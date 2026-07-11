use serde::{Deserialize, Serialize};

use crate::config::AppConfig;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorIdentity {
    pub username: String,
    pub display_avatar_url: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum MediaKind {
    Image,
    Gif,
    Video,
    Audio,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaEvent {
    pub kind: MediaKind,
    pub url: String,
    pub proxy_url: String,
    pub filename: String,
    pub content_type: String,
    #[serde(default)]
    pub artwork_id: Option<String>,
    #[serde(default)]
    pub audio_id: Option<String>,
    #[serde(default)]
    pub cached_media_id: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub artist: Option<String>,
    pub author: AuthorIdentity,
    pub timestamp: u64,
    pub message_id: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PendingMedia {
    pub id: u64,
    pub media: MediaEvent,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TtsEvent {
    pub id: String,
    pub text: String,
    pub author: AuthorIdentity,
    pub content_type: String,
    pub timestamp: u64,
}

#[derive(Clone, Debug)]
pub struct TtsRequest {
    pub id: String,
    pub text: String,
    pub author: AuthorIdentity,
    pub timestamp: u64,
}

#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BotStatus {
    pub connected: bool,
    pub username: Option<String>,
    pub display_avatar_url: Option<String>,
    pub error: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelSummary {
    pub id: String,
    pub name: String,
    pub guild_name: String,
}

#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerStatus {
    pub connected: bool,
    pub overlay_clients: usize,
    pub error: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InterfacePreferences {
    pub language: String,
    pub theme: String,
    pub accent_rgb: [u8; 3],
    pub font_scale: u8,
}

impl Default for InterfacePreferences {
    fn default() -> Self {
        Self {
            language: "en".into(),
            theme: "dark".into(),
            accent_rgb: [88, 185, 137],
            font_scale: 100,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "type", content = "payload", rename_all = "camelCase")]
pub enum RelayEvent {
    Media(MediaEvent),
    Tts(TtsEvent),
    Config(AppConfig),
    Clear,
    Skip,
    Appearance(InterfacePreferences),
}
