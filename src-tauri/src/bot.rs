use std::{
    sync::Arc,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use anyhow::{Context as _, Result, bail};
use futures_util::StreamExt;
use serenity::{
    all::{
        ActionRowComponent, ButtonStyle, Channel, ChannelId, ChannelType, Colour, Command,
        CommandDataOptionValue, CommandInteraction, CommandOptionType, ComponentInteraction,
        ComponentInteractionDataKind, Context, CreateActionRow, CreateAllowedMentions,
        CreateButton, CreateChannel, CreateCommand, CreateCommandOption, CreateEmbed,
        CreateInputText, CreateInteractionResponse, CreateInteractionResponseMessage,
        CreateMessage, CreateModal, CreateSelectMenu, CreateSelectMenuKind, CreateSelectMenuOption,
        EditInteractionResponse, EventHandler, GatewayIntents, GetMessages, GuildId,
        InputTextStyle, Interaction, Message, MessageId, MessageUpdateEvent, ModalInteraction,
        OnlineStatus, PermissionOverwrite, PermissionOverwriteType, Permissions, Ready,
        StickerFormatType, User, UserId,
    },
    async_trait,
    cache::Cache,
    client::Client,
    gateway::ActivityData,
    http::Http,
};

use crate::{
    artwork,
    commands::emit_output_test,
    config::{AppConfig, ChannelLockSnapshot, HoneypotAction, PermissionOverwriteSnapshot},
    credentials::{load_discord_credentials, load_youtube_api_key},
    custom_commands,
    model::{
        AuthorIdentity, BotStatus, ChannelSummary, GuildTagIdentity, MediaEvent, MediaKind,
        MusicPlaybackEvent, MusicPlaybackMode, OutputConnectionStatus, OutputTestTarget,
        ServerStatus, StickerEvent, TtsRequest, VisualSegment,
    },
    music::{
        MusicSelection, MusicSkipDecision, MusicStartResult, SearchSelection, SelectionTake,
        cooldown_wait_seconds, parse_timestamp,
    },
    music_i18n::{self, MusicStrings},
    privacy,
    stage_scheduler::{StageLane, StageTicket},
    state::{AppCore, BotRuntime},
    youtube,
};

const IMAGE_EXTENSIONS: [&str; 5] = ["png", "jpg", "jpeg", "webp", "bmp"];
const VIDEO_EXTENSIONS: [&str; 6] = ["mp4", "webm", "mov", "m4v", "ogv", "avi"];
const AUDIO_EXTENSIONS: [&str; 7] = ["mp3", "ogg", "wav", "m4a", "flac", "aac", "opus"];
const MEDIA_TEXT_LIMIT: usize = 180;
const MUSIC_COMPONENT_PREFIX: &str = "relay:music:";
const MUSIC_SEARCH_PREFIX: &str = "relay:music:search:";
const MUSIC_MODE_PREFIX: &str = "relay:music:mode:";
const MUSIC_CUSTOM_PREFIX: &str = "relay:music:custom:";
const MUSIC_SKIP_PREFIX: &str = "relay:music:skip:";
const MUSIC_CUSTOM_START_ID: &str = "start";
const MUSIC_CUSTOM_END_ID: &str = "end";

struct Handler {
    core: Arc<AppCore>,
}

const HONEYPOT_AUDIT_REASON: &str =
    "Relay compromised-account trap: message posted in the protected channel.";

fn honeypot_action_for_channel(config: &AppConfig, channel_id: &str) -> Option<HoneypotAction> {
    (!config.honeypot_channel_id.is_empty() && config.honeypot_channel_id == channel_id)
        .then_some(config.honeypot_action)
}

fn honeypot_notice(action: HoneypotAction) -> &'static str {
    match action {
        HoneypotAction::Kick => {
            "A message from your Discord account was posted in a protected honeypot channel used to identify compromised accounts. Your account may have been compromised, including by token-grabbing malware or a malicious application. As a precaution, you will be kicked from the server. Change your Discord password, enable two-factor authentication, and review Authorized Apps before rejoining."
        }
        HoneypotAction::Ban => {
            "A message from your Discord account was posted in a protected honeypot channel used to identify compromised accounts. Your account may have been compromised, including by token-grabbing malware or a malicious application. As a precaution, you will be banned from the server. Change your Discord password, enable two-factor authentication, and review Authorized Apps before contacting the server moderators."
        }
    }
}

async fn enforce_honeypot_action(
    core: &AppCore,
    context: &Context,
    message: &Message,
    action: HoneypotAction,
) {
    let Some(guild_id) = message.guild_id else {
        core.bot_status.write().await.error =
            Some("Compromised account trap ignored a non-server message.".into());
        return;
    };

    let dm_delivered = message
        .author
        .direct_message(
            &context.http,
            CreateMessage::new().content(honeypot_notice(action)),
        )
        .await
        .is_ok();
    let action_result = match action {
        HoneypotAction::Kick => {
            guild_id
                .kick_with_reason(&context.http, message.author.id, HONEYPOT_AUDIT_REASON)
                .await
        }
        HoneypotAction::Ban => {
            guild_id
                .ban_with_reason(&context.http, message.author.id, 0, HONEYPOT_AUDIT_REASON)
                .await
        }
    };

    core.bot_status.write().await.error = match action_result {
        Ok(()) if dm_delivered => None,
        Ok(()) => Some(
            "Compromised account trap acted, but the Discord DM could not be delivered.".into(),
        ),
        Err(error) => Some(format!("Compromised account trap action failed: {error}")),
    };
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, context: Context, ready: Ready) {
        let config = self.core.config.read().await.clone();
        let (activity, status) = presence_from_config(&config);
        context.set_presence(activity, status);
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

        if let Err(error) =
            Command::set_global_commands(&context.http, vec![relay_command(&config)]).await
        {
            // Non-fatal: the gateway stays connected, so keep the online status.
            self.core.bot_status.write().await.error =
                Some(format!("Command registration failed: {error}"));
        }
    }

    async fn message(&self, context: Context, message: Message) {
        if message.author.bot {
            return;
        }
        let config = self.core.config.read().await.clone();
        let channel_id = message.channel_id.to_string();

        if let Some(action) = honeypot_action_for_channel(&config, &channel_id) {
            enforce_honeypot_action(&self.core, &context, &message, action).await;
            return;
        }

        let role_ids = message_role_ids(&message);
        let scoped_config = privacy::scoped_config_for_roles(&config, &role_ids);

        if !config.music_channel_id.is_empty() && channel_id == config.music_channel_id {
            handle_music_message(&self.core, &context.http, &message, &scoped_config).await;
            return;
        }

        if !config.tts_channel_id.is_empty() && channel_id == config.tts_channel_id {
            let text_report = classify_message_privacy(&message, &scoped_config);
            if block_and_delete_message_if_needed(
                &self.core,
                &context.http,
                &message,
                &text_report,
                &scoped_config,
            )
            .await
            {
                return;
            }
            if privacy::privacy_rules_enabled(&scoped_config) {
                let action = privacy::action_for(&text_report, &scoped_config);
                if matches!(action, privacy::PrivacyAction::Review) {
                    privacy::log_decision(&text_report, action);
                    return;
                }
            }
            let stage_ticket =
                if !message.content.trim().is_empty() || !message.sticker_items.is_empty() {
                    Some(
                        self.core
                            .register_stage_output(
                                message_timestamp(&message),
                                &message.id.to_string(),
                                0,
                                StageLane::Tts,
                            )
                            .await,
                    )
                } else {
                    None
                };
            let author = message_author(&message);
            let guild_tag = guild_tag_from_user(&message.author);
            let mut sticker_segments = Vec::new();
            for sticker in message.sticker_items.iter().take(3) {
                let Some(url) = sticker.image_url() else {
                    continue;
                };
                let (format, _) = sticker_format(sticker.format_type);
                let sticker_text = format!("{}\n{}", message.content, sticker.name);
                let (mut report, _) =
                    inspect_sticker(&url, format, Some(&sticker_text), &scoped_config).await;
                let current_config = self.core.config.read().await.clone();
                let current_scoped_config =
                    privacy::scoped_config_for_roles(&current_config, &role_ids);
                report = reclassify_privacy_report(report, &sticker_text, &current_scoped_config);
                if block_and_delete_message_if_needed(
                    &self.core,
                    &context.http,
                    &message,
                    &report,
                    &current_scoped_config,
                )
                .await
                {
                    if let Some(ticket) = stage_ticket {
                        self.core.cancel_stage_output(ticket).await;
                    }
                    return;
                }
                if privacy::privacy_rules_enabled(&current_scoped_config) {
                    let action = privacy::action_for(&report, &current_scoped_config);
                    if matches!(action, privacy::PrivacyAction::Review) {
                        privacy::log_decision(&report, action);
                        if let Some(ticket) = stage_ticket {
                            self.core.cancel_stage_output(ticket).await;
                        }
                        return;
                    }
                }
                sticker_segments.push(sticker_visual_segment(
                    sticker.name.clone(),
                    Some(url),
                    sticker.format_type,
                ));
            }
            if !sticker_segments.is_empty() {
                let mut segments = match message.content.trim() {
                    "" => Vec::new(),
                    content => parse_visual_segments(content)
                        .unwrap_or_else(|| plain_text_segments(content.into())),
                };
                segments.append(&mut sticker_segments);
                let Some(ticket) = stage_ticket else {
                    return;
                };
                self.core
                    .publish_visual_tts_if_allowed_with_ticket_and_roles(
                        ticket,
                        message.id.to_string(),
                        message.content.clone(),
                        author,
                        guild_tag,
                        message_timestamp(&message),
                        segments,
                        &role_ids,
                    )
                    .await;
                return;
            }
            if let Some(segments) = parse_visual_segments(&message.content) {
                let Some(ticket) = stage_ticket else {
                    return;
                };
                self.core
                    .publish_visual_tts_if_allowed_with_ticket_and_roles(
                        ticket,
                        message.id.to_string(),
                        message.content.clone(),
                        author,
                        guild_tag,
                        message_timestamp(&message),
                        segments,
                        &role_ids,
                    )
                    .await;
                return;
            }
            if let Some(text) = prepare_tts_text(&message.content, config.tts_character_limit) {
                let Some(ticket) = stage_ticket else {
                    return;
                };
                if !config.tts_speech_enabled {
                    self.core
                        .publish_visual_tts_if_allowed_with_ticket_and_roles(
                            ticket,
                            message.id.to_string(),
                            text.clone(),
                            author,
                            guild_tag,
                            message_timestamp(&message),
                            plain_text_segments(text),
                            &role_ids,
                        )
                        .await;
                    return;
                }
                let request = TtsRequest {
                    id: message.id.to_string(),
                    text: text.clone(),
                    author,
                    guild_tag,
                    timestamp: message_timestamp(&message),
                };
                if let Err(error) = self
                    .core
                    .publish_tts_with_ticket_and_roles(ticket, request.clone(), &role_ids)
                    .await
                {
                    let visual_published = self
                        .core
                        .publish_visual_tts_if_allowed_with_ticket_and_roles(
                            ticket,
                            request.id,
                            request.text.clone(),
                            request.author,
                            request.guild_tag,
                            request.timestamp,
                            plain_text_segments(request.text),
                            &role_ids,
                        )
                        .await;
                    self.core.bot_status.write().await.error =
                        tts_failure_status(&error.to_string(), visual_published);
                } else {
                    self.core.bot_status.write().await.error = None;
                }
            } else if let Some(ticket) = stage_ticket {
                self.core.cancel_stage_output(ticket).await;
            }
            return;
        }

        if config.watched_channel_id.is_empty() || channel_id != config.watched_channel_id {
            return;
        }
        let message_report = classify_message_privacy(&message, &scoped_config);
        if block_and_delete_message_if_needed(
            &self.core,
            &context.http,
            &message,
            &message_report,
            &scoped_config,
        )
        .await
        {
            return;
        }
        let media_text = prepare_media_text(&message.content);
        let timestamp = message_timestamp(&message);
        let message_id = message.id.to_string();
        let mut stage_tickets = Vec::new();
        let mut sticker_tickets = Vec::new();
        for (index, sticker) in message.sticker_items.iter().take(3).enumerate() {
            if sticker.image_url().is_none() {
                continue;
            }
            let ticket = self
                .core
                .register_stage_output(timestamp, &message_id, 100 + index as u16, StageLane::Media)
                .await;
            stage_tickets.push(ticket);
            sticker_tickets.push((index, ticket));
        }
        let mut attachment_tickets = Vec::new();
        for (index, _) in message
            .attachments
            .iter()
            .filter_map(|item| classify_attachment(item).map(|kind| (item, kind)))
            .take(3)
            .enumerate()
        {
            let ticket = self
                .core
                .register_stage_output(timestamp, &message_id, 200 + index as u16, StageLane::Media)
                .await;
            stage_tickets.push(ticket);
            attachment_tickets.push(ticket);
        }

        for (index, sticker) in message.sticker_items.iter().take(3).enumerate() {
            let Some(url) = sticker.image_url() else {
                continue;
            };
            let (format, _) = sticker_format(sticker.format_type);
            let stage_ticket = sticker_tickets
                .iter()
                .find_map(|(ticket_index, ticket)| (*ticket_index == index).then_some(*ticket))
                .expect("recognized sticker has a stage ticket");
            let sticker_text = format!("{}\n{}", message.content, sticker.name);
            let (privacy_report, bytes) =
                inspect_sticker(&url, format, Some(&sticker_text), &scoped_config).await;
            let current_config = self.core.config.read().await.clone();
            let current_scoped_config =
                privacy::scoped_config_for_roles(&current_config, &role_ids);
            let privacy_report =
                reclassify_privacy_report(privacy_report, &sticker_text, &current_scoped_config);
            if block_and_delete_message_if_needed(
                &self.core,
                &context.http,
                &message,
                &privacy_report,
                &current_scoped_config,
            )
            .await
            {
                cancel_stage_tickets(&self.core, &stage_tickets).await;
                return;
            }
            self.core
                .submit_sticker_with_ticket_and_roles(
                    stage_ticket,
                    StickerEvent {
                        id: sticker.id.to_string(),
                        name: sticker.name.clone(),
                        format: format.into(),
                        url,
                        cached_media_id: None,
                        author: message_author(&message),
                        timestamp: message_timestamp(&message),
                        message_id: message.id.to_string(),
                    },
                    Some(&sticker_text),
                    bytes,
                    Some(privacy_report),
                    &role_ids,
                )
                .await;
        }

        for attachment in message
            .attachments
            .iter()
            .filter_map(|item| classify_attachment(item).map(|kind| (item, kind)))
            .take(3)
            .enumerate()
        {
            let (index, (attachment, kind)) = attachment;
            let stage_ticket = attachment_tickets[index];
            let mut audio_metadata = if matches!(kind, MediaKind::Audio) {
                artwork::extract(&attachment.url).await.ok()
            } else {
                None
            };
            let mut event = MediaEvent {
                kind,
                url: attachment.url.clone(),
                proxy_url: attachment.proxy_url.clone(),
                filename: attachment.filename.clone(),
                content_type: attachment.content_type.clone().unwrap_or_default(),
                artwork_id: None,
                audio_id: None,
                cached_media_id: None,
                title: audio_metadata
                    .as_ref()
                    .and_then(|metadata| metadata.title.clone()),
                artist: audio_metadata
                    .as_ref()
                    .and_then(|metadata| metadata.artist.clone()),
                text: media_text.clone(),
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
            let attachment_text = format!(
                "{}\n{}\n{}",
                attachment_privacy_text(&message.content, &event.filename),
                event.title.as_deref().unwrap_or_default(),
                event.artist.as_deref().unwrap_or_default()
            );
            let mut privacy_report = if matches!(kind, MediaKind::Image | MediaKind::Gif) {
                if attachment.size as usize > privacy::MAX_IMAGE_BYTES {
                    privacy::image_limit_report(Some(&attachment_text), &scoped_config)
                } else {
                    privacy::analyze_remote_image(
                        &event.url,
                        &event.proxy_url,
                        Some(&attachment_text),
                        &scoped_config,
                    )
                    .await
                }
            } else {
                privacy::classify_text(Some(&attachment_text), &scoped_config)
            };
            if let Some(embedded) = audio_metadata
                .as_ref()
                .and_then(|metadata| metadata.artwork.as_ref())
            {
                privacy_report.merge(
                    privacy::analyze_image_bytes_async(
                        &embedded.bytes,
                        Some(&attachment_text),
                        &scoped_config,
                    )
                    .await,
                );
            }
            let current_config = self.core.config.read().await.clone();
            let current_scoped_config =
                privacy::scoped_config_for_roles(&current_config, &role_ids);
            let privacy_report =
                reclassify_privacy_report(privacy_report, &attachment_text, &current_scoped_config);
            if block_and_delete_message_if_needed(
                &self.core,
                &context.http,
                &message,
                &privacy_report,
                &current_scoped_config,
            )
            .await
            {
                cancel_stage_tickets(&self.core, &stage_tickets).await;
                return;
            }
            if let Some(embedded) = audio_metadata
                .as_mut()
                .and_then(|metadata| metadata.artwork.take())
            {
                let id = attachment.id.to_string();
                self.core.cache_artwork(id.clone(), embedded).await;
                event.artwork_id = Some(id);
            }
            if let Some(metadata) = audio_metadata.as_mut() {
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
                event.audio_id = Some(id);
            }
            self.core
                .submit_analyzed_media_with_ticket_and_roles(
                    stage_ticket,
                    event,
                    Some(privacy_report),
                    Some(&message.content),
                    &role_ids,
                )
                .await;
        }

        submit_embedded_gifs(&self.core, &context.http, &message).await;
    }

    async fn message_update(
        &self,
        context: Context,
        _old: Option<Message>,
        new: Option<Message>,
        event: MessageUpdateEvent,
    ) {
        if let Some(message) = new {
            submit_embedded_gifs(&self.core, &context.http, &message).await;
            return;
        }
        let Some(embeds) = event.embeds else {
            return;
        };
        let watched_channel_id = self.core.config.read().await.watched_channel_id.clone();
        if watched_channel_id.is_empty() || event.channel_id.to_string() != watched_channel_id {
            return;
        }
        if let Ok(message) = context.http.get_message(event.channel_id, event.id).await {
            submit_embedded_gifs(&self.core, &context.http, &message).await;
            return;
        }
        if let Some(author) = event.author {
            let message = DeferredEmbedMessage {
                channel_id: event.channel_id.to_string(),
                message_id: event.id.to_string(),
                author,
                timestamp: event
                    .timestamp
                    .map(|timestamp| timestamp.unix_timestamp().max(0) as u64 * 1_000)
                    .unwrap_or_else(current_timestamp_ms),
                content: event.content.unwrap_or_default(),
                embeds,
                role_ids: Vec::new(),
            };
            submit_deferred_embeds(&self.core, &context.http, message).await;
            return;
        }
    }

    async fn interaction_create(&self, context: Context, interaction: Interaction) {
        let command = match interaction {
            Interaction::Component(component) => {
                if component.data.custom_id.starts_with(MUSIC_COMPONENT_PREFIX) {
                    handle_music_component(&self.core, &context, &component).await;
                } else {
                    custom_commands::handle_custom_component(&self.core, &context, &component)
                        .await;
                }
                return;
            }
            Interaction::Modal(modal) => {
                if modal.data.custom_id.starts_with(MUSIC_CUSTOM_PREFIX) {
                    handle_music_custom_modal(&self.core, &context, &modal).await;
                }
                return;
            }
            Interaction::Command(command) => command,
            _ => return,
        };
        if command.data.name != "relay" {
            return;
        }

        if let Some(response) =
            custom_commands::handle_custom_command(&self.core, &context, &command).await
        {
            if command
                .create_response(
                    &context.http,
                    CreateInteractionResponse::Message(response.into_message()),
                )
                .await
                .is_err()
            {
                self.core.bot_status.write().await.error =
                    Some("Discord did not accept a custom command response.".into());
            }
            return;
        }

        // Changelog fetches GitHub and posts embeds — defer so Discord does not time out.
        let defer_changelog = command
            .data
            .options
            .first()
            .is_some_and(|option| option.name == "changelog");
        if defer_changelog {
            let defer = CreateInteractionResponse::Defer(
                CreateInteractionResponseMessage::new().ephemeral(true),
            );
            if let Err(error) = command.create_response(&context.http, defer).await {
                self.core.bot_status.write().await.error =
                    Some(format!("Discord response failed: {error}"));
                return;
            }
            let content = handle_relay(&self.core, &context.http, &command)
                .await
                .unwrap_or_else(|error| format!("Unable to post the changelog: {error:#}"));
            let edit = EditInteractionResponse::new()
                .content(content)
                .allowed_mentions(CreateAllowedMentions::new());
            if let Err(error) = command.edit_response(&context.http, edit).await {
                self.core.bot_status.write().await.error =
                    Some(format!("Discord response failed: {error}"));
            }
            return;
        }

        let content = handle_relay(&self.core, &context.http, &command)
            .await
            .unwrap_or_else(|error| format!("Unable to update Relay: {error:#}"));
        let response = CreateInteractionResponse::Message(
            CreateInteractionResponseMessage::new()
                .content(content)
                .ephemeral(true)
                .allowed_mentions(CreateAllowedMentions::new()),
        );
        if let Err(error) = command.create_response(&context.http, response).await {
            // Non-fatal: the gateway stays connected, so keep the online status.
            self.core.bot_status.write().await.error =
                Some(format!("Discord response failed: {error}"));
        }
    }
}

async fn handle_music_message(
    core: &Arc<AppCore>,
    http: &Arc<Http>,
    message: &Message,
    scoped_config: &AppConfig,
) {
    let query = message.content.trim();
    if query.is_empty() || query.chars().any(char::is_control) {
        return;
    }
    let text_report = classify_message_privacy(message, scoped_config);
    if block_and_delete_message_if_needed(core, http, message, &text_report, scoped_config).await {
        return;
    }
    if privacy::privacy_rules_enabled(scoped_config)
        && matches!(
            privacy::action_for(&text_report, scoped_config),
            privacy::PrivacyAction::Review
        )
    {
        return;
    }

    let strings = music_locale(core).await;
    let api_key = match load_youtube_api_key() {
        Ok(Some(api_key)) => api_key,
        Ok(None) => {
            let _ = message.channel_id.say(http, strings.not_configured).await;
            return;
        }
        Err(_) => {
            core.bot_status.write().await.error =
                Some("Unable to read the saved YouTube API key.".into());
            let _ = message
                .channel_id
                .say(http, strings.search_unavailable)
                .await;
            return;
        }
    };

    let user_id = message.author.id.get();
    {
        let mut music = core.music.lock().await;
        let now = Instant::now();
        if let Some(remaining) = music.search_cooldown_remaining(user_id, now) {
            let seconds = cooldown_wait_seconds(remaining).to_string();
            let reply = music_i18n::fill(strings.search_cooldown, &[("seconds", &seconds)]);
            let _ = message.channel_id.say(http, reply).await;
            return;
        }
        // Mark before the API call so failed/retried spam still burns the cooldown.
        music.mark_search_attempt(user_id, now);
    }

    let results = match youtube::search(query, &api_key).await {
        Ok(results) => results,
        Err(error) => {
            let detail = error.to_string();
            core.bot_status.write().await.error = Some(detail.clone());
            let reply = if detail.to_ascii_lowercase().contains("quota") {
                "YouTube API quota exceeded for today. Try again after the daily reset, or raise the quota in Google Cloud."
            } else {
                strings.search_unavailable
            };
            let _ = message.channel_id.say(http, reply).await;
            return;
        }
    };
    if results.is_empty() {
        let _ = message.channel_id.say(http, strings.no_results).await;
        return;
    }

    let query = query.chars().take(200).collect::<String>();
    let search_id = core.music.lock().await.insert_search(
        message.author.id.get(),
        message.channel_id.get(),
        query.clone(),
        results.clone(),
    );
    let mut embed = CreateEmbed::new()
        .title(strings.results_title)
        .description(music_i18n::fill(
            strings.choose_title,
            &[("query", &truncate_text(&query, 180))],
        ));
    let mut options = Vec::with_capacity(results.len());
    for (index, track) in results.iter().enumerate() {
        embed = embed.field(
            format!("{}. {}", index + 1, truncate_text(&track.title, 180)),
            format!(
                "{} · {}",
                truncate_text(&track.channel_title, 80),
                format_duration(track.duration_seconds)
            ),
            false,
        );
        options.push(
            CreateSelectMenuOption::new(truncate_text(&track.title, 100), track.video_id.clone())
                .description(format!(
                    "{} · {}",
                    truncate_text(&track.channel_title, 80),
                    format_duration(track.duration_seconds)
                )),
        );
    }
    let message_builder = CreateMessage::new()
        .embed(embed)
        .components(vec![CreateActionRow::SelectMenu(
            CreateSelectMenu::new(
                format!("{MUSIC_SEARCH_PREFIX}{search_id}"),
                CreateSelectMenuKind::String { options },
            )
            .placeholder(strings.choose_placeholder),
        )])
        .allowed_mentions(CreateAllowedMentions::new());
    if message
        .channel_id
        .send_message(http, message_builder)
        .await
        .is_err()
    {
        core.bot_status.write().await.error =
            Some("Discord rejected the YouTube search results.".into());
    }
}

async fn handle_music_component(
    core: &Arc<AppCore>,
    context: &Context,
    component: &ComponentInteraction,
) {
    let strings = music_locale(core).await;
    let custom_id = component.data.custom_id.as_str();
    if let Some(search_id) = custom_id.strip_prefix(MUSIC_SEARCH_PREFIX) {
        let Some(video_id) = (match &component.data.kind {
            ComponentInteractionDataKind::StringSelect { values } => values.first().cloned(),
            _ => None,
        }) else {
            respond_music_component(
                core,
                context,
                component,
                strings.nothing_selected,
                Vec::new(),
            )
            .await;
            return;
        };
        let selection = core.music.lock().await.select_search(
            search_id,
            component.user.id.get(),
            &music_requester_name(&component.user),
            &video_id,
        );
        match selection {
            SearchSelection::Selected(selection_id) => {
                let duration_seconds = core
                    .music
                    .lock()
                    .await
                    .selection_duration_seconds(&selection_id)
                    .unwrap_or(0);
                let preview_seconds = duration_seconds.min(30);
                let preview_end = format_duration(preview_seconds);
                let full_end = format_duration(duration_seconds);
                let content = format!(
                    "{}\n{}\n{}\n{}\n{}",
                    strings.choose_mode,
                    music_i18n::fill(strings.preview_bullet, &[("end", &preview_end)]),
                    music_i18n::fill(strings.full_bullet, &[("end", &full_end)]),
                    strings.custom_bullet,
                    strings.owner_only_action,
                );
                respond_music_component(
                    core,
                    context,
                    component,
                    &content,
                    music_mode_components(&selection_id, duration_seconds, &strings),
                )
                .await;
            }
            SearchSelection::NotOwner => {
                respond_music_component(
                    core,
                    context,
                    component,
                    strings.search_not_owner,
                    Vec::new(),
                )
                .await;
            }
            SearchSelection::NotFound | SearchSelection::InvalidVideo => {
                respond_music_component(
                    core,
                    context,
                    component,
                    strings.search_expired,
                    Vec::new(),
                )
                .await;
            }
        }
        return;
    }

    if let Some(rest) = custom_id.strip_prefix(MUSIC_MODE_PREFIX) {
        let Some((selection_id, mode_name)) = rest.rsplit_once(':') else {
            return;
        };
        if mode_name == "cancel" {
            let result = core
                .music
                .lock()
                .await
                .cancel_selection(selection_id, component.user.id.get());
            let content = match result {
                SelectionTake::Taken(_) => strings.selection_cancelled,
                SelectionTake::NotOwner => strings.selection_not_owner,
                SelectionTake::NotFound => strings.selection_expired,
            };
            respond_music_component(core, context, component, content, Vec::new()).await;
            return;
        }
        if mode_name == "custom" {
            let access = core
                .music
                .lock()
                .await
                .peek_selection(selection_id, component.user.id.get());
            match access {
                SelectionTake::Taken(_) => {
                    let _ = core
                        .music
                        .lock()
                        .await
                        .touch_selection(selection_id, component.user.id.get());
                    let modal = CreateModal::new(
                        format!("{MUSIC_CUSTOM_PREFIX}{selection_id}"),
                        truncate_text(strings.custom_modal_title, 45),
                    )
                    .components(vec![
                        CreateActionRow::InputText(
                            CreateInputText::new(
                                InputTextStyle::Short,
                                truncate_text(strings.custom_start_label, 45),
                                MUSIC_CUSTOM_START_ID,
                            )
                            .placeholder(truncate_text(strings.custom_start_placeholder, 100))
                            .required(true)
                            .max_length(12),
                        ),
                        CreateActionRow::InputText(
                            CreateInputText::new(
                                InputTextStyle::Short,
                                truncate_text(strings.custom_end_label, 45),
                                MUSIC_CUSTOM_END_ID,
                            )
                            .placeholder(truncate_text(strings.custom_end_placeholder, 100))
                            .required(true)
                            .max_length(12),
                        ),
                    ]);
                    if component
                        .create_response(&context.http, CreateInteractionResponse::Modal(modal))
                        .await
                        .is_err()
                    {
                        core.bot_status.write().await.error =
                            Some("Discord rejected the custom clip modal.".into());
                    }
                }
                SelectionTake::NotOwner => {
                    respond_music_component(
                        core,
                        context,
                        component,
                        strings.selection_not_owner,
                        Vec::new(),
                    )
                    .await;
                }
                SelectionTake::NotFound => {
                    respond_music_component(
                        core,
                        context,
                        component,
                        strings.selection_expired,
                        Vec::new(),
                    )
                    .await;
                }
            }
            return;
        }
        let mode = match mode_name {
            "preview" => MusicPlaybackMode::Preview,
            "full" => MusicPlaybackMode::Full,
            _ => return,
        };
        let selection = match core
            .music
            .lock()
            .await
            .take_selection(selection_id, component.user.id.get())
        {
            SelectionTake::Taken(selection) => selection,
            SelectionTake::NotOwner => {
                respond_music_component(
                    core,
                    context,
                    component,
                    strings.selection_not_owner,
                    Vec::new(),
                )
                .await;
                return;
            }
            SelectionTake::NotFound => {
                respond_music_component(
                    core,
                    context,
                    component,
                    strings.selection_expired,
                    Vec::new(),
                )
                .await;
                return;
            }
        };
        let result = core
            .start_music(
                selection.clone(),
                mode,
                current_timestamp_ms(),
                &component.id.to_string(),
            )
            .await;
        match &result {
            MusicStartResult::QueueFull => {
                core.music
                    .lock()
                    .await
                    .restore_selection(selection_id, selection.clone());
                respond_music_component(core, context, component, strings.queue_full, Vec::new())
                    .await;
            }
            MusicStartResult::Started(_) | MusicStartResult::Queued { .. } => {
                announce_music_playback(
                    core,
                    context,
                    &selection,
                    &result,
                    &strings,
                    Some(component),
                )
                .await;
            }
        }
        return;
    }

    if let Some(playback_id) = custom_id.strip_prefix(MUSIC_SKIP_PREFIX) {
        let decision = {
            let music = core.music.lock().await;
            music.skip_decision(playback_id, component.user.id.get())
        };
        match decision {
            MusicSkipDecision::Allowed => {}
            MusicSkipDecision::NotOwner => {
                respond_music_component(
                    core,
                    context,
                    component,
                    strings.skip_not_owner,
                    Vec::new(),
                )
                .await;
                return;
            }
            MusicSkipDecision::NotCurrent => {
                // Server state may already be cleared (e.g. a failing OBS embed
                // reported musicEnded) while the Windows widget still plays.
                // Acknowledge before delete — UpdateMessage would fail once gone.
                let _ = component
                    .create_response(&context.http, CreateInteractionResponse::Acknowledge)
                    .await;
                core.force_stop_music(playback_id).await;
                if component.message.delete(&context.http).await.is_err() {
                    // Soft-fail: may already be deleted or missing Manage Messages.
                }
                return;
            }
        }
        // Acknowledge the Skip click first, then stop+delete the announce.
        let _ = component
            .create_response(&context.http, CreateInteractionResponse::Acknowledge)
            .await;
        if core.stop_music_if_current(playback_id).await.is_none() {
            core.force_stop_music(playback_id).await;
            let _ = component.message.delete(&context.http).await;
        }
    }
}

async fn respond_music_component(
    core: &Arc<AppCore>,
    context: &Context,
    component: &ComponentInteraction,
    content: &str,
    components: Vec<CreateActionRow>,
) {
    let response = CreateInteractionResponse::Message(
        CreateInteractionResponseMessage::new()
            .content(content)
            .ephemeral(true)
            .components(components)
            .allowed_mentions(CreateAllowedMentions::new()),
    );
    if component
        .create_response(&context.http, response)
        .await
        .is_err()
    {
        core.bot_status.write().await.error =
            Some("Discord rejected the music interaction.".into());
    }
}

async fn music_locale(core: &AppCore) -> MusicStrings {
    let language = core.interface_preferences.read().await.language.clone();
    music_i18n::music_strings_for_language(&language)
}

fn music_mode_components(
    selection_id: &str,
    duration_seconds: u64,
    strings: &MusicStrings,
) -> Vec<CreateActionRow> {
    let preview_seconds = duration_seconds.min(30);
    let preview_end = format_duration(preview_seconds);
    let full_duration = format_duration(duration_seconds);
    let preview_label = music_i18n::fill(strings.preview_button, &[("end", &preview_end)]);
    let full_label = music_i18n::fill(strings.full_button, &[("duration", &full_duration)]);
    vec![CreateActionRow::Buttons(vec![
        CreateButton::new(format!("{MUSIC_MODE_PREFIX}{selection_id}:preview"))
            .label(truncate_text(&preview_label, 80))
            .style(ButtonStyle::Primary),
        CreateButton::new(format!("{MUSIC_MODE_PREFIX}{selection_id}:full"))
            .label(truncate_text(&full_label, 80))
            .style(ButtonStyle::Secondary),
        CreateButton::new(format!("{MUSIC_MODE_PREFIX}{selection_id}:custom"))
            .label(truncate_text(strings.custom_button, 80))
            .style(ButtonStyle::Secondary),
        CreateButton::new(format!("{MUSIC_MODE_PREFIX}{selection_id}:cancel"))
            .label(truncate_text(strings.cancel, 80))
            .style(ButtonStyle::Danger),
    ])]
}

async fn handle_music_custom_modal(
    core: &Arc<AppCore>,
    context: &Context,
    modal: &ModalInteraction,
) {
    let strings = music_locale(core).await;
    let Some(selection_id) = modal.data.custom_id.strip_prefix(MUSIC_CUSTOM_PREFIX) else {
        return;
    };
    let start_raw = modal_input_value(modal, MUSIC_CUSTOM_START_ID).unwrap_or_default();
    let end_raw = modal_input_value(modal, MUSIC_CUSTOM_END_ID).unwrap_or_default();
    let (Some(start_seconds), Some(end_seconds)) =
        (parse_timestamp(&start_raw), parse_timestamp(&end_raw))
    else {
        respond_music_modal(core, context, modal, strings.custom_invalid).await;
        return;
    };

    let selection = match core
        .music
        .lock()
        .await
        .take_selection(selection_id, modal.user.id.get())
    {
        SelectionTake::Taken(selection) => selection,
        SelectionTake::NotOwner => {
            respond_music_modal(core, context, modal, strings.selection_not_owner).await;
            return;
        }
        SelectionTake::NotFound => {
            respond_music_modal(core, context, modal, strings.selection_expired).await;
            return;
        }
    };

    let result = match core
        .start_music_custom(
            selection.clone(),
            start_seconds,
            end_seconds,
            current_timestamp_ms(),
            &modal.id.to_string(),
        )
        .await
    {
        Ok(result) => result,
        Err(_) => {
            core.music
                .lock()
                .await
                .restore_selection(selection_id, selection);
            respond_music_modal(core, context, modal, strings.custom_invalid).await;
            return;
        }
    };

    match &result {
        MusicStartResult::QueueFull => {
            core.music
                .lock()
                .await
                .restore_selection(selection_id, selection.clone());
            respond_music_modal(core, context, modal, strings.queue_full).await;
        }
        MusicStartResult::Started(playback) | MusicStartResult::Queued { playback, .. } => {
            announce_music_playback(core, context, &selection, &result, &strings, None).await;
            let range = music_playback_range_label(playback);
            let started_title = truncate_text(&playback.title, 140);
            let user = truncate_text(&playback.requested_by, 40);
            let content = match &result {
                MusicStartResult::Queued { position, .. } => music_i18n::fill(
                    strings.playback_queued,
                    &[
                        ("title", &started_title),
                        ("position", &position.to_string()),
                        ("user", &user),
                    ],
                ),
                _ => music_i18n::fill(
                    strings.playback_started,
                    &[
                        ("title", &started_title),
                        ("range", &range),
                        ("user", &user),
                    ],
                ),
            };
            respond_music_modal(core, context, modal, &content).await;
        }
    }
}

fn modal_input_value(modal: &ModalInteraction, custom_id: &str) -> Option<String> {
    for row in &modal.data.components {
        for component in &row.components {
            if let ActionRowComponent::InputText(input) = component
                && input.custom_id == custom_id
            {
                return input.value.clone();
            }
        }
    }
    None
}

async fn announce_music_playback(
    core: &Arc<AppCore>,
    context: &Context,
    selection: &MusicSelection,
    result: &MusicStartResult,
    strings: &MusicStrings,
    component: Option<&ComponentInteraction>,
) {
    let (playback, queued_position) = match result {
        MusicStartResult::Started(playback) => (playback, None),
        MusicStartResult::Queued { playback, position } => (playback, Some(*position)),
        MusicStartResult::QueueFull => return,
    };
    let range = music_playback_range_label(playback);
    let title = truncate_text(&playback.title, 160);
    let channel = truncate_text(&playback.channel_title, 60);
    let user = truncate_text(&playback.requested_by, 40);

    if let Some(position) = queued_position {
        let position_label = position.to_string();
        let queued = CreateMessage::new()
            .content(music_i18n::fill(
                strings.playback_queued,
                &[
                    ("title", &title),
                    ("position", &position_label),
                    ("user", &user),
                ],
            ))
            .allowed_mentions(CreateAllowedMentions::new());
        let _ = ChannelId::new(selection.channel_id)
            .send_message(&context.http, queued)
            .await;
        if let Some(component) = component {
            let queued_title = truncate_text(&playback.title, 140);
            respond_music_component(
                core,
                context,
                component,
                &music_i18n::fill(
                    strings.playback_queued,
                    &[
                        ("title", &queued_title),
                        ("position", &position_label),
                        ("user", &user),
                    ],
                ),
                Vec::new(),
            )
            .await;
        }
        return;
    }

    let now_playing = CreateMessage::new()
        .content(music_i18n::fill(
            strings.now_playing,
            &[
                ("title", &title),
                ("channel", &channel),
                ("range", &range),
                ("user", &user),
            ],
        ))
        .components(music_skip_components(playback, strings))
        .allowed_mentions(CreateAllowedMentions::new());
    if let Ok(now_playing_message) = ChannelId::new(selection.channel_id)
        .send_message(&context.http, now_playing)
        .await
    {
        core.music
            .lock()
            .await
            .set_now_playing_message_id(&playback.playback_id, now_playing_message.id.get());
    }
    if let Some(component) = component {
        let started_title = truncate_text(&playback.title, 140);
        respond_music_component(
            core,
            context,
            component,
            &music_i18n::fill(
                strings.playback_started,
                &[
                    ("title", &started_title),
                    ("range", &range),
                    ("user", &user),
                ],
            ),
            Vec::new(),
        )
        .await;
    }
}

async fn respond_music_modal(
    core: &Arc<AppCore>,
    context: &Context,
    modal: &ModalInteraction,
    content: &str,
) {
    let response = CreateInteractionResponse::Message(
        CreateInteractionResponseMessage::new()
            .content(content)
            .ephemeral(true)
            .allowed_mentions(CreateAllowedMentions::new()),
    );
    if modal
        .create_response(&context.http, response)
        .await
        .is_err()
    {
        core.bot_status.write().await.error =
            Some("Discord rejected the music modal response.".into());
    }
}

fn music_requester_name(user: &User) -> String {
    user.global_name
        .clone()
        .filter(|name| !name.trim().is_empty())
        .unwrap_or_else(|| user.name.clone())
}

fn music_playback_range_label(playback: &MusicPlaybackEvent) -> String {
    let start = format_duration(playback.start_seconds);
    let end = playback
        .end_seconds
        .map(format_duration)
        .unwrap_or_else(|| format_duration(playback.duration_seconds));
    format!("{start}→{end}")
}

fn music_skip_components(
    playback: &MusicPlaybackEvent,
    strings: &MusicStrings,
) -> Vec<CreateActionRow> {
    vec![CreateActionRow::Buttons(vec![
        CreateButton::new(format!("{MUSIC_SKIP_PREFIX}{}", playback.playback_id))
            .label(truncate_text(strings.skip, 80))
            .style(ButtonStyle::Secondary),
    ])]
}

fn truncate_text(value: &str, max_chars: usize) -> String {
    let mut value = value.chars().take(max_chars).collect::<String>();
    if value.chars().count() == max_chars && value.chars().count() < value.len() {
        value.push('…');
    }
    value
}

fn format_duration(seconds: u64) -> String {
    format!("{}:{:02}", seconds / 60, seconds % 60)
}

fn bounded_privacy_text(value: &str) -> String {
    let value = value.trim();
    value
        .char_indices()
        .nth(privacy::PRIVACY_TEXT_LIMIT)
        .map_or_else(|| value.to_owned(), |(index, _)| value[..index].to_owned())
}

fn message_role_ids(message: &Message) -> Vec<String> {
    message
        .member
        .as_ref()
        .map(|member| member.roles.iter().map(ToString::to_string).collect())
        .unwrap_or_default()
}

fn classify_message_privacy(message: &Message, config: &AppConfig) -> privacy::PrivacyReport {
    classify_privacy_values(
        &message.content,
        message
            .sticker_items
            .iter()
            .take(3)
            .map(|sticker| sticker.name.as_str()),
        message
            .attachments
            .iter()
            .take(3)
            .map(|attachment| attachment.filename.as_str()),
        config,
    )
}

fn classify_privacy_values<'a>(
    content: &str,
    sticker_names: impl IntoIterator<Item = &'a str>,
    attachment_names: impl IntoIterator<Item = &'a str>,
    config: &AppConfig,
) -> privacy::PrivacyReport {
    if !privacy::privacy_rules_enabled(config) {
        return privacy::PrivacyReport::safe();
    }

    let mut report = privacy::PrivacyReport::safe();
    let mut classify = |value: &str| {
        let value = bounded_privacy_text(value);
        if !value.is_empty() {
            report.merge(privacy::classify_text(Some(&value), config));
        }
    };
    classify(content);
    for sticker_name in sticker_names.into_iter().take(3) {
        classify(sticker_name);
    }
    for attachment_name in attachment_names.into_iter().take(3) {
        classify(attachment_name);
        if let Some((stem, _extension)) = attachment_name.rsplit_once('.') {
            classify(stem);
        }
    }
    report.apply_score_policy(config);
    report
}

fn attachment_privacy_text(content: &str, filename: &str) -> String {
    let content = bounded_privacy_text(content);
    let filename = bounded_privacy_text(filename);
    if content.is_empty() {
        filename
    } else if filename.is_empty() {
        content
    } else {
        format!("{content}\n{filename}")
    }
}

fn reclassify_privacy_report(
    report: privacy::PrivacyReport,
    text: &str,
    config: &AppConfig,
) -> privacy::PrivacyReport {
    let signature = privacy::config_signature(config);
    if report
        .config_signature
        .is_some_and(|report_signature| report_signature != signature)
    {
        let mut current = privacy::classify_text(Some(text), config);
        if current.classification == privacy::PrivacyClassification::Safe {
            current.merge(privacy::PrivacyReport::suspicious("scan_config_changed"));
        }
        current.apply_score_policy(config);
        return current;
    }

    let mut current = report;
    current.merge(privacy::classify_text(Some(text), config));
    current.apply_score_policy(config);
    current
}

fn privacy_action_is_blocked(report: &privacy::PrivacyReport, config: &AppConfig) -> bool {
    if !privacy::privacy_rules_enabled(config) {
        return false;
    }
    let action = privacy::action_for(report, config);
    if matches!(action, privacy::PrivacyAction::Block) {
        privacy::log_decision(report, action);
        true
    } else {
        false
    }
}

fn should_auto_delete_blocked_message(report: &privacy::PrivacyReport, config: &AppConfig) -> bool {
    config.privacy_auto_delete_blocked_messages
        && matches!(
            privacy::action_for(report, config),
            privacy::PrivacyAction::Block
        )
        && report
            .categories
            .iter()
            .any(|category| !matches!(category, privacy::PrivacyCategory::MediaSafety))
}

async fn block_and_delete_message_if_needed(
    core: &Arc<AppCore>,
    http: &Http,
    message: &Message,
    report: &privacy::PrivacyReport,
    config: &AppConfig,
) -> bool {
    if !privacy_action_is_blocked(report, config) {
        return false;
    }
    if should_auto_delete_blocked_message(report, config)
        && message
            .channel_id
            .delete_message(http, message.id)
            .await
            .is_err()
    {
        core.bot_status.write().await.error =
            Some("Privacy deletion failed. Verify Manage Messages in this channel.".into());
    }
    true
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

fn guild_tag_from_user(user: &User) -> Option<GuildTagIdentity> {
    let primary_guild = user.primary_guild.as_ref()?;
    if primary_guild.identity_enabled != Some(true) {
        return None;
    }
    let name = primary_guild.tag.as_deref()?.trim();
    if name.is_empty() {
        return None;
    }
    Some(GuildTagIdentity {
        name: name.into(),
        badge_url: primary_guild.badge_url(),
    })
}

fn message_timestamp(message: &Message) -> u64 {
    message.timestamp.unix_timestamp().max(0) as u64 * 1_000
}

async fn cancel_stage_tickets(core: &AppCore, tickets: &[StageTicket]) {
    for ticket in tickets {
        core.cancel_stage_output(*ticket).await;
    }
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

async fn inspect_sticker(
    url: &str,
    format: &str,
    text: Option<&str>,
    config: &AppConfig,
) -> (privacy::PrivacyReport, Option<Vec<u8>>) {
    let visual_bytes = matches!(format, "png" | "apng" | "gif");
    let bytes = if visual_bytes {
        artwork::download_bounded(url, artwork::MAX_ARTWORK_BYTES)
            .await
            .ok()
    } else {
        None
    };
    if !config.privacy_scan_enabled {
        return (privacy::classify_text(text, config), bytes);
    }
    let mut report = match bytes.as_deref() {
        Some(bytes) => privacy::analyze_image_bytes_async(bytes, text, config).await,
        None => privacy::classify_text(text, config),
    };
    if !visual_bytes || bytes.is_none() {
        report.merge(privacy::PrivacyReport::suspicious("scan_incomplete"));
    }
    report.apply_score_policy(config);
    (report, bytes)
}

fn sticker_visual_segment(
    name: String,
    url: Option<String>,
    format: StickerFormatType,
) -> VisualSegment {
    VisualSegment {
        kind: "sticker".into(),
        value: name,
        url: url.filter(|_| {
            matches!(
                format,
                StickerFormatType::Png | StickerFormatType::Apng | StickerFormatType::Gif
            )
        }),
        animated: matches!(format, StickerFormatType::Apng | StickerFormatType::Gif),
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

        let character = remainder
            .chars()
            .next()
            .expect("cursor is on a character boundary");
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
    let body = token
        .strip_prefix(if animated { "<a:" } else { "<:" })?
        .strip_suffix('>')?;
    let (name, id) = body.rsplit_once(':')?;
    if name.is_empty() || id.len() > 20 || !id.chars().all(|character| character.is_ascii_digit()) {
        return None;
    }
    let url = format!("https://cdn.discordapp.com/emojis/{id}.webp?size=128&animated={animated}");
    Some((token.len(), format!(":{name}:"), url, animated))
}

fn plain_text_segments(text: String) -> Vec<VisualSegment> {
    vec![VisualSegment {
        kind: "text".into(),
        value: text,
        url: None,
        animated: false,
    }]
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
    let Some((credentials, _source)) = load_discord_credentials()? else {
        stop_bot(&core).await;
        *core.bot_status.write().await = BotStatus::default();
        return Ok(false);
    };

    let intents =
        GatewayIntents::GUILDS | GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;
    // Build the new client before stopping the running bot, so a build
    // failure leaves the current connection untouched.
    let mut client = Client::builder(&credentials.token, intents)
        .event_handler(Handler { core: core.clone() })
        .await
        .context("failed to create the Discord client")?;
    stop_bot(&core).await;
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
        let runtime = runtime.as_ref().context("the Discord bot is not running")?;
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

pub async fn apply_bot_presence(core: &Arc<AppCore>, config: &AppConfig) {
    let shard_manager = {
        let runtime = core.bot_runtime.lock().await;
        runtime
            .as_ref()
            .map(|runtime| Arc::clone(&runtime.shard_manager))
    };
    let Some(shard_manager) = shard_manager else {
        return;
    };
    let messengers = shard_manager
        .runners
        .lock()
        .await
        .values()
        .map(|runner| runner.runner_tx.clone())
        .collect::<Vec<_>>();
    let (activity, status) = presence_from_config(config);
    for messenger in messengers {
        messenger.set_presence(activity.clone(), status);
    }
}

fn presence_from_config(config: &AppConfig) -> (Option<ActivityData>, OnlineStatus) {
    let status = match config.bot_online_status.as_str() {
        "idle" => OnlineStatus::Idle,
        "dnd" => OnlineStatus::DoNotDisturb,
        "invisible" => OnlineStatus::Invisible,
        _ => OnlineStatus::Online,
    };
    let text = config.bot_activity_text.trim();
    let activity = if text.is_empty() || config.bot_activity_type == "none" {
        None
    } else {
        Some(match config.bot_activity_type.as_str() {
            "playing" => ActivityData::playing(text),
            "listening" => ActivityData::listening(text),
            "watching" => ActivityData::watching(text),
            "competing" => ActivityData::competing(text),
            _ => ActivityData::custom(text),
        })
    };
    (activity, status)
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

const MISSING_CHANNEL_WARNING: &str = "A selected Discord channel is private or inaccessible. Add Relay or its role to the channel permissions.";

async fn warn_if_watched_channel_missing(core: &Arc<AppCore>) {
    let configured_channels = {
        let config = core.config.read().await;
        [
            config.watched_channel_id.clone(),
            config.tts_channel_id.clone(),
            config.music_channel_id.clone(),
        ]
    };
    let available_channels = core.channels.read().await;
    let missing = configured_channels.iter().any(|configured_channel| {
        !configured_channel.is_empty()
            && !available_channels
                .iter()
                .any(|channel| channel.id == *configured_channel)
    });
    let mut status = core.bot_status.write().await;
    if missing {
        status.error = Some(MISSING_CHANNEL_WARNING.into());
    } else if status.error.as_deref() == Some(MISSING_CHANNEL_WARNING) {
        status.error = None;
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

pub fn invite_url(client_id: &str, config: &AppConfig) -> String {
    let permissions = (Permissions::VIEW_CHANNEL
        | Permissions::READ_MESSAGE_HISTORY
        | Permissions::MANAGE_CHANNELS
        | Permissions::MANAGE_ROLES
        | Permissions::MANAGE_MESSAGES
        | custom_commands::required_bot_permissions(&config.custom_commands))
    .bits();
    format!(
        "https://discord.com/oauth2/authorize?client_id={client_id}&permissions={permissions}&scope=bot%20applications.commands"
    )
}

fn relay_command(config: &AppConfig) -> CreateCommand {
    let mut command = CreateCommand::new("relay")
        .description("Configure the local OBS media relay")
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
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "nuke",
                "Recreate a channel to delete all of its messages",
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::Channel,
                    "channel",
                    "Channel to recreate",
                )
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
            "status",
            "Show live Relay, OBS, queue, and widget status",
        ))
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "test",
                "Send a local test to one connected output",
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    "output",
                    "Output to test locally",
                )
                .add_string_choice("Media", "visual")
                .add_string_choice("Audio", "audio")
                .add_string_choice("TTS", "tts")
                .add_string_choice("Notification", "notification")
                .add_string_choice("Sticker", "sticker")
                .required(true),
            ),
        )
        .add_option(CreateCommandOption::new(
            CommandOptionType::SubCommand,
            "regenerate",
            "Reconnect local relay outputs without changing their URLs",
        ))
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "clear",
                "Delete a chosen number of messages from one channel",
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::Channel,
                    "channel",
                    "Channel whose messages will be deleted",
                )
                .channel_types(vec![ChannelType::Text, ChannelType::News])
                .required(true),
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::Integer,
                    "count",
                    "Number of messages to delete (1-1000)",
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
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "changelog",
                "Post the latest Relay release notes from GitHub",
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::Channel,
                    "channel",
                    "Channel that receives the release notes",
                )
                .channel_types(vec![ChannelType::Text, ChannelType::News])
                .required(true),
            ),
        );
    for custom in config
        .custom_commands
        .iter()
        .filter(|command| command.enabled)
    {
        command = command.add_option(custom.command_option());
    }
    command
}

pub async fn sync_relay_command_schema(core: &Arc<AppCore>, config: &AppConfig) -> Result<()> {
    let http = {
        let runtime = core.bot_runtime.lock().await;
        runtime
            .as_ref()
            .map(|runtime| runtime.http.clone())
            .context("the Discord bot is not running")?
    };
    if !core.bot_status.read().await.connected {
        bail!("the Discord bot is not connected");
    }
    Command::set_global_commands(&http, vec![relay_command(config)])
        .await
        .context("failed to synchronize the Relay command schema")?;
    Ok(())
}

async fn handle_relay(
    core: &Arc<AppCore>,
    http: &Http,
    command: &CommandInteraction,
) -> Result<String> {
    if !default_command_authorized(
        command.guild_id,
        command
            .member
            .as_deref()
            .and_then(|member| member.permissions),
    ) {
        return Ok("Default Relay commands require Discord Administrator permission.".into());
    }
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
            "`/relay {}` is disabled on the Commands page in the Relay application.",
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
            core.update_config(|config| config.watched_channel_id = channel_id.clone())
                .await?;
            Ok(format!("Relay channel set to <#{channel_id}>."))
        }
        "url" => {
            let config = core.config.read().await.clone();
            Ok(connection_details(&config))
        }
        "show" => {
            let config = core.config.read().await.clone();
            let channel = if config.watched_channel_id.is_empty() {
                "not configured".to_owned()
            } else {
                format!("<#{}>", config.watched_channel_id)
            };
            Ok(format!(
                "Channel: {channel}\n{}",
                connection_details(&config)
            ))
        }
        "status" => relay_status(core).await,
        "test" => {
            let target = arguments
                .iter()
                .find_map(|argument| match &argument.value {
                    CommandDataOptionValue::String(value) => output_test_target(value),
                    _ => None,
                })
                .context("an output is required")?;
            relay_output_test(core, target).await
        }
        "regenerate" => {
            let config = core.config.read().await.clone();
            Ok(format!(
                "The permanent relay URL was preserved. No OBS update is required:\n{}",
                overlay_url(&config)
            ))
        }
        "clear" => {
            let channel_id = arguments
                .iter()
                .find_map(|argument| match argument.value {
                    CommandDataOptionValue::Channel(channel_id) => Some(channel_id),
                    _ => None,
                })
                .context("a channel is required")?;
            let count = arguments
                .iter()
                .find_map(|argument| match argument.value {
                    CommandDataOptionValue::Integer(value) => usize::try_from(value).ok(),
                    _ => None,
                })
                .filter(|count| (1..=1_000).contains(count))
                .context("a message count between 1 and 1000 is required")?;
            clear_selected_channel(http, channel_id, count).await
        }
        "nuke" => {
            let channel_id = arguments
                .iter()
                .find_map(|argument| match argument.value {
                    CommandDataOptionValue::Channel(channel_id) => Some(channel_id),
                    _ => None,
                })
                .context("a channel is required")?;
            nuke_selected_channel(core, http, channel_id).await
        }
        "lock" => toggle_channel_lock(core, http).await,
        "changelog" => {
            let channel_id = arguments
                .iter()
                .find_map(|argument| match argument.value {
                    CommandDataOptionValue::Channel(channel_id) => Some(channel_id),
                    _ => None,
                })
                .context("a channel is required")?;
            post_changelog(core, http, channel_id).await
        }
        _ => Ok("Unknown Relay subcommand.".into()),
    }
}

const CHANGELOG_URL: &str =
    "https://raw.githubusercontent.com/imnotStealthy/relay/main/CHANGELOG.md";
const CHANGELOG_PAGE_URL: &str = "https://github.com/imnotStealthy/relay/blob/main/CHANGELOG.md";
const CHANGELOG_MAX_BYTES: usize = 256 * 1024;
const CHANGELOG_EMBED_DESCRIPTION_LIMIT: usize = 3_900;
const CHANGELOG_MAX_EMBEDS: usize = 10;
const CHANGELOG_EMBED_COLOUR: u32 = 0x2F_B3_A8;

#[derive(Debug, Clone, PartialEq, Eq)]
struct ChangelogSection {
    heading: String,
    version: String,
    date: Option<String>,
    body: String,
}

async fn post_changelog(core: &Arc<AppCore>, http: &Http, channel_id: ChannelId) -> Result<String> {
    let language = core.interface_preferences.read().await.language.clone();
    let changelog = fetch_changelog_markdown()
        .await
        .context("failed to download CHANGELOG.md from GitHub")?;
    let mut section = latest_changelog_section(&changelog)
        .context("no published release section was found in CHANGELOG.md")?;
    section.body = crate::changelog::changelog_body_for_language(&section.body, &language);
    let (embeds, truncated) = build_changelog_embeds(&section);
    if embeds.is_empty() {
        bail!("the latest changelog section is empty");
    }

    for embed in embeds {
        channel_id
            .send_message(
                http,
                CreateMessage::new()
                    .embed(embed)
                    .allowed_mentions(CreateAllowedMentions::new()),
            )
            .await
            .context(
                "failed to post the changelog — ensure the bot can View Channel, Send Messages, and Embed Links in that channel",
            )?;
    }

    let mut confirmation = format!(
        "Posted Relay **{}** release notes as embed(s) to <#{}>.",
        section.version, channel_id
    );
    if truncated {
        confirmation.push_str(&format!(
            "\nSome content was truncated. Full notes: <{CHANGELOG_PAGE_URL}>"
        ));
    }
    Ok(confirmation)
}

async fn fetch_changelog_markdown() -> Result<String> {
    let url = reqwest::Url::parse(CHANGELOG_URL).context("invalid changelog URL")?;
    if url.scheme() != "https"
        || url.host_str() != Some("raw.githubusercontent.com")
        || url.path() != "/imnotStealthy/relay/main/CHANGELOG.md"
    {
        bail!("changelog URL is not the expected GitHub raw path");
    }

    let client = reqwest::Client::builder()
        .user_agent(concat!("Relay/", env!("CARGO_PKG_VERSION")))
        .timeout(Duration::from_secs(20))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .context("unable to create the changelog HTTP client")?;
    let response = client
        .get(url)
        .send()
        .await
        .context("unable to reach GitHub")?
        .error_for_status()
        .context("GitHub rejected the changelog request")?;
    if response
        .content_length()
        .is_some_and(|length| length > CHANGELOG_MAX_BYTES as u64)
    {
        bail!("CHANGELOG.md exceeds the download size limit");
    }

    let mut body = Vec::new();
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.context("failed while reading CHANGELOG.md")?;
        if body.len() + chunk.len() > CHANGELOG_MAX_BYTES {
            bail!("CHANGELOG.md exceeds the download size limit");
        }
        body.extend_from_slice(&chunk);
    }

    String::from_utf8(body).context("CHANGELOG.md is not valid UTF-8")
}

fn latest_changelog_section(changelog: &str) -> Option<ChangelogSection> {
    let mut heading = None;
    let mut lines = Vec::new();
    let mut in_release = false;
    for line in changelog.lines() {
        if line.starts_with("## ") {
            if in_release {
                break;
            }
            in_release = line.starts_with("## [") && !line.starts_with("## [Unreleased]");
            if !in_release {
                continue;
            }
            heading = Some(line.to_owned());
            continue;
        } else if line.starts_with('[') && line.contains("]: http") {
            continue;
        }
        if in_release {
            lines.push(line);
        }
    }
    let heading = heading?;
    let (version, date) = parse_changelog_heading(&heading)?;
    let body = lines.join("\n").trim().to_owned();
    if body.is_empty() {
        return None;
    }
    Some(ChangelogSection {
        heading,
        version,
        date,
        body,
    })
}

fn parse_changelog_heading(heading: &str) -> Option<(String, Option<String>)> {
    let rest = heading.strip_prefix("## [")?;
    let (version, after) = rest.split_once(']')?;
    let version = version.trim();
    if version.is_empty() || version.eq_ignore_ascii_case("unreleased") {
        return None;
    }
    let date = after
        .trim()
        .strip_prefix('-')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned);
    Some((version.to_owned(), date))
}

fn discord_format_changelog_body(body: &str) -> String {
    let mut formatted = String::new();
    for line in body.lines() {
        if let Some(title) = line.strip_prefix("#### ") {
            if !formatted.is_empty() {
                formatted.push('\n');
            }
            formatted.push_str("**");
            formatted.push_str(title.trim());
            formatted.push_str("**\n");
            continue;
        }
        if let Some(title) = line.strip_prefix("### ") {
            if !formatted.is_empty() {
                formatted.push('\n');
            }
            formatted.push_str("**");
            formatted.push_str(title.trim());
            formatted.push_str("**\n");
            continue;
        }
        if let Some(item) = line.strip_prefix("- ") {
            formatted.push('•');
            formatted.push(' ');
            formatted.push_str(item);
            formatted.push('\n');
            continue;
        }
        if line.starts_with("## ") {
            continue;
        }
        formatted.push_str(line);
        formatted.push('\n');
    }
    formatted.trim().to_owned()
}

fn build_changelog_embeds(section: &ChangelogSection) -> (Vec<CreateEmbed>, bool) {
    let colour = Colour::new(CHANGELOG_EMBED_COLOUR);
    let formatted = discord_format_changelog_body(&section.body);
    let mut descriptions = split_message_chunks(&formatted, CHANGELOG_EMBED_DESCRIPTION_LIMIT);
    if descriptions.is_empty() {
        descriptions.push(String::new());
    }

    let truncated = descriptions.len() > CHANGELOG_MAX_EMBEDS;
    if truncated {
        descriptions.truncate(CHANGELOG_MAX_EMBEDS);
        if let Some(last) = descriptions.last_mut() {
            let notice = format!("\n\n…truncated. Full changelog: {CHANGELOG_PAGE_URL}");
            while char_len(last) + char_len(&notice) > CHANGELOG_EMBED_DESCRIPTION_LIMIT
                && !last.is_empty()
            {
                last.pop();
            }
            last.push_str(&notice);
        }
    }

    let total = descriptions.len();
    let footer_text = match &section.date {
        Some(date) => format!("Released {date} · Synced from GitHub CHANGELOG.md"),
        None => "Synced from GitHub CHANGELOG.md".to_owned(),
    };
    let embeds = descriptions
        .into_iter()
        .enumerate()
        .map(|(index, description)| {
            let title = if total == 1 {
                format!("Relay {}", section.version)
            } else {
                format!("Relay {} ({}/{})", section.version, index + 1, total)
            };
            let mut embed = CreateEmbed::new()
                .colour(colour)
                .title(title)
                .url(CHANGELOG_PAGE_URL)
                .description(description);
            if index + 1 == total {
                embed = embed.footer(serenity::all::CreateEmbedFooter::new(footer_text.clone()));
            }
            embed
        })
        .collect();

    (embeds, truncated)
}

fn char_len(value: &str) -> usize {
    value.chars().count()
}

fn split_message_chunks(text: &str, limit: usize) -> Vec<String> {
    if limit == 0 {
        return Vec::new();
    }
    let mut chunks = Vec::new();
    let mut current = String::new();
    for line in text.lines() {
        let mut remaining = line;
        while char_len(remaining) > limit {
            let (head, tail) = split_at_char_boundary(remaining, limit);
            if !current.is_empty() {
                chunks.push(std::mem::take(&mut current));
            }
            chunks.push(head.to_owned());
            remaining = tail;
        }
        if !current.is_empty() && char_len(&current) + char_len(remaining) + 1 > limit {
            chunks.push(std::mem::take(&mut current));
        }
        if !current.is_empty() {
            current.push('\n');
        }
        current.push_str(remaining);
    }
    if !current.is_empty() {
        chunks.push(current);
    }
    chunks
}

fn split_at_char_boundary(value: &str, char_count: usize) -> (&str, &str) {
    match value.char_indices().nth(char_count) {
        Some((index, _)) => (&value[..index], &value[index..]),
        None => (value, ""),
    }
}

pub(crate) async fn clear_selected_channel(
    http: &Http,
    channel_id: ChannelId,
    count: usize,
) -> Result<String> {
    let deleted = clear_channel_messages(http, channel_id, count).await?;
    Ok(format!(
        "Cleared {deleted} message(s) from <#{channel_id}>."
    ))
}

async fn nuke_selected_channel(
    core: &Arc<AppCore>,
    http: &Http,
    channel_id: ChannelId,
) -> Result<String> {
    let Channel::Guild(channel) = channel_id.to_channel(http).await? else {
        bail!("only text and announcement channels can be recreated");
    };
    if !matches!(channel.kind, ChannelType::Text | ChannelType::News) {
        bail!("only text and announcement channels can be recreated");
    }

    let mut replacement = CreateChannel::new(channel.name.clone())
        .kind(channel.kind)
        .position(channel.position)
        .permissions(channel.permission_overwrites.clone())
        .nsfw(channel.nsfw)
        .rate_limit_per_user(channel.rate_limit_per_user.unwrap_or_default());
    if let Some(parent_id) = channel.parent_id {
        replacement = replacement.category(parent_id);
    }
    if let Some(topic) = channel.topic.as_deref() {
        replacement = replacement.topic(topic);
    }
    let replacement = channel.guild_id.create_channel(http, replacement).await?;
    let replacement_id = replacement.id;
    let old_id = channel.id;

    core.update_config(|config| {
        replace_configured_channel_id(config, old_id, replacement_id);
    })
    .await?;
    channel.delete(http).await?;
    Ok(format!(
        "Recreated <#{old_id}> as <#{replacement_id}>. Its message history was deleted."
    ))
}

fn replace_configured_channel_id(
    config: &mut AppConfig,
    old_channel_id: ChannelId,
    replacement_channel_id: ChannelId,
) {
    let old_channel_id = old_channel_id.to_string();
    let replacement_channel_id = replacement_channel_id.to_string();
    for channel_id in [
        &mut config.watched_channel_id,
        &mut config.tts_channel_id,
        &mut config.music_channel_id,
        &mut config.honeypot_channel_id,
    ] {
        if *channel_id == old_channel_id {
            *channel_id = replacement_channel_id.clone();
        }
    }
    if let Some(lock) = config.channel_lock.as_mut()
        && lock.channel_id == old_channel_id
    {
        lock.channel_id = replacement_channel_id;
    }
}

async fn clear_channel_messages(http: &Http, channel_id: ChannelId, limit: usize) -> Result<usize> {
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
        "status" => config.command_status_enabled,
        "test" => config.command_test_enabled,
        "regenerate" => config.command_regenerate_enabled,
        "clear" => config.command_clear_enabled,
        "nuke" => config.command_nuke_enabled,
        "lock" => config.command_lock_enabled,
        "changelog" => config.command_changelog_enabled,
        _ => false,
    }
}

fn default_command_authorized(guild_id: Option<GuildId>, permissions: Option<Permissions>) -> bool {
    guild_id.is_some()
        && permissions.is_some_and(|permissions| permissions.contains(Permissions::ADMINISTRATOR))
}

async fn relay_status(core: &AppCore) -> Result<String> {
    let config = core.config.read().await.clone();
    let bot = core.bot_status.read().await.clone();
    let server = core.server_status.read().await.clone();
    let moderation_pending = core.pending_media.read().await.len();
    Ok(format_relay_status(
        &config,
        &bot,
        &server,
        moderation_pending,
        core.tts_pending_count(),
    ))
}

fn format_relay_status(
    config: &AppConfig,
    bot: &BotStatus,
    server: &ServerStatus,
    moderation_pending: usize,
    tts_pending: usize,
) -> String {
    let bot_state = if bot.connected {
        bot.username
            .as_deref()
            .map(|username| format!("connected as {username}"))
            .unwrap_or_else(|| "connected".into())
    } else {
        "disconnected".into()
    };
    let media_channel = if config.watched_channel_id.is_empty() {
        "not configured".into()
    } else {
        format!("<#{}>", config.watched_channel_id)
    };
    let tts_channel = if config.tts_channel_id.is_empty() {
        "disabled".into()
    } else {
        format!("<#{}>", config.tts_channel_id)
    };
    let moderation = if config.moderation_enabled {
        format!("enabled ({moderation_pending} pending)")
    } else {
        "disabled".into()
    };
    let media_widget = widget_status(config.widget_visible, config.widget_locked);
    let notification_widget = widget_status(
        config.notification_widget_visible,
        config.notification_widget_locked,
    );

    format!(
        "**Relay status**\n\
         Bot: {bot_state}\n\
         Local server: {}\n\
         Media channel: {media_channel}\n\
         TTS channel: {tts_channel}\n\
         Moderation: {moderation}\n\
         TTS preparing: {tts_pending}\n\
         Media widget: {media_widget}\n\
         Notification widget: {notification_widget}\n\
         **Connected outputs (OBS / widget / preview)**\n\
         Visual: {}\n\
         Audio: {}\n\
         TTS: {}\n\
         Notifications: {}\n\
         Stickers: {}",
        if server.connected {
            "online"
        } else {
            "offline"
        },
        output_status(&server.outputs.visual),
        output_status(&server.outputs.audio),
        output_status(&server.outputs.tts),
        output_status(&server.outputs.notification),
        output_status(&server.outputs.sticker),
    )
}

fn widget_status(visible: bool, locked: bool) -> &'static str {
    match (visible, locked) {
        (false, _) => "hidden",
        (true, true) => "visible and locked",
        (true, false) => "visible and movable",
    }
}

fn output_status(status: &OutputConnectionStatus) -> String {
    format!(
        "{} / {} / {}",
        status.obs_clients, status.widget_clients, status.preview_clients
    )
}

async fn relay_output_test(core: &AppCore, target: OutputTestTarget) -> Result<String> {
    let connected = {
        let server = core.server_status.read().await;
        connected_output_count(&server, target)
    };
    let label = output_test_label(target);
    if connected == 0 {
        return Ok(format!(
            "No live {label} output is connected. Connect the matching OBS source or Windows output first."
        ));
    }
    emit_output_test(core, target).await?;
    Ok(format!(
        "Local {label} test sent to {connected} connected output(s). Nothing was posted to Discord or added to Relay history."
    ))
}

fn output_test_target(value: &str) -> Option<OutputTestTarget> {
    match value {
        "visual" | "media" => Some(OutputTestTarget::Visual),
        "audio" => Some(OutputTestTarget::Audio),
        "tts" => Some(OutputTestTarget::Tts),
        "notification" => Some(OutputTestTarget::Notification),
        "sticker" => Some(OutputTestTarget::Sticker),
        _ => None,
    }
}

fn connected_output_count(server: &ServerStatus, target: OutputTestTarget) -> usize {
    if !server.connected {
        return 0;
    }
    let status = match target {
        OutputTestTarget::Visual => &server.outputs.visual,
        OutputTestTarget::Audio => &server.outputs.audio,
        OutputTestTarget::Tts => &server.outputs.tts,
        OutputTestTarget::Notification => &server.outputs.notification,
        OutputTestTarget::Sticker => &server.outputs.sticker,
    };
    status.obs_clients + status.widget_clients
}

fn output_test_label(target: OutputTestTarget) -> &'static str {
    match target {
        OutputTestTarget::Visual => "media",
        OutputTestTarget::Audio => "audio",
        OutputTestTarget::Tts => "TTS",
        OutputTestTarget::Notification => "notification",
        OutputTestTarget::Sticker => "sticker",
    }
}

async fn toggle_channel_lock(core: &Arc<AppCore>, http: &Http) -> Result<String> {
    let config = core.config.read().await.clone();
    if let Some(snapshot) = config.channel_lock.clone() {
        restore_channel_permissions(http, &snapshot).await?;
        core.update_config(|next| next.channel_lock = None).await?;
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
    let lock_snapshot = snapshot.clone();
    core.update_config(|next| next.channel_lock = Some(lock_snapshot))
        .await?;

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
                let _ = core
                    .update_config(|rollback| rollback.channel_lock = None)
                    .await;
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

fn connection_details(config: &AppConfig) -> String {
    format!(
        "Relay URL: `http://127.0.0.1:{}`\nVisual overlay: `{}`\nAudio overlay: `{}`",
        config.port,
        overlay_url(config),
        audio_overlay_url(config)
    )
}

fn overlay_url(config: &AppConfig) -> String {
    format!(
        "http://{}:{}/obs/visual",
        crate::widget::youtube_embed_host(),
        config.port
    )
}

fn audio_overlay_url(config: &AppConfig) -> String {
    format!("http://127.0.0.1:{}/obs/audio", config.port)
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
    content: String,
    embeds: Vec<serenity::all::Embed>,
    role_ids: Vec<String>,
}

async fn submit_embedded_gifs(core: &Arc<AppCore>, http: &Http, message: &Message) {
    submit_deferred_embeds(
        core,
        http,
        DeferredEmbedMessage {
            channel_id: message.channel_id.to_string(),
            message_id: message.id.to_string(),
            author: message.author.clone(),
            timestamp: message.timestamp.unix_timestamp().max(0) as u64 * 1_000,
            content: message.content.clone(),
            embeds: message.embeds.clone(),
            role_ids: message_role_ids(message),
        },
    )
    .await;
}

async fn submit_deferred_embeds(core: &Arc<AppCore>, http: &Http, message: DeferredEmbedMessage) {
    if message.author.bot {
        return;
    }
    let config = core.config.read().await.clone();
    let scoped_config = privacy::scoped_config_for_roles(&config, &message.role_ids);
    if config.watched_channel_id.is_empty() || message.channel_id != config.watched_channel_id {
        return;
    }
    let message_report = classify_privacy_values(
        &message.content,
        std::iter::empty::<&str>(),
        std::iter::empty::<&str>(),
        &scoped_config,
    );
    if privacy_action_is_blocked(&message_report, &scoped_config) {
        delete_deferred_message_if_needed(core, http, &message, &message_report, &scoped_config)
            .await;
        return;
    }
    let media_text = prepare_media_text(&message.content);
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
        if let Some(bytes) = downloaded.as_deref() {
            content_type = sniff_media_type(bytes, content_type);
        }
        let event = MediaEvent {
            kind: MediaKind::Gif,
            url: embed.url,
            proxy_url: embed.proxy_url,
            filename: embed.title.unwrap_or_else(|| "Discord GIF".into()),
            content_type: content_type.into(),
            artwork_id: None,
            audio_id: None,
            cached_media_id: None,
            title: None,
            artist: None,
            text: media_text.clone(),
            author: AuthorIdentity {
                username: message.author.name.clone(),
                display_avatar_url: message
                    .author
                    .avatar_url()
                    .unwrap_or_else(|| message.author.default_avatar_url()),
            },
            timestamp: message.timestamp,
            message_id: message.message_id.clone(),
        };
        let privacy_report = if event
            .content_type
            .to_ascii_lowercase()
            .starts_with("image/")
        {
            match downloaded.as_deref() {
                Some(bytes) => {
                    privacy::analyze_image_bytes_async(
                        bytes,
                        Some(&message.content),
                        &scoped_config,
                    )
                    .await
                }
                None => {
                    privacy::analyze_remote_image(
                        &event.url,
                        &event.proxy_url,
                        Some(&message.content),
                        &scoped_config,
                    )
                    .await
                }
            }
        } else {
            privacy::classify_text(Some(&message.content), &scoped_config)
        };
        let current_config = core.config.read().await.clone();
        let current_scoped_config =
            privacy::scoped_config_for_roles(&current_config, &message.role_ids);
        let privacy_report =
            reclassify_privacy_report(privacy_report, &message.content, &current_scoped_config);
        if privacy_action_is_blocked(&privacy_report, &current_scoped_config) {
            delete_deferred_message_if_needed(
                core,
                http,
                &message,
                &privacy_report,
                &current_scoped_config,
            )
            .await;
            return;
        }
        core.submit_analyzed_media_with_text_and_roles(
            event,
            Some(privacy_report),
            Some(&message.content),
            &message.role_ids,
        )
        .await;
    }
}

async fn delete_deferred_message_if_needed(
    core: &Arc<AppCore>,
    http: &Http,
    message: &DeferredEmbedMessage,
    report: &privacy::PrivacyReport,
    config: &AppConfig,
) {
    let (Ok(channel_id), Ok(message_id)) = (
        message.channel_id.parse::<u64>(),
        message.message_id.parse::<u64>(),
    ) else {
        return;
    };
    if should_auto_delete_blocked_message(report, config)
        && ChannelId::new(channel_id)
            .delete_message(http, MessageId::new(message_id))
            .await
            .is_err()
    {
        core.bot_status.write().await.error =
            Some("Privacy deletion failed. Verify Manage Messages in this channel.".into());
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
    } else {
        let thumbnail = embed.thumbnail.as_ref()?;
        (
            thumbnail.url.clone(),
            thumbnail
                .proxy_url
                .clone()
                .unwrap_or_else(|| thumbnail.url.clone()),
        )
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

fn prepare_media_text(content: &str) -> Option<String> {
    let printable = content
        .chars()
        .map(|character| {
            if character.is_control() {
                ' '
            } else {
                character
            }
        })
        .collect::<String>();
    let normalized = printable
        .split_whitespace()
        .filter(|segment| !segment.starts_with("https://") && !segment.starts_with("http://"))
        .collect::<Vec<_>>()
        .join(" ");
    if normalized.is_empty() {
        return None;
    }
    let mut characters = normalized.chars();
    let mut text = characters
        .by_ref()
        .take(MEDIA_TEXT_LIMIT)
        .collect::<String>();
    if characters.next().is_some() {
        text.pop();
        text.push('…');
    }
    Some(text)
}

async fn set_bot_error(core: &Arc<AppCore>, error: String) {
    let mut status = core.bot_status.write().await;
    status.connected = false;
    status.error = Some(error);
}

fn current_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn tts_failure_status(error: &str, visual_published: bool) -> Option<String> {
    (!visual_published).then(|| format!("Windows TTS failed: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn successful_visual_fallback_clears_the_tts_error_status() {
        assert_eq!(tts_failure_status("voice unavailable", true), None);
        assert_eq!(
            tts_failure_status("voice unavailable", false),
            Some("Windows TTS failed: voice unavailable".into())
        );
    }

    #[test]
    fn honeypot_targets_only_the_configured_channel_with_english_notices() {
        let config = AppConfig {
            honeypot_channel_id: "123456789012345678".into(),
            honeypot_action: HoneypotAction::Ban,
            ..AppConfig::default()
        };

        assert_eq!(
            honeypot_action_for_channel(&config, "123456789012345678"),
            Some(HoneypotAction::Ban)
        );
        assert_eq!(
            honeypot_action_for_channel(&config, "223456789012345678"),
            None
        );
        assert!(honeypot_notice(HoneypotAction::Kick).contains("kicked from the server"));
        assert!(honeypot_notice(HoneypotAction::Ban).contains("banned from the server"));
        for notice in [
            honeypot_notice(HoneypotAction::Kick),
            honeypot_notice(HoneypotAction::Ban),
        ] {
            assert!(notice.contains("token-grabbing"));
            assert!(notice.contains("Change your Discord password"));
            assert!(notice.contains("two-factor authentication"));
        }
    }

    #[test]
    fn deferred_embed_updates_fetch_roles_before_using_the_partial_fallback() {
        let source = include_str!("bot.rs");
        let handler = source
            .split("async fn message_update(")
            .nth(1)
            .and_then(|value| value.split("async fn interaction_create(").next())
            .expect("message update handler");
        let fetch = handler
            .find("get_message(event.channel_id, event.id)")
            .unwrap();
        let partial = handler.find("role_ids: Vec::new()").unwrap();
        assert!(fetch < partial);
    }

    #[test]
    fn extracts_only_enabled_discord_guild_tags() {
        let tagged_user: User = serde_json::from_value(serde_json::json!({
            "id": "123456789012345678",
            "username": "Stealthy.",
            "primary_guild": {
                "identity_guild_id": "987654321098765432",
                "identity_enabled": true,
                "tag": "RE",
                "badge": "7d1734ae5a615e82bc7a4033b98fade8"
            }
        }))
        .unwrap();
        let tag = guild_tag_from_user(&tagged_user).unwrap();
        assert_eq!(tag.name, "RE");
        assert_eq!(
            tag.badge_url.as_deref(),
            Some(
                "https://cdn.discordapp.com/guild-tag-badges/987654321098765432/7d1734ae5a615e82bc7a4033b98fade8.png?size=1024"
            )
        );

        let hidden_user: User = serde_json::from_value(serde_json::json!({
            "id": "123456789012345678",
            "username": "Stealthy.",
            "primary_guild": {
                "identity_guild_id": "987654321098765432",
                "identity_enabled": false,
                "tag": "RE",
                "badge": "7d1734ae5a615e82bc7a4033b98fade8"
            }
        }))
        .unwrap();
        assert!(guild_tag_from_user(&hidden_user).is_none());
    }

    #[test]
    fn formats_local_overlay_urls_without_secret() {
        let config = AppConfig {
            port: 5_321,
            ..AppConfig::default()
        };
        assert_eq!(overlay_url(&config), "http://localhost:5321/obs/visual");
        assert_eq!(
            audio_overlay_url(&config),
            "http://127.0.0.1:5321/obs/audio"
        );
        let details = connection_details(&config);
        assert!(details.contains("http://localhost:5321/obs/visual"));
        assert!(!details.contains("secret"));
    }

    #[test]
    fn builds_invite_url_with_required_scopes_and_permissions() {
        assert_eq!(
            invite_url("123456789012345678", &AppConfig::default()),
            "https://discord.com/oauth2/authorize?client_id=123456789012345678&permissions=268510224&scope=bot%20applications.commands"
        );
    }

    #[test]
    fn disables_commands_individually() {
        let config = AppConfig {
            command_clear_enabled: false,
            command_status_enabled: false,
            command_test_enabled: false,
            ..AppConfig::default()
        };
        assert!(!command_enabled(&config, "clear"));
        assert!(!command_enabled(&config, "status"));
        assert!(!command_enabled(&config, "test"));
        assert!(command_enabled(&config, "lock"));
        assert!(!command_enabled(&config, "unknown"));
    }

    #[test]
    fn default_commands_still_require_an_administrator_inside_a_guild() {
        let guild_id = Some(GuildId::new(123_456_789_012_345_678));
        assert!(default_command_authorized(
            guild_id,
            Some(Permissions::ADMINISTRATOR)
        ));
        assert!(!default_command_authorized(
            guild_id,
            Some(Permissions::MANAGE_MESSAGES)
        ));
        assert!(!default_command_authorized(
            None,
            Some(Permissions::ADMINISTRATOR)
        ));
    }

    #[test]
    fn formats_live_status_for_obs_and_windows_outputs() {
        let config = AppConfig {
            watched_channel_id: "123456789012345678".into(),
            tts_channel_id: "223456789012345678".into(),
            moderation_enabled: true,
            widget_visible: true,
            widget_locked: true,
            ..AppConfig::default()
        };
        let bot = BotStatus {
            connected: true,
            username: Some("Relay".into()),
            ..BotStatus::default()
        };
        let mut server = ServerStatus {
            connected: true,
            ..ServerStatus::default()
        };
        server.outputs.visual.obs_clients = 1;
        server.outputs.visual.widget_clients = 1;
        server.outputs.notification.preview_clients = 1;

        let status = format_relay_status(&config, &bot, &server, 3, 2);

        assert!(status.contains("Bot: connected as Relay"));
        assert!(status.contains("Media channel: <#123456789012345678>"));
        assert!(status.contains("Moderation: enabled (3 pending)"));
        assert!(status.contains("TTS preparing: 2"));
        assert!(status.contains("Media widget: visible and locked"));
        assert!(status.contains("Visual: 1 / 1 / 0"));
        assert!(status.contains("Notifications: 0 / 0 / 1"));
        assert!(!status.contains("secret"));
    }

    #[test]
    fn clear_command_requires_a_bounded_message_count() {
        let command = serde_json::to_value(relay_command(&AppConfig::default())).unwrap();
        let clear = command["options"]
            .as_array()
            .unwrap()
            .iter()
            .find(|option| option["name"] == "clear")
            .unwrap();
        let channel = &clear["options"][0];
        assert_eq!(channel["name"], "channel");
        assert_eq!(channel["required"], true);
        let count = &clear["options"][1];
        assert_eq!(count["name"], "count");
        assert_eq!(count["required"], true);
        assert_eq!(count["min_value"], 1);
        assert_eq!(count["max_value"], 1_000);
    }

    #[test]
    fn nuke_command_requires_a_text_or_announcement_channel() {
        let command = serde_json::to_value(relay_command(&AppConfig::default())).unwrap();
        let nuke = command["options"]
            .as_array()
            .unwrap()
            .iter()
            .find(|option| option["name"] == "nuke")
            .expect("nuke subcommand must be registered");
        let channel = &nuke["options"][0];
        assert_eq!(channel["name"], "channel");
        assert_eq!(channel["required"], true);
        assert_eq!(channel["channel_types"], serde_json::json!([0, 5]));
    }

    #[test]
    fn nuke_replaces_configured_channel_references() {
        let mut config = AppConfig {
            watched_channel_id: "1".into(),
            tts_channel_id: "1".into(),
            music_channel_id: "2".into(),
            honeypot_channel_id: "1".into(),
            channel_lock: Some(ChannelLockSnapshot {
                channel_id: "1".into(),
                overwrites: Vec::new(),
            }),
            ..AppConfig::default()
        };

        replace_configured_channel_id(&mut config, ChannelId::new(1), ChannelId::new(3));

        assert_eq!(config.watched_channel_id, "3");
        assert_eq!(config.tts_channel_id, "3");
        assert_eq!(config.music_channel_id, "2");
        assert_eq!(config.honeypot_channel_id, "3");
        assert_eq!(config.channel_lock.unwrap().channel_id, "3");
    }

    #[test]
    fn custom_commands_share_the_relay_schema_without_a_global_admin_gate() {
        let config = AppConfig {
            custom_commands: vec![custom_commands::CustomCommandDefinition {
                name: "rules".into(),
                description: "Show the configured rules".into(),
                action: custom_commands::CustomCommandAction::Reply {
                    text: "Rules".into(),
                    ephemeral: true,
                },
                ..custom_commands::CustomCommandDefinition::default()
            }],
            ..AppConfig::default()
        };
        let command = serde_json::to_value(relay_command(&config)).unwrap();
        assert!(command.get("default_member_permissions").is_none());
        assert!(
            command["options"]
                .as_array()
                .unwrap()
                .iter()
                .any(|option| option["name"] == "rules")
        );
    }

    #[test]
    fn test_command_exposes_every_local_output() {
        let command = serde_json::to_value(relay_command(&AppConfig::default())).unwrap();
        let test = command["options"]
            .as_array()
            .unwrap()
            .iter()
            .find(|option| option["name"] == "test")
            .unwrap();
        let output = &test["options"][0];
        assert_eq!(output["name"], "output");
        assert_eq!(output["required"], true);
        let values = output["choices"]
            .as_array()
            .unwrap()
            .iter()
            .map(|choice| choice["value"].as_str().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(
            values,
            vec!["visual", "audio", "tts", "notification", "sticker"]
        );
    }

    #[tokio::test]
    async fn discord_output_tests_require_a_live_output_and_bypass_history() {
        let directory = tempfile::tempdir().unwrap();
        let core = AppCore::load(directory.path().join("config.json")).unwrap();

        let unavailable = relay_output_test(&core, OutputTestTarget::Visual)
            .await
            .unwrap();
        assert!(unavailable.contains("No live media output is connected"));

        {
            let mut server = core.server_status.write().await;
            server.connected = true;
            server.outputs.visual.obs_clients = 1;
        }
        let mut events = core.relay_tx.subscribe();
        let confirmation = relay_output_test(&core, OutputTestTarget::Visual)
            .await
            .unwrap();

        assert!(confirmation.contains("Local media test sent to 1 connected output"));
        assert!(confirmation.contains("Nothing was posted to Discord"));
        assert!(matches!(
            events.recv().await.unwrap(),
            crate::model::RelayEvent::TestOutput(_)
        ));
        assert!(core.history.read().await.is_empty());
    }

    #[test]
    fn extracts_the_latest_release_section_from_the_changelog() {
        let changelog = "# Changelog\n\nIntro text.\n\n## [Unreleased]\n\n- Pending change.\n\n## [1.1.0] - 2026-07-12\n\n### Added\n\n- New feature.\n\n## [1.0.0] - 2026-07-12\n\n- First release.\n\n[Unreleased]: https://example.com/compare\n[1.1.0]: https://example.com/tag\n";
        let section = latest_changelog_section(changelog).unwrap();
        assert_eq!(section.version, "1.1.0");
        assert_eq!(section.date.as_deref(), Some("2026-07-12"));
        assert_eq!(section.heading, "## [1.1.0] - 2026-07-12");
        assert!(section.body.contains("New feature."));
        assert!(!section.body.contains("Pending change."));
        assert!(!section.body.contains("First release."));
        assert!(!section.body.contains("example.com"));
        assert!(
            latest_changelog_section("# Changelog\n\n## [Unreleased]\n\n- Only pending.\n")
                .is_none()
        );
    }

    #[test]
    fn formats_changelog_markdown_for_discord_embeds() {
        let body = "### English\n\n#### Added\n\n- New feature.\n\n### Français\n\n#### Ajouté\n\n- Nouvelle fonctionnalité.\n";
        let formatted = discord_format_changelog_body(body);
        assert!(formatted.contains("**English**"));
        assert!(formatted.contains("**Added**"));
        assert!(formatted.contains("• New feature."));
        assert!(formatted.contains("**Français**"));
        assert!(formatted.contains("• Nouvelle fonctionnalité."));
        assert!(!formatted.contains("####"));
    }

    #[test]
    fn builds_changelog_embeds_with_version_title_and_github_link() {
        let section = ChangelogSection {
            heading: "## [1.2.6] - 2026-08-14".into(),
            version: "1.2.6".into(),
            date: Some("2026-08-14".into()),
            body: "### English\n\n#### Fixed\n\n- One fix.\n\n### Français\n\n#### Corrigé\n\n- Un correctif.\n".into(),
        };
        let (embeds, truncated) = build_changelog_embeds(&section);
        assert!(!truncated);
        assert_eq!(embeds.len(), 1);
    }

    #[test]
    fn splits_long_changelog_sections_into_discord_sized_messages() {
        let long_line = "x".repeat(80);
        let text = (0..60)
            .map(|_| long_line.clone())
            .collect::<Vec<_>>()
            .join("\n");
        let chunks = split_message_chunks(&text, 1_900);
        assert!(chunks.len() > 1);
        assert!(chunks.iter().all(|chunk| char_len(chunk) <= 1_900));
        assert_eq!(chunks.join("\n"), text);
    }

    #[test]
    fn hard_splits_oversized_changelog_lines() {
        let line = "y".repeat(5_000);
        let chunks = split_message_chunks(&line, 1_000);
        assert!(chunks.len() > 1);
        assert!(chunks.iter().all(|chunk| char_len(chunk) <= 1_000));
        assert_eq!(chunks.concat(), line);
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
    fn prepares_bounded_media_text_without_standalone_links() {
        assert_eq!(
            prepare_media_text("Regardez mon setup https://example.com/image"),
            Some("Regardez mon setup".into())
        );
        assert_eq!(prepare_media_text("https://example.com/image"), None);
        assert_eq!(
            prepare_media_text("Une ligne\navec\tdu texte"),
            Some("Une ligne avec du texte".into())
        );

        let caption = prepare_media_text(&"é".repeat(MEDIA_TEXT_LIMIT + 1)).unwrap();
        assert_eq!(caption.chars().count(), MEDIA_TEXT_LIMIT);
        assert!(caption.ends_with('…'));
    }

    #[test]
    fn automatic_privacy_filter_covers_sticker_and_attachment_names_without_scan() {
        let config = AppConfig {
            privacy_scan_enabled: false,
            privacy_concepts: vec![privacy::ForbiddenConcept {
                canonical: "hitler".into(),
                aliases: Vec::new(),
                regexes: Vec::new(),
            }],
            ..AppConfig::default()
        };
        let report = classify_privacy_values(
            "ordinary message",
            ["safe sticker"],
            ["hitler.png"],
            &config,
        );
        assert!(privacy_action_is_blocked(&report, &config));
        assert!(report.reasons.contains(&"forbidden_concept"));
    }

    #[test]
    fn auto_deletes_blocked_privacy_and_filter_word_messages() {
        let mut config = AppConfig {
            privacy_scan_enabled: true,
            ..AppConfig::default()
        };
        let address = privacy::classify_text(Some("1 rue canot massy"), &config);
        assert!(should_auto_delete_blocked_message(&address, &config));

        config.privacy_scan_enabled = false;
        config.privacy_concepts = vec![privacy::ForbiddenConcept {
            canonical: "blockedterm".into(),
            aliases: Vec::new(),
            regexes: Vec::new(),
        }];
        let filter_word = privacy::classify_text(Some("blockedterm"), &config);
        assert!(
            filter_word
                .categories
                .contains(&privacy::PrivacyCategory::ContentFilter)
        );
        assert!(should_auto_delete_blocked_message(&filter_word, &config));
        assert!(!should_auto_delete_blocked_message(
            &privacy::PrivacyReport::sensitive("image_limits"),
            &config,
        ));

        config.privacy_auto_delete_blocked_messages = false;
        assert!(!should_auto_delete_blocked_message(&address, &config));
        assert!(!should_auto_delete_blocked_message(&filter_word, &config));
    }

    #[test]
    fn converts_unicode_and_custom_emojis_to_visual_segments() {
        let segments = parse_visual_segments(
            "Hello 👋 <:relay:123456789012345678> <a:dance:223456789012345678>",
        )
        .expect("message contains emojis");
        assert_eq!(
            segments
                .iter()
                .filter(|segment| segment.kind == "emoji")
                .count(),
            3
        );
        assert!(segments.iter().any(|segment| segment.value == "👋"));
        assert!(segments.iter().any(|segment| {
            segment.value == ":dance:"
                && segment.animated
                && segment
                    .url
                    .as_deref()
                    .is_some_and(|url| url.contains("223456789012345678"))
        }));
    }

    #[test]
    fn wraps_disabled_speech_messages_in_a_single_text_segment() {
        let segments = plain_text_segments("test".into());
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].kind, "text");
        assert_eq!(segments[0].value, "test");
        assert!(segments[0].url.is_none());
        assert!(!segments[0].animated);
    }

    #[test]
    fn leaves_plain_tts_messages_on_the_audio_path() {
        assert!(parse_visual_segments("Relay reads this message").is_none());
        assert!(parse_visual_segments("invalid <:emoji:not-an-id>").is_none());
    }

    #[test]
    fn maps_configured_bot_presence() {
        let config = AppConfig {
            bot_online_status: "idle".into(),
            bot_activity_type: "custom".into(),
            bot_activity_text: "Send your memes".into(),
            ..AppConfig::default()
        };
        let (activity, status) = presence_from_config(&config);
        assert_eq!(status, OnlineStatus::Idle);
        assert_eq!(activity.unwrap().state.as_deref(), Some("Send your memes"));

        let hidden = AppConfig {
            bot_online_status: "invisible".into(),
            bot_activity_type: "none".into(),
            ..AppConfig::default()
        };
        let (activity, status) = presence_from_config(&hidden);
        assert_eq!(status, OnlineStatus::Invisible);
        assert!(activity.is_none());
    }

    #[test]
    fn maps_all_discord_sticker_formats() {
        assert_eq!(sticker_format(StickerFormatType::Png), ("png", "image/png"));
        assert_eq!(
            sticker_format(StickerFormatType::Apng),
            ("apng", "image/png")
        );
        assert_eq!(
            sticker_format(StickerFormatType::Lottie),
            ("lottie", "application/json")
        );
        assert_eq!(sticker_format(StickerFormatType::Gif), ("gif", "image/gif"));
    }

    #[test]
    fn converts_renderable_stickers_to_visual_notification_segments() {
        let gif = sticker_visual_segment(
            "Relay dance".into(),
            Some("https://media.discordapp.net/stickers/1.gif".into()),
            StickerFormatType::Gif,
        );
        assert_eq!(
            (gif.kind.as_str(), gif.value.as_str()),
            ("sticker", "Relay dance")
        );
        assert!(gif.url.is_some() && gif.animated);

        let lottie = sticker_visual_segment(
            "Relay wave".into(),
            Some("https://cdn.discordapp.com/stickers/2.json".into()),
            StickerFormatType::Lottie,
        );
        assert!(lottie.url.is_none() && !lottie.animated);
    }
}
