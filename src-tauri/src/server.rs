use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
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
    model::{RelayEvent, ServerStatus},
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
    overlay_clients: Arc<AtomicUsize>,
    client_shutdown: tokio::sync::broadcast::Sender<()>,
}

#[derive(Debug, Deserialize)]
struct AccessQuery {
    role: Option<String>,
    secret: Option<String>,
    token: Option<String>,
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
    let state = RelayServerState {
        core: core.clone(),
        relay_secret: Arc::new(load_or_create_relay_secret()?),
        overlay_clients: Arc::new(AtomicUsize::new(0)),
        client_shutdown: client_shutdown.clone(),
    };
    let router = Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .route("/overlay", get(overlay))
        .route("/medias", get(visual_overlay))
        .route("/audios", get(audio_overlay))
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
            header::REFERRER_POLICY,
            HeaderValue::from_static("no-referrer"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::CONTENT_SECURITY_POLICY,
            HeaderValue::from_static(
                "default-src 'none'; script-src 'self'; style-src 'self'; img-src 'self' https://cdn.discordapp.com https://media.discordapp.net https://*.discordapp.net https://*.klipy.com data:; media-src 'self' https://cdn.discordapp.com https://media.discordapp.net https://*.discordapp.net https://*.klipy.com; connect-src 'self' ws://127.0.0.1:*; frame-ancestors 'self' tauri://localhost http://tauri.localhost",
            ),
        ))
        .with_state(state);
    let (shutdown_tx, shutdown_rx) = oneshot::channel();
    let status_core = core.clone();
    let task = tokio::spawn(async move {
        *status_core.server_status.write().await = ServerStatus {
            connected: true,
            overlay_clients: 0,
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

fn short_overlay(state: &RelayServerState, mode: &str) -> Response {
    let html = OVERLAY_HTML.replace(
        "<meta name=\"relay-mode\" content=\"all\">",
        &format!("<meta name=\"relay-mode\" content=\"{mode}\">"),
    );
    short_page(state, html)
}

/// Deliberate trade-off: the short URLs (/medias, /audios, /tts, ...) are
/// meant to be pasted into OBS without any secret, so this response hands the
/// relay secret to any local caller via Set-Cookie. The server only binds
/// 127.0.0.1, so the boundary is "this machine", not "this user": any local
/// process can obtain the secret. Requiring the secret here would break the
/// paste-and-go OBS setup.
fn short_page(state: &RelayServerState, html: impl Into<Body>) -> Response {
    Response::builder()
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .header(
            header::SET_COOKIE,
            format!(
                "relay_secret={}; Path=/; HttpOnly; SameSite=Strict",
                state.relay_secret
            ),
        )
        .body(html.into())
        .expect("valid short overlay response")
}

async fn notification_sound(
    State(state): State<RelayServerState>,
    Query(query): Query<AccessQuery>,
    headers: HeaderMap,
) -> Response {
    if !request_secret_matches(query.secret.as_deref(), &headers, &state.relay_secret) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let Some(path) = state.core.config.read().await.notification_sound_path.clone() else {
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
    ([(header::CONTENT_TYPE, "text/css; charset=utf-8")], STICKERS_CSS)
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

    let is_output = matches!(role, "overlay" | "tts" | "notification" | "sticker");
    upgrade.on_upgrade(move |socket| handle_socket(socket, state, is_output))
}

async fn handle_socket(socket: WebSocket, state: RelayServerState, is_output: bool) {
    if is_output {
        update_overlay_count(&state, 1).await;
    }
    let (mut sender, mut receiver) = socket.split();
    let config = state.core.config.read().await.clone();
    let config_payload = if is_output {
        json!(OverlayConfig::from(&config))
    } else {
        json!(config)
    };
    if send_json(&mut sender, &json!({ "type": "config", "payload": config_payload }))
        .await
        .is_err()
    {
        if is_output {
            update_overlay_count(&state, -1).await;
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
        if is_output {
            update_overlay_count(&state, -1).await;
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
    }

    let mut relay_rx = state.core.relay_tx.subscribe();
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
                        let payload = json!({ "type": "config", "payload": OverlayConfig::from(&config) });
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
            incoming = receiver.next() => {
                match incoming {
                    Some(Ok(Message::Close(_))) | None | Some(Err(_)) => break,
                    Some(Ok(Message::Ping(payload))) => {
                        if sender.send(Message::Pong(payload)).await.is_err() { break; }
                    }
                    Some(Ok(_)) => {
                        let _ = sender.send(Message::Close(None)).await;
                        break;
                    }
                }
            }
        }
    }
    if is_output {
        update_overlay_count(&state, -1).await;
    }
}

async fn send_json<T: serde::Serialize>(
    sender: &mut futures_util::stream::SplitSink<WebSocket, Message>,
    value: &T,
) -> Result<(), axum::Error> {
    let serialized = serde_json::to_string(value).expect("serializable server event");
    sender.send(Message::Text(serialized.into())).await
}

async fn update_overlay_count(state: &RelayServerState, delta: isize) {
    let count = if delta > 0 {
        state
            .overlay_clients
            .fetch_add(delta as usize, Ordering::Relaxed)
            + delta as usize
    } else {
        state
            .overlay_clients
            .fetch_sub((-delta) as usize, Ordering::Relaxed)
            - (-delta) as usize
    };
    state.core.server_status.write().await.overlay_clients = count;
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
        model::{AuthorIdentity, MediaEvent, MediaKind},
        state::{CachedMedia, MediaArtwork, MediaAudio, TtsAudio},
    };

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
            "http://tauri.localhost"
        ))));
        assert!(!origin_allowed(Some(&HeaderValue::from_static(
            "https://example.com"
        ))));
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
        assert!(visual_response.contains("relay_secret="));
        assert!(visual_response.contains("content=\"visual\""));
        assert!(visual_response.contains("https://*.discordapp.net"));
        assert!(visual_response.contains("https://*.klipy.com"));
        let audio_response = http_response(port, "/audios");
        assert!(audio_response.starts_with("HTTP/1.1 200"));
        assert!(audio_response.contains("content=\"audio\""));
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
        for _ in 0..10 {
            let (mut client, _) = tokio_tungstenite::connect_async(format!(
                "ws://127.0.0.1:{port}/ws?role=overlay&secret={secret}"
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
            clients.push(client);
        }
        let (mut tts_client, _) = tokio_tungstenite::connect_async(format!(
            "ws://127.0.0.1:{port}/ws?role=tts&secret={secret}"
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
        clients.push(tts_client);
        let (mut notification_client, _) = tokio_tungstenite::connect_async(format!(
            "ws://127.0.0.1:{port}/ws?role=notification&secret={secret}"
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
        clients.push(notification_client);

        for index in 0..300 {
            core.publish_media(MediaEvent {
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
                author: AuthorIdentity {
                    username: "stability".into(),
                    display_avatar_url: "https://cdn.discordapp.com/avatar.png".into(),
                },
                timestamp: index,
                message_id: format!("10000000000000{index:04}"),
            })
            .await;
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
        assert_eq!(core.server_status.read().await.overlay_clients, 12);

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

    fn free_local_port() -> u16 {
        std::net::TcpListener::bind((HOST, 0))
            .unwrap()
            .local_addr()
            .unwrap()
            .port()
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
        let mut stream = std::net::TcpStream::connect((HOST, port)).unwrap();
        write!(
            stream,
            "GET {path} HTTP/1.1\r\nHost: {HOST}:{port}\r\nConnection: close\r\n\r\n"
        )
        .unwrap();
        let mut response = String::new();
        stream.read_to_string(&mut response).unwrap();
        response
    }
}
