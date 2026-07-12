use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context as _, Result, bail};
use serenity::{
    all::{
        Channel, ChannelId, ChannelType, Command, CommandDataOptionValue, CommandInteraction,
        CommandOptionType, Context, CreateCommand, CreateCommandOption,
        CreateInteractionResponse, CreateInteractionResponseMessage, EventHandler, GatewayIntents,
        GetMessages, GuildId, Interaction, Message, MessageId, MessageUpdateEvent,
        PermissionOverwrite, PermissionOverwriteType, Permissions, Ready, StickerFormatType, UserId,
    },
    async_trait,
    cache::Cache,
    client::Client,
    http::Http,
};

use crate::{
    artwork,
    config::{AppConfig, ChannelLockSnapshot, PermissionOverwriteSnapshot},
    credentials::{load_discord_credentials, load_or_create_relay_secret},
    model::{
        AuthorIdentity, BotStatus, ChannelSummary, MediaEvent, MediaKind, StickerEvent, TtsRequest,
        VisualSegment,
    },
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
            if let Some(segments) = parse_visual_segments(&message.content) {
                self.core.publish_visual_tts(
                    message.id.to_string(),
                    message.content.clone(),
                    message_author(&message),
                    message_timestamp(&message),
                    segments,
                );
                return;
            }
            if let Some(text) = prepare_tts_text(&message.content, config.tts_character_limit)
                && let Err(error) = self
                    .core
                    .publish_tts(TtsRequest {
                        id: message.id.to_string(),
                        text,
                        author: message_author(&message),
                        timestamp: message_timestamp(&message),
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

        for sticker in message.sticker_items.iter().take(3) {
            let Some(url) = sticker.image_url() else {
                continue;
            };
            let (format, content_type) = sticker_format(sticker.format_type);
            let cache_id = format!("sticker-{}", sticker.id);
            let cached_media_id = match artwork::download_bounded(
                &url,
                artwork::MAX_ARTWORK_BYTES,
            )
            .await
            {
                Ok(bytes) => {
                    self.core
                        .cache_media(cache_id.clone(), content_type.into(), bytes)
                        .await;
                    Some(cache_id)
                }
                Err(_) => None,
            };
            self.core.publish_sticker(StickerEvent {
                id: sticker.id.to_string(),
                name: sticker.name.clone(),
                format: format.into(),
                url,
                cached_media_id,
                author: message_author(&message),
                timestamp: message_timestamp(&message),
                message_id: message.id.to_string(),
            });
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

        let content = handle_relay(&self.core, &context.http, &command)
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

fn message_author(message: &Message) -> AuthorIdentity {
    AuthorIdentity {
        username: message.author.name.clone(),
        display_avatar_url: message
            .author
            .avatar_url()
            .unwrap_or_else(|| message.author.default_avatar_url()),
    }
}

fn message_timestamp(message: &Message) -> u64 {
    message.timestamp.unix_timestamp().max(0) as u64 * 1_000
}

fn sticker_format(format: StickerFormatType) -> (&'static str, &'static str) {
    match format {
        StickerFormatType::Png => ("png", "image/png"),
        StickerFormatType::Apng => ("apng", "image/png"),
        StickerFormatType::Lottie => ("lottie", "application/json"),
        StickerFormatType::Gif => ("gif", "image/gif"),
        StickerFormatType::Unknown(_) => ("unknown", "application/octet-stream"),
        _ => ("unknown", "application/octet-stream"),
    }
}

fn parse_visual_segments(content: &str) -> Option<Vec<VisualSegment>> {
    let mut segments = Vec::new();
    let mut text = String::new();
    let mut cursor = 0;
    let mut found_emoji = false;

    while cursor < content.len() {
        let remainder = &content[cursor..];
        if let Some((consumed, value, url, animated)) = parse_custom_emoji(remainder) {
            push_text_segment(&mut segments, &mut text);
            segments.push(VisualSegment {
                kind: "emoji".into(),
                value,
                url: Some(url),
                animated,
            });
            cursor += consumed;
            found_emoji = true;
            continue;
        }

        let character = remainder.chars().next().expect("cursor is on a character boundary");
        if is_unicode_emoji(character) {
            push_text_segment(&mut segments, &mut text);
            segments.push(VisualSegment {
                kind: "emoji".into(),
                value: character.to_string(),
                url: None,
                animated: false,
            });
            found_emoji = true;
        } else {
            text.push(character);
        }
        cursor += character.len_utf8();
    }
    push_text_segment(&mut segments, &mut text);
    found_emoji.then_some(segments)
}

fn parse_custom_emoji(content: &str) -> Option<(usize, String, String, bool)> {
    if !content.starts_with("<:") && !content.starts_with("<a:") {
        return None;
    }
    let end = content.find('>')?;
    let token = &content[..=end];
    let animated = token.starts_with("<a:");
    let body = token.strip_prefix(if animated { "<a:" } else { "<:" })?.strip_suffix('>')?;
    let (name, id) = body.rsplit_once(':')?;
    if name.is_empty() || id.len() > 20 || !id.chars().all(|character| character.is_ascii_digit()) {
        return None;
    }
    let url = format!(
        "https://cdn.discordapp.com/emojis/{id}.webp?size=128&animated={animated}"
    );
    Some((token.len(), format!(":{name}:"), url, animated))
}

fn push_text_segment(segments: &mut Vec<VisualSegment>, text: &mut String) {
    if !text.is_empty() {
        segments.push(VisualSegment {
            kind: "text".into(),
            value: std::mem::take(text),
            url: None,
            animated: false,
        });
    }
}

fn is_unicode_emoji(character: char) -> bool {
    matches!(
        character as u32,
        0x1F000..=0x1FAFF
            | 0x2600..=0x27BF
            | 0x2300..=0x23FF
            | 0x2B00..=0x2BFF
            | 0xFE0F
    )
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
    let permissions =
        (Permissions::VIEW_CHANNEL
            | Permissions::READ_MESSAGE_HISTORY
            | Permissions::MANAGE_ROLES
            | Permissions::MANAGE_MESSAGES)
            .bits();
    format!(
        "https://discord.com/oauth2/authorize?client_id={client_id}&permissions={permissions}&scope=bot%20applications.commands"
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
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "clear",
                "Delete a chosen number of messages from each configured channel",
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::Integer,
                    "count",
                    "Number of messages to delete from each channel (1-1000)",
                )
                .min_int_value(1)
                .max_int_value(1_000)
                .required(true),
            ),
        )
        .add_option(CreateCommandOption::new(
            CommandOptionType::SubCommand,
            "lock",
            "Toggle the configured media channel lock",
        ))
}

async fn handle_relay(
    core: &Arc<AppCore>,
    http: &Http,
    command: &CommandInteraction,
) -> Result<String> {
    let Some(option) = command.data.options.first() else {
        return Ok("Choose a Relay subcommand.".into());
    };
    let CommandDataOptionValue::SubCommand(arguments) = &option.value else {
        return Ok("Invalid Relay command.".into());
    };
    let config = core.config.read().await.clone();
    let lock_can_restore = option.name == "lock" && config.channel_lock.is_some();
    if !command_enabled(&config, &option.name) && !lock_can_restore {
        return Ok(format!(
            "`/relay {}` is disabled in the Relay application.",
            option.name
        ));
    }

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
        "clear" => {
            let count = arguments
                .iter()
                .find_map(|argument| match argument.value {
                    CommandDataOptionValue::Integer(value) => usize::try_from(value).ok(),
                    _ => None,
                })
                .filter(|count| (1..=1_000).contains(count))
                .context("a message count between 1 and 1000 is required")?;
            clear_configured_channels(core, http, count).await
        }
        "lock" => toggle_channel_lock(core, http).await,
        _ => Ok("Unknown Relay subcommand.".into()),
    }
}

async fn clear_configured_channels(
    core: &Arc<AppCore>,
    http: &Http,
    count: usize,
) -> Result<String> {
    let config = core.config.read().await.clone();
    let channels = [
        ("Media", config.watched_channel_id),
        ("TTS", config.tts_channel_id),
    ]
    .into_iter()
    .filter(|(_, channel_id)| !channel_id.is_empty())
    .collect::<Vec<_>>();
    if channels.is_empty() {
        bail!("configure a media or TTS channel before clearing Discord messages");
    }

    let mut cleared = Vec::new();
    let mut failures = Vec::new();
    for (label, channel_id) in channels {
        let id = ChannelId::new(channel_id.parse()?);
        match clear_channel_messages(http, id, count).await {
            Ok(count) => cleared.push(format!("{label} <#{channel_id}>: {count} messages")),
            Err(error) => failures.push(format!("{label} <#{channel_id}>: {error}")),
        }
    }
    if !failures.is_empty() {
        let completed = if cleared.is_empty() {
            "No channel was cleared.".into()
        } else {
            format!("Completed: {}.", cleared.join(", "))
        };
        bail!("Discord channel cleanup was partial. {completed} Failed: {}", failures.join(", "));
    }
    Ok(format!("Discord channels cleared: {}.", cleared.join(", ")))
}

async fn clear_channel_messages(
    http: &Http,
    channel_id: ChannelId,
    limit: usize,
) -> Result<usize> {
    let mut before = None;
    let mut deleted = 0;
    while deleted < limit {
        let page_limit = (limit - deleted).min(100) as u8;
        let mut request = GetMessages::new().limit(page_limit);
        if let Some(message_id) = before {
            request = request.before(message_id);
        }
        let messages = channel_id.messages(http, request).await?;
        if messages.is_empty() {
            break;
        }
        before = messages.last().map(|message| message.id);
        let now = current_timestamp_ms() / 1_000;
        let (recent, old): (Vec<MessageId>, Vec<MessageId>) = messages
            .iter()
            .take(limit - deleted)
            .map(|message| message.id)
            .partition(|message_id| is_bulk_deletable(*message_id, now));

        if recent.len() >= 2 {
            channel_id.delete_messages(http, &recent).await?;
            deleted += recent.len();
        } else if let Some(message_id) = recent.first() {
            channel_id.delete_message(http, message_id).await?;
            deleted += 1;
        }
        for message_id in old {
            channel_id.delete_message(http, message_id).await?;
            deleted += 1;
        }
        if messages.len() < usize::from(page_limit) {
            break;
        }
    }
    Ok(deleted)
}

fn is_bulk_deletable(message_id: MessageId, now_seconds: u64) -> bool {
    const SAFE_BULK_DELETE_AGE_SECONDS: u64 = 13 * 24 * 60 * 60 + 23 * 60 * 60;
    let created = message_id.created_at().unix_timestamp().max(0) as u64;
    now_seconds.saturating_sub(created) < SAFE_BULK_DELETE_AGE_SECONDS
}

fn command_enabled(config: &AppConfig, command: &str) -> bool {
    match command {
        "channel" => config.command_channel_enabled,
        "url" => config.command_url_enabled,
        "show" => config.command_show_enabled,
        "regenerate" => config.command_regenerate_enabled,
        "clear" => config.command_clear_enabled,
        "lock" => config.command_lock_enabled,
        _ => false,
    }
}

async fn toggle_channel_lock(core: &Arc<AppCore>, http: &Http) -> Result<String> {
    let config = core.config.read().await.clone();
    if let Some(snapshot) = config.channel_lock.clone() {
        restore_channel_permissions(http, &snapshot).await?;
        let mut next = core.config.read().await.clone();
        next.channel_lock = None;
        core.set_config(next).await?;
        return Ok(format!("<#{0}> is unlocked.", snapshot.channel_id));
    }
    if config.watched_channel_id.is_empty() {
        bail!("configure a media channel before locking it");
    }

    let channel_id = ChannelId::new(config.watched_channel_id.parse()?);
    let Channel::Guild(channel) = channel_id.to_channel(http).await? else {
        bail!("the configured media channel is not a server text channel");
    };
    let roles = channel.guild_id.roles(http).await?;
    let everyone = channel.guild_id.everyone_role();
    let mut targets = vec![PermissionOverwriteType::Role(everyone)];
    targets.extend(roles.values().filter_map(|role| {
        (role.id != everyone
            && role.permissions.intersects(
                Permissions::ADMINISTRATOR
                    | Permissions::MANAGE_CHANNELS
                    | Permissions::MANAGE_MESSAGES,
            ))
        .then_some(PermissionOverwriteType::Role(role.id))
    }));

    let snapshot = ChannelLockSnapshot {
        channel_id: channel.id.to_string(),
        overwrites: targets
            .iter()
            .filter_map(|kind| snapshot_permission(&channel.permission_overwrites, *kind))
            .collect(),
    };
    let mut next = config;
    next.channel_lock = Some(snapshot.clone());
    core.set_config(next).await?;

    for kind in targets {
        let existing = channel
            .permission_overwrites
            .iter()
            .find(|overwrite| overwrite.kind == kind);
        let mut allow = existing.map_or(Permissions::empty(), |overwrite| overwrite.allow);
        let mut deny = existing.map_or(Permissions::empty(), |overwrite| overwrite.deny);
        if kind == PermissionOverwriteType::Role(everyone) {
            allow.remove(Permissions::SEND_MESSAGES);
            deny.insert(Permissions::SEND_MESSAGES);
        } else {
            deny.remove(Permissions::SEND_MESSAGES);
            allow.insert(Permissions::SEND_MESSAGES);
        }
        if let Err(error) = channel
            .create_permission(http, PermissionOverwrite { allow, deny, kind })
            .await
        {
            if restore_channel_permissions(http, &snapshot).await.is_ok() {
                let mut rollback = core.config.read().await.clone();
                rollback.channel_lock = None;
                let _ = core.set_config(rollback).await;
            }
            bail!("Discord refused the channel lock: {error}");
        }
    }
    Ok(format!(
        "<#{0}> is locked. Administrators and moderation roles can still write.",
        snapshot.channel_id
    ))
}

fn snapshot_permission(
    overwrites: &[PermissionOverwrite],
    kind: PermissionOverwriteType,
) -> Option<PermissionOverwriteSnapshot> {
    let (target_kind, target_id) = match kind {
        PermissionOverwriteType::Member(id) => ("member", id.to_string()),
        PermissionOverwriteType::Role(id) => ("role", id.to_string()),
        _ => return None,
    };
    let existing = overwrites.iter().find(|overwrite| overwrite.kind == kind);
    Some(PermissionOverwriteSnapshot {
        target_id,
        target_kind: target_kind.into(),
        allow: existing.map_or(0, |overwrite| overwrite.allow.bits()),
        deny: existing.map_or(0, |overwrite| overwrite.deny.bits()),
        existed: existing.is_some(),
    })
}

async fn restore_channel_permissions(http: &Http, snapshot: &ChannelLockSnapshot) -> Result<()> {
    let channel_id = ChannelId::new(snapshot.channel_id.parse()?);
    let Channel::Guild(channel) = channel_id.to_channel(http).await? else {
        bail!("the locked channel is no longer a server channel");
    };
    for saved in &snapshot.overwrites {
        let id = saved.target_id.parse::<u64>()?;
        let kind = match saved.target_kind.as_str() {
            "member" => PermissionOverwriteType::Member(UserId::new(id)),
            "role" => PermissionOverwriteType::Role(serenity::all::RoleId::new(id)),
            _ => bail!("the saved channel permission target is invalid"),
        };
        if saved.existed {
            channel
                .create_permission(
                    http,
                    PermissionOverwrite {
                        allow: Permissions::from_bits_truncate(saved.allow),
                        deny: Permissions::from_bits_truncate(saved.deny),
                        kind,
                    },
                )
                .await?;
        } else if channel
            .permission_overwrites
            .iter()
            .any(|overwrite| overwrite.kind == kind)
        {
            channel.delete_permission(http, kind).await?;
        }
    }
    Ok(())
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
    let image_is_gif = [
        embed.url.as_deref(),
        embed.image.as_ref().map(|image| image.url.as_str()),
        embed
            .image
            .as_ref()
            .and_then(|image| image.proxy_url.as_deref()),
        embed
            .thumbnail
            .as_ref()
            .map(|thumbnail| thumbnail.url.as_str()),
        embed
            .thumbnail
            .as_ref()
            .and_then(|thumbnail| thumbnail.proxy_url.as_deref()),
    ]
    .into_iter()
    .flatten()
    .any(|url| url_has_extension(url, "gif"));
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
    let (url, proxy_url) = if let Some(image) = embed.image.as_ref() {
        (
            image.url.clone(),
            image.proxy_url.clone().unwrap_or_else(|| image.url.clone()),
        )
    } else if let Some(thumbnail) = embed.thumbnail.as_ref() {
        (
            thumbnail.url.clone(),
            thumbnail
                .proxy_url
                .clone()
                .unwrap_or_else(|| thumbnail.url.clone()),
        )
    } else {
        return None;
    };
    Some(EmbeddedGif {
        url: url.clone(),
        proxy_url,
        title: embed.title.clone(),
        content_type: if url_has_extension(&url, "webp") {
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
            "https://discord.com/oauth2/authorize?client_id=123456789012345678&permissions=268510208&scope=bot%20applications.commands"
        );
    }

    #[test]
    fn disables_commands_individually() {
        let config = AppConfig {
            command_clear_enabled: false,
            ..AppConfig::default()
        };
        assert!(!command_enabled(&config, "clear"));
        assert!(command_enabled(&config, "lock"));
        assert!(!command_enabled(&config, "unknown"));
    }

    #[test]
    fn clear_command_requires_a_bounded_message_count() {
        let command = serde_json::to_value(relay_command()).unwrap();
        let clear = command["options"]
            .as_array()
            .unwrap()
            .iter()
            .find(|option| option["name"] == "clear")
            .unwrap();
        let count = &clear["options"][0];
        assert_eq!(count["name"], "count");
        assert_eq!(count["required"], true);
        assert_eq!(count["min_value"], 1);
        assert_eq!(count["max_value"], 1_000);
    }

    #[test]
    fn snapshots_missing_permission_overwrites_for_exact_restoration() {
        let kind = PermissionOverwriteType::Role(serenity::all::RoleId::new(42));
        let saved = snapshot_permission(&[], kind).unwrap();
        assert_eq!(saved.target_kind, "role");
        assert_eq!(saved.target_id, "42");
        assert!(!saved.existed);
        assert_eq!(saved.allow, 0);
        assert_eq!(saved.deny, 0);
    }

    #[test]
    fn bulk_deletes_only_messages_safely_inside_discords_two_week_limit() {
        let now = current_timestamp_ms() / 1_000;
        let recent = MessageId::new(((now - 60) * 1_000 - 1_420_070_400_000) << 22);
        let old = MessageId::new(((now - 14 * 24 * 60 * 60) * 1_000 - 1_420_070_400_000) << 22);
        assert!(is_bulk_deletable(recent, now));
        assert!(!is_bulk_deletable(old, now));
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
    fn accepts_discord_favorite_gifs_stored_as_thumbnail_only_images() {
        let embed: serenity::all::Embed = serde_json::from_value(serde_json::json!({
            "type": "image",
            "url": "https://media.tenor.com/example/john-pork-is-calling.gif",
            "thumbnail": {
                "url": "https://media.tenor.com/example/john-pork-is-calling.gif",
                "proxy_url": "https://images-ext-1.discordapp.net/external/example/john-pork-is-calling.gif",
                "height": 387,
                "width": 220
            }
        }))
        .unwrap();

        let gif = embedded_gif(&embed).expect("thumbnail-only Discord favorite should be relayed");
        assert_eq!(
            gif.url,
            "https://media.tenor.com/example/john-pork-is-calling.gif"
        );
        assert_eq!(
            gif.proxy_url,
            "https://images-ext-1.discordapp.net/external/example/john-pork-is-calling.gif"
        );
        assert_eq!(gif.content_type, "image/gif");
    }

    #[test]
    fn accepts_thumbnail_only_direct_gifs_without_a_known_provider() {
        let embed: serenity::all::Embed = serde_json::from_value(serde_json::json!({
            "type": "image",
            "thumbnail": { "url": "https://example.com/animation.gif" }
        }))
        .unwrap();

        let gif = embedded_gif(&embed).expect("direct GIF thumbnails should be relayed");
        assert_eq!(gif.url, "https://example.com/animation.gif");
        assert_eq!(gif.content_type, "image/gif");
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

    #[test]
    fn converts_unicode_and_custom_emojis_to_visual_segments() {
        let segments = parse_visual_segments(
            "Hello 👋 <:relay:123456789012345678> <a:dance:223456789012345678>",
        )
        .expect("message contains emojis");
        assert_eq!(segments.iter().filter(|segment| segment.kind == "emoji").count(), 3);
        assert!(segments.iter().any(|segment| segment.value == "👋"));
        assert!(segments.iter().any(|segment| {
            segment.value == ":dance:"
                && segment.animated
                && segment.url.as_deref().is_some_and(|url| url.contains("223456789012345678"))
        }));
    }

    #[test]
    fn leaves_plain_tts_messages_on_the_audio_path() {
        assert!(parse_visual_segments("Relay reads this message").is_none());
        assert!(parse_visual_segments("invalid <:emoji:not-an-id>").is_none());
    }

    #[test]
    fn maps_all_discord_sticker_formats() {
        assert_eq!(sticker_format(StickerFormatType::Png), ("png", "image/png"));
        assert_eq!(sticker_format(StickerFormatType::Apng), ("apng", "image/png"));
        assert_eq!(sticker_format(StickerFormatType::Lottie), ("lottie", "application/json"));
        assert_eq!(sticker_format(StickerFormatType::Gif), ("gif", "image/gif"));
    }
}
