use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};

use crate::{
    artwork,
    bot::{apply_bot_presence, invite_url, refresh_channel_list, start_bot},
    config::{AppConfig, OutputGeometry},
    credentials::{
        CredentialStatus, DiscordCredentials, credential_status, load_or_create_relay_secret,
        save_discord_credentials,
    },
    model::{
        AudioControlAction, AudioControlEvent, AuthorIdentity, BotStatus, ChannelSummary,
        GuildTagIdentity, InterfacePreferences, MediaEvent, MediaKind, OutputTestEvent,
        OutputTestTarget, PendingMedia, RelayEvent, ServerStatus, StickerEvent, TtsEvent,
        VisualSegment,
    },
    notification_widget::{self, NotificationWidgetState},
    server::start_server,
    state::AppCore,
    widget::{self, WidgetState},
};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Bootstrap {
    config: AppConfig,
    bot: BotStatus,
    server: ServerStatus,
    credentials: CredentialStatus,
    channels: Vec<ChannelSummary>,
    history: Vec<MediaEvent>,
    pending_media: Vec<PendingMedia>,
    overlay_url: String,
    audio_url: String,
    tts_url: String,
    notification_url: String,
    sticker_url: String,
    ws_url: String,
    invite_url: Option<String>,
    widget: WidgetState,
    notification_widget: NotificationWidgetState,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeStatus {
    bot: BotStatus,
    server: ServerStatus,
    channels: Vec<ChannelSummary>,
    widget: WidgetState,
    notification_widget: NotificationWidgetState,
    pending_media: Vec<PendingMedia>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WidgetBootstrap {
    overlay_url: String,
    locked: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PanelConfig {
    watched_channel_id: String,
    tts_channel_id: String,
    port: u16,
    display_duration_ms: u64,
    gif_duration_ms: u64,
    sticker_duration_ms: u64,
    notification_duration_ms: u64,
    media_volume: u8,
    tts_character_limit: u32,
    tts_queue_limit: u8,
    tts_speech_enabled: bool,
    tts_notifications_obs_enabled: bool,
    bot_online_status: String,
    bot_activity_type: String,
    bot_activity_text: String,
    show_author: bool,
    widget_sound_enabled: bool,
    moderation_enabled: bool,
    moderation_allow_images: bool,
    moderation_allow_videos: bool,
    moderation_allow_audio: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandSettings {
    channel: bool,
    url: bool,
    show: bool,
    status: bool,
    regenerate: bool,
    clear: bool,
    lock: bool,
    changelog: bool,
}

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum OutputTarget {
    MediaObs,
    MediaWidget,
    NotificationObs,
    NotificationWidget,
}

#[tauri::command]
pub async fn get_bootstrap(
    app: AppHandle,
    core: State<'_, Arc<AppCore>>,
) -> Result<Bootstrap, String> {
    build_bootstrap(&app, &core).await.map_err(display_error)
}

#[tauri::command]
pub async fn get_widget_bootstrap(
    core: State<'_, Arc<AppCore>>,
) -> Result<WidgetBootstrap, String> {
    let config = core.config.read().await.clone();
    let secret = load_or_create_relay_secret().map_err(display_error)?;
    Ok(WidgetBootstrap {
        overlay_url: overlay_url(config.port, &secret),
        locked: config.widget_locked,
    })
}

#[tauri::command]
pub async fn get_runtime_status(
    app: AppHandle,
    core: State<'_, Arc<AppCore>>,
) -> Result<RuntimeStatus, String> {
    Ok(RuntimeStatus {
        bot: core.bot_status.read().await.clone(),
        server: core.server_status.read().await.clone(),
        channels: core.channels.read().await.clone(),
        widget: widget::state(&app, &core).await,
        notification_widget: notification_widget::state(&app, &core).await,
        pending_media: core.pending_media.read().await.iter().cloned().collect(),
    })
}

#[tauri::command]
pub async fn refresh_channels(
    core: State<'_, Arc<AppCore>>,
) -> Result<Vec<ChannelSummary>, String> {
    refresh_channel_list(&core).await.map_err(display_error)?;
    Ok(core.channels.read().await.clone())
}

#[tauri::command]
pub async fn set_interface_preferences(
    core: State<'_, Arc<AppCore>>,
    language: String,
    theme: String,
    accent_rgb: [u8; 3],
    font_scale: u8,
) -> Result<(), String> {
    if !matches!(language.as_str(), "en" | "fr" | "es" | "de") {
        return Err("unsupported interface language".into());
    }
    if !matches!(theme.as_str(), "light" | "dark") {
        return Err("unsupported interface theme".into());
    }
    if !(80..=140).contains(&font_scale) {
        return Err("font scale must be between 80 and 140".into());
    }
    let preferences = InterfacePreferences {
        language,
        theme,
        accent_rgb,
        font_scale,
    };
    *core.interface_preferences.write().await = preferences.clone();
    let _ = core.relay_tx.send(RelayEvent::Appearance(preferences));
    Ok(())
}

#[tauri::command]
pub async fn set_output_geometry(
    app: AppHandle,
    core: State<'_, Arc<AppCore>>,
    target: OutputTarget,
    geometry: OutputGeometry,
    width: Option<f64>,
    height: Option<f64>,
    keep_aspect_ratio: Option<bool>,
) -> Result<AppConfig, String> {
    let current = core.config.read().await.clone();
    let media_size = if matches!(target, OutputTarget::MediaWidget)
        && (width.is_some() || height.is_some() || keep_aspect_ratio.is_some())
    {
        let keep_ratio = keep_aspect_ratio.unwrap_or(current.widget_keep_aspect_ratio);
        let (width, height) = widget::clamp_requested_size(
            &app,
            width.unwrap_or(current.widget_width),
            height.unwrap_or(current.widget_height),
            keep_ratio,
        )
        .map_err(display_error)?;
        Some((width, height, keep_ratio))
    } else {
        None
    };
    let notification_size = if matches!(target, OutputTarget::NotificationWidget)
        && (width.is_some() || height.is_some())
    {
        Some(
            notification_widget::clamp_requested_size(
                &app,
                width.unwrap_or(current.notification_widget_width),
                height.unwrap_or(current.notification_widget_height),
                geometry.content_scale,
            )
            .map_err(display_error)?,
        )
    } else {
        None
    };
    let next = core
        .update_config(|config| {
            match target {
                OutputTarget::MediaObs => config.media_obs_geometry = geometry,
                OutputTarget::MediaWidget => config.media_widget_geometry = geometry,
                OutputTarget::NotificationObs => config.notification_obs_geometry = geometry,
                OutputTarget::NotificationWidget => {
                    config.notification_widget_geometry = geometry;
                }
            }
            if let Some((width, height, keep_ratio)) = media_size {
                config.widget_width = width;
                config.widget_height = height;
                config.widget_keep_aspect_ratio = keep_ratio;
            }
            if let Some((width, height)) = notification_size {
                config.notification_widget_width = width;
                config.notification_widget_height = height;
            }
        })
        .await
        .map_err(display_error)?;
    if let Some((width, height, _)) = media_size {
        widget::apply_configured_size(&app, width, height).map_err(display_error)?;
    }
    if let Some((width, height)) = notification_size {
        notification_widget::apply_configured_size(&app, width, height, geometry.content_scale)
            .map_err(display_error)?;
    }
    Ok(next)
}

#[tauri::command]
pub async fn save_credentials(
    app: AppHandle,
    core: State<'_, Arc<AppCore>>,
    client_id: String,
    token: String,
) -> Result<Bootstrap, String> {
    save_discord_credentials(&DiscordCredentials { client_id, token }).map_err(display_error)?;
    start_bot(core.inner().clone())
        .await
        .map_err(display_error)?;
    build_bootstrap(&app, &core).await.map_err(display_error)
}

#[tauri::command]
pub async fn apply_config(
    app: AppHandle,
    core: State<'_, Arc<AppCore>>,
    config: PanelConfig,
) -> Result<Bootstrap, String> {
    let previous = core.config.read().await.clone();
    let next = core
        .update_config(|current| {
            current.watched_channel_id = config.watched_channel_id;
            current.tts_channel_id = config.tts_channel_id;
            current.port = config.port;
            current.display_duration_ms = config.display_duration_ms;
            current.gif_duration_ms = config.gif_duration_ms;
            current.sticker_duration_ms = config.sticker_duration_ms;
            current.notification_duration_ms = config.notification_duration_ms;
            current.media_volume = config.media_volume;
            current.tts_character_limit = config.tts_character_limit;
            current.tts_queue_limit = config.tts_queue_limit;
            current.tts_speech_enabled = config.tts_speech_enabled;
            current.tts_notifications_obs_enabled = config.tts_notifications_obs_enabled;
            current.bot_online_status = config.bot_online_status;
            current.bot_activity_type = config.bot_activity_type;
            current.bot_activity_text = config.bot_activity_text.trim().to_owned();
            current.show_author = config.show_author;
            current.widget_sound_enabled = config.widget_sound_enabled;
            current.moderation_enabled = config.moderation_enabled;
            current.moderation_allow_images = config.moderation_allow_images;
            current.moderation_allow_videos = config.moderation_allow_videos;
            current.moderation_allow_audio = config.moderation_allow_audio;
        })
        .await
        .map_err(display_error)?;
    let port_changed = previous.port != next.port;
    let server_down = !core.server_status.read().await.connected;

    if (port_changed || server_down)
        && let Err(error) = start_server(core.inner().clone()).await
    {
        let _ = core.set_config(previous).await;
        if let Err(rollback_error) = start_server(core.inner().clone()).await {
            core.server_status.write().await.error = Some(rollback_error.to_string());
        }
        return Err(format!("Unable to use the requested local port: {error}"));
    }
    if previous.bot_online_status != next.bot_online_status
        || previous.bot_activity_type != next.bot_activity_type
        || previous.bot_activity_text != next.bot_activity_text
    {
        apply_bot_presence(&core, &next).await;
    }
    widget::refresh(&app, &core).await.map_err(display_error)?;
    notification_widget::refresh(&app, &core)
        .await
        .map_err(display_error)?;
    build_bootstrap(&app, &core).await.map_err(display_error)
}

#[tauri::command]
pub async fn clear_overlay(core: State<'_, Arc<AppCore>>) -> Result<(), String> {
    let _ = core.relay_tx.send(RelayEvent::Clear);
    Ok(())
}

#[tauri::command]
pub async fn save_command_settings(
    core: State<'_, Arc<AppCore>>,
    settings: CommandSettings,
) -> Result<AppConfig, String> {
    core.update_config(|config| {
        config.command_channel_enabled = settings.channel;
        config.command_url_enabled = settings.url;
        config.command_show_enabled = settings.show;
        config.command_status_enabled = settings.status;
        config.command_regenerate_enabled = settings.regenerate;
        config.command_clear_enabled = settings.clear;
        config.command_lock_enabled = settings.lock || config.channel_lock.is_some();
        config.command_changelog_enabled = settings.changelog;
    })
    .await
    .map_err(display_error)
}

#[tauri::command]
pub async fn replay_media(core: State<'_, Arc<AppCore>>, message_id: String) -> Result<(), String> {
    if message_id.is_empty()
        || message_id.len() > 64
        || !message_id
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '-')
    {
        return Err("Invalid Discord message ID.".into());
    }
    let events = core
        .history
        .read()
        .await
        .iter()
        .filter(|event| event.message_id == message_id)
        .cloned()
        .collect::<Vec<_>>();
    if events.is_empty() {
        return Err("The media is no longer in history.".into());
    }
    for mut event in events.into_iter().rev() {
        if matches!(event.kind, crate::model::MediaKind::Gif) && event.cached_media_id.is_none() {
            let cache_id = format!("{}-replay", event.message_id);
            let bytes =
                match artwork::download_bounded(&event.url, artwork::MAX_EMBED_MEDIA_BYTES).await {
                    Ok(bytes) => Some(bytes),
                    Err(_) if event.proxy_url != event.url => {
                        artwork::download_bounded(&event.proxy_url, artwork::MAX_EMBED_MEDIA_BYTES)
                            .await
                            .ok()
                    }
                    Err(_) => None,
                };
            if let Some(bytes) = bytes {
                core.cache_media(cache_id.clone(), event.content_type.clone(), bytes)
                    .await;
                event.cached_media_id = Some(cache_id);
            }
        }
        let _ = core.relay_tx.send(RelayEvent::Media(event));
    }
    Ok(())
}

#[tauri::command]
pub async fn skip_media(core: State<'_, Arc<AppCore>>) -> Result<(), String> {
    let _ = core.relay_tx.send(RelayEvent::Skip);
    Ok(())
}

#[tauri::command]
pub async fn test_output(
    core: State<'_, Arc<AppCore>>,
    target: OutputTestTarget,
) -> Result<(), String> {
    emit_output_test(&core, target).await.map_err(display_error)
}

async fn emit_output_test(core: &AppCore, target: OutputTestTarget) -> anyhow::Result<()> {
    const TEST_AUTHOR: &str = "Relay test";
    const TEST_AVATAR: &str = "/overlay-assets/relay-radar.png";
    const TEST_AUDIO_ID: &str = "999999999999999998";
    const TEST_TTS_ID: &str = "999999999999999999";

    let author = AuthorIdentity {
        username: TEST_AUTHOR.into(),
        display_avatar_url: TEST_AVATAR.into(),
    };
    let event = match target {
        OutputTestTarget::Visual => OutputTestEvent {
            target,
            media: Some(test_media(MediaKind::Image, "Relay visual test", None)),
            tts: None,
            sticker: None,
        },
        OutputTestTarget::Audio => {
            core.cache_audio(TEST_AUDIO_ID.into(), "audio/wav".into(), silent_wav())
                .await;
            OutputTestEvent {
                target,
                media: Some(test_media(
                    MediaKind::Audio,
                    "Relay audio test",
                    Some(TEST_AUDIO_ID.into()),
                )),
                tts: None,
                sticker: None,
            }
        }
        OutputTestTarget::Tts => {
            core.cache_tts_audio(TEST_TTS_ID.into(), "audio/wav".into(), silent_wav())
                .await;
            OutputTestEvent {
                target,
                media: None,
                tts: Some(TtsEvent {
                    id: TEST_TTS_ID.into(),
                    text: "Relay TTS test".into(),
                    author,
                    guild_tag: None,
                    content_type: "audio/wav".into(),
                    timestamp: 0,
                    visual_only: false,
                    segments: Vec::new(),
                }),
                sticker: None,
            }
        }
        OutputTestTarget::Notification => OutputTestEvent {
            target,
            media: None,
            tts: Some(TtsEvent {
                id: "relay-test-notification".into(),
                text: "Relay notification test".into(),
                author,
                guild_tag: Some(GuildTagIdentity {
                    name: "RE".into(),
                    badge_url: None,
                }),
                content_type: String::new(),
                timestamp: 0,
                visual_only: true,
                segments: vec![VisualSegment {
                    kind: "text".into(),
                    value: "Relay notification test".into(),
                    url: None,
                    animated: false,
                }],
            }),
            sticker: None,
        },
        OutputTestTarget::Sticker => OutputTestEvent {
            target,
            media: None,
            tts: None,
            sticker: Some(StickerEvent {
                id: "relay-test-sticker".into(),
                name: "Relay sticker test".into(),
                format: "png".into(),
                url: TEST_AVATAR.into(),
                cached_media_id: None,
                author,
                timestamp: 0,
                message_id: "relay-test-sticker".into(),
            }),
        },
    };
    let _ = core.relay_tx.send(RelayEvent::TestOutput(Box::new(event)));
    Ok(())
}

fn test_media(kind: MediaKind, filename: &str, audio_id: Option<String>) -> MediaEvent {
    MediaEvent {
        kind,
        url: "/overlay-assets/relay-radar.png".into(),
        proxy_url: "/overlay-assets/relay-radar.png".into(),
        filename: filename.into(),
        content_type: if matches!(kind, MediaKind::Audio) {
            "audio/wav".into()
        } else {
            "image/png".into()
        },
        artwork_id: None,
        audio_id,
        cached_media_id: None,
        title: None,
        artist: None,
        author: AuthorIdentity {
            username: "Relay test".into(),
            display_avatar_url: "/overlay-assets/relay-radar.png".into(),
        },
        timestamp: 0,
        message_id: format!("relay-test-{}", filename.replace(' ', "-")),
    }
}

fn silent_wav() -> Vec<u8> {
    const SAMPLE_RATE: u32 = 8_000;
    const SAMPLE_COUNT: u32 = 1_600;
    const BYTES_PER_SAMPLE: u16 = 2;
    let data_length = SAMPLE_COUNT * u32::from(BYTES_PER_SAMPLE);
    let mut bytes = Vec::with_capacity(44 + data_length as usize);
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&(36 + data_length).to_le_bytes());
    bytes.extend_from_slice(b"WAVEfmt ");
    bytes.extend_from_slice(&16_u32.to_le_bytes());
    bytes.extend_from_slice(&1_u16.to_le_bytes());
    bytes.extend_from_slice(&1_u16.to_le_bytes());
    bytes.extend_from_slice(&SAMPLE_RATE.to_le_bytes());
    bytes.extend_from_slice(&(SAMPLE_RATE * u32::from(BYTES_PER_SAMPLE)).to_le_bytes());
    bytes.extend_from_slice(&BYTES_PER_SAMPLE.to_le_bytes());
    bytes.extend_from_slice(&16_u16.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&data_length.to_le_bytes());
    bytes.resize(44 + data_length as usize, 0);
    bytes
}

#[tauri::command]
pub async fn control_audio(
    core: State<'_, Arc<AppCore>>,
    action: String,
    current_url: Option<String>,
) -> Result<(), String> {
    let (action, media) = match action.as_str() {
        "pause" => (AudioControlAction::Pause, None),
        "resume" => (AudioControlAction::Resume, None),
        "skip" => (AudioControlAction::Skip, None),
        "previous" => {
            let history = core.history.read().await;
            let audio = history
                .iter()
                .filter(|event| matches!(event.kind, crate::model::MediaKind::Audio))
                .collect::<Vec<_>>();
            let current_index = current_url
                .as_deref()
                .and_then(|url| audio.iter().position(|event| event.url == url));
            let previous = current_index
                .and_then(|index| audio.get(index + 1))
                .or_else(|| current_index.is_none().then(|| audio.first()).flatten())
                .ok_or_else(|| "No previous audio is available.".to_string())?;
            (AudioControlAction::Previous, Some((*previous).clone()))
        }
        _ => return Err("Invalid audio control action.".into()),
    };
    let _ = core
        .relay_tx
        .send(RelayEvent::AudioControl(AudioControlEvent {
            action,
            media,
        }));
    Ok(())
}

#[tauri::command]
pub async fn get_media_artwork(
    core: State<'_, Arc<AppCore>>,
    artwork_id: String,
) -> Result<tauri::ipc::Response, String> {
    if artwork_id.is_empty()
        || artwork_id.len() > 64
        || !artwork_id
            .chars()
            .all(|character| character.is_ascii_digit())
    {
        return Err("Invalid media artwork ID.".into());
    }
    let cache = core.media_artwork.read().await;
    let artwork = cache
        .iter()
        .find(|artwork| artwork.id == artwork_id)
        .ok_or_else(|| "The media artwork is no longer available.".to_string())?;
    Ok(tauri::ipc::Response::new(artwork.bytes.to_vec()))
}

#[tauri::command]
pub async fn approve_pending_media(core: State<'_, Arc<AppCore>>, id: u64) -> Result<(), String> {
    core.approve_media(id)
        .await
        .then_some(())
        .ok_or_else(|| "The media is no longer pending.".into())
}

#[tauri::command]
pub async fn reject_pending_media(core: State<'_, Arc<AppCore>>, id: u64) -> Result<(), String> {
    core.reject_media(id)
        .await
        .then_some(())
        .ok_or_else(|| "The media is no longer pending.".into())
}

#[tauri::command]
pub async fn clear_pending_media(core: State<'_, Arc<AppCore>>) -> Result<(), String> {
    core.clear_pending_media().await;
    Ok(())
}

#[tauri::command]
pub async fn regenerate_secret(
    app: AppHandle,
    core: State<'_, Arc<AppCore>>,
) -> Result<Bootstrap, String> {
    start_server(core.inner().clone())
        .await
        .map_err(display_error)?;
    widget::refresh(&app, &core).await.map_err(display_error)?;
    notification_widget::refresh(&app, &core)
        .await
        .map_err(display_error)?;
    build_bootstrap(&app, &core).await.map_err(display_error)
}

#[tauri::command]
pub async fn toggle_widget(
    app: AppHandle,
    core: State<'_, Arc<AppCore>>,
) -> Result<WidgetState, String> {
    widget::toggle(&app, core.inner().clone())
        .await
        .map_err(display_error)
}

#[tauri::command]
pub async fn set_widget_locked(
    app: AppHandle,
    core: State<'_, Arc<AppCore>>,
    locked: bool,
) -> Result<WidgetState, String> {
    widget::set_locked(&app, core.inner().clone(), locked)
        .await
        .map_err(display_error)
}

#[tauri::command]
pub async fn set_notification_widget_visible(
    app: AppHandle,
    core: State<'_, Arc<AppCore>>,
    visible: bool,
) -> Result<NotificationWidgetState, String> {
    notification_widget::set_visible(&app, core.inner().clone(), visible)
        .await
        .map_err(display_error)
}

pub const NOTIFICATION_SOUND_MAX_SECONDS: u64 = 10;
pub const NOTIFICATION_SOUND_MAX_BYTES: u64 = 15 * 1024 * 1024;

#[tauri::command]
pub async fn set_notification_sound_enabled(
    core: State<'_, Arc<AppCore>>,
    enabled: bool,
) -> Result<AppConfig, String> {
    core.update_config(|config| config.notification_sound_enabled = enabled)
        .await
        .map_err(display_error)
}

#[tauri::command]
pub async fn set_notification_sound_obs_enabled(
    core: State<'_, Arc<AppCore>>,
    enabled: bool,
) -> Result<AppConfig, String> {
    core.update_config(|config| config.notification_sound_obs_enabled = enabled)
        .await
        .map_err(display_error)
}

#[tauri::command]
pub async fn pick_notification_sound(
    core: State<'_, Arc<AppCore>>,
) -> Result<Option<AppConfig>, String> {
    let file = rfd::AsyncFileDialog::new()
        .add_filter(
            "Audio",
            &[
                "mp3", "flac", "wav", "ogg", "oga", "opus", "m4a", "aac", "webm", "weba",
            ],
        )
        .pick_file()
        .await;
    let Some(file) = file else {
        return Ok(None);
    };
    let path = file.path().to_path_buf();
    validate_notification_sound(path.clone())
        .await
        .map_err(display_error)?;
    let path_string = path.to_string_lossy().into_owned();
    let config = core
        .update_config(|config| config.notification_sound_path = Some(path_string))
        .await
        .map_err(display_error)?;
    Ok(Some(config))
}

#[tauri::command]
pub async fn clear_notification_sound(core: State<'_, Arc<AppCore>>) -> Result<AppConfig, String> {
    core.update_config(|config| config.notification_sound_path = None)
        .await
        .map_err(display_error)
}

async fn validate_notification_sound(path: std::path::PathBuf) -> anyhow::Result<()> {
    use lofty::prelude::AudioFile;

    let metadata = std::fs::metadata(&path)?;
    if metadata.len() > NOTIFICATION_SOUND_MAX_BYTES {
        anyhow::bail!("The notification sound must stay under 15 MB.");
    }
    let duration = tokio::task::spawn_blocking(move || -> anyhow::Result<std::time::Duration> {
        let tagged = lofty::probe::Probe::open(&path)?.read()?;
        Ok(tagged.properties().duration())
    })
    .await??;
    if duration > std::time::Duration::from_secs(NOTIFICATION_SOUND_MAX_SECONDS) {
        anyhow::bail!(
            "The notification sound must last {NOTIFICATION_SOUND_MAX_SECONDS} seconds or less."
        );
    }
    Ok(())
}

#[tauri::command]
pub async fn set_notification_widget_locked(
    app: AppHandle,
    core: State<'_, Arc<AppCore>>,
    locked: bool,
) -> Result<NotificationWidgetState, String> {
    notification_widget::set_locked(&app, core.inner().clone(), locked)
        .await
        .map_err(display_error)
}

async fn build_bootstrap(app: &AppHandle, core: &Arc<AppCore>) -> anyhow::Result<Bootstrap> {
    let config = core.config.read().await.clone();
    let credentials = credential_status()?;
    let invite_url = credentials.client_id.as_deref().map(invite_url);
    Ok(Bootstrap {
        overlay_url: short_overlay_url(config.port, "medias"),
        audio_url: short_overlay_url(config.port, "audios"),
        tts_url: short_overlay_url(config.port, "tts"),
        notification_url: short_overlay_url(config.port, "notifications"),
        sticker_url: short_overlay_url(config.port, "stickers"),
        ws_url: format!(
            "ws://127.0.0.1:{}/ws?role=panel&token={}",
            config.port, core.panel_token
        ),
        invite_url,
        widget: widget::state(app, core).await,
        notification_widget: notification_widget::state(app, core).await,
        config,
        bot: core.bot_status.read().await.clone(),
        server: core.server_status.read().await.clone(),
        credentials,
        channels: core.channels.read().await.clone(),
        history: core.history.read().await.iter().cloned().collect(),
        pending_media: core.pending_media.read().await.iter().cloned().collect(),
    })
}

fn overlay_url(port: u16, secret: &str) -> String {
    format!("http://127.0.0.1:{port}/overlay?secret={secret}")
}

fn short_overlay_url(port: u16, path: &str) -> String {
    format!("http://127.0.0.1:{port}/{path}")
}

fn display_error(error: impl std::fmt::Display) -> String {
    error.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_a_valid_silent_wav() {
        let wav = silent_wav();
        assert_eq!(&wav[..4], b"RIFF");
        assert_eq!(&wav[8..12], b"WAVE");
        assert_eq!(u32::from_le_bytes(wav[40..44].try_into().unwrap()), 3_200);
    }

    #[tokio::test]
    async fn output_tests_bypass_history_and_cache_their_audio() {
        let directory = tempfile::tempdir().unwrap();
        let core = AppCore::load(directory.path().join("config.json")).unwrap();
        let mut events = core.relay_tx.subscribe();

        emit_output_test(&core, OutputTestTarget::Visual)
            .await
            .unwrap();
        let RelayEvent::TestOutput(visual) = events.recv().await.unwrap() else {
            panic!("expected visual output test");
        };
        assert_eq!(visual.target, OutputTestTarget::Visual);
        assert!(visual.media.is_some());
        assert!(core.history.read().await.is_empty());

        emit_output_test(&core, OutputTestTarget::Audio)
            .await
            .unwrap();
        let RelayEvent::TestOutput(audio) = events.recv().await.unwrap() else {
            panic!("expected audio output test");
        };
        assert_eq!(audio.target, OutputTestTarget::Audio);
        assert_eq!(
            audio.media.and_then(|media| media.audio_id).as_deref(),
            Some("999999999999999998")
        );
        assert_eq!(core.media_audio.read().await.len(), 1);

        emit_output_test(&core, OutputTestTarget::Tts)
            .await
            .unwrap();
        let RelayEvent::TestOutput(tts) = events.recv().await.unwrap() else {
            panic!("expected TTS output test");
        };
        assert_eq!(tts.target, OutputTestTarget::Tts);
        assert_eq!(
            tts.tts.as_ref().map(|event| event.id.as_str()),
            Some("999999999999999999")
        );
        assert_eq!(core.tts_audio.read().await.len(), 1);
        assert!(core.history.read().await.is_empty());
    }
}
