use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

pub const DEFAULT_PORT: u16 = 4590;
pub const DEFAULT_DISPLAY_DURATION_MS: u64 = 8_000;
pub const DEFAULT_GIF_DURATION_MS: u64 = 8_000;
pub const DEFAULT_STICKER_DURATION_MS: u64 = 8_000;
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

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct AppConfig {
    pub watched_channel_id: String,
    pub tts_channel_id: String,
    pub port: u16,
    pub display_duration_ms: u64,
    pub gif_duration_ms: u64,
    pub sticker_duration_ms: u64,
    pub media_volume: u8,
    pub tts_character_limit: u32,
    pub tts_queue_limit: u8,
    pub tts_notifications_obs_enabled: bool,
    pub show_author: bool,
    pub moderation_enabled: bool,
    pub moderation_allow_images: bool,
    pub moderation_allow_videos: bool,
    pub moderation_allow_audio: bool,
    pub command_channel_enabled: bool,
    pub command_url_enabled: bool,
    pub command_show_enabled: bool,
    pub command_regenerate_enabled: bool,
    pub command_clear_enabled: bool,
    pub command_lock_enabled: bool,
    pub channel_lock: Option<ChannelLockSnapshot>,
    pub widget_x: Option<i32>,
    pub widget_y: Option<i32>,
    pub widget_visible: bool,
    pub widget_locked: bool,
    pub notification_widget_x: Option<i32>,
    pub notification_widget_y: Option<i32>,
    pub notification_widget_visible: bool,
    pub notification_widget_locked: bool,
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
            media_volume: 50,
            tts_character_limit: 0,
            tts_queue_limit: 50,
            tts_notifications_obs_enabled: false,
            show_author: true,
            moderation_enabled: false,
            moderation_allow_images: true,
            moderation_allow_videos: true,
            moderation_allow_audio: true,
            command_channel_enabled: true,
            command_url_enabled: true,
            command_show_enabled: true,
            command_regenerate_enabled: true,
            command_clear_enabled: true,
            command_lock_enabled: true,
            channel_lock: None,
            widget_x: None,
            widget_y: None,
            widget_visible: false,
            widget_locked: false,
            notification_widget_x: None,
            notification_widget_y: None,
            notification_widget_visible: false,
            notification_widget_locked: false,
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
        if self.media_volume > 100 {
            bail!("The media volume must be between 0 and 100 percent.");
        }
        if !(1..=50).contains(&self.tts_queue_limit) {
            bail!("The TTS queue limit must be between 1 and 50.");
        }
        Ok(())
    }
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

#[derive(Debug)]
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
            media_volume: 65,
            tts_character_limit: 280,
            tts_queue_limit: 24,
            tts_notifications_obs_enabled: true,
            show_author: false,
            moderation_enabled: true,
            moderation_allow_images: true,
            moderation_allow_videos: false,
            moderation_allow_audio: true,
            command_channel_enabled: true,
            command_url_enabled: false,
            command_show_enabled: true,
            command_regenerate_enabled: false,
            command_clear_enabled: true,
            command_lock_enabled: true,
            channel_lock: None,
            widget_x: Some(-640),
            widget_y: Some(120),
            widget_visible: true,
            widget_locked: true,
            notification_widget_x: Some(900),
            notification_widget_y: Some(40),
            notification_widget_visible: true,
            notification_widget_locked: true,
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
