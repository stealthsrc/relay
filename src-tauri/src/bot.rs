use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context as _, Result, bail};
use serenity::{
    all::{
        ChannelType, Command, CommandDataOptionValue, CommandInteraction, CommandOptionType,
        Context, CreateCommand, CreateCommandOption, CreateInteractionResponse,
        CreateInteractionResponseMessage, EventHandler, GatewayIntents, GuildId, Interaction,
        Message, MessageUpdateEvent, Permissions, Ready, UserId,
    },
    async_trait,
    cache::Cache,
    client::Client,
    http::Http,
};

use crate::{
    artwork,
    config::AppConfig,
    credentials::{load_discord_credentials, load_or_create_relay_secret},
    model::{AuthorIdentity, BotStatus, ChannelSummary, MediaEvent, MediaKind, TtsRequest},
    state::{AppCore, BotRuntime},
};

const IMAGE_EXTENSIONS: [&str; 5] = ["png", "jpg", "jpeg", "webp", "bmp"];
const VIDEO_EXTENSIONS: [&str; 6] = ["mp4", "webm", "mov", "m4v", "ogv", "avi"];
const AUDIO_EXTENSIONS: [&str; 7] = ["mp3", "ogg", "wav", "m4a", "flac", "aac", "opus"];

struct Handler {
    core: Arc<AppCore>,
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, context: Context, ready: Ready) {
        let avatar = ready
            .user
            .avatar_url()
            .unwrap_or_else(|| ready.user.default_avatar_url());
        *self.core.bot_status.write().await = BotStatus {
            connected: true,
            username: Some(ready.user.name.clone()),
            display_avatar_url: Some(avatar),
            error: None,
        };

        let guild_ids = ready.guilds.iter().map(|guild| guild.id).collect();
        let channels =
            discover_channels(&context.http, &context.cache, ready.user.id, guild_ids).await;
        *self.core.channels.write().await = channels;
        warn_if_watched_channel_missing(&self.core).await;

        if let Err(error) = Command::set_global_commands(&context.http, vec![relay_command()]).await
        {
            set_bot_error(&self.core, format!("Command registration failed: {error}")).await;
        }
    }

    async fn message(&self, _context: Context, message: Message) {
        if message.author.bot {
            return;
        }
        let config = self.core.config.read().await.clone();
        let channel_id = message.channel_id.to_string();

        if !config.tts_channel_id.is_empty() && channel_id == config.tts_channel_id {
            if let Some(text) = prepare_tts_text(&message.content, config.tts_character_limit)
                && let Err(error) = self
                    .core
                    .publish_tts(TtsRequest {
                        id: message.id.to_string(),
                        text,
                        author: AuthorIdentity {
                            username: message.author.name.clone(),
                            display_avatar_url: message
                                .author
                                .avatar_url()
                                .unwrap_or_else(|| message.author.default_avatar_url()),
                        },
                        timestamp: message.timestamp.unix_timestamp().max(0) as u64 * 1_000,
                    })
                    .await
            {
                self.core.bot_status.write().await.error =
                    Some(format!("Windows TTS failed: {error}"));
            }
            return;
        }

        if config.watched_channel_id.is_empty() || channel_id != config.watched_channel_id {
            return;
        }

        for attachment in message
            .attachments
            .iter()
            .filter_map(|item| classify_attachment(item).map(|kind| (item, kind)))
            .take(3)
        {
            let (attachment, kind) = attachment;
            let mut audio_metadata = if matches!(kind, MediaKind::Audio) {
                artwork::extract(&attachment.url).await.ok()
            } else {
                None
            };
            let artwork_id = if let Some(embedded) = audio_metadata
                .as_mut()
                .and_then(|metadata| metadata.artwork.take())
            {
                let id = attachment.id.to_string();
                self.core.cache_artwork(id.clone(), embedded).await;
                Some(id)
            } else {
                None
            };
            let audio_id = if let Some(metadata) = audio_metadata.as_mut() {
                let id = attachment.id.to_string();
                let content_type = attachment
                    .content_type
                    .clone()
                    .unwrap_or_else(|| "application/octet-stream".into());
                self.core
                    .cache_audio(
                        id.clone(),
                        content_type,
                        std::mem::take(&mut metadata.audio),
                    )
                    .await;
                Some(id)
            } else {
                None
            };
            let event = MediaEvent {
                kind,
                url: attachment.url.clone(),
                proxy_url: attachment.proxy_url.clone(),
                filename: attachment.filename.clone(),
                content_type: attachment.content_type.clone().unwrap_or_default(),
                artwork_id,
                audio_id,
                cached_media_id: None,
                title: audio_metadata
                    .as_ref()
                    .and_then(|metadata| metadata.title.clone()),
                artist: audio_metadata.and_then(|metadata| metadata.artist),
                author: AuthorIdentity {
                    username: message.author.name.clone(),
                    display_avatar_url: message
                        .author
                        .avatar_url()
                        .unwrap_or_else(|| message.author.default_avatar_url()),
                },
                timestamp: message.timestamp.unix_timestamp().max(0) as u64 * 1_000,
                message_id: message.id.to_string(),
            };
            self.core.submit_media(event).await;
        }

        submit_embedded_gifs(&self.core, &message).await;
    }

    async fn message_update(
        &self,
        context: Context,
        _old: Option<Message>,
        new: Option<Message>,
        event: MessageUpdateEvent,
    ) {
        if let Some(message) = new {
            submit_embedded_gifs(&self.core, &message).await;
            return;
        }
        let Some(embeds) = event.embeds else {
            return;
        };
        if let Some(author) = event.author {
            let message = DeferredEmbedMessage {
                channel_id: event.channel_id.to_string(),
                message_id: event.id.to_string(),
                author,
                timestamp: event
                    .timestamp
                    .map(|timestamp| timestamp.unix_timestamp().max(0) as u64 * 1_000)
                    .unwrap_or_else(current_timestamp_ms),
                embeds,
            };
            submit_deferred_embeds(&self.core, message).await;
            return;
        }

        let watched_channel_id = self.core.config.read().await.watched_channel_id.clone();
        if watched_channel_id.is_empty() || event.channel_id.to_string() != watched_channel_id {
            return;
        }
        if let Ok(message) = context.http.get_message(event.channel_id, event.id).await {
            submit_embedded_gifs(&self.core, &message).await;
        }
    }

    async fn interaction_create(&self, context: Context, interaction: Interaction) {
        let Interaction::Command(command) = interaction else {
            return;
        };
        if command.data.name != "relay" {
            return;
        }

        let content = handle_relay(&self.core, &command)
            .await
            .unwrap_or_else(|error| format!("Unable to update Relay: {error}"));
        let response = CreateInteractionResponse::Message(
            CreateInteractionResponseMessage::new()
                .content(content)
                .ephemeral(true),
        );
        if let Err(error) = command.create_response(&context.http, response).await {
            set_bot_error(&self.core, format!("Discord response failed: {error}")).await;
        }
    }
}

pub async fn start_bot(core: Arc<AppCore>) -> Result<bool> {
    stop_bot(&core).await;
    let Some((credentials, _source)) = load_discord_credentials()? else {
        *core.bot_status.write().await = BotStatus::default();
        return Ok(false);
    };

    let intents =
        GatewayIntents::GUILDS | GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(&credentials.token, intents)
        .event_handler(Handler { core: core.clone() })
        .await
        .context("failed to create the Discord client")?;
    let shard_manager = client.shard_manager.clone();
    let http = client.http.clone();
    let cache = client.cache.clone();
    let status_core = core.clone();
    let task = tokio::spawn(async move {
        if let Err(error) = client.start().await {
            set_bot_error(&status_core, format!("Discord connection failed: {error}")).await;
        }
        status_core.bot_status.write().await.connected = false;
    });
    *core.bot_runtime.lock().await = Some(BotRuntime {
        shard_manager,
        task,
        http,
        cache,
    });
    Ok(true)
}

pub async fn refresh_channel_list(core: &Arc<AppCore>) -> Result<()> {
    let (http, cache) = {
        let runtime = core.bot_runtime.lock().await;
        let runtime = runtime
            .as_ref()
            .context("the Discord bot is not running")?;
        (runtime.http.clone(), runtime.cache.clone())
    };
    if !core.bot_status.read().await.connected {
        bail!("the Discord bot is not connected");
    }
    let bot_id = cache.current_user().id;
    let guild_ids = cache.guilds();
    let channels = discover_channels(&http, &cache, bot_id, guild_ids).await;
    *core.channels.write().await = channels;
    warn_if_watched_channel_missing(core).await;
    Ok(())
}

async fn discover_channels(
    http: &Arc<Http>,
    cache: &Arc<Cache>,
    bot_id: UserId,
    guild_ids: Vec<GuildId>,
) -> Vec<ChannelSummary> {
    let mut channels = Vec::new();
    for guild_id in guild_ids {
        let guild_name = guild_id
            .to_partial_guild(http)
            .await
            .map(|guild| guild.name)
            .unwrap_or_else(|_| guild_id.to_string());
        if let Ok(guild_channels) = guild_id.channels(http).await {
            channels.extend(guild_channels.into_values().filter_map(|channel| {
                (matches!(channel.kind, ChannelType::Text | ChannelType::News)
                    && bot_can_view_channel(cache, &channel, bot_id))
                .then(|| ChannelSummary {
                    id: channel.id.to_string(),
                    name: channel.name,
                    guild_name: guild_name.clone(),
                })
            }));
        }
    }
    channels.sort_by(|left, right| {
        left.guild_name
            .cmp(&right.guild_name)
            .then(left.name.cmp(&right.name))
    });
    channels
}

async fn warn_if_watched_channel_missing(core: &Arc<AppCore>) {
    let configured_channel = core.config.read().await.watched_channel_id.clone();
    if !configured_channel.is_empty()
        && !core
            .channels
            .read()
            .await
            .iter()
            .any(|channel| channel.id == configured_channel)
    {
        core.bot_status.write().await.error = Some(
            "The selected media channel is private or inaccessible. Add Relay or its role to the channel permissions."
                .into(),
        );
    }
}

pub async fn stop_bot(core: &Arc<AppCore>) {
    if let Some(runtime) = core.bot_runtime.lock().await.take() {
        runtime.shard_manager.shutdown_all().await;
        runtime.task.abort();
    }
    *core.bot_status.write().await = BotStatus::default();
    core.channels.write().await.clear();
}

pub fn invite_url(client_id: &str) -> String {
    format!(
        "https://discord.com/oauth2/authorize?client_id={client_id}&permissions=66560&scope=bot%20applications.commands"
    )
}

fn relay_command() -> CreateCommand {
    CreateCommand::new("relay")
        .description("Configure the local OBS media relay")
        .default_member_permissions(Permissions::ADMINISTRATOR)
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "channel",
                "Set the channel whose media is relayed to OBS",
            )
            .add_sub_option(
                CreateCommandOption::new(CommandOptionType::Channel, "channel", "Channel to watch")
                    .channel_types(vec![ChannelType::Text, ChannelType::News])
                    .required(true),
            ),
        )
        .add_option(CreateCommandOption::new(
            CommandOptionType::SubCommand,
            "url",
            "Show the local relay and overlay URLs",
        ))
        .add_option(CreateCommandOption::new(
            CommandOptionType::SubCommand,
            "show",
            "Show relay configuration and connection details",
        ))
        .add_option(CreateCommandOption::new(
            CommandOptionType::SubCommand,
            "regenerate",
            "Reconnect local relay outputs without changing their URLs",
        ))
}

async fn handle_relay(core: &Arc<AppCore>, command: &CommandInteraction) -> Result<String> {
    let Some(option) = command.data.options.first() else {
        return Ok("Choose a Relay subcommand.".into());
    };
    let CommandDataOptionValue::SubCommand(arguments) = &option.value else {
        return Ok("Invalid Relay command.".into());
    };

    match option.name.as_str() {
        "channel" => {
            let channel_id = arguments
                .iter()
                .find_map(|argument| match argument.value {
                    CommandDataOptionValue::Channel(channel_id) => Some(channel_id.to_string()),
                    _ => None,
                })
                .context("a channel is required")?;
            let config = AppConfig {
                watched_channel_id: channel_id.clone(),
                ..core.config.read().await.clone()
            };
            core.set_config(config).await?;
            Ok(format!("Relay channel set to <#{channel_id}>."))
        }
        "url" => {
            let config = core.config.read().await.clone();
            let secret = load_or_create_relay_secret()?;
            Ok(connection_details(&config, &secret))
        }
        "show" => {
            let config = core.config.read().await.clone();
            let secret = load_or_create_relay_secret()?;
            let channel = if config.watched_channel_id.is_empty() {
                "not configured".to_owned()
            } else {
                format!("<#{}>", config.watched_channel_id)
            };
            Ok(format!(
                "Channel: {channel}\n{}\nThe relay secret is separate from the Discord token. Keep it private.",
                connection_details(&config, &secret)
            ))
        }
        "regenerate" => {
            let config = core.config.read().await.clone();
            let secret = load_or_create_relay_secret()?;
            Ok(format!(
                "The permanent relay URL was preserved. No OBS update is required:\n{}",
                overlay_url(&config, &secret)
            ))
        }
        _ => Ok("Unknown Relay subcommand.".into()),
    }
}

fn connection_details(config: &AppConfig, secret: &str) -> String {
    format!(
        "Relay URL: `http://127.0.0.1:{}`\nOverlay URL: `{}`\nSecret: `{secret}`",
        config.port,
        overlay_url(config, secret)
    )
}

fn overlay_url(config: &AppConfig, secret: &str) -> String {
    format!("http://127.0.0.1:{}/overlay?secret={secret}", config.port)
}

fn classify_attachment(attachment: &serenity::all::Attachment) -> Option<MediaKind> {
    classify_media(&attachment.filename, attachment.content_type.as_deref())
}

struct EmbeddedGif {
    url: String,
    proxy_url: String,
    title: Option<String>,
    content_type: &'static str,
}

struct DeferredEmbedMessage {
    channel_id: String,
    message_id: String,
    author: serenity::all::User,
    timestamp: u64,
    embeds: Vec<serenity::all::Embed>,
}

async fn submit_embedded_gifs(core: &Arc<AppCore>, message: &Message) {
    submit_deferred_embeds(
        core,
        DeferredEmbedMessage {
            channel_id: message.channel_id.to_string(),
            message_id: message.id.to_string(),
            author: message.author.clone(),
            timestamp: message.timestamp.unix_timestamp().max(0) as u64 * 1_000,
            embeds: message.embeds.clone(),
        },
    )
    .await;
}

async fn submit_deferred_embeds(core: &Arc<AppCore>, message: DeferredEmbedMessage) {
    if message.author.bot {
        return;
    }
    let config = core.config.read().await;
    if config.watched_channel_id.is_empty() || message.channel_id != config.watched_channel_id {
        return;
    }
    drop(config);
    for (index, embed) in message.embeds.iter().filter_map(embedded_gif).enumerate() {
        let event_id = format!("{}-embed-{index}", message.message_id);
        if !core.claim_embed(event_id.clone()).await {
            continue;
        }
        let downloaded =
            match artwork::download_bounded(&embed.url, artwork::MAX_EMBED_MEDIA_BYTES).await {
                Ok(bytes) => Some(bytes),
                Err(_) if embed.proxy_url != embed.url => {
                    artwork::download_bounded(&embed.proxy_url, artwork::MAX_EMBED_MEDIA_BYTES)
                        .await
                        .ok()
                }
                Err(_) => None,
            };
        let mut content_type = embed.content_type;
        let cached_media_id = if let Some(bytes) = downloaded {
            content_type = sniff_media_type(&bytes, content_type);
            core.cache_media(event_id.clone(), content_type.into(), bytes)
                .await;
            Some(event_id.clone())
        } else {
            None
        };
        core.submit_media(MediaEvent {
            kind: MediaKind::Gif,
            url: embed.url,
            proxy_url: embed.proxy_url,
            filename: embed.title.unwrap_or_else(|| "Discord GIF".into()),
            content_type: content_type.into(),
            artwork_id: None,
            audio_id: None,
            cached_media_id,
            title: None,
            artist: None,
            author: AuthorIdentity {
                username: message.author.name.clone(),
                display_avatar_url: message
                    .author
                    .avatar_url()
                    .unwrap_or_else(|| message.author.default_avatar_url()),
            },
            timestamp: message.timestamp,
            message_id: message.message_id.clone(),
        })
        .await;
    }
}

fn embedded_gif(embed: &serenity::all::Embed) -> Option<EmbeddedGif> {
    let is_gifv = embed.kind.as_deref() == Some("gifv");
    let image_is_gif = embed
        .image
        .as_ref()
        .is_some_and(|image| url_has_extension(&image.url, "gif"));
    let known_provider = [
        embed.url.as_deref(),
        embed
            .provider
            .as_ref()
            .and_then(|provider| provider.name.as_deref()),
        embed
            .provider
            .as_ref()
            .and_then(|provider| provider.url.as_deref()),
        embed.image.as_ref().map(|image| image.url.as_str()),
        embed.video.as_ref().map(|video| video.url.as_str()),
    ]
    .into_iter()
    .flatten()
    .any(|value| {
        let value = value.to_ascii_lowercase();
        value.contains("klipy") || value.contains("tenor") || value.contains("giphy")
    });
    if !is_gifv && !image_is_gif && !known_provider {
        return None;
    }
    if let Some(video) = embed.video.as_ref() {
        let content_type = video_mime_type(&video.url)
            .or_else(|| video.proxy_url.as_deref().and_then(video_mime_type))
            .unwrap_or("video/mp4");
        return Some(EmbeddedGif {
            url: video.url.clone(),
            proxy_url: video.proxy_url.clone().unwrap_or_else(|| video.url.clone()),
            title: embed.title.clone(),
            content_type,
        });
    }
    let video_url = [
        embed.image.as_ref().map(|media| media.url.as_str()),
        embed
            .image
            .as_ref()
            .and_then(|media| media.proxy_url.as_deref()),
        embed.thumbnail.as_ref().map(|media| media.url.as_str()),
        embed
            .thumbnail
            .as_ref()
            .and_then(|media| media.proxy_url.as_deref()),
    ]
    .into_iter()
    .flatten()
    .find_map(|url| video_mime_type(url).map(|content_type| (url, content_type)));
    if let Some((url, content_type)) = video_url {
        return Some(EmbeddedGif {
            url: url.to_owned(),
            proxy_url: url.to_owned(),
            title: embed.title.clone(),
            content_type,
        });
    }
    let image = embed.image.as_ref()?;
    Some(EmbeddedGif {
        url: image.url.clone(),
        proxy_url: image.proxy_url.clone().unwrap_or_else(|| image.url.clone()),
        title: embed.title.clone(),
        content_type: if url_has_extension(&image.url, "webp") {
            "image/webp"
        } else {
            "image/gif"
        },
    })
}

fn video_mime_type(url: &str) -> Option<&'static str> {
    if url_has_extension(url, "mp4") || url_has_extension(url, "m4v") {
        Some("video/mp4")
    } else if url_has_extension(url, "webm") {
        Some("video/webm")
    } else {
        None
    }
}

fn sniff_media_type(bytes: &[u8], fallback: &'static str) -> &'static str {
    if bytes.get(4..8) == Some(b"ftyp".as_slice()) {
        "video/mp4"
    } else if bytes.starts_with(&[0x1a, 0x45, 0xdf, 0xa3]) {
        "video/webm"
    } else if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        "image/gif"
    } else if bytes.starts_with(b"RIFF") && bytes.get(8..12) == Some(b"WEBP".as_slice()) {
        "image/webp"
    } else {
        fallback
    }
}

#[allow(deprecated)]
fn bot_can_view_channel(
    cache: &Arc<Cache>,
    channel: &serenity::all::GuildChannel,
    bot_id: UserId,
) -> bool {
    channel
        .permissions_for_user(cache, bot_id)
        .map(|permissions| permissions.contains(Permissions::VIEW_CHANNEL))
        .unwrap_or(true)
}

fn url_has_extension(url: &str, extension: &str) -> bool {
    url.split(['?', '#']).next().is_some_and(|path| {
        path.to_ascii_lowercase()
            .ends_with(&format!(".{extension}"))
    })
}

fn classify_media(filename: &str, content_type: Option<&str>) -> Option<MediaKind> {
    let content_type = content_type.unwrap_or_default().to_ascii_lowercase();
    if content_type == "image/gif" {
        return Some(MediaKind::Gif);
    }
    if content_type.starts_with("image/") {
        return Some(MediaKind::Image);
    }
    if content_type.starts_with("video/") {
        return Some(MediaKind::Video);
    }
    if content_type.starts_with("audio/") {
        return Some(MediaKind::Audio);
    }

    let extension = filename
        .rsplit_once('.')
        .map(|(_, extension)| extension.to_ascii_lowercase())?;
    if extension == "gif" {
        Some(MediaKind::Gif)
    } else if IMAGE_EXTENSIONS.contains(&extension.as_str()) {
        Some(MediaKind::Image)
    } else if VIDEO_EXTENSIONS.contains(&extension.as_str()) {
        Some(MediaKind::Video)
    } else if AUDIO_EXTENSIONS.contains(&extension.as_str()) {
        Some(MediaKind::Audio)
    } else {
        None
    }
}

fn prepare_tts_text(content: &str, character_limit: u32) -> Option<String> {
    let content = content.trim();
    if content.is_empty() {
        return None;
    }
    if character_limit == 0 {
        return Some(content.to_owned());
    }
    Some(content.chars().take(character_limit as usize).collect())
}

async fn set_bot_error(core: &Arc<AppCore>, error: String) {
    let mut status = core.bot_status.write().await;
    status.connected = false;
    status.error = Some(error);
}

#[allow(dead_code)]
fn current_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_local_overlay_url_with_secret() {
        let config = AppConfig {
            port: 5_321,
            ..AppConfig::default()
        };
        assert_eq!(
            overlay_url(&config, "private"),
            "http://127.0.0.1:5321/overlay?secret=private"
        );
    }

    #[test]
    fn builds_invite_url_with_required_scopes_and_permissions() {
        assert_eq!(
            invite_url("123456789012345678"),
            "https://discord.com/oauth2/authorize?client_id=123456789012345678&permissions=66560&scope=bot%20applications.commands"
        );
    }

    #[test]
    fn classifies_supported_media_by_content_type_and_extension() {
        assert!(matches!(
            classify_media("still.png", None),
            Some(MediaKind::Image)
        ));
        assert!(matches!(
            classify_media("loop.gif", None),
            Some(MediaKind::Gif)
        ));
        assert!(matches!(
            classify_media("clip.bin", Some("video/mp4")),
            Some(MediaKind::Video)
        ));
        assert!(matches!(
            classify_media("track.opus", None),
            Some(MediaKind::Audio)
        ));
        assert!(classify_media("notes.txt", Some("text/plain")).is_none());
    }

    #[test]
    fn extracts_discord_gifv_embeds_as_muted_video_gifs() {
        let embed: serenity::all::Embed = serde_json::from_value(serde_json::json!({
            "type": "gifv",
            "title": "Tenor animation",
            "url": "https://tenor.com/view/example",
            "video": {
                "url": "https://media.tenor.com/example.mp4",
                "proxy_url": "https://images-ext-1.discordapp.net/example.mp4"
            },
            "thumbnail": { "url": "https://media.tenor.com/example.gif" }
        }))
        .unwrap();
        let gif = embedded_gif(&embed).expect("Discord gifv should be relayed");
        assert_eq!(gif.url, "https://media.tenor.com/example.mp4");
        assert_eq!(gif.content_type, "video/mp4");
        assert_eq!(gif.title.as_deref(), Some("Tenor animation"));
    }

    #[test]
    fn accepts_klipy_image_embeds_even_without_gifv_type() {
        let embed: serenity::all::Embed = serde_json::from_value(serde_json::json!({
            "type": "image",
            "provider": { "name": "KLIPY", "url": "https://klipy.com" },
            "image": { "url": "https://cdn.klipy.com/animated.webp" }
        }))
        .unwrap();
        let gif = embedded_gif(&embed).expect("KLIPY image embed should be relayed");
        assert_eq!(gif.url, "https://cdn.klipy.com/animated.webp");
        assert_eq!(gif.content_type, "image/webp");
    }

    #[test]
    fn treats_klipy_image_proxy_mp4_as_video_gif() {
        let embed: serenity::all::Embed = serde_json::from_value(serde_json::json!({
            "type": "image",
            "title": "Klipy picker GIF",
            "provider": { "name": "KLIPY", "url": "https://klipy.com" },
            "image": {
                "url": "https://static.klipy.com/preview.jpg",
                "proxy_url": "https://images-ext-1.discordapp.net/external/example/clip.mp4"
            }
        }))
        .unwrap();
        let gif = embedded_gif(&embed).expect("KLIPY MP4 proxy should be relayed");
        assert_eq!(
            gif.url,
            "https://images-ext-1.discordapp.net/external/example/clip.mp4"
        );
        assert_eq!(gif.content_type, "video/mp4");
    }

    #[test]
    fn sniffs_mp4_bytes_when_discord_reports_an_image() {
        let bytes = b"\0\0\0\x18ftypisom\0\0\0\0isom";
        assert_eq!(sniff_media_type(bytes, "image/gif"), "video/mp4");
    }

    #[test]
    fn prepares_plain_tts_messages_with_an_optional_unicode_limit() {
        assert_eq!(
            prepare_tts_text("  Bonjour Relay  ", 0).as_deref(),
            Some("Bonjour Relay")
        );
        assert_eq!(
            prepare_tts_text("\u{e9}l\u{e9}phant", 3).as_deref(),
            Some("\u{e9}l\u{e9}")
        );
        assert!(prepare_tts_text("   ", 0).is_none());
    }
}
