use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};

use crate::{
    artwork,
    bot::{invite_url, refresh_channel_list, start_bot},
    config::AppConfig,
    credentials::{
        CredentialStatus, DiscordCredentials, credential_status, load_or_create_relay_secret,
        save_discord_credentials,
    },
    model::{
        BotStatus, ChannelSummary, InterfacePreferences, MediaEvent, PendingMedia, RelayEvent,
        ServerStatus,
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
    media_volume: u8,
    tts_character_limit: u32,
    tts_queue_limit: u8,
    tts_notifications_obs_enabled: bool,
    show_author: bool,
    moderation_enabled: bool,
    moderation_allow_images: bool,
    moderation_allow_videos: bool,
    moderation_allow_audio: bool,
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
    let next = AppConfig {
        watched_channel_id: config.watched_channel_id,
        tts_channel_id: config.tts_channel_id,
        port: config.port,
        display_duration_ms: config.display_duration_ms,
        gif_duration_ms: config.gif_duration_ms,
        media_volume: config.media_volume,
        tts_character_limit: config.tts_character_limit,
        tts_queue_limit: config.tts_queue_limit,
        tts_notifications_obs_enabled: config.tts_notifications_obs_enabled,
        show_author: config.show_author,
        moderation_enabled: config.moderation_enabled,
        moderation_allow_images: config.moderation_allow_images,
        moderation_allow_videos: config.moderation_allow_videos,
        moderation_allow_audio: config.moderation_allow_audio,
        ..previous.clone()
    };
    let port_changed = previous.port != next.port;
    core.set_config(next).await.map_err(display_error)?;

    if port_changed && let Err(error) = start_server(core.inner().clone()).await {
        let _ = core.set_config(previous).await;
        let _ = start_server(core.inner().clone()).await;
        return Err(format!("Unable to use the requested local port: {error}"));
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
