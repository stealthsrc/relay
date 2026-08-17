use std::{
    sync::{Arc, Mutex},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result};
use axum::{
    Router,
    body::Body,
    extract::{
        Path, Query, State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{Html, IntoResponse, Response},
    routing::get,
};
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use serde_json::json;
use subtle::ConstantTimeEq;
use tokio::{net::TcpListener, sync::oneshot};
use tower_http::set_header::SetResponseHeaderLayer;

use crate::{
    config::{AppConfig, OutputGeometry},
    credentials::load_or_create_relay_secret,
    model::{
        AudioPlaybackState, MediaKind, MusicEndedEvent, OutputConnectionStatus, OutputStatuses,
        RelayEvent, ServerStatus,
    },
    state::{AppCore, ServerRuntime},
};

/// Display-only settings sent to overlay/tts/notification/sticker clients.
/// The full AppConfig (channel IDs, lock snapshots, widget positions) is
/// reserved for the panel role.
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct OverlayConfig {
    port: u16,
    display_duration_ms: u64,
    gif_duration_ms: u64,
    sticker_duration_ms: u64,
    notification_duration_ms: u64,
    media_volume: u8,
    tts_queue_limit: u8,
    tts_speech_enabled: bool,
    tts_notifications_obs_enabled: bool,
    show_author: bool,
    show_media_text_obs: bool,
    show_media_text_widget: bool,
    widget_sound_enabled: bool,
    notification_sound_enabled: bool,
    notification_sound_obs_enabled: bool,
    media_obs_geometry: OutputGeometry,
    media_widget_geometry: OutputGeometry,
    notification_obs_geometry: OutputGeometry,
    notification_widget_geometry: OutputGeometry,
}

impl From<&AppConfig> for OverlayConfig {
    fn from(config: &AppConfig) -> Self {
        Self {
            port: config.port,
            display_duration_ms: config.display_duration_ms,
            gif_duration_ms: config.gif_duration_ms,
            sticker_duration_ms: config.sticker_duration_ms,
            notification_duration_ms: config.notification_duration_ms,
            media_volume: config.media_volume,
            tts_queue_limit: config.tts_queue_limit,
            tts_speech_enabled: config.tts_speech_enabled,
            tts_notifications_obs_enabled: config.tts_notifications_obs_enabled,
            show_author: config.show_author,
            show_media_text_obs: config.show_media_text_obs,
            show_media_text_widget: config.show_media_text_widget,
            widget_sound_enabled: config.widget_sound_enabled,
            notification_sound_enabled: config.notification_sound_enabled,
            notification_sound_obs_enabled: config.notification_sound_obs_enabled,
            media_obs_geometry: config.media_obs_geometry,
            media_widget_geometry: config.media_widget_geometry,
            notification_obs_geometry: config.notification_obs_geometry,
            notification_widget_geometry: config.notification_widget_geometry,
        }
    }
}

const HOST: &str = "127.0.0.1";
const OVERLAY_HTML: &str = include_str!("../../overlay/index.html");
const OVERLAY_CSS: &str = include_str!("../../overlay/overlay.css");
const OVERLAY_JS: &str = include_str!("../../overlay/overlay.js");
const OBS_VISUAL_HTML: &str = include_str!("../../overlay/obs-visual.html");
const OBS_VISUAL_CSS: &str = include_str!("../../overlay/obs-visual.css");
const OBS_AUDIO_HTML: &str = include_str!("../../overlay/obs-audio.html");
const OBS_AUDIO_CSS: &str = include_str!("../../overlay/obs-audio.css");
const RADAR_PNG: &[u8] = include_bytes!("../../gui/assets/relay-radar.png");
const TTS_HTML: &str = include_str!("../../tts/index.html");
const TTS_CSS: &str = include_str!("../../tts/tts.css");
const TTS_JS: &str = include_str!("../../tts/tts.js");
const NOTIFICATIONS_HTML: &str = include_str!("../../notifications/index.html");
const NOTIFICATIONS_CSS: &str = include_str!("../../notifications/notifications.css");
const NOTIFICATIONS_JS: &str = include_str!("../../notifications/notifications.js");
const STICKERS_HTML: &str = include_str!("../../stickers/index.html");
const STICKERS_CSS: &str = include_str!("../../stickers/stickers.css");
const STICKERS_JS: &str = include_str!("../../stickers/stickers.js");

#[derive(Clone)]
struct RelayServerState {
    core: Arc<AppCore>,
    relay_secret: Arc<String>,
    client_shutdown: tokio::sync::broadcast::Sender<()>,
    media_clock_tx: tokio::sync::watch::Sender<MediaClockState>,
    media_clock_counts: Arc<Mutex<MediaClockCounts>>,
    stage_clock_tx: tokio::sync::watch::Sender<StageClockState>,
    stage_clock_counts: Arc<Mutex<StageClockCounts>>,
}

#[derive(Clone, Copy, Default, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct MediaClockState {
    video_busy: bool,
    audio_busy: bool,
}

#[derive(Default)]
struct MediaClockCounts {
    video_busy: usize,
    audio_busy: usize,
}

/// Cross-output stage lock so media / YouTube and TTS never overlap.
#[derive(Clone, Copy, Default, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct StageClockState {
    media_busy: bool,
    /// Authoritative YouTube/jukebox occupancy from the server music queue.
    /// Clients must prefer this over local musicPlay/musicIdle flags so a
    /// lagged WebSocket cannot leave TTS/notifications blocked forever.
    music_busy: bool,
    tts_busy: bool,
}

#[derive(Default)]
struct StageClockCounts {
    media_busy: usize,
    tts_busy: usize,
    music_busy: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
enum StageLane {
    Media,
    Tts,
}

#[derive(Debug, Deserialize)]
struct AccessQuery {
    role: Option<String>,
    secret: Option<String>,
    token: Option<String>,
    source: Option<String>,
    client: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OutputSource {
    Visual,
    Audio,
    Tts,
    Notification,
    Sticker,
    All,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OutputClient {
    Obs,
    Widget,
    Preview,
    Probe,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct OutputConnection {
    source: OutputSource,
    client: OutputClient,
}

impl OutputConnection {
    fn is_tracked(self) -> bool {
        !matches!(self.client, OutputClient::Probe)
    }
}

#[derive(Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "camelCase")]
enum OutputClientMessage {
    AudioPlayback(Box<AudioPlaybackState>),
    MusicEnded(MusicEndedEvent),
    MediaClock(MediaClockReport),
    StageClock(StageClockReport),
}

#[derive(Deserialize)]
struct MediaClockReport {
    busy: bool,
}

#[derive(Deserialize)]
struct StageClockReport {
    lane: StageLane,
    busy: bool,
}

pub async fn start_server(core: Arc<AppCore>) -> Result<()> {
    let port = core.config.read().await.port;
    let running_port = core
        .server_runtime
        .lock()
        .await
        .as_ref()
        .map(|runtime| runtime.port);
    // When moving to a different port, bind it before stopping the running
    // server so a failed bind leaves the current server (and its clients) intact.
    let listener = if running_port == Some(port) {
        stop_server(&core).await;
        TcpListener::bind((HOST, port))
            .await
            .with_context(|| format!("failed to bind {HOST}:{port}"))?
    } else {
        let listener = TcpListener::bind((HOST, port))
            .await
            .with_context(|| format!("failed to bind {HOST}:{port}"))?;
        stop_server(&core).await;
        listener
    };
    let (client_shutdown, _) = tokio::sync::broadcast::channel(1);
    let (media_clock_tx, _) = tokio::sync::watch::channel(MediaClockState::default());
    let (stage_clock_tx, _) = tokio::sync::watch::channel(StageClockState::default());
    let state = RelayServerState {
        core: core.clone(),
        relay_secret: Arc::new(load_or_create_relay_secret()?),
        client_shutdown: client_shutdown.clone(),
        media_clock_tx,
        media_clock_counts: Arc::new(Mutex::new(MediaClockCounts::default())),
        stage_clock_tx,
        stage_clock_counts: Arc::new(Mutex::new(StageClockCounts::default())),
    };
    // Keep musicBusy on the stage clock in sync with the server jukebox so
    // lagged overlay/TTS/notification sockets cannot strand musicActive=true.
    let music_clock_state = state.clone();
    let mut music_events = core.relay_tx.subscribe();
    tokio::spawn(async move {
        loop {
            match music_events.recv().await {
                Ok(RelayEvent::MusicPlay(_)) => {
                    set_stage_music_busy(&music_clock_state, true);
                    sync_stage_scheduler(&music_clock_state).await;
                }
                Ok(RelayEvent::MusicIdle) | Ok(RelayEvent::Clear) => {
                    set_stage_music_busy(&music_clock_state, false);
                    sync_stage_scheduler(&music_clock_state).await;
                }
                Ok(RelayEvent::MusicStop(_)) => {
                    // Next event is MusicPlay (still busy) or MusicIdle/Clear.
                }
                Ok(_) => {}
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                    let busy = music_clock_state.core.current_music().await.is_some();
                    set_stage_music_busy(&music_clock_state, busy);
                    sync_stage_scheduler(&music_clock_state).await;
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
            }
        }
    });
    let router = Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .route("/overlay", get(overlay))
        .route("/medias", get(visual_overlay))
        .route("/audios", get(audio_overlay))
        .route("/youtube", get(youtube_overlay))
        .route("/obs/visual", get(obs_visual_page))
        .route("/obs/audio", get(obs_audio_page))
        .route("/obs-assets/obs-visual.css", get(obs_visual_css))
        .route("/obs-assets/obs-audio.css", get(obs_audio_css))
        .route("/overlay-assets/overlay.css", get(overlay_css))
        .route("/overlay-assets/overlay.js", get(overlay_js))
        .route("/overlay-assets/relay-radar.png", get(radar_png))
        .route("/tts", get(tts_page))
        .route("/tts-assets/tts.css", get(tts_css))
        .route("/tts-assets/tts.js", get(tts_js))
        .route("/tts-audio/{id}", get(tts_audio))
        .route("/media-artwork/{id}", get(media_artwork))
        .route("/media-audio/{id}", get(media_audio))
        .route("/media-cache/{id}", get(cached_media))
        .route("/notifications", get(notifications_page))
        .route("/notification-sound", get(notification_sound))
        .route("/stickers", get(stickers_page))
        .route("/sticker-assets/stickers.css", get(stickers_css))
        .route("/sticker-assets/stickers.js", get(stickers_js))
        .route(
            "/notification-assets/notifications.css",
            get(notifications_css),
        )
        .route(
            "/notification-assets/notifications.js",
            get(notifications_js),
        )
        .route("/ws", get(websocket))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            // YouTube IFrame embeds require a referrer (error 150/153 if stripped).
            header::REFERRER_POLICY,
            HeaderValue::from_static("strict-origin-when-cross-origin"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::CONTENT_SECURITY_POLICY,
            // frame-src 'self' allows /obs/* composites to embed legacy short pages.
            HeaderValue::from_static(
                "default-src 'none'; script-src 'self' https://www.youtube.com https://www.youtube-nocookie.com; style-src 'self'; img-src 'self' https://cdn.discordapp.com https://media.discordapp.net https://*.discordapp.net https://*.klipy.com https://i.ytimg.com data:; media-src 'self' https://cdn.discordapp.com https://media.discordapp.net https://*.discordapp.net https://*.klipy.com; connect-src 'self' ws://127.0.0.1:* ws://localhost:* https://www.youtube.com https://www.youtube-nocookie.com https://*.googlevideo.com https://youtubei.googleapis.com; frame-src 'self' https://www.youtube.com https://www.youtube-nocookie.com; frame-ancestors 'self' tauri://localhost http://tauri.localhost",
            ),
        ))
        .with_state(state);
    let (shutdown_tx, shutdown_rx) = oneshot::channel();
    let status_core = core.clone();
    let task = tokio::spawn(async move {
        *status_core.server_status.write().await = ServerStatus {
            connected: true,
            overlay_clients: 0,
            outputs: OutputStatuses::default(),
            error: None,
        };
        let result = axum::serve(listener, router)
            .with_graceful_shutdown(async {
                let _ = shutdown_rx.await;
            })
            .await;
        if let Err(error) = result {
            status_core.server_status.write().await.error = Some(error.to_string());
        }
        status_core.server_status.write().await.connected = false;
    });
    *core.server_runtime.lock().await = Some(ServerRuntime {
        shutdown: shutdown_tx,
        client_shutdown,
        task,
        port,
    });
    Ok(())
}

pub async fn stop_server(core: &Arc<AppCore>) {
    if let Some(runtime) = core.server_runtime.lock().await.take() {
        let _ = runtime.client_shutdown.send(());
        let _ = runtime.shutdown.send(());
        let mut task = runtime.task;
        if tokio::time::timeout(Duration::from_secs(3), &mut task)
            .await
            .is_err()
        {
            task.abort();
        }
    }
    *core.server_status.write().await = ServerStatus::default();
}

async fn root() -> impl IntoResponse {
    (StatusCode::FOUND, [(header::LOCATION, "/overlay")])
}

async fn health() -> impl IntoResponse {
    axum::Json(json!({
        "status": "ok",
    }))
}

async fn overlay(
    State(state): State<RelayServerState>,
    Query(query): Query<AccessQuery>,
) -> Response {
    if !secret_matches(query.secret.as_deref(), &state.relay_secret) {
        return (StatusCode::UNAUTHORIZED, "Invalid relay secret.").into_response();
    }
    Html(OVERLAY_HTML).into_response()
}

async fn visual_overlay(State(state): State<RelayServerState>) -> Response {
    short_overlay(&state, "visual")
}

async fn audio_overlay(State(state): State<RelayServerState>) -> Response {
    short_overlay(&state, "audio")
}

async fn youtube_overlay(State(state): State<RelayServerState>, headers: HeaderMap) -> Response {
    // Existing OBS sources may still point at 127.0.0.1; YouTube rejects that Referer.
    if let Some(response) = redirect_path_off_loopback_ip(&headers, "/youtube") {
        return response;
    }
    short_overlay(&state, "youtube")
}

async fn obs_visual_page(State(state): State<RelayServerState>, headers: HeaderMap) -> Response {
    // Composite embeds /youtube; parent must also be localhost for a valid Referer chain.
    if let Some(response) = redirect_path_off_loopback_ip(&headers, "/obs/visual") {
        return response;
    }
    short_page(&state, OBS_VISUAL_HTML)
}

async fn obs_audio_page(State(state): State<RelayServerState>) -> Response {
    short_page(&state, OBS_AUDIO_HTML)
}

async fn obs_visual_css() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/css; charset=utf-8")],
        OBS_VISUAL_CSS,
    )
}

async fn obs_audio_css() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/css; charset=utf-8")],
        OBS_AUDIO_CSS,
    )
}

/// YouTube error 150 rejects embeds whose page Referer is `http://127.0.0.1`.
/// Serve YouTube-bearing pages only via `http://localhost` (same loopback, accepted Referer).
fn redirect_path_off_loopback_ip(headers: &HeaderMap, path: &str) -> Option<Response> {
    let host = headers.get(header::HOST)?.to_str().ok()?;
    let (hostname, port) = match host.split_once(':') {
        Some((hostname, port)) => (hostname, Some(port)),
        None => (host, None),
    };
    if !hostname.eq_ignore_ascii_case("127.0.0.1") {
        return None;
    }
    let embed_host = crate::widget::youtube_embed_host();
    let location = match port.filter(|value| !value.is_empty()) {
        Some(port) => format!("http://{embed_host}:{port}{path}"),
        None => format!("http://{embed_host}{path}"),
    };
    Some(
        (
            StatusCode::TEMPORARY_REDIRECT,
            [(header::LOCATION, location)],
        )
            .into_response(),
    )
}

fn short_overlay(state: &RelayServerState, mode: &str) -> Response {
    let html = OVERLAY_HTML.replace(
        "<meta name=\"relay-mode\" content=\"all\">",
        &format!("<meta name=\"relay-mode\" content=\"{mode}\">"),
    );
    short_page(state, html)
}

/// Short OBS URLs have no query secret. The page embeds the secret in a meta
/// tag so overlay JS can authenticate WS/media requests without a host-wide
/// cookie (cookies on 127.0.0.1 are not port-scoped). A Max-Age=0 Set-Cookie
/// expires any leftover `relay_secret` cookie from earlier builds.
fn short_page(state: &RelayServerState, html: impl AsRef<str>) -> Response {
    Response::builder()
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .header(
            header::SET_COOKIE,
            "relay_secret=; Path=/; HttpOnly; SameSite=Strict; Max-Age=0",
        )
        .body(inject_relay_secret(html.as_ref(), &state.relay_secret).into())
        .expect("valid short overlay response")
}

fn inject_relay_secret(html: &str, secret: &str) -> String {
    if !secret.bytes().all(|byte| byte.is_ascii_hexdigit())
        || html.contains("name=\"relay-secret\"")
    {
        return html.to_owned();
    }
    let meta = format!("<meta name=\"relay-secret\" content=\"{secret}\">");
    if let Some(index) = html.find("</head>") {
        let mut injected = String::with_capacity(html.len() + meta.len());
        injected.push_str(&html[..index]);
        injected.push_str(&meta);
        injected.push_str(&html[index..]);
        injected
    } else {
        format!("{meta}{html}")
    }
}

async fn notification_sound(
    State(state): State<RelayServerState>,
    Query(query): Query<AccessQuery>,
    headers: HeaderMap,
) -> Response {
    if !request_secret_matches(query.secret.as_deref(), &headers, &state.relay_secret) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let Some(path) = state
        .core
        .config
        .read()
        .await
        .notification_sound_path
        .clone()
    else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let bytes = tokio::task::spawn_blocking(move || -> anyhow::Result<(Vec<u8>, String)> {
        let metadata = std::fs::metadata(&path)?;
        if metadata.len() > crate::commands::NOTIFICATION_SOUND_MAX_BYTES {
            anyhow::bail!("notification sound exceeds the size limit");
        }
        let bytes = std::fs::read(&path)?;
        let extension = std::path::Path::new(&path)
            .extension()
            .and_then(|extension| extension.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();
        let content_type = match extension.as_str() {
            "mp3" => "audio/mpeg",
            "flac" => "audio/flac",
            "wav" => "audio/wav",
            "ogg" | "oga" | "opus" => "audio/ogg",
            "m4a" => "audio/mp4",
            "aac" => "audio/aac",
            "webm" | "weba" => "audio/webm",
            _ => "application/octet-stream",
        };
        Ok((bytes, content_type.to_owned()))
    })
    .await;
    let Ok(Ok((bytes, content_type))) = bytes else {
        return StatusCode::NOT_FOUND.into_response();
    };
    Response::builder()
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CACHE_CONTROL, "private, no-store")
        .body(Body::from(bytes))
        .expect("valid notification sound response")
}

async fn overlay_css() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/css; charset=utf-8")],
        OVERLAY_CSS,
    )
}

async fn overlay_js() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/javascript; charset=utf-8")],
        OVERLAY_JS,
    )
}

async fn radar_png() -> impl IntoResponse {
    ([(header::CONTENT_TYPE, "image/png")], RADAR_PNG)
}

async fn tts_page(
    State(state): State<RelayServerState>,
    Query(query): Query<AccessQuery>,
) -> Response {
    if query.secret.is_some() && !secret_matches(query.secret.as_deref(), &state.relay_secret) {
        return (StatusCode::UNAUTHORIZED, "Invalid relay secret.").into_response();
    }
    short_page(&state, TTS_HTML)
}

async fn tts_css() -> impl IntoResponse {
    ([(header::CONTENT_TYPE, "text/css; charset=utf-8")], TTS_CSS)
}

async fn tts_js() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/javascript; charset=utf-8")],
        TTS_JS,
    )
}

async fn notifications_page(
    State(state): State<RelayServerState>,
    Query(query): Query<AccessQuery>,
) -> Response {
    if query.secret.is_some() && !secret_matches(query.secret.as_deref(), &state.relay_secret) {
        return (StatusCode::UNAUTHORIZED, "Invalid relay secret.").into_response();
    }
    short_page(&state, NOTIFICATIONS_HTML)
}

async fn notifications_css() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/css; charset=utf-8")],
        NOTIFICATIONS_CSS,
    )
}

async fn notifications_js() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/javascript; charset=utf-8")],
        NOTIFICATIONS_JS,
    )
}

async fn stickers_page(
    State(state): State<RelayServerState>,
    Query(query): Query<AccessQuery>,
) -> Response {
    if query.secret.is_some() && !secret_matches(query.secret.as_deref(), &state.relay_secret) {
        return (StatusCode::UNAUTHORIZED, "Invalid relay secret.").into_response();
    }
    short_page(&state, STICKERS_HTML)
}

async fn stickers_css() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/css; charset=utf-8")],
        STICKERS_CSS,
    )
}

async fn stickers_js() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/javascript; charset=utf-8")],
        STICKERS_JS,
    )
}

async fn tts_audio(
    Path(id): Path<String>,
    State(state): State<RelayServerState>,
    Query(query): Query<AccessQuery>,
    headers: HeaderMap,
) -> Response {
    if !request_secret_matches(query.secret.as_deref(), &headers, &state.relay_secret)
        || id.is_empty()
        || id.len() > 20
        || !id.chars().all(|character| character.is_ascii_digit())
    {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let audio = state
        .core
        .tts_audio
        .read()
        .await
        .iter()
        .find(|item| item.id == id)
        .cloned();
    let Some(audio) = audio else {
        return StatusCode::NOT_FOUND.into_response();
    };
    Response::builder()
        .header(header::CONTENT_TYPE, audio.content_type)
        .header(header::CACHE_CONTROL, "private, no-store")
        .body(Body::from(audio.bytes))
        .expect("valid TTS audio response")
}

async fn media_artwork(
    Path(id): Path<String>,
    State(state): State<RelayServerState>,
    Query(query): Query<AccessQuery>,
    headers: HeaderMap,
) -> Response {
    if !request_secret_matches(query.secret.as_deref(), &headers, &state.relay_secret)
        || id.is_empty()
        || id.len() > 20
        || !id.chars().all(|character| character.is_ascii_digit())
    {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let artwork = state
        .core
        .media_artwork
        .read()
        .await
        .iter()
        .find(|item| item.id == id)
        .cloned();
    let Some(artwork) = artwork else {
        return StatusCode::NOT_FOUND.into_response();
    };
    Response::builder()
        .header(header::CONTENT_TYPE, artwork.content_type)
        .header(header::CACHE_CONTROL, "private, no-store")
        .body(Body::from(artwork.bytes))
        .expect("valid media artwork response")
}

async fn media_audio(
    Path(id): Path<String>,
    State(state): State<RelayServerState>,
    Query(query): Query<AccessQuery>,
    headers: HeaderMap,
) -> Response {
    if !request_secret_matches(query.secret.as_deref(), &headers, &state.relay_secret)
        || id.is_empty()
        || id.len() > 20
        || !id.chars().all(|character| character.is_ascii_digit())
    {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let audio = state
        .core
        .media_audio
        .read()
        .await
        .iter()
        .find(|item| item.id == id)
        .cloned();
    let Some(audio) = audio else {
        return StatusCode::NOT_FOUND.into_response();
    };
    Response::builder()
        .header(header::CONTENT_TYPE, audio.content_type)
        .header(header::CACHE_CONTROL, "private, no-store")
        .body(Body::from(audio.bytes))
        .expect("valid media audio response")
}

async fn cached_media(
    Path(id): Path<String>,
    State(state): State<RelayServerState>,
    Query(query): Query<AccessQuery>,
    headers: HeaderMap,
) -> Response {
    let authorized = request_secret_matches(query.secret.as_deref(), &headers, &state.relay_secret)
        || secret_matches(query.token.as_deref(), &state.core.panel_token);
    if !authorized
        || id.is_empty()
        || id.len() > 64
        || !id
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '-')
    {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let media = state
        .core
        .cached_media
        .read()
        .await
        .iter()
        .find(|item| item.id == id)
        .cloned();
    let Some(media) = media else {
        return StatusCode::NOT_FOUND.into_response();
    };
    ranged_media_response(media, headers.get(header::RANGE))
}

fn ranged_media_response(
    media: crate::state::CachedMedia,
    range: Option<&HeaderValue>,
) -> Response {
    let total = media.bytes.len();
    let requested = range
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("bytes="))
        .and_then(|value| value.split_once('-'))
        .and_then(|(start, end)| {
            let start = start.parse::<usize>().ok()?;
            let end = if end.is_empty() {
                total.checked_sub(1)?
            } else {
                end.parse::<usize>().ok()?.min(total.checked_sub(1)?)
            };
            (start <= end && start < total).then_some((start, end))
        });
    let (status, body, content_range) = if let Some((start, end)) = requested {
        (
            StatusCode::PARTIAL_CONTENT,
            media.bytes.slice(start..=end),
            Some(format!("bytes {start}-{end}/{total}")),
        )
    } else {
        (StatusCode::OK, media.bytes, None)
    };
    let mut response = Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, media.content_type)
        .header(header::ACCEPT_RANGES, "bytes")
        .header(header::CACHE_CONTROL, "private, no-store")
        .header(header::CONTENT_LENGTH, body.len());
    if let Some(content_range) = content_range {
        response = response.header(header::CONTENT_RANGE, content_range);
    }
    response
        .body(Body::from(body))
        .expect("valid cached media response")
}

async fn websocket(
    upgrade: WebSocketUpgrade,
    State(state): State<RelayServerState>,
    Query(query): Query<AccessQuery>,
    headers: HeaderMap,
) -> Response {
    if !origin_allowed(headers.get(header::ORIGIN)) {
        return StatusCode::FORBIDDEN.into_response();
    }
    let role = query.role.as_deref().unwrap_or("overlay");
    let authorized = match role {
        "overlay" | "tts" | "notification" | "sticker" => {
            request_secret_matches(query.secret.as_deref(), &headers, &state.relay_secret)
        }
        "panel" => secret_matches(query.token.as_deref(), &state.core.panel_token),
        _ => false,
    };
    if !authorized {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    let output = match role {
        "overlay" | "tts" | "notification" | "sticker" => {
            let Some(output) = output_connection(role, &query) else {
                return StatusCode::BAD_REQUEST.into_response();
            };
            Some(output)
        }
        "panel" => None,
        _ => return StatusCode::BAD_REQUEST.into_response(),
    };
    upgrade.on_upgrade(move |socket| handle_socket(socket, state, output))
}

async fn handle_socket(
    socket: WebSocket,
    state: RelayServerState,
    output: Option<OutputConnection>,
) {
    let is_output = output.is_some();
    let tracked_clock_source = match output {
        Some(OutputConnection {
            source: OutputSource::Visual,
            client: OutputClient::Obs,
        }) => Some(OutputSource::Visual),
        Some(OutputConnection {
            source: OutputSource::Audio,
            client: OutputClient::Obs,
        }) => Some(OutputSource::Audio),
        _ => None,
    };
    let receives_music = output_receives_music(output);
    let receives_media_clock = tracked_clock_source.is_some();
    let receives_stage_clock = output_receives_stage_clock(output);
    let reports_stage_clock = output_reports_stage_clock(output);
    let mut reported_busy = false;
    let mut reported_stage_lane: Option<StageLane> = None;
    let mut relay_rx = state.core.relay_tx.subscribe();
    let mut media_clock_rx = state.media_clock_tx.subscribe();
    let mut stage_clock_rx = state.stage_clock_tx.subscribe();
    if let Some(output) = output {
        update_output_connection(&state, output, 1).await;
    }
    let (mut sender, mut receiver) = socket.split();
    let config = state.core.config.read().await.clone();
    let config_payload = if is_output {
        json!(OverlayConfig::from(&config))
    } else {
        json!(config)
    };
    if send_json(
        &mut sender,
        &json!({ "type": "config", "payload": config_payload }),
    )
    .await
    .is_err()
    {
        if let Some(output) = output {
            update_output_connection(&state, output, -1).await;
        }
        return;
    }
    let appearance = state.core.interface_preferences.read().await.clone();
    if send_json(
        &mut sender,
        &json!({ "type": "appearance", "payload": appearance }),
    )
    .await
    .is_err()
    {
        if let Some(output) = output {
            update_output_connection(&state, output, -1).await;
        }
        return;
    }
    if !is_output {
        let history = state.core.history.read().await.clone();
        let _ = send_json(
            &mut sender,
            &json!({ "type": "history", "payload": history }),
        )
        .await;
    } else if receives_media_clock
        && send_json(
            &mut sender,
            &json!({ "type": "mediaClock", "payload": *media_clock_rx.borrow_and_update() }),
        )
        .await
        .is_err()
    {
        if let Some(output) = output {
            update_output_connection(&state, output, -1).await;
        }
        return;
    }

    if receives_stage_clock
        && send_json(
            &mut sender,
            &json!({ "type": "stageClock", "payload": *stage_clock_rx.borrow_and_update() }),
        )
        .await
        .is_err()
    {
        if let Some(output) = output {
            update_output_connection(&state, output, -1).await;
        }
        return;
    }

    if receives_music
        && let Some(playback) = state.core.current_music().await
        && send_json(&mut sender, &json!(RelayEvent::MusicPlay(playback)))
            .await
            .is_err()
    {
        if let Some(output) = output {
            update_output_connection(&state, output, -1).await;
        }
        return;
    }

    let mut shutdown_rx = state.client_shutdown.subscribe();
    loop {
        tokio::select! {
            _ = shutdown_rx.recv() => {
                let _ = sender.send(Message::Close(None)).await;
                break;
            }
            event = relay_rx.recv() => {
                match event {
                    Ok(RelayEvent::Config(config)) if is_output => {
                        let payload = json!({ "type": "config", "payload": OverlayConfig::from(config.as_ref()) });
                        if send_json(&mut sender, &payload).await.is_err() { break; }
                    }
                    Ok(event) => {
                        if send_json(&mut sender, &event).await.is_err() { break; }
                    }
                    // A lagging client only misses events; keep the socket
                    // alive instead of interrupting the media it displays.
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {}
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                }
            }
            changed = media_clock_rx.changed(), if receives_media_clock => {
                if changed.is_err() { break; }
                let clock = *media_clock_rx.borrow_and_update();
                if send_json(&mut sender, &json!({ "type": "mediaClock", "payload": clock })).await.is_err() { break; }
            }
            changed = stage_clock_rx.changed(), if receives_stage_clock => {
                if changed.is_err() { break; }
                let clock = *stage_clock_rx.borrow_and_update();
                if send_json(&mut sender, &json!({ "type": "stageClock", "payload": clock })).await.is_err() { break; }
            }
            incoming = receiver.next() => {
                match incoming {
                    Some(Ok(Message::Close(_))) | None | Some(Err(_)) => break,
                    Some(Ok(Message::Ping(payload))) => {
                        if sender.send(Message::Pong(payload)).await.is_err() { break; }
                    }
                    Some(Ok(Message::Text(text))) if is_output => {
                        let Ok(message) = serde_json::from_str(text.as_str()) else {
                            let _ = sender.send(Message::Close(None)).await;
                            break;
                        };
                        match message {
                            OutputClientMessage::AudioPlayback(playback)
                                if matches!(playback.media.kind, MediaKind::Audio)
                                    && matches!(playback.target.as_str(), "obs" | "widget") =>
                            {
                                let _ = state
                                    .core
                                    .relay_tx
                                    .send(RelayEvent::AudioPlayback(*playback));
                            }
                            OutputClientMessage::MusicEnded(event) if receives_music => {
                                let _ = state.core.finish_music(&event.playback_id).await;
                            }
                            OutputClientMessage::MediaClock(clock) if tracked_clock_source.is_some() => {
                                let source = tracked_clock_source.expect("tracked clock source");
                                if clock.busy {
                                    let granted = try_acquire_output_lease(
                                        &state,
                                        source,
                                        &mut reported_busy,
                                    );
                                    let clock = media_clock_state(&state);
                                    if send_json(
                                        &mut sender,
                                        &json!({
                                            "type": "mediaGrant",
                                            "payload": { "granted": granted, "clock": clock },
                                        }),
                                    )
                                    .await
                                    .is_err()
                                    {
                                        break;
                                    }
                                } else {
                                    release_output_lease(&state, source, &mut reported_busy);
                                }
                            }
                            OutputClientMessage::StageClock(report) if reports_stage_clock => {
                                let granted = apply_stage_clock_report(
                                    &state,
                                    report.lane,
                                    report.busy,
                                    &mut reported_stage_lane,
                                );
                                sync_stage_scheduler(&state).await;
                                // Rejected busy claims do not change the watch value, so echo
                                // the clock to this claimant so pending UI can clear/retry.
                                if report.busy && !granted {
                                    let clock = stage_clock_state(&state);
                                    if send_json(
                                        &mut sender,
                                        &json!({
                                            "type": "stageClock",
                                            "payload": {
                                                "mediaBusy": clock.media_busy,
                                                "musicBusy": clock.music_busy,
                                                "ttsBusy": clock.tts_busy,
                                                "granted": false,
                                                "lane": match report.lane {
                                                    StageLane::Media => "media",
                                                    StageLane::Tts => "tts",
                                                },
                                            }
                                        }),
                                    )
                                    .await
                                    .is_err()
                                    {
                                        break;
                                    }
                                }
                            }
                            _ => {
                                let _ = sender.send(Message::Close(None)).await;
                                break;
                            }
                        }
                    }
                    Some(Ok(_)) => {
                        let _ = sender.send(Message::Close(None)).await;
                        break;
                    }
                }
            }
        }
    }
    if let Some(source) = tracked_clock_source {
        release_output_lease(&state, source, &mut reported_busy);
    }
    if let Some(lane) = reported_stage_lane {
        apply_stage_clock_report(&state, lane, false, &mut reported_stage_lane);
        sync_stage_scheduler(&state).await;
    }
    if let Some(output) = output {
        update_output_connection(&state, output, -1).await;
    }
}

async fn send_json<T: serde::Serialize>(
    sender: &mut futures_util::stream::SplitSink<WebSocket, Message>,
    value: &T,
) -> Result<(), axum::Error> {
    let serialized = serde_json::to_string(value).expect("serializable server event");
    sender.send(Message::Text(serialized.into())).await
}

fn output_connection(role: &str, query: &AccessQuery) -> Option<OutputConnection> {
    let source = match (role, query.source.as_deref()) {
        ("overlay", None | Some("all")) => OutputSource::All,
        ("overlay", Some("visual")) => OutputSource::Visual,
        ("overlay", Some("audio") | Some("youtube")) => OutputSource::Audio,
        ("tts", None | Some("tts")) => OutputSource::Tts,
        ("notification", None | Some("notification")) => OutputSource::Notification,
        ("sticker", None | Some("sticker")) => OutputSource::Sticker,
        _ => return None,
    };
    let client = match query.client.as_deref().unwrap_or("obs") {
        "obs" => OutputClient::Obs,
        "widget" => OutputClient::Widget,
        "preview" => OutputClient::Preview,
        "probe" => OutputClient::Probe,
        _ => return None,
    };
    if matches!(client, OutputClient::Widget | OutputClient::Preview)
        && !matches!(
            source,
            OutputSource::Visual | OutputSource::Notification | OutputSource::All
        )
    {
        return None;
    }
    Some(OutputConnection { source, client })
}

fn output_receives_music(output: Option<OutputConnection>) -> bool {
    matches!(
        output,
        Some(OutputConnection {
            // Visual OBS (/medias) needs MusicPlay/Idle hints so GIFs wait for
            // the jukebox even before the stageClock watch updates arrive.
            source: OutputSource::Audio
                | OutputSource::Visual
                | OutputSource::All
                | OutputSource::Notification
                | OutputSource::Tts,
            client: OutputClient::Obs | OutputClient::Widget,
        })
    )
}

fn output_receives_stage_clock(output: Option<OutputConnection>) -> bool {
    matches!(
        output,
        Some(OutputConnection {
            source: OutputSource::Visual
                | OutputSource::Audio
                | OutputSource::All
                | OutputSource::Tts
                | OutputSource::Notification
                | OutputSource::Sticker,
            client: OutputClient::Obs | OutputClient::Widget,
        })
    )
}

fn output_reports_stage_clock(output: Option<OutputConnection>) -> bool {
    output_receives_stage_clock(output)
}

async fn update_output_connection(
    state: &RelayServerState,
    connection: OutputConnection,
    delta: isize,
) {
    if !connection.is_tracked() {
        return;
    }
    let mut status = state.core.server_status.write().await;
    if matches!(connection.client, OutputClient::Obs) {
        adjust_count(&mut status.overlay_clients, delta);
    }
    for source in output_sources(connection.source) {
        let output = output_status_mut(&mut status.outputs, *source);
        let count = match connection.client {
            OutputClient::Obs => &mut output.obs_clients,
            OutputClient::Widget => &mut output.widget_clients,
            OutputClient::Preview => &mut output.preview_clients,
            OutputClient::Probe => continue,
        };
        adjust_count(count, delta);
        if delta > 0 {
            output.last_connected_at = Some(current_timestamp_ms());
        }
    }
}

fn try_acquire_output_lease(
    state: &RelayServerState,
    source: OutputSource,
    reported_busy: &mut bool,
) -> bool {
    if *reported_busy {
        return true;
    }
    let mut counts = state
        .media_clock_counts
        .lock()
        .expect("media clock mutex poisoned");
    let blocked = match source {
        OutputSource::Visual => counts.audio_busy > 0,
        OutputSource::Audio => counts.video_busy > 0,
        _ => return false,
    };
    if blocked {
        return false;
    }
    match source {
        OutputSource::Visual => counts.video_busy = counts.video_busy.saturating_add(1),
        OutputSource::Audio => counts.audio_busy = counts.audio_busy.saturating_add(1),
        _ => unreachable!("only split media outputs can acquire a lease"),
    }
    *reported_busy = true;
    drop(counts);
    broadcast_media_clock(state);
    true
}

fn release_output_lease(state: &RelayServerState, source: OutputSource, reported_busy: &mut bool) {
    if !*reported_busy {
        return;
    }
    let mut counts = state
        .media_clock_counts
        .lock()
        .expect("media clock mutex poisoned");
    match source {
        OutputSource::Visual => adjust_count(&mut counts.video_busy, -1),
        OutputSource::Audio => adjust_count(&mut counts.audio_busy, -1),
        _ => return,
    }
    *reported_busy = false;
    drop(counts);
    broadcast_media_clock(state);
}

fn media_clock_state(state: &RelayServerState) -> MediaClockState {
    let counts = state
        .media_clock_counts
        .lock()
        .expect("media clock mutex poisoned");
    MediaClockState {
        video_busy: counts.video_busy > 0,
        audio_busy: counts.audio_busy > 0,
    }
}

fn broadcast_media_clock(state: &RelayServerState) {
    let next = media_clock_state(state);
    state.media_clock_tx.send_if_modified(|current| {
        let changed = *current != next;
        *current = next;
        changed
    });
}

fn stage_clock_state(state: &RelayServerState) -> StageClockState {
    let counts = state
        .stage_clock_counts
        .lock()
        .expect("stage clock mutex poisoned");
    StageClockState {
        media_busy: counts.media_busy > 0,
        music_busy: counts.music_busy,
        tts_busy: counts.tts_busy > 0,
    }
}

async fn sync_stage_scheduler(state: &RelayServerState) {
    let clock = stage_clock_state(state);
    state
        .core
        .stage_scheduler
        .stage_state(clock.media_busy, clock.music_busy, clock.tts_busy)
        .await;
}

fn broadcast_stage_clock(state: &RelayServerState) {
    let next = stage_clock_state(state);
    // Every ownership change must wake claimants. The public booleans can stay
    // identical when OBS and the Windows widget overlap on the same media lane.
    state.stage_clock_tx.send_replace(next);
}

fn set_stage_music_busy(state: &RelayServerState, busy: bool) {
    let changed = {
        let mut counts = state
            .stage_clock_counts
            .lock()
            .expect("stage clock mutex poisoned");
        if counts.music_busy == busy {
            false
        } else {
            counts.music_busy = busy;
            true
        }
    };
    if changed {
        broadcast_stage_clock(state);
    }
}

fn apply_stage_clock_report(
    state: &RelayServerState,
    lane: StageLane,
    busy: bool,
    reported_lane: &mut Option<StageLane>,
) -> bool {
    let mut counts = state
        .stage_clock_counts
        .lock()
        .expect("stage clock mutex poisoned");
    if busy {
        if *reported_lane == Some(lane) {
            return true;
        }
        // Exclusive stage: media, YouTube, and TTS must never overlap.
        let blocked = match lane {
            StageLane::Media => counts.tts_busy > 0 || counts.music_busy,
            StageLane::Tts => counts.media_busy > 0 || counts.music_busy,
        };
        if blocked {
            return false;
        }
        if let Some(previous) = reported_lane.take() {
            match previous {
                StageLane::Media => adjust_count(&mut counts.media_busy, -1),
                StageLane::Tts => adjust_count(&mut counts.tts_busy, -1),
            }
        }
        match lane {
            StageLane::Media => counts.media_busy = counts.media_busy.saturating_add(1),
            StageLane::Tts => counts.tts_busy = counts.tts_busy.saturating_add(1),
        }
        *reported_lane = Some(lane);
    } else {
        let Some(previous) = *reported_lane else {
            return true;
        };
        if previous != lane {
            return true;
        }
        match previous {
            StageLane::Media => adjust_count(&mut counts.media_busy, -1),
            StageLane::Tts => adjust_count(&mut counts.tts_busy, -1),
        }
        *reported_lane = None;
    }
    drop(counts);
    broadcast_stage_clock(state);
    true
}

fn output_sources(source: OutputSource) -> &'static [OutputSource] {
    match source {
        OutputSource::All => &[OutputSource::Visual, OutputSource::Audio],
        OutputSource::Visual => &[OutputSource::Visual],
        OutputSource::Audio => &[OutputSource::Audio],
        OutputSource::Tts => &[OutputSource::Tts],
        OutputSource::Notification => &[OutputSource::Notification],
        OutputSource::Sticker => &[OutputSource::Sticker],
    }
}

fn output_status_mut(
    statuses: &mut OutputStatuses,
    source: OutputSource,
) -> &mut OutputConnectionStatus {
    match source {
        OutputSource::Visual => &mut statuses.visual,
        OutputSource::Audio => &mut statuses.audio,
        OutputSource::Tts => &mut statuses.tts,
        OutputSource::Notification => &mut statuses.notification,
        OutputSource::Sticker => &mut statuses.sticker,
        OutputSource::All => unreachable!("combined output sources are expanded before tracking"),
    }
}

fn adjust_count(count: &mut usize, delta: isize) {
    if delta > 0 {
        *count = count.saturating_add(delta as usize);
    } else {
        *count = count.saturating_sub((-delta) as usize);
    }
}

fn current_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn secret_matches(candidate: Option<&str>, expected: &str) -> bool {
    candidate.is_some_and(|candidate| {
        candidate.len() == expected.len()
            && bool::from(candidate.as_bytes().ct_eq(expected.as_bytes()))
    })
}

fn request_secret_matches(candidate: Option<&str>, headers: &HeaderMap, expected: &str) -> bool {
    secret_matches(candidate, expected)
        || secret_matches(cookie_value(headers, "relay_secret"), expected)
}

fn cookie_value<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers
        .get(header::COOKIE)?
        .to_str()
        .ok()?
        .split(';')
        .filter_map(|part| part.trim().split_once('='))
        .find_map(|(key, value)| (key == name).then_some(value))
}

fn origin_allowed(origin: Option<&HeaderValue>) -> bool {
    let Some(origin) = origin.and_then(|value| value.to_str().ok()) else {
        return true;
    };
    origin.starts_with("http://127.0.0.1:")
        || origin.starts_with("http://localhost:")
        || origin == "tauri://localhost"
        || origin == "http://tauri.localhost"
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::{Read, Write};

    use crate::{
        config::AppConfig,
        credentials::load_or_create_relay_secret,
        model::{AuthorIdentity, MediaEvent, MediaKind, RelayEvent},
        state::{CachedMedia, MediaArtwork, MediaAudio, TtsAudio},
    };

    #[test]
    fn injects_hex_secret_into_html_head() {
        let html = "<html><head></head><body></body></html>";
        assert_eq!(
            inject_relay_secret(html, "abc123"),
            "<html><head><meta name=\"relay-secret\" content=\"abc123\"></head><body></body></html>"
        );
        assert_eq!(inject_relay_secret(html, "not hex!"), html);
        let already = "<html><head><meta name=\"relay-secret\" content=\"abc123\"></head></html>";
        assert_eq!(inject_relay_secret(already, "abc123"), already);
    }

    #[test]
    fn compares_secrets_without_prefix_matches() {
        assert!(secret_matches(Some("private"), "private"));
        assert!(!secret_matches(Some("priv"), "private"));
        assert!(!secret_matches(None, "private"));

        let mut headers = HeaderMap::new();
        headers.insert(
            header::COOKIE,
            HeaderValue::from_static("theme=dark; relay_secret=private"),
        );
        assert!(request_secret_matches(None, &headers, "private"));
    }

    #[test]
    fn accepts_only_local_overlay_and_tauri_origins() {
        assert!(origin_allowed(None));
        assert!(origin_allowed(Some(&HeaderValue::from_static(
            "http://127.0.0.1:4590"
        ))));
        assert!(origin_allowed(Some(&HeaderValue::from_static(
            "http://localhost:4590"
        ))));
        assert!(origin_allowed(Some(&HeaderValue::from_static(
            "http://tauri.localhost"
        ))));
        assert!(!origin_allowed(Some(&HeaderValue::from_static(
            "https://example.com"
        ))));
    }

    #[test]
    fn youtube_widget_pages_use_localhost_referrer_host() {
        // YouTube rejects embeds whose Referer is http://127.0.0.1 (error 150)
        // but accepts http://localhost on the same loopback interface.
        assert!(crate::widget::youtube_embed_host().starts_with("localhost"));
        assert_ne!(crate::widget::youtube_embed_host(), "127.0.0.1");
    }

    #[test]
    fn youtube_loopback_ip_redirects_to_localhost() {
        let mut headers = HeaderMap::new();
        headers.insert(header::HOST, HeaderValue::from_static("127.0.0.1:4590"));
        let response = redirect_path_off_loopback_ip(&headers, "/youtube").expect("redirect");
        assert_eq!(response.status(), StatusCode::TEMPORARY_REDIRECT);
        assert_eq!(
            response.headers().get(header::LOCATION).unwrap(),
            "http://localhost:4590/youtube"
        );

        let mut localhost = HeaderMap::new();
        localhost.insert(header::HOST, HeaderValue::from_static("localhost:4590"));
        assert!(redirect_path_off_loopback_ip(&localhost, "/youtube").is_none());
    }

    #[test]
    fn obs_visual_loopback_ip_redirects_to_localhost() {
        let mut headers = HeaderMap::new();
        headers.insert(header::HOST, HeaderValue::from_static("127.0.0.1:4590"));
        let response = redirect_path_off_loopback_ip(&headers, "/obs/visual").expect("redirect");
        assert_eq!(
            response.headers().get(header::LOCATION).unwrap(),
            "http://localhost:4590/obs/visual"
        );
    }

    #[test]
    fn output_config_never_serializes_private_scanner_values() {
        let config = AppConfig {
            privacy_custom_patterns: vec!["private-value-marker".into()],
            privacy_allowlist: vec!["allowlist-value-marker".into()],
            ..AppConfig::default()
        };
        let serialized = serde_json::to_string(&OverlayConfig::from(&config)).unwrap();

        assert!(!serialized.contains("private-value-marker"));
        assert!(!serialized.contains("allowlist-value-marker"));
        assert!(!serialized.contains("privacyCustomPatterns"));
        assert!(!serialized.contains("privacyAllowlist"));
    }

    #[test]
    fn classifies_valid_output_sources_and_client_contexts() {
        let visual_preview =
            output_connection("overlay", &access_query(Some("visual"), Some("preview")))
                .expect("visual preview should be accepted");
        assert_eq!(visual_preview.source, OutputSource::Visual);
        assert_eq!(visual_preview.client, OutputClient::Preview);

        let combined = output_connection("overlay", &access_query(None, None))
            .expect("legacy overlay should remain supported");
        assert_eq!(combined.source, OutputSource::All);
        assert_eq!(combined.client, OutputClient::Obs);
        let widget = output_connection("overlay", &access_query(None, Some("widget")))
            .expect("the Windows widget should be accepted for the combined overlay");
        assert!(output_receives_music(Some(widget)));

        assert!(output_connection("tts", &access_query(Some("audio"), None)).is_none());
        assert!(output_connection("tts", &access_query(None, Some("widget"))).is_none());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn serves_authenticated_overlay_and_broadcasts_under_load() {
        let port = free_local_port();
        let directory = tempfile::tempdir().unwrap();
        let core = AppCore::load(directory.path().join("config.json")).unwrap();
        core.set_config(AppConfig {
            port,
            ..AppConfig::default()
        })
        .await
        .unwrap();
        start_server(core.clone()).await.unwrap();
        let secret = load_or_create_relay_secret().unwrap();

        assert!(http_status(port, "/overlay").starts_with("HTTP/1.1 401"));
        assert!(
            http_status(port, &format!("/overlay?secret={secret}")).starts_with("HTTP/1.1 200")
        );
        let visual_response = http_response(port, "/medias");
        assert!(visual_response.starts_with("HTTP/1.1 200"));
        assert!(visual_response.contains(&format!(
            "<meta name=\"relay-secret\" content=\"{secret}\">"
        )));
        assert!(!visual_response.contains(&format!("relay_secret={secret}")));
        assert!(
            visual_response.to_ascii_lowercase().contains(
                "set-cookie: relay_secret=; path=/; httponly; samesite=strict; max-age=0"
            )
        );
        assert!(visual_response.contains("content=\"visual\""));
        assert!(visual_response.contains("https://*.discordapp.net"));
        assert!(visual_response.contains("https://*.klipy.com"));
        assert!(visual_response.contains("https://i.ytimg.com"));
        let visual_headers = visual_response.to_ascii_lowercase();
        assert!(visual_headers.contains("referrer-policy: strict-origin-when-cross-origin"));
        assert!(!visual_headers.contains("referrer-policy: no-referrer"));
        let audio_response = http_response(port, "/audios");
        assert!(audio_response.starts_with("HTTP/1.1 200"));
        assert!(audio_response.contains("content=\"audio\""));
        // /youtube on 127.0.0.1 must redirect: YouTube rejects that Referer (error 150).
        let youtube_redirect = http_response(port, "/youtube");
        assert!(youtube_redirect.starts_with("HTTP/1.1 307"));
        assert!(youtube_redirect.contains(&format!("location: http://localhost:{port}/youtube")));
        let youtube_response = http_response_with_host(port, "/youtube", "localhost");
        assert!(youtube_response.starts_with("HTTP/1.1 200"));
        assert!(youtube_response.contains("content=\"youtube\""));
        let obs_visual_redirect = http_response(port, "/obs/visual");
        assert!(obs_visual_redirect.starts_with("HTTP/1.1 307"));
        assert!(
            obs_visual_redirect.contains(&format!("location: http://localhost:{port}/obs/visual"))
        );
        let obs_visual = http_response_with_host(port, "/obs/visual", "localhost");
        assert!(obs_visual.starts_with("HTTP/1.1 200"));
        assert!(obs_visual.contains("src=\"/medias\""));
        assert!(obs_visual.contains("src=\"/stickers\""));
        assert!(obs_visual.contains("src=\"/notifications\""));
        assert!(obs_visual.contains("src=\"/youtube\""));
        assert!(obs_visual.contains("allowtransparency=\"true\""));
        assert!(!obs_visual.contains("color-scheme\" content=\"light only\""));
        let obs_audio = http_response(port, "/obs/audio");
        assert!(obs_audio.starts_with("HTTP/1.1 200"));
        assert!(obs_audio.contains("src=\"/audios\""));
        assert!(obs_audio.contains("src=\"/tts\""));
        assert!(http_status(port, "/tts").starts_with("HTTP/1.1 200"));
        assert!(http_status(port, "/tts?secret=wrong").starts_with("HTTP/1.1 401"));
        assert!(http_status(port, &format!("/tts?secret={secret}")).starts_with("HTTP/1.1 200"));
        assert!(http_status(port, "/notifications").starts_with("HTTP/1.1 200"));
        assert!(http_status(port, "/notifications?secret=wrong").starts_with("HTTP/1.1 401"));
        assert!(
            http_status(port, &format!("/notifications?secret={secret}"))
                .starts_with("HTTP/1.1 200")
        );
        core.tts_audio.write().await.push_front(TtsAudio {
            id: "123456789012345678".into(),
            content_type: "audio/wav".into(),
            bytes: axum::body::Bytes::from_static(b"RIFF-test"),
        });
        assert!(http_status(port, "/tts-audio/123456789012345678").starts_with("HTTP/1.1 401"));
        assert!(
            http_status(
                port,
                &format!("/tts-audio/123456789012345678?secret={secret}")
            )
            .starts_with("HTTP/1.1 200")
        );
        core.media_artwork.write().await.push_front(MediaArtwork {
            id: "223456789012345678".into(),
            content_type: "image/png".into(),
            bytes: axum::body::Bytes::from_static(b"PNG-test"),
        });
        assert!(http_status(port, "/media-artwork/223456789012345678").starts_with("HTTP/1.1 401"));
        assert!(
            http_status(
                port,
                &format!("/media-artwork/223456789012345678?secret={secret}")
            )
            .starts_with("HTTP/1.1 200")
        );
        core.media_audio.write().await.push_front(MediaAudio {
            id: "323456789012345678".into(),
            content_type: "audio/mpeg".into(),
            bytes: axum::body::Bytes::from_static(b"ID3-original-audio"),
        });
        assert!(http_status(port, "/media-audio/323456789012345678").starts_with("HTTP/1.1 401"));
        assert!(
            http_status(
                port,
                &format!("/media-audio/323456789012345678?secret={secret}")
            )
            .starts_with("HTTP/1.1 200")
        );
        core.cached_media.write().await.push_front(CachedMedia {
            id: "423456789012345678-embed-0".into(),
            content_type: "video/mp4".into(),
            bytes: axum::body::Bytes::from_static(b"0123456789"),
        });
        assert!(
            http_status(port, "/media-cache/423456789012345678-embed-0")
                .starts_with("HTTP/1.1 401")
        );
        assert!(
            http_status(
                port,
                &format!("/media-cache/423456789012345678-embed-0?secret={secret}")
            )
            .starts_with("HTTP/1.1 200")
        );

        let mut clients = Vec::new();
        for _ in 0..8 {
            let (mut client, _) = tokio_tungstenite::connect_async(format!(
                "ws://127.0.0.1:{port}/ws?role=overlay&source=visual&client=obs&secret={secret}"
            ))
            .await
            .unwrap();
            let initial = client.next().await.unwrap().unwrap();
            assert!(initial.to_text().unwrap().contains("\"type\":\"config\""));
            let appearance = client.next().await.unwrap().unwrap();
            assert!(
                appearance
                    .to_text()
                    .unwrap()
                    .contains("\"type\":\"appearance\"")
            );
            let clock = client.next().await.unwrap().unwrap();
            assert!(clock.to_text().unwrap().contains("\"type\":\"mediaClock\""));
            let stage = client.next().await.unwrap().unwrap();
            assert!(stage.to_text().unwrap().contains("\"type\":\"stageClock\""));
            clients.push(client);
        }
        let (mut preview_client, _) = tokio_tungstenite::connect_async(format!(
            "ws://127.0.0.1:{port}/ws?role=overlay&source=visual&client=preview&secret={secret}"
        ))
        .await
        .unwrap();
        let initial = preview_client.next().await.unwrap().unwrap();
        assert!(initial.to_text().unwrap().contains("\"type\":\"config\""));
        let appearance = preview_client.next().await.unwrap().unwrap();
        assert!(
            appearance
                .to_text()
                .unwrap()
                .contains("\"type\":\"appearance\"")
        );
        clients.push(preview_client);
        let (mut widget_client, _) = tokio_tungstenite::connect_async(format!(
            "ws://127.0.0.1:{port}/ws?role=overlay&source=visual&client=widget&secret={secret}"
        ))
        .await
        .unwrap();
        let initial = widget_client.next().await.unwrap().unwrap();
        assert!(initial.to_text().unwrap().contains("\"type\":\"config\""));
        let appearance = widget_client.next().await.unwrap().unwrap();
        assert!(
            appearance
                .to_text()
                .unwrap()
                .contains("\"type\":\"appearance\"")
        );
        let stage = widget_client.next().await.unwrap().unwrap();
        assert!(stage.to_text().unwrap().contains("\"type\":\"stageClock\""));
        clients.push(widget_client);
        let (mut tts_client, _) = tokio_tungstenite::connect_async(format!(
            "ws://127.0.0.1:{port}/ws?role=tts&source=tts&client=obs&secret={secret}"
        ))
        .await
        .unwrap();
        let initial = tts_client.next().await.unwrap().unwrap();
        assert!(initial.to_text().unwrap().contains("\"type\":\"config\""));
        let appearance = tts_client.next().await.unwrap().unwrap();
        assert!(
            appearance
                .to_text()
                .unwrap()
                .contains("\"type\":\"appearance\"")
        );
        let stage = tts_client.next().await.unwrap().unwrap();
        assert!(stage.to_text().unwrap().contains("\"type\":\"stageClock\""));
        clients.push(tts_client);
        let (mut notification_client, _) = tokio_tungstenite::connect_async(format!(
            "ws://127.0.0.1:{port}/ws?role=notification&source=notification&client=obs&secret={secret}"
        ))
        .await
        .unwrap();
        let initial = notification_client.next().await.unwrap().unwrap();
        assert!(initial.to_text().unwrap().contains("\"type\":\"config\""));
        let appearance = notification_client.next().await.unwrap().unwrap();
        assert!(
            appearance
                .to_text()
                .unwrap()
                .contains("\"type\":\"appearance\"")
        );
        let stage = notification_client.next().await.unwrap().unwrap();
        assert!(stage.to_text().unwrap().contains("\"type\":\"stageClock\""));
        clients.push(notification_client);

        for index in 0..300 {
            let event = MediaEvent {
                kind: MediaKind::Image,
                url: format!("https://cdn.discordapp.com/test/{index}.png"),
                proxy_url: format!("https://media.discordapp.net/test/{index}.png"),
                filename: format!("{index}.png"),
                content_type: "image/png".into(),
                artwork_id: None,
                audio_id: None,
                cached_media_id: None,
                title: None,
                artist: None,
                text: None,
                author: AuthorIdentity {
                    username: "stability".into(),
                    display_avatar_url: "https://cdn.discordapp.com/avatar.png".into(),
                },
                timestamp: index,
                message_id: format!("10000000000000{index:04}"),
            };
            {
                let mut history = core.history.write().await;
                history.push_front(event.clone());
                history.truncate(crate::state::HISTORY_LIMIT);
            }
            let _ = core.relay_tx.send(RelayEvent::Media(event));
        }

        for client in &mut clients {
            for _ in 0..300 {
                let message = tokio::time::timeout(Duration::from_secs(5), client.next())
                    .await
                    .expect("broadcast timed out")
                    .expect("socket closed")
                    .expect("websocket error");
                assert!(message.to_text().unwrap().contains("\"type\":\"media\""));
            }
        }
        assert_eq!(core.history.read().await.len(), 50);
        let status = core.server_status.read().await.clone();
        assert_eq!(status.overlay_clients, 10);
        assert_eq!(status.outputs.visual.obs_clients, 8);
        assert_eq!(status.outputs.visual.preview_clients, 1);
        assert_eq!(status.outputs.visual.widget_clients, 1);
        assert!(status.outputs.visual.last_connected_at.is_some());
        assert_eq!(status.outputs.tts.obs_clients, 1);
        assert_eq!(status.outputs.notification.obs_clients, 1);

        start_server(core.clone()).await.unwrap();
        for client in &mut clients {
            let closed = tokio::time::timeout(Duration::from_secs(5), client.next())
                .await
                .expect("old socket did not close after restart");
            assert!(
                closed.is_none()
                    || closed.is_some_and(|message| message.is_ok_and(|value| value.is_close()))
            );
        }
        drop(clients);
        stop_server(&core).await;
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn coordinates_split_video_and_audio_outputs() {
        let port = free_local_port();
        let directory = tempfile::tempdir().unwrap();
        let core = AppCore::load(directory.path().join("config.json")).unwrap();
        core.set_config(AppConfig {
            port,
            ..AppConfig::default()
        })
        .await
        .unwrap();
        start_server(core.clone()).await.unwrap();
        let secret = load_or_create_relay_secret().unwrap();

        let mut audio = connect_test_output(port, &secret, "audio").await;
        let initial_audio_clock = next_test_event(&mut audio, "mediaClock").await;
        assert_eq!(initial_audio_clock["payload"]["videoBusy"], false);
        assert_eq!(initial_audio_clock["payload"]["audioBusy"], false);

        let mut visual = connect_test_output(port, &secret, "visual").await;
        let initial_visual_clock = next_test_event(&mut visual, "mediaClock").await;
        assert_eq!(initial_visual_clock["payload"]["videoBusy"], false);
        assert_eq!(initial_visual_clock["payload"]["audioBusy"], false);

        send_test_clock(&mut visual, true).await;
        let visual_grant = next_test_event(&mut visual, "mediaGrant").await;
        assert_eq!(visual_grant["payload"]["granted"], true);
        let visual_busy = next_test_event(&mut visual, "mediaClock").await;
        assert_eq!(visual_busy["payload"]["videoBusy"], true);
        let visual_busy = next_test_event(&mut audio, "mediaClock").await;
        assert_eq!(visual_busy["payload"]["videoBusy"], true);

        send_test_clock(&mut audio, true).await;
        let audio_grant = next_test_event(&mut audio, "mediaGrant").await;
        assert_eq!(audio_grant["payload"]["granted"], false);

        core.publish_media(test_media(MediaKind::Video, "video"))
            .await;
        let video_event = next_test_event(&mut audio, "media").await;
        assert_eq!(video_event["payload"]["kind"], "video");
        let video_event = next_test_event(&mut visual, "media").await;
        assert_eq!(video_event["payload"]["kind"], "video");

        core.publish_media(test_media(MediaKind::Audio, "audio"))
            .await;
        let audio_event = next_test_event(&mut visual, "media").await;
        assert_eq!(audio_event["payload"]["kind"], "audio");
        let audio_event = next_test_event(&mut audio, "media").await;
        assert_eq!(audio_event["payload"]["kind"], "audio");

        drop(visual);
        let video_idle = next_test_event(&mut audio, "mediaClock").await;
        assert_eq!(video_idle["payload"]["videoBusy"], false);

        send_test_clock(&mut audio, true).await;
        let audio_grant = next_test_event(&mut audio, "mediaGrant").await;
        assert_eq!(audio_grant["payload"]["granted"], true);
        let audio_busy = next_test_event(&mut audio, "mediaClock").await;
        assert_eq!(audio_busy["payload"]["audioBusy"], true);

        send_test_clock(&mut audio, false).await;
        let audio_idle = next_test_event(&mut audio, "mediaClock").await;
        assert_eq!(audio_idle["payload"]["audioBusy"], false);

        stop_server(&core).await;
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn coordinates_media_and_tts_stage_clock() {
        let port = free_local_port();
        let directory = tempfile::tempdir().unwrap();
        let core = AppCore::load(directory.path().join("config.json")).unwrap();
        core.set_config(AppConfig {
            port,
            ..AppConfig::default()
        })
        .await
        .unwrap();
        start_server(core.clone()).await.unwrap();
        let secret = load_or_create_relay_secret().unwrap();

        let mut overlay = connect_test_output(port, &secret, "all").await;
        let overlay_stage = next_test_event(&mut overlay, "stageClock").await;
        assert_eq!(overlay_stage["payload"]["mediaBusy"], false);
        assert_eq!(overlay_stage["payload"]["musicBusy"], false);
        assert_eq!(overlay_stage["payload"]["ttsBusy"], false);

        let (mut notification, _) = tokio_tungstenite::connect_async(format!(
            "ws://127.0.0.1:{port}/ws?role=notification&source=notification&client=widget&secret={secret}"
        ))
        .await
        .unwrap();
        assert_eq!(
            next_test_event(&mut notification, "config").await["type"],
            "config"
        );
        assert_eq!(
            next_test_event(&mut notification, "appearance").await["type"],
            "appearance"
        );
        let notification_stage = next_test_event(&mut notification, "stageClock").await;
        assert_eq!(notification_stage["payload"]["mediaBusy"], false);
        assert_eq!(notification_stage["payload"]["musicBusy"], false);
        assert_eq!(notification_stage["payload"]["ttsBusy"], false);

        notification
            .send(tokio_tungstenite::tungstenite::Message::Text(
                json!({ "type": "stageClock", "payload": { "lane": "tts", "busy": true } })
                    .to_string()
                    .into(),
            ))
            .await
            .unwrap();
        let busy = next_test_event(&mut overlay, "stageClock").await;
        assert_eq!(busy["payload"]["ttsBusy"], true);
        assert_eq!(busy["payload"]["mediaBusy"], false);
        let busy = next_test_event(&mut notification, "stageClock").await;
        assert_eq!(busy["payload"]["ttsBusy"], true);

        overlay
            .send(tokio_tungstenite::tungstenite::Message::Text(
                json!({ "type": "stageClock", "payload": { "lane": "media", "busy": true } })
                    .to_string()
                    .into(),
            ))
            .await
            .unwrap();
        // Media claim is rejected while TTS holds the exclusive stage.
        let rejected = next_test_event(&mut overlay, "stageClock").await;
        assert_eq!(rejected["payload"]["mediaBusy"], false);
        assert_eq!(rejected["payload"]["ttsBusy"], true);
        assert_eq!(rejected["payload"]["granted"], false);

        notification
            .send(tokio_tungstenite::tungstenite::Message::Text(
                json!({ "type": "stageClock", "payload": { "lane": "tts", "busy": false } })
                    .to_string()
                    .into(),
            ))
            .await
            .unwrap();
        let idle = next_test_event(&mut overlay, "stageClock").await;
        assert_eq!(idle["payload"]["ttsBusy"], false);
        let idle = next_test_event(&mut notification, "stageClock").await;
        assert_eq!(idle["payload"]["ttsBusy"], false);

        overlay
            .send(tokio_tungstenite::tungstenite::Message::Text(
                json!({ "type": "stageClock", "payload": { "lane": "media", "busy": true } })
                    .to_string()
                    .into(),
            ))
            .await
            .unwrap();
        let media_only = next_test_event(&mut overlay, "stageClock").await;
        assert_eq!(media_only["payload"]["mediaBusy"], true);
        assert_eq!(media_only["payload"]["ttsBusy"], false);
        let media_only = next_test_event(&mut notification, "stageClock").await;
        assert_eq!(media_only["payload"]["mediaBusy"], true);
        assert_eq!(media_only["payload"]["ttsBusy"], false);

        let mut peer_overlay = connect_test_output(port, &secret, "all").await;
        let peer_initial = next_test_event(&mut peer_overlay, "stageClock").await;
        assert_eq!(peer_initial["payload"]["mediaBusy"], true);
        peer_overlay
            .send(tokio_tungstenite::tungstenite::Message::Text(
                json!({ "type": "stageClock", "payload": { "lane": "media", "busy": true } })
                    .to_string()
                    .into(),
            ))
            .await
            .unwrap();
        // A same-lane grant must wake the claimant even though the global
        // mediaBusy boolean was already true because another output owns it.
        let peer_grant = next_test_event(&mut peer_overlay, "stageClock").await;
        assert_eq!(peer_grant["payload"]["mediaBusy"], true);
        assert_eq!(peer_grant["payload"]["ttsBusy"], false);

        stop_server(&core).await;
    }

    type TestWebSocket = tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >;

    async fn connect_test_output(port: u16, secret: &str, source: &str) -> TestWebSocket {
        let (mut client, _) = tokio_tungstenite::connect_async(format!(
            "ws://127.0.0.1:{port}/ws?role=overlay&source={source}&client=obs&secret={secret}"
        ))
        .await
        .unwrap();
        assert_eq!(
            next_test_event(&mut client, "config").await["type"],
            "config"
        );
        assert_eq!(
            next_test_event(&mut client, "appearance").await["type"],
            "appearance"
        );
        client
    }

    async fn next_test_event(client: &mut TestWebSocket, expected_type: &str) -> serde_json::Value {
        loop {
            let message = tokio::time::timeout(Duration::from_secs(5), client.next())
                .await
                .expect("websocket event timed out")
                .expect("websocket closed")
                .expect("websocket error");
            let event: serde_json::Value =
                serde_json::from_str(message.to_text().expect("text websocket event")).unwrap();
            if event["type"] == expected_type {
                return event;
            }
        }
    }

    async fn send_test_clock(client: &mut TestWebSocket, busy: bool) {
        client
            .send(tokio_tungstenite::tungstenite::Message::Text(
                json!({ "type": "mediaClock", "payload": { "busy": busy } })
                    .to_string()
                    .into(),
            ))
            .await
            .unwrap();
    }

    fn test_media(kind: MediaKind, id: &str) -> MediaEvent {
        MediaEvent {
            kind,
            url: format!("https://cdn.discordapp.com/{id}"),
            proxy_url: format!("https://media.discordapp.net/{id}"),
            filename: id.into(),
            content_type: "application/octet-stream".into(),
            artwork_id: None,
            audio_id: None,
            cached_media_id: None,
            title: None,
            artist: None,
            text: None,
            author: AuthorIdentity {
                username: "clock-test".into(),
                display_avatar_url: "https://cdn.discordapp.com/avatar.png".into(),
            },
            timestamp: 1,
            message_id: id.into(),
        }
    }

    fn free_local_port() -> u16 {
        std::net::TcpListener::bind((HOST, 0))
            .unwrap()
            .local_addr()
            .unwrap()
            .port()
    }

    fn access_query(source: Option<&str>, client: Option<&str>) -> AccessQuery {
        AccessQuery {
            role: None,
            secret: None,
            token: None,
            source: source.map(str::to_owned),
            client: client.map(str::to_owned),
        }
    }

    #[test]
    fn serves_cached_video_byte_ranges_inline() {
        let response = ranged_media_response(
            CachedMedia {
                id: "gif".into(),
                content_type: "video/mp4".into(),
                bytes: axum::body::Bytes::from_static(b"0123456789"),
            },
            Some(&HeaderValue::from_static("bytes=2-5")),
        );
        assert_eq!(response.status(), StatusCode::PARTIAL_CONTENT);
        assert_eq!(response.headers()[header::CONTENT_RANGE], "bytes 2-5/10");
        assert_eq!(response.headers()[header::CONTENT_TYPE], "video/mp4");
        assert!(
            response
                .headers()
                .get(header::CONTENT_DISPOSITION)
                .is_none()
        );
    }

    fn http_status(port: u16, path: &str) -> String {
        http_response(port, path)
            .lines()
            .next()
            .unwrap_or_default()
            .to_owned()
    }

    fn http_response(port: u16, path: &str) -> String {
        http_response_with_host(port, path, HOST)
    }

    fn http_response_with_host(port: u16, path: &str, hostname: &str) -> String {
        let mut stream = std::net::TcpStream::connect((HOST, port)).unwrap();
        write!(
            stream,
            "GET {path} HTTP/1.1\r\nHost: {hostname}:{port}\r\nConnection: close\r\n\r\n"
        )
        .unwrap();
        let mut response = String::new();
        stream.read_to_string(&mut response).unwrap();
        response
    }
}
