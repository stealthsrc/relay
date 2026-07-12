use std::{
    collections::VecDeque,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};

use anyhow::Result;
use axum::body::Bytes;
use serenity::{cache::Cache, gateway::ShardManager, http::Http};
use tokio::{
    sync::{Mutex, RwLock, broadcast},
    task::JoinHandle,
};

use crate::{
    artwork::EmbeddedArtwork,
    config::{AppConfig, ConfigStore},
    model::{
        BotStatus, ChannelSummary, InterfacePreferences, MediaEvent, MediaKind, PendingMedia,
        RelayEvent, ServerStatus, StickerEvent, TtsEvent, TtsRequest, VisualSegment,
    },
    tts,
};

pub const HISTORY_LIMIT: usize = 50;
pub const MODERATION_QUEUE_LIMIT: usize = 50;
pub const ARTWORK_CACHE_LIMIT: usize = 50;
pub const MEDIA_AUDIO_CACHE_LIMIT: usize = 50;
pub const TTS_CACHE_LIMIT: usize = 50;
pub const PROCESSED_EMBED_LIMIT: usize = 500;
pub const MEDIA_CACHE_ITEM_LIMIT: usize = 30;
pub const MEDIA_CACHE_BYTE_LIMIT: usize = 100 * 1024 * 1024;
const TTS_SYNTHESIS_TIMEOUT: Duration = Duration::from_secs(15);

#[derive(Clone)]
pub struct TtsAudio {
    pub id: String,
    pub content_type: String,
    pub bytes: Bytes,
}

#[derive(Clone)]
pub struct MediaArtwork {
    pub id: String,
    pub content_type: String,
    pub bytes: Bytes,
}

#[derive(Clone)]
pub struct MediaAudio {
    pub id: String,
    pub content_type: String,
    pub bytes: Bytes,
}

#[derive(Clone)]
pub struct CachedMedia {
    pub id: String,
    pub content_type: String,
    pub bytes: Bytes,
}

pub struct BotRuntime {
    pub shard_manager: Arc<ShardManager>,
    pub task: JoinHandle<()>,
    pub http: Arc<Http>,
    pub cache: Arc<Cache>,
}

pub struct ServerRuntime {
    pub shutdown: tokio::sync::oneshot::Sender<()>,
    pub client_shutdown: broadcast::Sender<()>,
    pub task: JoinHandle<()>,
}

pub struct AppCore {
    pub config: RwLock<AppConfig>,
    pub config_store: ConfigStore,
    pub bot_status: RwLock<BotStatus>,
    pub server_status: RwLock<ServerStatus>,
    pub channels: RwLock<Vec<ChannelSummary>>,
    pub history: RwLock<VecDeque<MediaEvent>>,
    pub pending_media: RwLock<VecDeque<PendingMedia>>,
    pub tts_audio: RwLock<VecDeque<TtsAudio>>,
    pub media_artwork: RwLock<VecDeque<MediaArtwork>>,
    pub media_audio: RwLock<VecDeque<MediaAudio>>,
    pub tts_synthesis_lock: Mutex<()>,
    pub relay_tx: broadcast::Sender<RelayEvent>,
    pub bot_runtime: Mutex<Option<BotRuntime>>,
    pub server_runtime: Mutex<Option<ServerRuntime>>,
    pub panel_token: String,
    pub widget_move_generation: AtomicU64,
    pub notification_widget_move_generation: AtomicU64,
    pub interface_preferences: RwLock<InterfacePreferences>,
    pub processed_embed_ids: RwLock<VecDeque<String>>,
    pub cached_media: RwLock<VecDeque<CachedMedia>>,
    next_moderation_id: AtomicU64,
}

impl AppCore {
    pub fn load(config_path: PathBuf) -> Result<Arc<Self>> {
        let config_store = ConfigStore::new(config_path);
        let config = config_store.load()?;
        let (relay_tx, _) = broadcast::channel(2_048);
        Ok(Arc::new(Self {
            config: RwLock::new(config),
            config_store,
            bot_status: RwLock::new(BotStatus::default()),
            server_status: RwLock::new(ServerStatus::default()),
            channels: RwLock::new(Vec::new()),
            history: RwLock::new(VecDeque::with_capacity(HISTORY_LIMIT)),
            pending_media: RwLock::new(VecDeque::with_capacity(MODERATION_QUEUE_LIMIT)),
            tts_audio: RwLock::new(VecDeque::with_capacity(TTS_CACHE_LIMIT)),
            media_artwork: RwLock::new(VecDeque::with_capacity(ARTWORK_CACHE_LIMIT)),
            media_audio: RwLock::new(VecDeque::with_capacity(MEDIA_AUDIO_CACHE_LIMIT)),
            tts_synthesis_lock: Mutex::new(()),
            relay_tx,
            bot_runtime: Mutex::new(None),
            server_runtime: Mutex::new(None),
            panel_token: random_session_token(),
            widget_move_generation: AtomicU64::new(0),
            notification_widget_move_generation: AtomicU64::new(0),
            interface_preferences: RwLock::new(InterfacePreferences::default()),
            processed_embed_ids: RwLock::new(VecDeque::with_capacity(PROCESSED_EMBED_LIMIT)),
            cached_media: RwLock::new(VecDeque::with_capacity(MEDIA_CACHE_ITEM_LIMIT)),
            next_moderation_id: AtomicU64::new(1),
        }))
    }

    pub async fn set_config(&self, config: AppConfig) -> Result<()> {
        config.validate()?;
        self.config_store.save(&config)?;
        *self.config.write().await = config.clone();
        let mut pending = self.pending_media.write().await;
        if config.moderation_enabled {
            pending.retain(|item| media_type_allowed(&config, item.media.kind));
        } else {
            pending.clear();
        }
        drop(pending);
        let _ = self.relay_tx.send(RelayEvent::Config(config));
        Ok(())
    }

    pub async fn submit_media(&self, media: MediaEvent) {
        let config = self.config.read().await;
        if !config.moderation_enabled {
            drop(config);
            self.publish_media(media).await;
            return;
        }
        if !media_type_allowed(&config, media.kind) {
            return;
        }
        let mut pending = self.pending_media.write().await;
        if pending.len() >= MODERATION_QUEUE_LIMIT {
            return;
        }
        pending.push_back(PendingMedia {
            id: self.next_moderation_id.fetch_add(1, Ordering::Relaxed),
            media,
        });
    }

    pub async fn approve_media(&self, id: u64) -> bool {
        let media = {
            let mut pending = self.pending_media.write().await;
            pending
                .iter()
                .position(|item| item.id == id)
                .and_then(|index| pending.remove(index))
                .map(|item| item.media)
        };
        let Some(media) = media else {
            return false;
        };
        self.publish_media(media).await;
        true
    }

    pub async fn reject_media(&self, id: u64) -> bool {
        let mut pending = self.pending_media.write().await;
        let Some(index) = pending.iter().position(|item| item.id == id) else {
            return false;
        };
        pending.remove(index);
        true
    }

    pub async fn clear_pending_media(&self) {
        self.pending_media.write().await.clear();
    }

    pub async fn cache_artwork(&self, id: String, artwork: EmbeddedArtwork) {
        let mut cache = self.media_artwork.write().await;
        cache.retain(|item| item.id != id);
        cache.push_front(MediaArtwork {
            id,
            content_type: artwork.content_type,
            bytes: Bytes::from(artwork.bytes),
        });
        cache.truncate(ARTWORK_CACHE_LIMIT);
    }

    pub async fn cache_audio(&self, id: String, content_type: String, bytes: Vec<u8>) {
        let mut cache = self.media_audio.write().await;
        cache.retain(|item| item.id != id);
        cache.push_front(MediaAudio {
            id,
            content_type,
            bytes: Bytes::from(bytes),
        });
        cache.truncate(MEDIA_AUDIO_CACHE_LIMIT);
    }

    pub async fn claim_embed(&self, id: String) -> bool {
        let mut processed = self.processed_embed_ids.write().await;
        if processed.contains(&id) {
            return false;
        }
        processed.push_front(id);
        processed.truncate(PROCESSED_EMBED_LIMIT);
        true
    }

    pub async fn cache_media(&self, id: String, content_type: String, bytes: Vec<u8>) {
        let mut cache = self.cached_media.write().await;
        cache.retain(|item| item.id != id);
        cache.push_front(CachedMedia {
            id,
            content_type,
            bytes: Bytes::from(bytes),
        });
        while cache.len() > MEDIA_CACHE_ITEM_LIMIT
            || cache.iter().map(|item| item.bytes.len()).sum::<usize>() > MEDIA_CACHE_BYTE_LIMIT
        {
            cache.pop_back();
        }
    }

    pub async fn publish_media(&self, media: MediaEvent) {
        {
            let mut history = self.history.write().await;
            history.push_front(media.clone());
            history.truncate(HISTORY_LIMIT);
        }
        let _ = self.relay_tx.send(RelayEvent::Media(media));
    }

    pub fn publish_sticker(&self, sticker: StickerEvent) {
        let _ = self.relay_tx.send(RelayEvent::Sticker(sticker));
    }

    pub async fn publish_tts(&self, request: TtsRequest) -> Result<()> {
        let _synthesis = self.tts_synthesis_lock.lock().await;
        let speech = tokio::time::timeout(
            TTS_SYNTHESIS_TIMEOUT,
            tts::synthesize(request.text.clone()),
        )
        .await
        .map_err(|_| anyhow::anyhow!("Windows TTS timed out; the queue was released"))??;
        let event = TtsEvent {
            id: request.id.clone(),
            text: request.text,
            author: request.author,
            content_type: speech.content_type.clone(),
            timestamp: request.timestamp,
            visual_only: false,
            segments: Vec::new(),
        };
        {
            let mut cache = self.tts_audio.write().await;
            cache.push_front(TtsAudio {
                id: request.id,
                content_type: speech.content_type,
                bytes: Bytes::from(speech.bytes),
            });
            cache.truncate(TTS_CACHE_LIMIT);
        }
        let _ = self.relay_tx.send(RelayEvent::Tts(event));
        Ok(())
    }

    pub fn publish_visual_tts(
        &self,
        id: String,
        text: String,
        author: crate::model::AuthorIdentity,
        timestamp: u64,
        segments: Vec<VisualSegment>,
    ) {
        let event = TtsEvent {
            id,
            text,
            author,
            content_type: String::new(),
            timestamp,
            visual_only: true,
            segments,
        };
        let _ = self.relay_tx.send(RelayEvent::Tts(event));
    }
}

fn media_type_allowed(config: &AppConfig, kind: MediaKind) -> bool {
    match kind {
        MediaKind::Image | MediaKind::Gif => config.moderation_allow_images,
        MediaKind::Video => config.moderation_allow_videos,
        MediaKind::Audio => config.moderation_allow_audio,
    }
}

fn random_session_token() -> String {
    use rand::{RngCore, rng};

    let mut bytes = [0_u8; 32];
    rng().fill_bytes(&mut bytes);
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn media(kind: MediaKind, message_id: &str) -> MediaEvent {
        MediaEvent {
            kind,
            url: "https://cdn.discordapp.com/media".into(),
            proxy_url: "https://media.discordapp.net/media".into(),
            filename: "media.bin".into(),
            content_type: "application/octet-stream".into(),
            artwork_id: None,
            audio_id: None,
            cached_media_id: None,
            title: None,
            artist: None,
            author: crate::model::AuthorIdentity {
                username: "Moderator".into(),
                display_avatar_url: "https://cdn.discordapp.com/avatar.png".into(),
            },
            timestamp: 42,
            message_id: message_id.into(),
        }
    }

    #[tokio::test]
    async fn moderates_allowed_media_and_rejects_disabled_types() {
        let directory = tempfile::tempdir().unwrap();
        let core = AppCore::load(directory.path().join("config.json")).unwrap();
        let mut events = core.relay_tx.subscribe();
        core.set_config(AppConfig {
            moderation_enabled: true,
            moderation_allow_videos: false,
            ..AppConfig::default()
        })
        .await
        .unwrap();
        let _ = events.recv().await.unwrap();

        core.submit_media(media(MediaKind::Image, "1")).await;
        core.submit_media(media(MediaKind::Video, "2")).await;
        assert_eq!(core.pending_media.read().await.len(), 1);
        assert!(events.try_recv().is_err());

        let id = core.pending_media.read().await[0].id;
        assert!(core.approve_media(id).await);
        assert!(matches!(events.recv().await.unwrap(), RelayEvent::Media(_)));
        assert_eq!(core.history.read().await.len(), 1);
        assert!(!core.reject_media(id).await);
    }

    #[tokio::test]
    async fn synthesizes_caches_and_broadcasts_tts_audio() {
        let directory = tempfile::tempdir().unwrap();
        let core = AppCore::load(directory.path().join("config.json")).unwrap();
        let mut events = core.relay_tx.subscribe();

        core.publish_tts(TtsRequest {
            id: "123456789012345678".into(),
            text: "Relay queue test".into(),
            author: crate::model::AuthorIdentity {
                username: "Queue tester".into(),
                display_avatar_url: "https://cdn.discordapp.com/avatar.png".into(),
            },
            timestamp: 42,
        })
        .await
        .unwrap();

        let event = events.recv().await.unwrap();
        let RelayEvent::Tts(event) = event else {
            panic!("expected a TTS relay event");
        };
        assert_eq!(event.id, "123456789012345678");
        assert_eq!(event.text, "Relay queue test");
        assert_eq!(event.author.username, "Queue tester");
        assert_eq!(event.content_type, "audio/wav");
        let cache = core.tts_audio.read().await;
        assert_eq!(cache.len(), 1);
        assert!(cache[0].bytes.starts_with(b"RIFF"));
    }

    #[tokio::test]
    async fn broadcasts_visual_tts_without_touching_the_audio_cache() {
        let directory = tempfile::tempdir().unwrap();
        let core = AppCore::load(directory.path().join("config.json")).unwrap();
        let mut events = core.relay_tx.subscribe();

        core.publish_visual_tts(
            "123456789012345678".into(),
            "test".into(),
            crate::model::AuthorIdentity {
                username: "Silent tester".into(),
                display_avatar_url: "https://cdn.discordapp.com/avatar.png".into(),
            },
            42,
            vec![VisualSegment {
                kind: "text".into(),
                value: "test".into(),
                url: None,
                animated: false,
            }],
        );

        let RelayEvent::Tts(event) = events.recv().await.unwrap() else {
            panic!("expected a TTS relay event");
        };
        assert!(event.visual_only);
        assert_eq!(event.text, "test");
        assert_eq!(event.segments.len(), 1);
        assert!(core.tts_audio.read().await.is_empty());
    }

    #[tokio::test]
    async fn claims_delayed_embeds_only_once() {
        let directory = tempfile::tempdir().unwrap();
        let core = AppCore::load(directory.path().join("config.json")).unwrap();
        assert!(core.claim_embed("message-embed-0".into()).await);
        assert!(!core.claim_embed("message-embed-0".into()).await);
    }

}
