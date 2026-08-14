use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

use crate::custom_commands::{CustomCommandDefinition, validate_custom_commands};
use crate::privacy::{
    ForbiddenConcept, MAX_CONFIGURED_REGEXES, MAX_PRIVACY_LIST_ENTRIES,
    MAX_PRIVACY_LIST_VALUE_CHARS, PrivacyCategory, PrivacyClassification, ProtectionLevel,
    SuspiciousPolicy, default_privacy_categories,
};

pub const DEFAULT_PORT: u16 = 4590;
pub const DEFAULT_DISPLAY_DURATION_MS: u64 = 8_000;
pub const DEFAULT_GIF_DURATION_MS: u64 = 8_000;
pub const DEFAULT_STICKER_DURATION_MS: u64 = 8_000;
pub const DEFAULT_NOTIFICATION_DURATION_MS: u64 = 8_000;
pub const DEFAULT_WIDGET_WIDTH: f64 = 640.0;
pub const DEFAULT_WIDGET_HEIGHT: f64 = 360.0;
pub const DEFAULT_NOTIFICATION_WIDGET_WIDTH: f64 = 510.0;
pub const DEFAULT_NOTIFICATION_WIDGET_HEIGHT: f64 = 130.0;
pub const MAX_PRIVACY_EXEMPT_ROLE_IDS: usize = 100;
pub const MIN_WIDGET_WIDTH: f64 = 160.0;
pub const MIN_WIDGET_HEIGHT: f64 = 90.0;
const MAX_WIDGET_DIMENSION: f64 = 16_384.0;
const LEGACY_CONFIG_DIRECTORIES: [&str; 2] =
    ["eu.stealthylabs.discord-obs-relay", "discord-obs-relay"];

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionOverwriteSnapshot {
    pub target_id: String,
    pub target_kind: String,
    pub allow: u64,
    pub deny: u64,
    pub existed: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelLockSnapshot {
    pub channel_id: String,
    pub overwrites: Vec<PermissionOverwriteSnapshot>,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct OutputGeometry {
    pub crop_top: u8,
    pub crop_right: u8,
    pub crop_bottom: u8,
    pub crop_left: u8,
    pub content_scale: u16,
}

impl Default for OutputGeometry {
    fn default() -> Self {
        Self {
            crop_top: 0,
            crop_right: 0,
            crop_bottom: 0,
            crop_left: 0,
            content_scale: 100,
        }
    }
}

impl OutputGeometry {
    pub fn validate(&self) -> Result<()> {
        if [
            self.crop_top,
            self.crop_right,
            self.crop_bottom,
            self.crop_left,
        ]
        .into_iter()
        .any(|crop| crop > 40)
        {
            bail!("Output crop values must be between 0 and 40 percent.");
        }
        if !(50..=200).contains(&self.content_scale) {
            bail!("Output content scale must be between 50 and 200 percent.");
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct AppConfig {
    pub watched_channel_id: String,
    pub tts_channel_id: String,
    pub port: u16,
    pub display_duration_ms: u64,
    pub gif_duration_ms: u64,
    pub sticker_duration_ms: u64,
    pub notification_duration_ms: u64,
    pub media_volume: u8,
    pub tts_character_limit: u32,
    pub tts_queue_limit: u8,
    pub tts_speech_enabled: bool,
    pub tts_notifications_obs_enabled: bool,
    pub bot_online_status: String,
    pub bot_activity_type: String,
    pub bot_activity_text: String,
    pub show_author: bool,
    pub show_media_text_obs: bool,
    pub show_media_text_widget: bool,
    pub moderation_enabled: bool,
    pub moderation_allow_images: bool,
    pub moderation_allow_videos: bool,
    pub moderation_allow_audio: bool,
    pub privacy_scan_enabled: bool,
    pub privacy_suspicious_policy: SuspiciousPolicy,
    pub privacy_suspicious_threshold: u8,
    pub privacy_sensitive_threshold: u8,
    pub privacy_similarity_boost: u8,
    pub privacy_concepts: Vec<ForbiddenConcept>,
    pub privacy_filter_exempt_role_ids: Vec<String>,
    pub privacy_protection_level: ProtectionLevel,
    pub privacy_enabled_categories: Vec<PrivacyCategory>,
    pub privacy_block_threshold: PrivacyClassification,
    pub privacy_review_intermediate: bool,
    pub privacy_auto_delete_blocked_messages: bool,
    pub privacy_allowlist: Vec<String>,
    pub privacy_custom_patterns: Vec<String>,
    pub command_channel_enabled: bool,
    pub command_url_enabled: bool,
    pub command_show_enabled: bool,
    pub command_status_enabled: bool,
    pub command_test_enabled: bool,
    pub command_regenerate_enabled: bool,
    pub command_clear_enabled: bool,
    pub command_lock_enabled: bool,
    pub command_changelog_enabled: bool,
    pub custom_commands: Vec<CustomCommandDefinition>,
    pub channel_lock: Option<ChannelLockSnapshot>,
    pub widget_x: Option<i32>,
    pub widget_y: Option<i32>,
    pub widget_width: f64,
    pub widget_height: f64,
    pub widget_keep_aspect_ratio: bool,
    pub widget_visible: bool,
    pub widget_locked: bool,
    pub widget_sound_enabled: bool,
    pub notification_widget_x: Option<i32>,
    pub notification_widget_y: Option<i32>,
    pub notification_widget_width: f64,
    pub notification_widget_height: f64,
    pub notification_widget_visible: bool,
    pub notification_widget_locked: bool,
    pub media_obs_geometry: OutputGeometry,
    pub media_widget_geometry: OutputGeometry,
    pub notification_obs_geometry: OutputGeometry,
    pub notification_widget_geometry: OutputGeometry,
    pub notification_sound_enabled: bool,
    pub notification_sound_obs_enabled: bool,
    pub notification_sound_path: Option<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            watched_channel_id: String::new(),
            tts_channel_id: String::new(),
            port: DEFAULT_PORT,
            display_duration_ms: DEFAULT_DISPLAY_DURATION_MS,
            gif_duration_ms: DEFAULT_GIF_DURATION_MS,
            sticker_duration_ms: DEFAULT_STICKER_DURATION_MS,
            notification_duration_ms: DEFAULT_NOTIFICATION_DURATION_MS,
            media_volume: 50,
            tts_character_limit: 0,
            tts_queue_limit: 50,
            tts_speech_enabled: true,
            tts_notifications_obs_enabled: false,
            bot_online_status: "online".into(),
            bot_activity_type: "custom".into(),
            bot_activity_text: String::new(),
            show_author: true,
            show_media_text_obs: false,
            show_media_text_widget: false,
            moderation_enabled: false,
            moderation_allow_images: true,
            moderation_allow_videos: true,
            moderation_allow_audio: true,
            privacy_scan_enabled: false,
            privacy_suspicious_policy: SuspiciousPolicy::Review,
            privacy_suspicious_threshold: 2,
            privacy_sensitive_threshold: 4,
            privacy_similarity_boost: 4,
            privacy_concepts: Vec::new(),
            privacy_filter_exempt_role_ids: Vec::new(),
            privacy_protection_level: ProtectionLevel::Balanced,
            privacy_enabled_categories: default_privacy_categories(),
            privacy_block_threshold: PrivacyClassification::High,
            privacy_review_intermediate: true,
            privacy_auto_delete_blocked_messages: true,
            privacy_allowlist: Vec::new(),
            privacy_custom_patterns: Vec::new(),
            command_channel_enabled: true,
            command_url_enabled: true,
            command_show_enabled: true,
            command_status_enabled: true,
            command_test_enabled: true,
            command_regenerate_enabled: true,
            command_clear_enabled: true,
            command_lock_enabled: true,
            command_changelog_enabled: true,
            custom_commands: Vec::new(),
            channel_lock: None,
            widget_x: None,
            widget_y: None,
            widget_width: DEFAULT_WIDGET_WIDTH,
            widget_height: DEFAULT_WIDGET_HEIGHT,
            widget_keep_aspect_ratio: true,
            widget_visible: false,
            widget_locked: false,
            widget_sound_enabled: false,
            notification_widget_x: None,
            notification_widget_y: None,
            notification_widget_width: DEFAULT_NOTIFICATION_WIDGET_WIDTH,
            notification_widget_height: DEFAULT_NOTIFICATION_WIDGET_HEIGHT,
            notification_widget_visible: false,
            notification_widget_locked: false,
            media_obs_geometry: OutputGeometry::default(),
            media_widget_geometry: OutputGeometry::default(),
            notification_obs_geometry: OutputGeometry::default(),
            notification_widget_geometry: OutputGeometry::default(),
            notification_sound_enabled: false,
            notification_sound_obs_enabled: false,
            notification_sound_path: None,
        }
    }
}

impl AppConfig {
    pub fn validate(&self) -> Result<()> {
        validate_channel_id(&self.watched_channel_id, "watched")?;
        validate_channel_id(&self.tts_channel_id, "TTS")?;
        if !self.watched_channel_id.is_empty() && self.watched_channel_id == self.tts_channel_id {
            bail!("Media and TTS must use separate Discord channels.");
        }
        if self.port < 1024 {
            bail!("The local port must be between 1024 and 65535.");
        }
        if !(1_000..=60_000).contains(&self.display_duration_ms) {
            bail!("The image duration must be between 1 and 60 seconds.");
        }
        if !(1_000..=60_000).contains(&self.gif_duration_ms) {
            bail!("The GIF duration must be between 1 and 60 seconds.");
        }
        if !(1_000..=60_000).contains(&self.sticker_duration_ms) {
            bail!("The sticker duration must be between 1 and 60 seconds.");
        }
        if !(1_000..=60_000).contains(&self.notification_duration_ms) {
            bail!("The notification duration must be between 1 and 60 seconds.");
        }
        if self.media_volume > 100 {
            bail!("The media volume must be between 0 and 100 percent.");
        }
        if !(1..=50).contains(&self.tts_queue_limit) {
            bail!("The TTS queue limit must be between 1 and 50.");
        }
        if !(1..=100).contains(&self.privacy_suspicious_threshold)
            || !(1..=100).contains(&self.privacy_sensitive_threshold)
            || self.privacy_sensitive_threshold <= self.privacy_suspicious_threshold
        {
            bail!("Privacy score thresholds are invalid.");
        }
        if !(1..=100).contains(&self.privacy_similarity_boost) {
            bail!("Privacy similarity boost must be between 1 and 100.");
        }
        if self.privacy_concepts.len() > 100 {
            bail!("At most 100 forbidden concepts may be configured.");
        }
        if self
            .privacy_concepts
            .iter()
            .map(|concept| concept.regexes.len())
            .sum::<usize>()
            > MAX_CONFIGURED_REGEXES
        {
            bail!("At most {MAX_CONFIGURED_REGEXES} filter regular expressions may be configured.");
        }
        for concept in &self.privacy_concepts {
            concept.validate()?;
        }
        if self.privacy_filter_exempt_role_ids.len() > MAX_PRIVACY_EXEMPT_ROLE_IDS {
            bail!(
                "At most {MAX_PRIVACY_EXEMPT_ROLE_IDS} privacy filter exempt roles may be configured."
            );
        }
        for role_id in &self.privacy_filter_exempt_role_ids {
            validate_snowflake_id(role_id, "privacy filter exempt role")?;
        }
        if !matches!(
            self.privacy_block_threshold,
            PrivacyClassification::High | PrivacyClassification::Critical
        ) {
            bail!("The privacy block threshold must be HIGH or CRITICAL.");
        }
        if self.privacy_enabled_categories.len() > PrivacyCategory::USER_CONFIGURABLE.len() {
            bail!("Too many privacy detection categories were configured.");
        }
        let mut unique_categories = std::collections::HashSet::new();
        for category in &self.privacy_enabled_categories {
            if !PrivacyCategory::USER_CONFIGURABLE.contains(category)
                || !unique_categories.insert(*category)
            {
                bail!("Privacy detection categories are invalid or duplicated.");
            }
        }
        validate_privacy_list(&self.privacy_allowlist, "privacy allowlist")?;
        validate_privacy_list(&self.privacy_custom_patterns, "private data list")?;
        if !matches!(
            self.bot_online_status.as_str(),
            "online" | "idle" | "dnd" | "invisible"
        ) {
            bail!("The Discord bot status is invalid.");
        }
        if !matches!(
            self.bot_activity_type.as_str(),
            "none" | "custom" | "playing" | "listening" | "watching" | "competing"
        ) {
            bail!("The Discord bot activity type is invalid.");
        }
        if self.bot_activity_text.chars().count() > 128
            || self.bot_activity_text.chars().any(char::is_control)
        {
            bail!("The Discord bot activity must contain at most 128 printable characters.");
        }
        validate_custom_commands(&self.custom_commands)?;
        validate_widget_size(self.widget_width, self.widget_height)?;
        validate_widget_size(
            self.notification_widget_width,
            self.notification_widget_height,
        )?;
        self.media_obs_geometry.validate()?;
        self.media_widget_geometry.validate()?;
        self.notification_obs_geometry.validate()?;
        self.notification_widget_geometry.validate()?;
        Ok(())
    }
}

fn validate_privacy_list(values: &[String], label: &str) -> Result<()> {
    if values.len() > MAX_PRIVACY_LIST_ENTRIES {
        bail!("The {label} may contain at most {MAX_PRIVACY_LIST_ENTRIES} entries.");
    }
    for value in values {
        let value = value.trim();
        if !(3..=MAX_PRIVACY_LIST_VALUE_CHARS).contains(&value.chars().count())
            || value.chars().any(char::is_control)
        {
            bail!(
                "Each {label} entry must contain 3 to {MAX_PRIVACY_LIST_VALUE_CHARS} printable characters."
            );
        }
    }
    Ok(())
}

fn validate_widget_size(width: f64, height: f64) -> Result<()> {
    if !width.is_finite()
        || !height.is_finite()
        || !(MIN_WIDGET_WIDTH..=MAX_WIDGET_DIMENSION).contains(&width)
        || !(MIN_WIDGET_HEIGHT..=MAX_WIDGET_DIMENSION).contains(&height)
    {
        bail!("Widget size is outside the supported bounds.");
    }
    Ok(())
}

fn validate_channel_id(channel_id: &str, label: &str) -> Result<()> {
    if !channel_id.is_empty()
        && (channel_id.len() > 20
            || !channel_id
                .chars()
                .all(|character| character.is_ascii_digit()))
    {
        bail!("The {label} channel ID is invalid.");
    }
    Ok(())
}

fn validate_snowflake_id(value: &str, label: &str) -> Result<()> {
    if !(17..=20).contains(&value.len())
        || !value.chars().all(|character| character.is_ascii_digit())
        || value.parse::<u64>().map_or(true, |id| id == 0)
    {
        bail!("The {label} ID is invalid.");
    }
    Ok(())
}

pub fn migrate_legacy_config(config_directory: &Path) -> Result<()> {
    let destination = config_directory.join("config.json");
    let Some(parent) = config_directory.parent() else {
        return Ok(());
    };
    for directory in LEGACY_CONFIG_DIRECTORIES {
        let legacy_path = parent.join(directory).join("config.json");
        if legacy_path.is_file() {
            let bytes = fs::read(&legacy_path)
                .with_context(|| format!("failed to read {}", legacy_path.display()))?;
            let (config, _) = deserialize_config(&bytes)
                .with_context(|| format!("failed to parse {}", legacy_path.display()))?;
            if destination.exists() {
                let store = ConfigStore::new(destination.clone());
                let mut current = store.load()?;
                let mut changed = false;
                if current.watched_channel_id.is_empty() && !config.watched_channel_id.is_empty() {
                    current.watched_channel_id = config.watched_channel_id;
                    changed = true;
                }
                if current.tts_channel_id.is_empty() && !config.tts_channel_id.is_empty() {
                    current.tts_channel_id = config.tts_channel_id;
                    changed = true;
                }
                if changed {
                    store.save(&current)?;
                }
            } else {
                ConfigStore::new(destination.clone()).save(&config)?;
            }
            break;
        }
    }
    Ok(())
}

#[derive(Clone, Debug)]
pub struct ConfigStore {
    path: PathBuf,
}

impl ConfigStore {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn load(&self) -> Result<AppConfig> {
        if !self.path.exists() {
            let config = AppConfig::default();
            self.save(&config)?;
            return Ok(config);
        }

        let bytes = fs::read(&self.path)
            .with_context(|| format!("failed to read {}", self.path.display()))?;
        let (config, migrated) = deserialize_config(&bytes)
            .with_context(|| format!("failed to parse {}", self.path.display()))?;
        config.validate()?;
        if migrated {
            self.save(&config)?;
        }
        Ok(config)
    }

    pub fn save(&self, config: &AppConfig) -> Result<()> {
        config.validate()?;
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }

        let temporary_path = temporary_path(&self.path);
        let bytes = serde_json::to_vec_pretty(config)?;
        let mut file = fs::File::create(&temporary_path)
            .with_context(|| format!("failed to create {}", temporary_path.display()))?;
        file.write_all(&bytes)?;
        file.write_all(b"\n")?;
        file.sync_all()?;
        drop(file);
        fs::rename(&temporary_path, &self.path)
            .with_context(|| format!("failed to replace {}", self.path.display()))?;
        Ok(())
    }
}

fn deserialize_config(bytes: &[u8]) -> Result<(AppConfig, bool)> {
    let mut value: serde_json::Value = serde_json::from_slice(bytes)?;
    let missing_gif_duration = value.get("gifDurationMs").is_none();
    let missing_sticker_duration = value.get("stickerDurationMs").is_none();
    let migrated = missing_gif_duration || missing_sticker_duration;
    if missing_gif_duration {
        let duration = value
            .get("displayDurationMs")
            .cloned()
            .unwrap_or_else(|| serde_json::json!(DEFAULT_DISPLAY_DURATION_MS));
        if let Some(config) = value.as_object_mut() {
            config.insert("gifDurationMs".into(), duration);
        }
    }
    if missing_sticker_duration && let Some(config) = value.as_object_mut() {
        config.insert(
            "stickerDurationMs".into(),
            serde_json::json!(DEFAULT_STICKER_DURATION_MS),
        );
    }
    Ok((serde_json::from_value(value)?, migrated))
}

fn temporary_path(path: &Path) -> PathBuf {
    let mut temporary = path.as_os_str().to_owned();
    temporary.push(".tmp");
    PathBuf::from(temporary)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_and_round_trips_default_config() {
        let directory = tempfile::tempdir().unwrap();
        let store = ConfigStore::new(directory.path().join("config.json"));

        let config = store.load().unwrap();
        assert_eq!(config, AppConfig::default());

        let updated = AppConfig {
            watched_channel_id: "123456789012345678".into(),
            tts_channel_id: "223456789012345678".into(),
            port: 5_000,
            display_duration_ms: 4_000,
            gif_duration_ms: 6_000,
            sticker_duration_ms: 7_000,
            notification_duration_ms: 9_000,
            media_volume: 65,
            tts_character_limit: 280,
            tts_queue_limit: 24,
            tts_speech_enabled: false,
            tts_notifications_obs_enabled: true,
            bot_online_status: "idle".into(),
            bot_activity_type: "watching".into(),
            bot_activity_text: "the media queue".into(),
            show_author: false,
            show_media_text_obs: true,
            show_media_text_widget: true,
            moderation_enabled: true,
            moderation_allow_images: true,
            moderation_allow_videos: false,
            moderation_allow_audio: true,
            privacy_scan_enabled: true,
            privacy_suspicious_policy: SuspiciousPolicy::Review,
            privacy_suspicious_threshold: 2,
            privacy_sensitive_threshold: 4,
            privacy_similarity_boost: 4,
            privacy_concepts: Vec::new(),
            privacy_filter_exempt_role_ids: Vec::new(),
            privacy_protection_level: ProtectionLevel::Strict,
            privacy_enabled_categories: default_privacy_categories(),
            privacy_block_threshold: PrivacyClassification::Critical,
            privacy_review_intermediate: true,
            privacy_auto_delete_blocked_messages: false,
            privacy_allowlist: vec!["public@example.com".into()],
            privacy_custom_patterns: vec!["private alias".into()],
            command_channel_enabled: true,
            command_url_enabled: false,
            command_show_enabled: true,
            command_status_enabled: false,
            command_test_enabled: false,
            command_regenerate_enabled: false,
            command_clear_enabled: true,
            command_lock_enabled: true,
            command_changelog_enabled: false,
            custom_commands: vec![CustomCommandDefinition {
                name: "announce".into(),
                description: "Post the configured announcement".into(),
                action: crate::custom_commands::CustomCommandAction::Reply {
                    text: "Configured locally".into(),
                    ephemeral: false,
                },
                ..CustomCommandDefinition::default()
            }],
            channel_lock: None,
            widget_x: Some(-640),
            widget_y: Some(120),
            widget_width: 960.0,
            widget_height: 540.0,
            widget_keep_aspect_ratio: false,
            widget_visible: true,
            widget_locked: true,
            widget_sound_enabled: true,
            notification_widget_x: Some(900),
            notification_widget_y: Some(40),
            notification_widget_width: 620.0,
            notification_widget_height: 180.0,
            notification_widget_visible: true,
            notification_widget_locked: true,
            media_obs_geometry: OutputGeometry {
                crop_top: 4,
                crop_right: 8,
                crop_bottom: 12,
                crop_left: 16,
                content_scale: 125,
            },
            media_widget_geometry: OutputGeometry {
                content_scale: 80,
                ..OutputGeometry::default()
            },
            notification_obs_geometry: OutputGeometry {
                crop_left: 10,
                content_scale: 150,
                ..OutputGeometry::default()
            },
            notification_widget_geometry: OutputGeometry {
                crop_bottom: 20,
                content_scale: 90,
                ..OutputGeometry::default()
            },
            notification_sound_enabled: true,
            notification_sound_obs_enabled: true,
            notification_sound_path: Some("C:/sounds/ping.mp3".into()),
        };
        store.save(&updated).unwrap();
        assert_eq!(store.load().unwrap(), updated);
    }

    #[test]
    fn rejects_invalid_config() {
        let config = AppConfig {
            port: 80,
            ..AppConfig::default()
        };
        assert!(config.validate().is_err());

        let duplicate_channels = AppConfig {
            watched_channel_id: "123456789012345678".into(),
            tts_channel_id: "123456789012345678".into(),
            ..AppConfig::default()
        };
        assert!(duplicate_channels.validate().is_err());

        let invalid_geometry = AppConfig {
            media_obs_geometry: OutputGeometry {
                crop_top: 41,
                ..OutputGeometry::default()
            },
            ..AppConfig::default()
        };
        assert!(invalid_geometry.validate().is_err());

        let invalid_scale = AppConfig {
            notification_widget_geometry: OutputGeometry {
                content_scale: 201,
                ..OutputGeometry::default()
            },
            ..AppConfig::default()
        };
        assert!(invalid_scale.validate().is_err());

        let invalid_size = AppConfig {
            widget_width: 159.0,
            ..AppConfig::default()
        };
        assert!(invalid_size.validate().is_err());

        let invalid_presence = AppConfig {
            bot_online_status: "offline".into(),
            ..AppConfig::default()
        };
        assert!(invalid_presence.validate().is_err());

        let invalid_activity = AppConfig {
            bot_activity_type: "streaming".into(),
            ..AppConfig::default()
        };
        assert!(invalid_activity.validate().is_err());

        let activity_too_long = AppConfig {
            bot_activity_text: "x".repeat(129),
            ..AppConfig::default()
        };
        assert!(activity_too_long.validate().is_err());

        let invalid_similarity_boost = AppConfig {
            privacy_similarity_boost: 0,
            ..AppConfig::default()
        };
        assert!(invalid_similarity_boost.validate().is_err());

        let invalid_exempt_role = AppConfig {
            privacy_filter_exempt_role_ids: vec!["123".into()],
            ..AppConfig::default()
        };
        assert!(invalid_exempt_role.validate().is_err());
        let zero_exempt_role = AppConfig {
            privacy_filter_exempt_role_ids: vec!["00000000000000000".into()],
            ..AppConfig::default()
        };
        assert!(zero_exempt_role.validate().is_err());

        let valid_exempt_role = AppConfig {
            privacy_filter_exempt_role_ids: vec!["123456789012345678".into()],
            ..AppConfig::default()
        };
        assert!(valid_exempt_role.validate().is_ok());

        let too_many_exempt_roles = AppConfig {
            privacy_filter_exempt_role_ids: (0..=MAX_PRIVACY_EXEMPT_ROLE_IDS)
                .map(|index| format!("{index:017}"))
                .collect(),
            ..AppConfig::default()
        };
        assert!(too_many_exempt_roles.validate().is_err());

        let invalid_block_threshold = AppConfig {
            privacy_block_threshold: PrivacyClassification::Medium,
            ..AppConfig::default()
        };
        assert!(invalid_block_threshold.validate().is_err());

        let duplicate_categories = AppConfig {
            privacy_enabled_categories: vec![PrivacyCategory::Email, PrivacyCategory::Email],
            ..AppConfig::default()
        };
        assert!(duplicate_categories.validate().is_err());

        let invalid_private_value = AppConfig {
            privacy_custom_patterns: vec!["ab".into()],
            ..AppConfig::default()
        };
        assert!(invalid_private_value.validate().is_err());

        let invalid_allowlist_value = AppConfig {
            privacy_allowlist: vec!["public\nvalue".into()],
            ..AppConfig::default()
        };
        assert!(invalid_allowlist_value.validate().is_err());
    }

    #[test]
    fn migrates_missing_gif_duration_from_the_image_duration() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("config.json");
        let mut legacy = serde_json::to_value(AppConfig::default()).unwrap();
        legacy.as_object_mut().unwrap().remove("gifDurationMs");
        legacy["displayDurationMs"] = serde_json::json!(15_000);
        fs::write(&path, serde_json::to_vec_pretty(&legacy).unwrap()).unwrap();

        let migrated = ConfigStore::new(path.clone()).load().unwrap();
        assert_eq!(migrated.display_duration_ms, 15_000);
        assert_eq!(migrated.gif_duration_ms, 15_000);
        let persisted: serde_json::Value =
            serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
        assert_eq!(persisted["gifDurationMs"], 15_000);
    }

    #[test]
    fn migrates_missing_sticker_duration_to_eight_seconds() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("config.json");
        let mut legacy = serde_json::to_value(AppConfig::default()).unwrap();
        legacy.as_object_mut().unwrap().remove("stickerDurationMs");
        fs::write(&path, serde_json::to_vec_pretty(&legacy).unwrap()).unwrap();

        let migrated = ConfigStore::new(path.clone()).load().unwrap();
        assert_eq!(migrated.sticker_duration_ms, 8_000);
        let persisted: serde_json::Value =
            serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
        assert_eq!(persisted["stickerDurationMs"], 8_000);
    }

    #[test]
    fn migrates_legacy_config_without_overwriting_relay_config() {
        let root = tempfile::tempdir().unwrap();
        let legacy_directory = root.path().join(LEGACY_CONFIG_DIRECTORIES[0]);
        let relay_directory = root.path().join("eu.stealthylabs.relay");
        let legacy = AppConfig {
            watched_channel_id: "123456789012345678".into(),
            ..AppConfig::default()
        };
        ConfigStore::new(legacy_directory.join("config.json"))
            .save(&legacy)
            .unwrap();
        ConfigStore::new(relay_directory.join("config.json"))
            .save(&AppConfig::default())
            .unwrap();

        migrate_legacy_config(&relay_directory).unwrap();
        assert_eq!(
            ConfigStore::new(relay_directory.join("config.json"))
                .load()
                .unwrap(),
            legacy
        );

        let relay = AppConfig {
            watched_channel_id: "323456789012345678".into(),
            tts_channel_id: "423456789012345678".into(),
            ..AppConfig::default()
        };
        ConfigStore::new(relay_directory.join("config.json"))
            .save(&relay)
            .unwrap();
        migrate_legacy_config(&relay_directory).unwrap();
        assert_eq!(
            ConfigStore::new(relay_directory.join("config.json"))
                .load()
                .unwrap(),
            relay
        );
    }
}
