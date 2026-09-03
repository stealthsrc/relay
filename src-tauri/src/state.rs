use std::{
    collections::{HashMap, VecDeque},
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
    },
    time::Duration,
};

use anyhow::{Result, bail};
use axum::body::Bytes;
use serenity::{cache::Cache, gateway::ShardManager, http::Http};
use tokio::{
    sync::{Mutex, RwLock, broadcast, mpsc, oneshot},
    task::JoinHandle,
};

use crate::{
    artwork,
    artwork::EmbeddedArtwork,
    config::{AppConfig, ConfigStore},
    custom_commands::CustomCommandConfirmations,
    media_compat::{self, VideoCompatibility},
    model::{
        BotStatus, ChannelSummary, InterfacePreferences, MediaEvent, MediaKind, MusicPlaybackEvent,
        MusicPlaybackMode, MusicStopEvent, PendingMedia, RelayEvent, ServerStatus, StickerEvent,
        TtsEvent, TtsRequest, VisualSegment,
    },
    music::{MusicSelection, MusicState},
    privacy::{self, PrivacyAction, PrivacyReport},
    stage_scheduler::{StageLane, StageScheduler, StageTicket},
    tts,
};

pub const HISTORY_LIMIT: usize = 50;
pub const MODERATION_QUEUE_LIMIT: usize = 50;
pub const ARTWORK_CACHE_LIMIT: usize = 50;
pub const MEDIA_AUDIO_CACHE_LIMIT: usize = 50;
pub const MEDIA_AUDIO_CACHE_BYTE_LIMIT: usize = 200 * 1024 * 1024;
pub const TTS_CACHE_LIMIT: usize = 50;
pub const PROCESSED_EMBED_LIMIT: usize = 500;
pub const MEDIA_CACHE_ITEM_LIMIT: usize = 30;
pub const MEDIA_CACHE_BYTE_LIMIT: usize = 100 * 1024 * 1024;
const TTS_SYNTHESIS_TIMEOUT: Duration = Duration::from_secs(15);
const MEDIA_DELIVERY_TIMEOUT: Duration = Duration::from_secs(3);

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
    pub port: u16,
}

pub struct MediaDeliveryRequest {
    pub kind: MediaKind,
    pub ready: oneshot::Sender<()>,
}

pub struct AppCore {
    pub config: RwLock<AppConfig>,
    pub config_store: ConfigStore,
    config_mutation: Mutex<()>,
    tts_pending_count: AtomicUsize,
    pub bot_status: RwLock<BotStatus>,
    pub server_status: RwLock<ServerStatus>,
    pub channels: RwLock<Vec<ChannelSummary>>,
    pub history: RwLock<VecDeque<MediaEvent>>,
    pub pending_media: RwLock<VecDeque<PendingMedia>>,
    pub tts_audio: RwLock<VecDeque<TtsAudio>>,
    pub media_artwork: RwLock<VecDeque<MediaArtwork>>,
    pub media_audio: RwLock<VecDeque<MediaAudio>>,
    pub tts_synthesis_lock: Mutex<()>,
    pub music: Mutex<MusicState>,
    pub music_cleanup: Mutex<crate::music_cleanup::MusicCleanup>,
    music_stage_tickets: Mutex<HashMap<String, StageTicket>>,
    pub relay_tx: broadcast::Sender<RelayEvent>,
    pub stage_scheduler: StageScheduler,
    pub bot_runtime: Mutex<Option<BotRuntime>>,
    pub custom_command_sync: Mutex<()>,
    pub custom_command_confirmations: Mutex<CustomCommandConfirmations>,
    pub server_runtime: Mutex<Option<ServerRuntime>>,
    pub panel_token: String,
    pub widget_move_generation: AtomicU64,
    pub widget_resize_generation: AtomicU64,
    pub notification_widget_move_generation: AtomicU64,
    pub notification_widget_resize_generation: AtomicU64,
    /// Runtime-only marker for a media widget auto-woken by music playback.
    pub widget_ephemeral_wake: AtomicBool,
    pub interface_preferences: RwLock<InterfacePreferences>,
    pub processed_embed_ids: RwLock<VecDeque<String>>,
    pub cached_media: RwLock<VecDeque<CachedMedia>>,
    pending_privacy_roles: RwLock<HashMap<u64, Vec<String>>>,
    media_delivery: RwLock<Option<mpsc::UnboundedSender<MediaDeliveryRequest>>>,
    next_moderation_id: AtomicU64,
}

impl AppCore {
    pub fn load(config_path: PathBuf) -> Result<Arc<Self>> {
        let config_store = ConfigStore::new(config_path);
        let config = config_store.load()?;
        let interface_preferences = config.interface_preferences.clone();
        let (relay_tx, _) = broadcast::channel(2_048);
        let stage_scheduler = StageScheduler::new(relay_tx.clone());
        Ok(Arc::new(Self {
            config: RwLock::new(config),
            config_store,
            config_mutation: Mutex::new(()),
            tts_pending_count: AtomicUsize::new(0),
            bot_status: RwLock::new(BotStatus::default()),
            server_status: RwLock::new(ServerStatus::default()),
            channels: RwLock::new(Vec::new()),
            history: RwLock::new(VecDeque::with_capacity(HISTORY_LIMIT)),
            pending_media: RwLock::new(VecDeque::with_capacity(MODERATION_QUEUE_LIMIT)),
            tts_audio: RwLock::new(VecDeque::with_capacity(TTS_CACHE_LIMIT)),
            media_artwork: RwLock::new(VecDeque::with_capacity(ARTWORK_CACHE_LIMIT)),
            media_audio: RwLock::new(VecDeque::with_capacity(MEDIA_AUDIO_CACHE_LIMIT)),
            tts_synthesis_lock: Mutex::new(()),
            music: Mutex::new(MusicState::default()),
            music_cleanup: Mutex::new(crate::music_cleanup::MusicCleanup::default()),
            music_stage_tickets: Mutex::new(HashMap::new()),
            relay_tx,
            stage_scheduler,
            bot_runtime: Mutex::new(None),
            custom_command_sync: Mutex::new(()),
            custom_command_confirmations: Mutex::new(CustomCommandConfirmations::default()),
            server_runtime: Mutex::new(None),
            panel_token: random_session_token(),
            widget_move_generation: AtomicU64::new(0),
            widget_resize_generation: AtomicU64::new(0),
            notification_widget_move_generation: AtomicU64::new(0),
            notification_widget_resize_generation: AtomicU64::new(0),
            widget_ephemeral_wake: AtomicBool::new(false),
            interface_preferences: RwLock::new(interface_preferences),
            processed_embed_ids: RwLock::new(VecDeque::with_capacity(PROCESSED_EMBED_LIMIT)),
            cached_media: RwLock::new(VecDeque::with_capacity(MEDIA_CACHE_ITEM_LIMIT)),
            pending_privacy_roles: RwLock::new(HashMap::new()),
            media_delivery: RwLock::new(None),
            next_moderation_id: AtomicU64::new(1),
        }))
    }

    pub async fn set_media_delivery(&self, sender: mpsc::UnboundedSender<MediaDeliveryRequest>) {
        *self.media_delivery.write().await = Some(sender);
    }

    pub async fn register_stage_output(
        &self,
        timestamp_ms: u64,
        message_id: &str,
        part: u16,
        lane: StageLane,
    ) -> StageTicket {
        self.stage_scheduler
            .reserve(timestamp_ms, message_id, part, lane)
            .await
    }

    pub async fn complete_media(&self, ticket: StageTicket, event: MediaEvent) {
        self.stage_scheduler
            .ready(ticket, RelayEvent::Media(event))
            .await;
    }

    pub async fn complete_sticker(&self, ticket: StageTicket, event: StickerEvent) {
        self.stage_scheduler
            .ready(ticket, RelayEvent::Sticker(event))
            .await;
    }

    pub async fn complete_tts(&self, ticket: StageTicket, event: TtsEvent) {
        self.stage_scheduler
            .ready(ticket, RelayEvent::Tts(event))
            .await;
    }

    pub async fn complete_music(&self, ticket: StageTicket, event: MusicPlaybackEvent) {
        self.stage_scheduler
            .ready(ticket, RelayEvent::MusicPlay(event))
            .await;
    }

    pub async fn cancel_stage_output(&self, ticket: StageTicket) {
        self.stage_scheduler.cancel(ticket).await;
    }

    pub async fn start_music(
        &self,
        selection: MusicSelection,
        mode: MusicPlaybackMode,
        order_timestamp: u64,
        order_id: &str,
    ) -> crate::music::MusicStartResult {
        let ticket = self
            .register_stage_output(order_timestamp, order_id, 300, StageLane::Music)
            .await;
        let result = self.music.lock().await.start(selection, mode);
        self.emit_music_start(result, ticket).await
    }

    pub async fn start_music_custom(
        &self,
        selection: MusicSelection,
        start_seconds: u64,
        end_seconds: u64,
        order_timestamp: u64,
        order_id: &str,
    ) -> Result<crate::music::MusicStartResult, crate::music::CustomRangeError> {
        let ticket = self
            .register_stage_output(order_timestamp, order_id, 300, StageLane::Music)
            .await;
        let result = self
            .music
            .lock()
            .await
            .start_custom(selection, start_seconds, end_seconds);
        match result {
            Ok(result) => Ok(self.emit_music_start(result, ticket).await),
            Err(error) => {
                self.cancel_stage_output(ticket).await;
                Err(error)
            }
        }
    }

    async fn emit_music_start(
        &self,
        result: crate::music::MusicStartResult,
        ticket: StageTicket,
    ) -> crate::music::MusicStartResult {
        match &result {
            crate::music::MusicStartResult::Started(playback) => {
                self.complete_music(ticket, playback.clone()).await;
            }
            crate::music::MusicStartResult::Queued { playback, .. } => {
                self.music_stage_tickets
                    .lock()
                    .await
                    .insert(playback.playback_id.clone(), ticket);
            }
            crate::music::MusicStartResult::QueueFull => {
                self.cancel_stage_output(ticket).await;
            }
        }
        result
    }

    pub async fn current_music(&self) -> Option<MusicPlaybackEvent> {
        self.music.lock().await.current_event()
    }

    pub async fn cancel_pending_music(&self, playback_id: &str) {
        let stopped = self.music.lock().await.remove_pending(playback_id);
        if let Some(stopped) = stopped {
            let ticket = self.music_stage_tickets.lock().await.remove(playback_id);
            if let Some(ticket) = ticket {
                self.cancel_stage_output(ticket).await;
            }
            self.delete_now_playing_message(stopped.channel_id, stopped.now_playing_message_id)
                .await;
        }
    }

    pub async fn stop_current_music(&self) -> Option<MusicPlaybackEvent> {
        let (stopped, next) = {
            let mut music = self.music.lock().await;
            let stopped = music.stop_current();
            let next = if stopped.is_some() {
                music.promote_next()
            } else {
                None
            };
            (stopped, next)
        };
        if let Some(stopped) = &stopped {
            self.delete_now_playing_message(stopped.channel_id, stopped.now_playing_message_id)
                .await;
            let _ = self.relay_tx.send(RelayEvent::MusicStop(MusicStopEvent {
                playback_id: stopped.playback.playback_id.clone(),
            }));
            self.emit_music_follow_up(next).await;
            return Some(stopped.playback.clone());
        }
        None
    }

    /// Clear current music and the entire pending queue (force-clear / overlay clear).
    pub async fn clear_all_music(&self) -> Option<MusicPlaybackEvent> {
        let (cards, stopped) = {
            let mut music = self.music.lock().await;
            (music.active_message_ids(), music.clear_all())
        };
        self.music_stage_tickets.lock().await.clear();
        for (channel, message) in cards {
            self.delete_now_playing_message(channel, Some(message))
                .await;
        }
        if let Some(stopped) = &stopped {
            let _ = self.relay_tx.send(RelayEvent::MusicStop(MusicStopEvent {
                playback_id: stopped.playback.playback_id.clone(),
            }));
            // Always idle after a full clear so TTS/notification clients resume
            // even when the following Clear event is coalesced or lagged.
            let _ = self.relay_tx.send(RelayEvent::MusicIdle);
            return Some(stopped.playback.clone());
        }
        None
    }

    pub async fn stop_music_if_current(&self, playback_id: &str) -> Option<MusicPlaybackEvent> {
        let (stopped, next) = {
            let mut music = self.music.lock().await;
            let stopped = music.stop_if_current(playback_id);
            let next = if stopped.is_some() {
                music.promote_next()
            } else {
                None
            };
            (stopped, next)
        };
        if let Some(stopped) = &stopped {
            self.delete_now_playing_message(stopped.channel_id, stopped.now_playing_message_id)
                .await;
            let _ = self.relay_tx.send(RelayEvent::MusicStop(MusicStopEvent {
                playback_id: stopped.playback.playback_id.clone(),
            }));
            self.emit_music_follow_up(next).await;
            return Some(stopped.playback.clone());
        }
        None
    }

    /// GUI / hotkey skip: skip current YouTube if any (promoting the queue), else skip media.
    pub async fn skip_playback(&self) {
        if self.stop_current_music().await.is_some() {
            return;
        }
        let _ = self.relay_tx.send(RelayEvent::MusicStop(MusicStopEvent {
            playback_id: String::new(),
        }));
        let _ = self.relay_tx.send(RelayEvent::Skip);
        self.stage_scheduler.skip_active().await;
    }

    pub async fn finish_music(&self, playback_id: &str) -> bool {
        let (stopped, repeated, next) = {
            let mut music = self.music.lock().await;
            let Some((stopped, repeated)) = music.finish_current(playback_id) else {
                return false;
            };
            let next = music.promote_next();
            (stopped, repeated, next)
        };
        if !repeated {
            self.delete_now_playing_message(stopped.channel_id, stopped.now_playing_message_id)
                .await;
        }
        let _ = self.relay_tx.send(RelayEvent::MusicStop(MusicStopEvent {
            playback_id: stopped.playback.playback_id.clone(),
        }));
        // Promote next or signal idle so overlays can resume media.
        self.emit_music_follow_up(next).await;
        true
    }

    pub(crate) async fn delete_now_playing_message(
        &self,
        channel_id: u64,
        message_id: Option<u64>,
    ) {
        let Some(message_id) = message_id else {
            return;
        };
        let config = self.config.read().await;
        if config.music_channel_id == channel_id.to_string()
            && config.music_welcome_message_id == message_id.to_string()
        {
            return;
        }
        drop(config);
        let http = {
            let runtime = self.bot_runtime.lock().await;
            runtime.as_ref().map(|runtime| runtime.http.clone())
        };
        let Some(http) = http else {
            return;
        };
        if serenity::model::id::ChannelId::new(channel_id)
            .delete_message(&*http, serenity::model::id::MessageId::new(message_id))
            .await
            .is_err()
        {
            // Soft-fail: missing Manage Messages (or already deleted) must not break playback.
            eprintln!(
                "relay: failed to delete now-playing message {message_id} in channel {channel_id}"
            );
        }
    }

    async fn emit_music_follow_up(&self, next: Option<MusicPlaybackEvent>) {
        if let Some(playback) = next {
            // Release the previous scheduler ticket before the next queued track
            // competes for the shared stage.
            let _ = self.relay_tx.send(RelayEvent::MusicIdle);
            let ticket = self
                .music_stage_tickets
                .lock()
                .await
                .remove(&playback.playback_id);
            if let Some(ticket) = ticket {
                self.complete_music(ticket, playback).await;
            } else {
                self.stage_scheduler
                    .enqueue(RelayEvent::MusicPlay(playback), StageLane::Music)
                    .await;
            }
            crate::bot::refresh_music_card(self).await;
        } else {
            let _ = self.relay_tx.send(RelayEvent::MusicIdle);
        }
    }

    pub async fn set_config(&self, config: AppConfig) -> Result<()> {
        let _mutation = self.config_mutation.lock().await;
        self.persist_config(config).await
    }

    /// Applies a partial mutation while holding the mutation lock across the
    /// read-modify-save cycle, so concurrent writers cannot lose updates.
    pub async fn update_config<F>(&self, mutate: F) -> Result<AppConfig>
    where
        F: FnOnce(&mut AppConfig),
    {
        let _mutation = self.config_mutation.lock().await;
        let mut config = self.config.read().await.clone();
        mutate(&mut config);
        self.persist_config(config.clone()).await?;
        Ok(config)
    }

    async fn persist_config(&self, config: AppConfig) -> Result<()> {
        config.validate()?;
        let store = self.config_store.clone();
        let to_save = config.clone();
        tokio::task::spawn_blocking(move || store.save(&to_save)).await??;
        *self.config.write().await = config.clone();
        let mut pending = self.pending_media.write().await;
        if config.moderation_enabled {
            pending.retain(|item| {
                item.privacy_classification.is_some()
                    || media_type_allowed(&config, item.media.kind)
            });
        } else {
            // A privacy Review item is independent from manual moderation and
            // must remain actionable when the manual switch is turned off.
            pending.retain(|item| item.privacy_classification.is_some());
        }
        drop(pending);
        let pending_ids = self
            .pending_media
            .read()
            .await
            .iter()
            .map(|item| item.id)
            .collect::<std::collections::HashSet<_>>();
        self.pending_privacy_roles
            .write()
            .await
            .retain(|pending_id, _| pending_ids.contains(pending_id));
        let _ = self.relay_tx.send(RelayEvent::Config(Box::new(config)));
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn submit_media(&self, media: MediaEvent) {
        self.submit_analyzed_media_with_text(media, None, None)
            .await;
    }

    #[allow(dead_code)]
    pub async fn submit_analyzed_media(&self, media: MediaEvent, report: Option<PrivacyReport>) {
        self.submit_analyzed_media_with_text(media, report, None)
            .await;
    }

    pub async fn submit_analyzed_media_with_text(
        &self,
        media: MediaEvent,
        report: Option<PrivacyReport>,
        full_text: Option<&str>,
    ) {
        self.submit_analyzed_media_with_text_and_roles(media, report, full_text, &[])
            .await;
    }

    pub async fn submit_analyzed_media_with_text_and_roles(
        &self,
        media: MediaEvent,
        report: Option<PrivacyReport>,
        full_text: Option<&str>,
        role_ids: &[String],
    ) {
        let ticket = self
            .register_stage_output(media.timestamp, &media.message_id, 200, StageLane::Media)
            .await;
        self.submit_analyzed_media_with_ticket_and_roles(
            ticket, media, report, full_text, role_ids,
        )
        .await;
    }

    pub async fn submit_analyzed_media_with_ticket_and_roles(
        &self,
        ticket: StageTicket,
        media: MediaEvent,
        report: Option<PrivacyReport>,
        full_text: Option<&str>,
        role_ids: &[String],
    ) {
        let config = self.config.read().await.clone();
        let scoped_config = privacy::scoped_config_for_roles(&config, role_ids);
        let role_exempt = privacy::has_exempt_role(&config, role_ids);
        let analysis_text = privacy_media_text(&media, full_text);
        // Always classify the message text again at this boundary. The bot's
        // asynchronous report is advisory, while this state gate is the last
        // decision point before history and relay publication.
        let mut authoritative_report = privacy::classify_text(Some(&analysis_text), &scoped_config);
        if let Some(report) = report {
            let mut report = if role_exempt {
                report.without_filter_signals()
            } else {
                report
            };
            if role_exempt {
                report.config_signature = Some(privacy::config_signature(&scoped_config));
            }
            let report_is_stale = privacy::privacy_rules_enabled(&scoped_config)
                && report
                    .config_signature
                    .is_none_or(|signature| signature != privacy::config_signature(&scoped_config));
            authoritative_report.merge(report);
            if report_is_stale
                && authoritative_report.classification == privacy::PrivacyClassification::Safe
            {
                authoritative_report.merge(PrivacyReport::suspicious("scan_config_changed"));
            }
        } else if config.privacy_scan_enabled
            && media_requires_image_scan(&media)
            && authoritative_report.classification == privacy::PrivacyClassification::Safe
        {
            authoritative_report = PrivacyReport::suspicious("image_scan_unavailable");
        }
        authoritative_report.apply_score_policy(&scoped_config);
        let privacy_action = if privacy::privacy_rules_enabled(&scoped_config) {
            privacy::action_for(&authoritative_report, &scoped_config)
        } else {
            PrivacyAction::Allow
        };
        privacy::log_decision(&authoritative_report, privacy_action);
        if matches!(privacy_action, PrivacyAction::Block) {
            self.cancel_stage_output(ticket).await;
            return;
        }
        let manually_moderated = config.moderation_enabled;
        let type_allowed = media_type_allowed(&config, media.kind);
        let requires_review =
            matches!(privacy_action, PrivacyAction::Review) || (manually_moderated && type_allowed);
        if manually_moderated && !type_allowed {
            self.cancel_stage_output(ticket).await;
            return;
        }
        if !requires_review {
            drop(config);
            self.publish_media_with_ticket(ticket, media).await;
            return;
        }
        self.cancel_stage_output(ticket).await;
        let mut pending = self.pending_media.write().await;
        // Evict the oldest unreviewed item instead of silently dropping new media.
        let evicted_pending_id = if pending.len() >= MODERATION_QUEUE_LIMIT {
            pending.pop_front().map(|item| item.id)
        } else {
            None
        };
        let pending_id = self.next_moderation_id.fetch_add(1, Ordering::Relaxed);
        pending.push_back(PendingMedia {
            id: pending_id,
            media,
            sticker: None,
            sticker_bytes: None,
            privacy_classification: (authoritative_report.classification
                != privacy::PrivacyClassification::Safe)
                .then_some(authoritative_report.classification),
            privacy_categories: authoritative_report.categories.clone(),
            privacy_reason: authoritative_report.primary_reason().map(str::to_owned),
        });
        drop(pending);
        if let Some(evicted_pending_id) = evicted_pending_id {
            self.pending_privacy_roles
                .write()
                .await
                .remove(&evicted_pending_id);
        }
        if !role_ids.is_empty() {
            self.pending_privacy_roles
                .write()
                .await
                .insert(pending_id, role_ids.to_vec());
        }
    }

    #[allow(dead_code)]
    pub async fn submit_sticker(
        &self,
        sticker: StickerEvent,
        text: Option<&str>,
        bytes: Option<Vec<u8>>,
        report: Option<PrivacyReport>,
    ) {
        self.submit_sticker_with_roles(sticker, text, bytes, report, &[])
            .await;
    }

    pub async fn submit_sticker_with_roles(
        &self,
        sticker: StickerEvent,
        text: Option<&str>,
        bytes: Option<Vec<u8>>,
        report: Option<PrivacyReport>,
        role_ids: &[String],
    ) {
        let ticket = self
            .register_stage_output(
                sticker.timestamp,
                &sticker.message_id,
                100,
                StageLane::Media,
            )
            .await;
        self.submit_sticker_with_ticket_and_roles(ticket, sticker, text, bytes, report, role_ids)
            .await;
    }

    pub async fn submit_sticker_with_ticket_and_roles(
        &self,
        ticket: StageTicket,
        mut sticker: StickerEvent,
        text: Option<&str>,
        bytes: Option<Vec<u8>>,
        report: Option<PrivacyReport>,
        role_ids: &[String],
    ) {
        let config = self.config.read().await;
        let scoped_config = privacy::scoped_config_for_roles(&config, role_ids);
        let role_exempt = privacy::has_exempt_role(&config, role_ids);
        let analysis_text = format!("{}\n{}", text.unwrap_or_default(), sticker.name);
        let mut authoritative_report = privacy::classify_text(Some(&analysis_text), &scoped_config);
        if let Some(report) = report {
            let mut report = if role_exempt {
                report.without_filter_signals()
            } else {
                report
            };
            if role_exempt {
                report.config_signature = Some(privacy::config_signature(&scoped_config));
            }
            let report_is_stale = privacy::privacy_rules_enabled(&scoped_config)
                && report
                    .config_signature
                    .is_none_or(|signature| signature != privacy::config_signature(&scoped_config));
            authoritative_report.merge(report);
            if report_is_stale
                && authoritative_report.classification == privacy::PrivacyClassification::Safe
            {
                authoritative_report.merge(PrivacyReport::suspicious("scan_config_changed"));
            }
        } else if config.privacy_scan_enabled {
            match bytes.as_deref() {
                Some(bytes) => authoritative_report.merge(
                    privacy::analyze_image_bytes_async(bytes, Some(&analysis_text), &scoped_config)
                        .await,
                ),
                None => authoritative_report.merge(PrivacyReport::suspicious("scan_incomplete")),
            }
        }
        if config.privacy_scan_enabled
            && bytes.is_none()
            && authoritative_report.classification == privacy::PrivacyClassification::Safe
        {
            authoritative_report.merge(PrivacyReport::suspicious("scan_incomplete"));
        }
        authoritative_report.apply_score_policy(&scoped_config);
        let privacy_action = if privacy::privacy_rules_enabled(&scoped_config) {
            privacy::action_for(&authoritative_report, &scoped_config)
        } else {
            PrivacyAction::Allow
        };
        privacy::log_decision(&authoritative_report, privacy_action);
        if matches!(privacy_action, PrivacyAction::Block) {
            self.cancel_stage_output(ticket).await;
            return;
        }
        let media_kind = if sticker.format == "gif" {
            MediaKind::Gif
        } else {
            MediaKind::Image
        };
        let content_type = match sticker.format.as_str() {
            "gif" => "image/gif",
            "apng" => "image/apng",
            _ => "image/png",
        };
        let media = MediaEvent {
            kind: media_kind,
            url: sticker.url.clone(),
            proxy_url: sticker.url.clone(),
            filename: sticker.name.clone(),
            content_type: content_type.into(),
            artwork_id: None,
            audio_id: None,
            cached_media_id: None,
            title: None,
            artist: None,
            text: None,
            author: sticker.author.clone(),
            timestamp: sticker.timestamp,
            message_id: sticker.message_id.clone(),
        };
        let manually_moderated = config.moderation_enabled;
        let type_allowed = media_type_allowed(&config, media.kind);
        if manually_moderated && !type_allowed {
            self.cancel_stage_output(ticket).await;
            return;
        }
        let requires_review =
            matches!(privacy_action, PrivacyAction::Review) || (manually_moderated && type_allowed);
        if requires_review {
            self.cancel_stage_output(ticket).await;
            let mut pending = self.pending_media.write().await;
            let evicted_pending_id = if pending.len() >= MODERATION_QUEUE_LIMIT {
                pending.pop_front().map(|item| item.id)
            } else {
                None
            };
            let pending_id = self.next_moderation_id.fetch_add(1, Ordering::Relaxed);
            pending.push_back(PendingMedia {
                id: pending_id,
                media,
                sticker: Some(sticker),
                sticker_bytes: bytes.map(Arc::new),
                privacy_classification: (authoritative_report.classification
                    != privacy::PrivacyClassification::Safe)
                    .then_some(authoritative_report.classification),
                privacy_categories: authoritative_report.categories.clone(),
                privacy_reason: authoritative_report.primary_reason().map(str::to_owned),
            });
            drop(pending);
            if let Some(evicted_pending_id) = evicted_pending_id {
                self.pending_privacy_roles
                    .write()
                    .await
                    .remove(&evicted_pending_id);
            }
            if !role_ids.is_empty() {
                self.pending_privacy_roles
                    .write()
                    .await
                    .insert(pending_id, role_ids.to_vec());
            }
            return;
        }
        if let Some(bytes) = bytes {
            let cache_id = format!("sticker-{}", sticker.id);
            self.cache_media(cache_id.clone(), content_type.into(), bytes)
                .await;
            sticker.cached_media_id = Some(cache_id);
        }
        self.publish_sticker_with_ticket(ticket, sticker).await;
    }

    pub async fn approve_media(&self, id: u64) -> bool {
        let item = {
            let pending = self.pending_media.read().await;
            pending.iter().find(|item| item.id == id).cloned()
        };
        let Some(item) = item else {
            return false;
        };
        let config = self.config.read().await.clone();
        let role_ids = self
            .pending_privacy_roles
            .read()
            .await
            .get(&item.id)
            .cloned()
            .unwrap_or_default();
        let scoped_config = privacy::scoped_config_for_roles(&config, &role_ids);
        if privacy::privacy_rules_enabled(&scoped_config) {
            let analysis_text = privacy_media_text(&item.media, None);
            let mut report = if scoped_config.privacy_scan_enabled
                && media_requires_image_scan(&item.media)
            {
                if let Some(bytes) = item.sticker_bytes.as_deref() {
                    privacy::analyze_image_bytes_async(bytes, Some(&analysis_text), &scoped_config)
                        .await
                } else {
                    privacy::analyze_remote_image(
                        &item.media.url,
                        &item.media.proxy_url,
                        Some(&analysis_text),
                        &scoped_config,
                    )
                    .await
                }
            } else {
                privacy::classify_text(Some(&analysis_text), &scoped_config)
            };
            if scoped_config.privacy_scan_enabled
                && let Some(artwork_id) = item.media.artwork_id.as_deref()
            {
                if let Some(bytes) = self.cached_artwork_bytes(artwork_id).await {
                    report.merge(
                        privacy::analyze_image_bytes_async(
                            &bytes,
                            Some(&analysis_text),
                            &scoped_config,
                        )
                        .await,
                    );
                } else {
                    report.merge(PrivacyReport::low("scan_incomplete"));
                }
            }
            report.apply_score_policy(&scoped_config);
            let action = privacy::action_for(&report, &scoped_config);
            if matches!(action, PrivacyAction::Block) {
                privacy::log_decision(&report, action);
                return false;
            }
        }
        let Some((media, sticker, sticker_bytes)) = ({
            let mut pending = self.pending_media.write().await;
            pending
                .iter()
                .position(|pending_item| pending_item.id == id)
                .and_then(|index| pending.remove(index))
                .map(|pending_item| {
                    (
                        pending_item.media,
                        pending_item.sticker,
                        pending_item.sticker_bytes,
                    )
                })
        }) else {
            return false;
        };
        self.pending_privacy_roles.write().await.remove(&item.id);
        if let Some(mut sticker_event) = sticker {
            if let Some(bytes) = sticker_bytes {
                let cache_id = format!("sticker-{}", sticker_event.id);
                self.cache_media(
                    cache_id.clone(),
                    media.content_type.clone(),
                    (*bytes).clone(),
                )
                .await;
                sticker_event.cached_media_id = Some(cache_id);
            }
            self.publish_sticker(sticker_event).await;
            return true;
        }
        self.publish_media(media).await;
        true
    }

    pub async fn reject_media(&self, id: u64) -> bool {
        let mut pending = self.pending_media.write().await;
        let Some(index) = pending.iter().position(|item| item.id == id) else {
            return false;
        };
        let pending_id = pending[index].id;
        pending.remove(index);
        drop(pending);
        self.pending_privacy_roles.write().await.remove(&pending_id);
        true
    }

    pub async fn clear_pending_media(&self) {
        self.pending_media.write().await.clear();
        self.pending_privacy_roles.write().await.clear();
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

    async fn cached_artwork_bytes(&self, id: &str) -> Option<Vec<u8>> {
        self.media_artwork
            .read()
            .await
            .iter()
            .find(|item| item.id == id)
            .map(|item| item.bytes.to_vec())
    }

    pub async fn cache_audio(&self, id: String, content_type: String, bytes: Vec<u8>) {
        let mut cache = self.media_audio.write().await;
        cache.retain(|item| item.id != id);
        cache.push_front(MediaAudio {
            id,
            content_type,
            bytes: Bytes::from(bytes),
        });
        while cache.len() > MEDIA_AUDIO_CACHE_LIMIT
            || cache.iter().map(|item| item.bytes.len()).sum::<usize>()
                > MEDIA_AUDIO_CACHE_BYTE_LIMIT
        {
            cache.pop_back();
        }
    }

    pub async fn cache_tts_audio(&self, id: String, content_type: String, bytes: Vec<u8>) {
        let mut cache = self.tts_audio.write().await;
        cache.retain(|item| item.id != id);
        cache.push_front(TtsAudio {
            id,
            content_type,
            bytes: Bytes::from(bytes),
        });
        cache.truncate(TTS_CACHE_LIMIT);
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

    pub async fn cached_media_bytes(&self, id: &str) -> Option<Vec<u8>> {
        self.cached_media
            .read()
            .await
            .iter()
            .find(|item| item.id == id)
            .map(|item| item.bytes.to_vec())
    }

    pub async fn replay_media_event(&self, mut event: MediaEvent) -> Result<()> {
        let initial_config = self.config.read().await.clone();
        let analysis_text = privacy_media_text(&event, None);
        let mut scanned_bytes = None;
        if privacy::privacy_rules_enabled(&initial_config) {
            let mut report = if media_requires_image_scan(&event) {
                if let Some(cache_id) = event.cached_media_id.as_deref() {
                    if let Some(bytes) = self.cached_media_bytes(cache_id).await {
                        privacy::analyze_image_bytes_async(
                            &bytes,
                            Some(&analysis_text),
                            &initial_config,
                        )
                        .await
                    } else {
                        let bytes = download_replay_bytes(&event).await;
                        if let Some(bytes) = bytes {
                            let report = privacy::analyze_image_bytes_async(
                                &bytes,
                                Some(&analysis_text),
                                &initial_config,
                            )
                            .await;
                            scanned_bytes = Some(bytes);
                            report
                        } else {
                            privacy::analyze_remote_image(
                                &event.url,
                                &event.proxy_url,
                                Some(&analysis_text),
                                &initial_config,
                            )
                            .await
                        }
                    }
                } else {
                    let bytes = download_replay_bytes(&event).await;
                    if let Some(bytes) = bytes {
                        let report = privacy::analyze_image_bytes_async(
                            &bytes,
                            Some(&analysis_text),
                            &initial_config,
                        )
                        .await;
                        scanned_bytes = Some(bytes);
                        report
                    } else {
                        privacy::analyze_remote_image(
                            &event.url,
                            &event.proxy_url,
                            Some(&analysis_text),
                            &initial_config,
                        )
                        .await
                    }
                }
            } else {
                privacy::classify_text(Some(&analysis_text), &initial_config)
            };
            if initial_config.privacy_scan_enabled
                && let Some(artwork_id) = event.artwork_id.as_deref()
            {
                if let Some(bytes) = self.cached_artwork_bytes(artwork_id).await {
                    report.merge(
                        privacy::analyze_image_bytes_async(
                            &bytes,
                            Some(&analysis_text),
                            &initial_config,
                        )
                        .await,
                    );
                } else {
                    report.merge(PrivacyReport::low("scan_incomplete"));
                }
            }
            let current_config = self.config.read().await.clone();
            if !privacy::privacy_rules_enabled(&current_config) {
                // The user explicitly disabled scanning while the async scan
                // was in flight; preserve the documented bypass behavior.
            } else {
                let current_signature = privacy::config_signature(&current_config);
                if report.config_signature != Some(current_signature)
                    && report.classification == privacy::PrivacyClassification::Safe
                {
                    report.merge(PrivacyReport::suspicious("scan_config_changed"));
                }
                report.apply_score_policy(&current_config);
                let action = privacy::action_for(&report, &current_config);
                privacy::log_decision(&report, action);
                match action {
                    PrivacyAction::Allow => {}
                    PrivacyAction::Review => {
                        bail!("Media requires privacy review before replay.");
                    }
                    PrivacyAction::Block => {
                        bail!("Media blocked by the local privacy scan.");
                    }
                }
            }
        }
        if matches!(event.kind, MediaKind::Gif)
            && event.cached_media_id.is_none()
            && scanned_bytes.is_none()
        {
            let bytes = download_replay_bytes(&event).await;
            if let Some(bytes) = bytes {
                scanned_bytes = Some(bytes);
            }
        }
        if let Some(bytes) = scanned_bytes {
            if event.cached_media_id.is_none() {
                let cache_id = format!("{}-replay", event.message_id);
                self.cache_media(cache_id.clone(), event.content_type.clone(), bytes)
                    .await;
                event.cached_media_id = Some(cache_id);
            } else {
                drop(bytes);
            }
        }
        self.prepare_video_compatibility(&mut event).await;
        self.stage_scheduler
            .enqueue(RelayEvent::Media(event), StageLane::Media)
            .await;
        Ok(())
    }

    pub async fn publish_media(&self, media: MediaEvent) {
        let ticket = self
            .register_stage_output(media.timestamp, &media.message_id, 200, StageLane::Media)
            .await;
        self.publish_media_with_ticket(ticket, media).await;
    }

    pub async fn publish_media_with_ticket(&self, ticket: StageTicket, mut media: MediaEvent) {
        self.prepare_video_compatibility(&mut media).await;
        self.prepare_media_delivery(media.kind).await;
        {
            let mut history = self.history.write().await;
            history.push_front(media.clone());
            history.truncate(HISTORY_LIMIT);
        }
        self.complete_media(ticket, media).await;
    }

    async fn prepare_video_compatibility(&self, media: &mut MediaEvent) {
        if !matches!(media.kind, MediaKind::Video) {
            return;
        }
        let cached_media_available = if let Some(cache_id) = media.cached_media_id.as_deref() {
            self.cached_media
                .read()
                .await
                .iter()
                .any(|item| item.id == cache_id)
        } else {
            false
        };
        if cached_media_available {
            return;
        }
        media.cached_media_id = None;

        match media_compat::make_webview_compatible(
            &media.url,
            &media.proxy_url,
            &media.filename,
            &media.content_type,
        )
        .await
        {
            VideoCompatibility::Unchanged => {}
            VideoCompatibility::Transcoded(bytes) => {
                let cache_id = format!(
                    "h264-{:016x}{:016x}",
                    rand::random::<u64>(),
                    rand::random::<u64>()
                );
                self.cache_media(cache_id.clone(), "video/mp4".into(), bytes)
                    .await;
                media.content_type = "video/mp4".into();
                media.cached_media_id = Some(cache_id);
                eprintln!("[media] Codec: HEVC Action: TRANSCODED_LOCAL");
            }
            VideoCompatibility::HevcFallback => {
                eprintln!("[media] Codec: HEVC Action: SOURCE_FALLBACK");
            }
        }
    }

    async fn prepare_media_delivery(&self, kind: MediaKind) {
        let Some(sender) = self.media_delivery.read().await.clone() else {
            return;
        };
        let (ready, receiver) = oneshot::channel();
        if sender.send(MediaDeliveryRequest { kind, ready }).is_ok() {
            let _ = tokio::time::timeout(MEDIA_DELIVERY_TIMEOUT, receiver).await;
        }
    }

    pub fn tts_pending_count(&self) -> usize {
        self.tts_pending_count.load(Ordering::SeqCst)
    }

    async fn publish_sticker(&self, sticker: StickerEvent) {
        let ticket = self
            .register_stage_output(
                sticker.timestamp,
                &sticker.message_id,
                100,
                StageLane::Media,
            )
            .await;
        self.publish_sticker_with_ticket(ticket, sticker).await;
    }

    async fn publish_sticker_with_ticket(&self, ticket: StageTicket, sticker: StickerEvent) {
        self.complete_sticker(ticket, sticker).await;
    }

    #[allow(dead_code)]
    pub async fn publish_tts(&self, request: TtsRequest) -> Result<()> {
        self.publish_tts_with_roles(request, &[]).await
    }

    pub async fn publish_tts_with_roles(
        &self,
        request: TtsRequest,
        role_ids: &[String],
    ) -> Result<()> {
        let ticket = self
            .register_stage_output(request.timestamp, &request.id, 0, StageLane::Tts)
            .await;
        let result = self
            .publish_tts_with_ticket_and_roles(ticket, request, role_ids)
            .await;
        if result.is_err() {
            self.cancel_stage_output(ticket).await;
        }
        result
    }

    pub async fn publish_tts_with_ticket_and_roles(
        &self,
        ticket: StageTicket,
        request: TtsRequest,
        role_ids: &[String],
    ) -> Result<()> {
        struct PendingGuard<'a>(&'a AtomicUsize);
        impl Drop for PendingGuard<'_> {
            fn drop(&mut self) {
                self.0.fetch_sub(1, Ordering::SeqCst);
            }
        }
        let limit = self.config.read().await.tts_queue_limit as usize;
        let waiting = self.tts_pending_count.fetch_add(1, Ordering::SeqCst);
        let _pending = PendingGuard(&self.tts_pending_count);
        if waiting >= limit {
            anyhow::bail!("the TTS queue is full");
        }
        let _synthesis = self.tts_synthesis_lock.lock().await;
        let config = self.config.read().await;
        let scoped_config = privacy::scoped_config_for_roles(&config, role_ids);
        if privacy::privacy_rules_enabled(&scoped_config) {
            let report = privacy::classify_text(Some(&request.text), &scoped_config);
            let action = privacy::action_for(&report, &scoped_config);
            if !matches!(action, PrivacyAction::Allow) {
                privacy::log_decision(&report, action);
                bail!("TTS privacy policy denied publication.");
            }
        }
        let speech =
            tokio::time::timeout(TTS_SYNTHESIS_TIMEOUT, tts::synthesize(request.text.clone()))
                .await
                .map_err(|_| anyhow::anyhow!("Windows TTS timed out; the queue was released"))??;
        let event = TtsEvent {
            id: request.id.clone(),
            text: request.text,
            author: request.author,
            guild_tag: request.guild_tag,
            content_type: speech.content_type.clone(),
            timestamp: request.timestamp,
            visual_only: false,
            segments: Vec::new(),
        };
        self.cache_tts_audio(request.id, speech.content_type, speech.bytes)
            .await;
        self.complete_tts(ticket, event).await;
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn publish_visual_tts(
        &self,
        id: String,
        text: String,
        author: crate::model::AuthorIdentity,
        guild_tag: Option<crate::model::GuildTagIdentity>,
        timestamp: u64,
        segments: Vec<VisualSegment>,
    ) {
        let ticket = self
            .register_stage_output(timestamp, &id, 0, StageLane::Tts)
            .await;
        self.publish_visual_tts_with_ticket(
            ticket, id, text, author, guild_tag, timestamp, segments,
        )
        .await;
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn publish_visual_tts_with_ticket(
        &self,
        ticket: StageTicket,
        id: String,
        text: String,
        author: crate::model::AuthorIdentity,
        guild_tag: Option<crate::model::GuildTagIdentity>,
        timestamp: u64,
        segments: Vec<VisualSegment>,
    ) {
        let event = TtsEvent {
            id,
            text,
            author,
            guild_tag,
            content_type: String::new(),
            timestamp,
            visual_only: true,
            segments,
        };
        self.complete_tts(ticket, event).await;
    }

    #[allow(dead_code)]
    pub async fn publish_visual_tts_if_allowed(
        &self,
        id: String,
        text: String,
        author: crate::model::AuthorIdentity,
        guild_tag: Option<crate::model::GuildTagIdentity>,
        timestamp: u64,
        segments: Vec<VisualSegment>,
    ) -> bool {
        self.publish_visual_tts_if_allowed_with_roles(
            id,
            text,
            author,
            guild_tag,
            timestamp,
            segments,
            &[],
        )
        .await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn publish_visual_tts_if_allowed_with_roles(
        &self,
        id: String,
        text: String,
        author: crate::model::AuthorIdentity,
        guild_tag: Option<crate::model::GuildTagIdentity>,
        timestamp: u64,
        segments: Vec<VisualSegment>,
        role_ids: &[String],
    ) -> bool {
        let ticket = self
            .register_stage_output(timestamp, &id, 0, StageLane::Tts)
            .await;
        self.publish_visual_tts_if_allowed_with_ticket_and_roles(
            ticket, id, text, author, guild_tag, timestamp, segments, role_ids,
        )
        .await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn publish_visual_tts_if_allowed_with_ticket_and_roles(
        &self,
        ticket: StageTicket,
        id: String,
        text: String,
        author: crate::model::AuthorIdentity,
        guild_tag: Option<crate::model::GuildTagIdentity>,
        timestamp: u64,
        segments: Vec<VisualSegment>,
        role_ids: &[String],
    ) -> bool {
        let config = self.config.read().await;
        let scoped_config = privacy::scoped_config_for_roles(&config, role_ids);
        if privacy::privacy_rules_enabled(&scoped_config) {
            let report = classify_tts_privacy(&text, &segments, &scoped_config);
            let action = privacy::action_for(&report, &scoped_config);
            if !matches!(action, PrivacyAction::Allow) {
                privacy::log_decision(&report, action);
                drop(config);
                self.cancel_stage_output(ticket).await;
                return false;
            }
        }
        drop(config);
        self.publish_visual_tts_with_ticket(
            ticket, id, text, author, guild_tag, timestamp, segments,
        )
        .await;
        true
    }
}

const MAX_TTS_PRIVACY_SEGMENTS: usize = 64;
const MAX_TTS_PRIVACY_COMPOSITE_CHARS: usize = privacy::PRIVACY_TEXT_LIMIT + 1;

fn classify_tts_privacy(
    text: &str,
    segments: &[VisualSegment],
    config: &AppConfig,
) -> privacy::PrivacyReport {
    let mut composite = String::new();
    let mut truncated = false;
    append_tts_privacy_field(&mut composite, text, &mut truncated);
    for segment in segments.iter().take(MAX_TTS_PRIVACY_SEGMENTS) {
        append_tts_privacy_field(&mut composite, &segment.value, &mut truncated);
    }
    if segments.len() > MAX_TTS_PRIVACY_SEGMENTS {
        truncated = true;
    }
    let mut report = privacy::classify_text(Some(&composite), config);
    if truncated {
        report.merge(privacy::PrivacyReport::suspicious("scan_incomplete"));
    }
    report
}

fn append_tts_privacy_field(target: &mut String, value: &str, truncated: &mut bool) {
    if value.is_empty() {
        return;
    }
    if !target.is_empty() {
        append_tts_privacy_bounded(target, "\n", truncated);
    }
    append_tts_privacy_bounded(target, value, truncated);
}

fn append_tts_privacy_bounded(target: &mut String, value: &str, truncated: &mut bool) {
    let current = target.chars().count();
    if current >= MAX_TTS_PRIVACY_COMPOSITE_CHARS {
        *truncated = true;
        return;
    }
    let remaining = MAX_TTS_PRIVACY_COMPOSITE_CHARS - current;
    let mut appended = 0;
    for character in value.chars().take(remaining) {
        target.push(character);
        appended += 1;
    }
    if appended < value.chars().count() {
        *truncated = true;
    }
}

fn privacy_media_text(media: &MediaEvent, full_text: Option<&str>) -> String {
    let mut fields = Vec::with_capacity(7);
    if let Some(text) = full_text.filter(|text| !text.trim().is_empty()) {
        fields.push(text.to_owned());
    }
    if let Some(text) = media.text.as_deref().filter(|text| !text.trim().is_empty()) {
        fields.push(text.to_owned());
    }
    if !media.filename.trim().is_empty() {
        fields.push(media.filename.clone());
        if let Some(stem) = media.filename.rsplit_once('.').map(|(stem, _)| stem)
            && !stem.trim().is_empty()
        {
            fields.push(stem.to_owned());
        }
    }
    if let Some(title) = media
        .title
        .as_deref()
        .filter(|text| !text.trim().is_empty())
    {
        fields.push(title.to_owned());
    }
    if let Some(artist) = media
        .artist
        .as_deref()
        .filter(|text| !text.trim().is_empty())
    {
        fields.push(artist.to_owned());
    }
    fields.join("\n")
}

async fn download_replay_bytes(event: &MediaEvent) -> Option<Vec<u8>> {
    match artwork::download_bounded(&event.url, artwork::MAX_EMBED_MEDIA_BYTES).await {
        Ok(bytes) => Some(bytes),
        Err(_) if event.proxy_url != event.url => {
            artwork::download_bounded(&event.proxy_url, artwork::MAX_EMBED_MEDIA_BYTES)
                .await
                .ok()
        }
        Err(_) => None,
    }
}

fn media_type_allowed(config: &AppConfig, kind: MediaKind) -> bool {
    match kind {
        MediaKind::Image | MediaKind::Gif => config.moderation_allow_images,
        MediaKind::Video => config.moderation_allow_videos,
        MediaKind::Audio => config.moderation_allow_audio,
    }
}

fn media_requires_image_scan(media: &MediaEvent) -> bool {
    matches!(media.kind, MediaKind::Image | MediaKind::Gif)
        && media
            .content_type
            .to_ascii_lowercase()
            .starts_with("image/")
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

    #[tokio::test]
    async fn app_core_restores_persisted_interface_preferences() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("config.json");
        let config = AppConfig {
            interface_preferences: InterfacePreferences {
                language: "ko".into(),
                theme: "light".into(),
                accent_rgb: [42, 84, 126],
                font_scale: 120,
            },
            ..AppConfig::default()
        };
        ConfigStore::new(path.clone()).save(&config).unwrap();

        let core = AppCore::load(path).unwrap();

        assert_eq!(
            *core.interface_preferences.read().await,
            config.interface_preferences
        );
    }

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
            text: None,
            author: crate::model::AuthorIdentity {
                username: "Moderator".into(),
                display_avatar_url: "https://cdn.discordapp.com/avatar.png".into(),
            },
            timestamp: 42,
            message_id: message_id.into(),
        }
    }

    fn sticker(message_id: &str) -> StickerEvent {
        StickerEvent {
            id: format!("sticker-{message_id}"),
            name: "Test sticker".into(),
            format: "png".into(),
            url: "https://cdn.discordapp.com/stickers/test.png".into(),
            cached_media_id: None,
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
        let (delivery, mut requests) = mpsc::unbounded_channel();
        core.set_media_delivery(delivery).await;
        let ready = tokio::spawn(async move {
            let request = requests.recv().await.unwrap();
            assert!(matches!(request.kind, MediaKind::Image));
            request.ready.send(()).unwrap();
        });
        assert!(core.approve_media(id).await);
        ready.await.unwrap();
        assert!(matches!(events.recv().await.unwrap(), RelayEvent::Media(_)));
        assert_eq!(core.history.read().await.len(), 1);
        assert!(!core.reject_media(id).await);
    }

    #[tokio::test]
    async fn sensitive_media_never_reaches_history_or_relay() {
        let directory = tempfile::tempdir().unwrap();
        let core = AppCore::load(directory.path().join("config.json")).unwrap();
        core.set_config(AppConfig {
            privacy_scan_enabled: true,
            ..AppConfig::default()
        })
        .await
        .unwrap();
        let mut events = core.relay_tx.subscribe();
        core.submit_analyzed_media(
            media(MediaKind::Image, "sensitive"),
            Some(PrivacyReport::sensitive("gps")),
        )
        .await;
        assert!(core.history.read().await.is_empty());
        assert!(core.pending_media.read().await.is_empty());
        assert!(events.try_recv().is_err());
    }

    #[tokio::test]
    async fn medium_privacy_risk_uses_existing_queue_before_publication() {
        let directory = tempfile::tempdir().unwrap();
        let core = AppCore::load(directory.path().join("config.json")).unwrap();
        core.set_config(AppConfig {
            privacy_scan_enabled: true,
            privacy_review_intermediate: true,
            moderation_enabled: false,
            ..AppConfig::default()
        })
        .await
        .unwrap();
        let mut events = core.relay_tx.subscribe();

        core.submit_analyzed_media_with_text(
            media(MediaKind::Image, "medium-phone"),
            None,
            Some("Call 06 12 34 56 78"),
        )
        .await;

        let pending = core.pending_media.read().await;
        assert_eq!(pending.len(), 1);
        assert_eq!(
            pending[0].privacy_classification,
            Some(privacy::PrivacyClassification::Medium)
        );
        assert!(
            pending[0]
                .privacy_categories
                .contains(&privacy::PrivacyCategory::Phone)
        );
        assert!(core.history.read().await.is_empty());
        assert!(events.try_recv().is_err());
    }

    #[tokio::test]
    async fn custom_private_value_is_blocked_before_history_and_relay() {
        let directory = tempfile::tempdir().unwrap();
        let core = AppCore::load(directory.path().join("config.json")).unwrap();
        core.set_config(AppConfig {
            privacy_scan_enabled: true,
            privacy_custom_patterns: vec!["private-room-42".into()],
            moderation_enabled: false,
            ..AppConfig::default()
        })
        .await
        .unwrap();
        let mut events = core.relay_tx.subscribe();

        core.submit_analyzed_media_with_text(
            media(MediaKind::Image, "custom-pattern"),
            None,
            Some("private room 42"),
        )
        .await;

        assert!(core.pending_media.read().await.is_empty());
        assert!(core.history.read().await.is_empty());
        assert!(events.try_recv().is_err());
    }

    #[tokio::test]
    async fn filter_words_block_without_scan_or_manual_moderation() {
        let directory = tempfile::tempdir().unwrap();
        let core = AppCore::load(directory.path().join("config.json")).unwrap();
        core.set_config(AppConfig {
            moderation_enabled: false,
            privacy_scan_enabled: false,
            privacy_concepts: vec![privacy::ForbiddenConcept {
                canonical: "hitler".into(),
                aliases: Vec::new(),
                regexes: vec![r"\bsecret\b".into()],
            }],
            ..AppConfig::default()
        })
        .await
        .unwrap();
        let mut events = core.relay_tx.subscribe();

        core.submit_analyzed_media_with_text(
            media(MediaKind::Image, "filter-exact"),
            Some(PrivacyReport::safe()),
            Some("hitler"),
        )
        .await;
        core.submit_analyzed_media_with_text(
            media(MediaKind::Image, "filter-regex"),
            Some(PrivacyReport::safe()),
            Some("a SECRET message"),
        )
        .await;
        assert!(core.history.read().await.is_empty());
        assert!(core.pending_media.read().await.is_empty());
        assert!(events.try_recv().is_err());

        core.submit_analyzed_media_with_text(
            media(MediaKind::Image, "filter-safe"),
            None,
            Some("public monument"),
        )
        .await;
        assert_eq!(core.history.read().await.len(), 1);
        assert!(matches!(events.recv().await.unwrap(), RelayEvent::Media(_)));
    }

    #[tokio::test]
    async fn exempt_role_publishes_filter_word_media_and_tts_but_not_gps() {
        let directory = tempfile::tempdir().unwrap();
        let core = AppCore::load(directory.path().join("config.json")).unwrap();
        let role_id = "123456789012345678".to_owned();
        core.set_config(AppConfig {
            moderation_enabled: false,
            privacy_scan_enabled: false,
            privacy_concepts: vec![privacy::ForbiddenConcept {
                canonical: "hitler".into(),
                aliases: Vec::new(),
                regexes: vec![r"\bsecret\b".into()],
            }],
            privacy_filter_exempt_role_ids: vec![role_id.clone()],
            ..AppConfig::default()
        })
        .await
        .unwrap();
        let mut events = core.relay_tx.subscribe();

        core.submit_analyzed_media_with_text_and_roles(
            media(MediaKind::Image, "exempt-media"),
            Some(PrivacyReport::sensitive("forbidden_concept")),
            Some("hitler"),
            std::slice::from_ref(&role_id),
        )
        .await;
        assert_eq!(core.history.read().await.len(), 1);
        assert!(matches!(events.recv().await.unwrap(), RelayEvent::Media(_)));

        core.submit_analyzed_media_with_text_and_roles(
            media(MediaKind::Image, "exempt-regex"),
            Some(PrivacyReport::sensitive("forbidden_regex")),
            Some("secret"),
            std::slice::from_ref(&role_id),
        )
        .await;
        assert_eq!(core.history.read().await.len(), 2);
        assert!(matches!(events.recv().await.unwrap(), RelayEvent::Media(_)));

        assert!(
            core.publish_visual_tts_if_allowed_with_roles(
                "exempt-tts".into(),
                "hitler".into(),
                crate::model::AuthorIdentity {
                    username: "Moderator".into(),
                    display_avatar_url: String::new(),
                },
                None,
                42,
                vec![VisualSegment {
                    kind: "text".into(),
                    value: "hitler".into(),
                    url: None,
                    animated: false,
                }],
                std::slice::from_ref(&role_id),
            )
            .await
        );
        assert!(matches!(events.recv().await.unwrap(), RelayEvent::Tts(_)));

        core.update_config(|config| config.privacy_scan_enabled = true)
            .await
            .unwrap();
        assert!(matches!(
            events.recv().await.unwrap(),
            RelayEvent::Config(_)
        ));
        core.submit_analyzed_media_with_text_and_roles(
            media(MediaKind::Image, "exempt-gps"),
            Some(PrivacyReport::sensitive("gps")),
            None,
            std::slice::from_ref(&role_id),
        )
        .await;
        assert_eq!(core.history.read().await.len(), 2);
        assert!(events.try_recv().is_err());
    }

    #[tokio::test]
    async fn pending_role_scope_is_kept_per_moderation_item() {
        let directory = tempfile::tempdir().unwrap();
        let core = AppCore::load(directory.path().join("config.json")).unwrap();
        let role_id = "123456789012345678".to_owned();
        core.set_config(AppConfig {
            moderation_enabled: true,
            privacy_scan_enabled: false,
            privacy_concepts: vec![privacy::ForbiddenConcept {
                canonical: "hitler".into(),
                aliases: Vec::new(),
                regexes: Vec::new(),
            }],
            privacy_filter_exempt_role_ids: vec![role_id.clone()],
            ..AppConfig::default()
        })
        .await
        .unwrap();
        let roles = std::slice::from_ref(&role_id);
        core.submit_analyzed_media_with_text_and_roles(
            media(MediaKind::Image, "shared-message"),
            None,
            Some("hitler"),
            roles,
        )
        .await;
        core.submit_analyzed_media_with_text_and_roles(
            media(MediaKind::Image, "shared-message"),
            None,
            Some("hitler"),
            roles,
        )
        .await;
        let ids = core
            .pending_media
            .read()
            .await
            .iter()
            .map(|item| item.id)
            .collect::<Vec<_>>();
        assert_eq!(ids.len(), 2);
        assert!(core.approve_media(ids[0]).await);
        assert!(core.approve_media(ids[1]).await);
        assert_eq!(core.history.read().await.len(), 2);
    }

    #[tokio::test]
    async fn state_gate_rechecks_text_before_publication() {
        let directory = tempfile::tempdir().unwrap();
        let core = AppCore::load(directory.path().join("config.json")).unwrap();
        core.set_config(AppConfig {
            privacy_scan_enabled: true,
            privacy_concepts: vec![privacy::ForbiddenConcept {
                canonical: "hitler".into(),
                aliases: Vec::new(),
                regexes: Vec::new(),
            }],
            ..AppConfig::default()
        })
        .await
        .unwrap();
        let mut media = media(MediaKind::Image, "concept");
        media.text = Some("h1tl3r".into());
        let mut events = core.relay_tx.subscribe();
        core.submit_analyzed_media(media, Some(PrivacyReport::safe()))
            .await;
        assert!(core.history.read().await.is_empty());
        assert!(core.pending_media.read().await.is_empty());
        assert!(events.try_recv().is_err());
    }

    #[tokio::test]
    async fn state_gate_uses_full_discord_text_beyond_caption_limit() {
        let directory = tempfile::tempdir().unwrap();
        let core = AppCore::load(directory.path().join("config.json")).unwrap();
        core.set_config(AppConfig {
            privacy_scan_enabled: true,
            privacy_concepts: vec![privacy::ForbiddenConcept {
                canonical: "hitler".into(),
                aliases: Vec::new(),
                regexes: Vec::new(),
            }],
            ..AppConfig::default()
        })
        .await
        .unwrap();
        let mut event = media(MediaKind::Image, "concept-after-caption");
        event.text = Some("a".repeat(180));
        let full_text = format!("{} h1tl3r", "a".repeat(180));
        let mut events = core.relay_tx.subscribe();
        core.submit_analyzed_media_with_text(event, Some(PrivacyReport::safe()), Some(&full_text))
            .await;
        assert!(core.history.read().await.is_empty());
        assert!(core.pending_media.read().await.is_empty());
        assert!(events.try_recv().is_err());
    }

    #[tokio::test]
    async fn state_gate_scans_untrusted_media_fields_for_concepts() {
        let directory = tempfile::tempdir().unwrap();
        let core = AppCore::load(directory.path().join("config.json")).unwrap();
        core.set_config(AppConfig {
            privacy_scan_enabled: true,
            privacy_concepts: vec![privacy::ForbiddenConcept {
                canonical: "hitler".into(),
                aliases: Vec::new(),
                regexes: Vec::new(),
            }],
            ..AppConfig::default()
        })
        .await
        .unwrap();
        let mut filename_media = media(MediaKind::Image, "filename-concept");
        filename_media.filename = "h1tl3r.png".into();
        let mut title_media = media(MediaKind::Image, "title-concept");
        title_media.title = Some("h1tl3r".into());
        core.submit_analyzed_media(filename_media, Some(PrivacyReport::safe()))
            .await;
        core.submit_analyzed_media(title_media, Some(PrivacyReport::safe()))
            .await;
        core.submit_sticker(
            StickerEvent {
                name: "h1tl3r".into(),
                ..sticker("sticker-concept")
            },
            None,
            None,
            Some(PrivacyReport::safe()),
        )
        .await;
        assert!(core.history.read().await.is_empty());
        assert!(core.pending_media.read().await.is_empty());
    }

    #[tokio::test]
    async fn approval_rechecks_sensitive_pending_entries() {
        let directory = tempfile::tempdir().unwrap();
        let core = AppCore::load(directory.path().join("config.json")).unwrap();
        core.set_config(AppConfig {
            privacy_scan_enabled: true,
            ..AppConfig::default()
        })
        .await
        .unwrap();
        let mut pending_media = media(MediaKind::Image, "pending-sensitive");
        pending_media.content_type = "image/png".into();
        core.pending_media.write().await.push_back(PendingMedia {
            id: 99,
            media: pending_media,
            sticker: None,
            sticker_bytes: Some(Arc::new(b"not an image".to_vec())),
            privacy_classification: Some(privacy::PrivacyClassification::Sensitive),
            privacy_categories: vec![privacy::PrivacyCategory::GpsLocation],
            privacy_reason: Some("gps".into()),
        });
        let mut events = core.relay_tx.subscribe();
        assert!(!core.approve_media(99).await);
        assert!(core.history.read().await.is_empty());
        assert!(events.try_recv().is_err());
    }

    #[tokio::test]
    async fn approval_rechecks_cached_audio_artwork_before_publication() {
        let directory = tempfile::tempdir().unwrap();
        let core = AppCore::load(directory.path().join("config.json")).unwrap();
        core.set_config(AppConfig {
            privacy_scan_enabled: true,
            ..AppConfig::default()
        })
        .await
        .unwrap();
        core.cache_artwork(
            "audio-artwork".into(),
            EmbeddedArtwork {
                content_type: "image/png".into(),
                bytes: b"not an image".to_vec(),
            },
        )
        .await;
        let mut event = media(MediaKind::Audio, "pending-audio-artwork");
        event.artwork_id = Some("audio-artwork".into());
        core.pending_media.write().await.push_back(PendingMedia {
            id: 100,
            media: event,
            sticker: None,
            sticker_bytes: None,
            privacy_classification: Some(privacy::PrivacyClassification::Medium),
            privacy_categories: vec![privacy::PrivacyCategory::Ocr],
            privacy_reason: Some("ocr_text".into()),
        });
        let mut events = core.relay_tx.subscribe();

        assert!(!core.approve_media(100).await);
        assert!(core.history.read().await.is_empty());
        assert!(events.try_recv().is_err());
    }

    #[tokio::test]
    async fn suspicious_media_uses_review_queue_even_when_manual_moderation_is_off() {
        let directory = tempfile::tempdir().unwrap();
        let core = AppCore::load(directory.path().join("config.json")).unwrap();
        core.set_config(AppConfig {
            privacy_scan_enabled: true,
            moderation_enabled: false,
            ..AppConfig::default()
        })
        .await
        .unwrap();
        let mut events = core.relay_tx.subscribe();
        core.submit_analyzed_media(
            media(MediaKind::Image, "suspicious"),
            Some(PrivacyReport::suspicious("privacy_signal")),
        )
        .await;
        assert_eq!(core.pending_media.read().await.len(), 1);
        assert!(core.history.read().await.is_empty());
        assert!(events.try_recv().is_err());
    }

    #[tokio::test]
    async fn sensitive_sticker_never_reaches_cache_or_relay() {
        let directory = tempfile::tempdir().unwrap();
        let core = AppCore::load(directory.path().join("config.json")).unwrap();
        core.set_config(AppConfig {
            privacy_scan_enabled: true,
            ..AppConfig::default()
        })
        .await
        .unwrap();
        let mut events = core.relay_tx.subscribe();
        core.submit_sticker(
            sticker("sensitive"),
            Some("h1tl3r"),
            Some(vec![1, 2, 3]),
            Some(PrivacyReport::sensitive("forbidden_concept")),
        )
        .await;
        assert!(core.pending_media.read().await.is_empty());
        assert!(core.cached_media.read().await.is_empty());
        assert!(events.try_recv().is_err());
    }

    #[tokio::test]
    async fn sticker_scan_review_uses_the_existing_queue() {
        let directory = tempfile::tempdir().unwrap();
        let core = AppCore::load(directory.path().join("config.json")).unwrap();
        core.set_config(AppConfig {
            privacy_scan_enabled: true,
            moderation_enabled: false,
            ..AppConfig::default()
        })
        .await
        .unwrap();
        let mut events = core.relay_tx.subscribe();
        core.submit_sticker(
            sticker("review"),
            Some("safe caption"),
            Some(vec![1, 2, 3]),
            Some(PrivacyReport::suspicious("scan_incomplete")),
        )
        .await;
        let pending = core.pending_media.read().await;
        assert_eq!(pending.len(), 1);
        assert!(pending[0].sticker.is_some());
        assert!(pending[0].sticker_bytes.is_some());
        assert!(core.cached_media.read().await.is_empty());
        assert!(events.try_recv().is_err());
    }

    #[tokio::test]
    async fn privacy_review_pending_survives_manual_moderation_disable() {
        let directory = tempfile::tempdir().unwrap();
        let core = AppCore::load(directory.path().join("config.json")).unwrap();
        core.set_config(AppConfig {
            privacy_scan_enabled: true,
            moderation_enabled: true,
            ..AppConfig::default()
        })
        .await
        .unwrap();
        core.submit_analyzed_media(
            media(MediaKind::Image, "privacy-review"),
            Some(PrivacyReport::suspicious("scan_incomplete")),
        )
        .await;
        assert_eq!(core.pending_media.read().await.len(), 1);
        core.update_config(|config| config.moderation_enabled = false)
            .await
            .unwrap();
        assert_eq!(core.pending_media.read().await.len(), 1);
    }

    #[tokio::test]
    async fn replay_blocks_sensitive_media_when_filter_words_are_enabled() {
        let directory = tempfile::tempdir().unwrap();
        let core = AppCore::load(directory.path().join("config.json")).unwrap();
        core.set_config(AppConfig {
            privacy_scan_enabled: false,
            privacy_concepts: vec![privacy::ForbiddenConcept {
                canonical: "hitler".into(),
                aliases: Vec::new(),
                regexes: Vec::new(),
            }],
            ..AppConfig::default()
        })
        .await
        .unwrap();
        let mut event = media(MediaKind::Video, "replay-sensitive");
        event.text = Some("h1tl3r".into());
        let mut events = core.relay_tx.subscribe();
        assert!(core.replay_media_event(event).await.is_err());
        assert!(events.try_recv().is_err());
    }

    #[tokio::test]
    async fn replay_keeps_the_privacy_off_bypass() {
        let directory = tempfile::tempdir().unwrap();
        let core = AppCore::load(directory.path().join("config.json")).unwrap();
        let mut event = media(MediaKind::Video, "replay-disabled");
        event.text = Some("h1tl3r".into());
        let mut events = core.relay_tx.subscribe();
        core.replay_media_event(event).await.unwrap();
        assert!(matches!(events.recv().await.unwrap(), RelayEvent::Media(_)));
    }

    #[tokio::test]
    async fn stale_safe_report_enters_the_current_privacy_policy() {
        let directory = tempfile::tempdir().unwrap();
        let core = AppCore::load(directory.path().join("config.json")).unwrap();
        core.set_config(AppConfig {
            privacy_scan_enabled: true,
            ..AppConfig::default()
        })
        .await
        .unwrap();
        let old_config = core.config.read().await.clone();
        let report = privacy::classify_text(Some("landscape"), &old_config);
        core.update_config(|config| config.privacy_similarity_boost = 3)
            .await
            .unwrap();
        core.submit_analyzed_media(media(MediaKind::Image, "stale-safe"), Some(report))
            .await;
        let pending = core.pending_media.read().await;
        assert_eq!(pending.len(), 1);
        assert_eq!(
            pending[0].privacy_reason.as_deref(),
            Some("scan_config_changed")
        );
    }

    #[tokio::test]
    async fn disabled_privacy_scan_preserves_existing_immediate_flow() {
        let directory = tempfile::tempdir().unwrap();
        let core = AppCore::load(directory.path().join("config.json")).unwrap();
        core.set_config(AppConfig {
            privacy_scan_enabled: false,
            privacy_concepts: Vec::new(),
            ..AppConfig::default()
        })
        .await
        .unwrap();
        let mut events = core.relay_tx.subscribe();
        core.submit_analyzed_media(
            media(MediaKind::Image, "disabled"),
            Some(PrivacyReport::sensitive("forbidden_concept")),
        )
        .await;
        assert_eq!(core.history.read().await.len(), 1);
        assert!(matches!(events.recv().await.unwrap(), RelayEvent::Media(_)));
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
            guild_tag: Some(crate::model::GuildTagIdentity {
                name: "RE".into(),
                badge_url: Some("https://cdn.discordapp.com/tag.png".into()),
            }),
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
        assert_eq!(event.guild_tag.unwrap().name, "RE");
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
            Some(crate::model::GuildTagIdentity {
                name: "RE".into(),
                badge_url: None,
            }),
            42,
            vec![VisualSegment {
                kind: "text".into(),
                value: "test".into(),
                url: None,
                animated: false,
            }],
        )
        .await;

        let RelayEvent::Tts(event) = events.recv().await.unwrap() else {
            panic!("expected a TTS relay event");
        };
        assert!(event.visual_only);
        assert_eq!(event.text, "test");
        assert_eq!(event.guild_tag.unwrap().name, "RE");
        assert_eq!(event.segments.len(), 1);
        assert!(core.tts_audio.read().await.is_empty());
    }

    #[tokio::test]
    async fn tts_privacy_gate_blocks_filter_concepts_and_holds_medium_risk() {
        let directory = tempfile::tempdir().unwrap();
        let core = AppCore::load(directory.path().join("config.json")).unwrap();
        core.set_config(AppConfig {
            privacy_scan_enabled: false,
            privacy_concepts: vec![privacy::ForbiddenConcept {
                canonical: "hitler".into(),
                aliases: Vec::new(),
                regexes: Vec::new(),
            }],
            ..AppConfig::default()
        })
        .await
        .unwrap();
        let mut events = core.relay_tx.subscribe();
        let author = crate::model::AuthorIdentity {
            username: "TTS tester".into(),
            display_avatar_url: "https://cdn.discordapp.com/avatar.png".into(),
        };
        let blocked_concept = core
            .publish_visual_tts_if_allowed(
                "tts-concept".into(),
                "sticker h1tl3r".into(),
                author.clone(),
                None,
                42,
                Vec::new(),
            )
            .await;
        assert!(!blocked_concept);
        let blocked_segment_concept = core
            .publish_visual_tts_if_allowed(
                "tts-segment-concept".into(),
                "safe text".into(),
                author.clone(),
                None,
                42,
                vec![VisualSegment {
                    kind: "text".into(),
                    value: "h1tl3r".into(),
                    url: None,
                    animated: false,
                }],
            )
            .await;
        assert!(!blocked_segment_concept);
        assert!(events.try_recv().is_err());
        let blocked_cross_field_concept = core
            .publish_visual_tts_if_allowed(
                "tts-cross-field-concept".into(),
                "hi".into(),
                author.clone(),
                None,
                42,
                vec![VisualSegment {
                    kind: "sticker".into(),
                    value: "tler".into(),
                    url: None,
                    animated: false,
                }],
            )
            .await;
        assert!(!blocked_cross_field_concept);
        assert!(events.try_recv().is_err());
        let blocked_cross_segment_concept = core
            .publish_visual_tts_if_allowed(
                "tts-cross-segment-concept".into(),
                "safe".into(),
                author.clone(),
                None,
                42,
                vec![
                    VisualSegment {
                        kind: "text".into(),
                        value: "hi".into(),
                        url: None,
                        animated: false,
                    },
                    VisualSegment {
                        kind: "text".into(),
                        value: "tler".into(),
                        url: None,
                        animated: false,
                    },
                ],
            )
            .await;
        assert!(!blocked_cross_segment_concept);
        assert!(events.try_recv().is_err());
        let unrelated_split = core
            .publish_visual_tts_if_allowed(
                "tts-unrelated-split".into(),
                "safe".into(),
                author.clone(),
                None,
                42,
                vec![VisualSegment {
                    kind: "text".into(),
                    value: "hello world".into(),
                    url: None,
                    animated: false,
                }],
            )
            .await;
        assert!(unrelated_split);
        assert!(matches!(events.try_recv(), Ok(RelayEvent::Tts(_))));
        core.update_config(|config| {
            config.privacy_scan_enabled = true;
            config.privacy_review_intermediate = true;
        })
        .await
        .unwrap();
        let held_for_review = core
            .publish_visual_tts_if_allowed(
                "tts-medium".into(),
                "call 06 12 34 56 78".into(),
                author.clone(),
                None,
                42,
                Vec::new(),
            )
            .await;
        assert!(!held_for_review);

        core.update_config(|config| config.privacy_review_intermediate = false)
            .await
            .unwrap();
        let allowed_medium = core
            .publish_visual_tts_if_allowed(
                "tts-medium-allowed".into(),
                "call 06 12 34 56 78".into(),
                author,
                None,
                42,
                Vec::new(),
            )
            .await;
        assert!(allowed_medium);
    }

    #[tokio::test]
    async fn claims_delayed_embeds_only_once() {
        let directory = tempfile::tempdir().unwrap();
        let core = AppCore::load(directory.path().join("config.json")).unwrap();
        assert!(core.claim_embed("message-embed-0".into()).await);
        assert!(!core.claim_embed("message-embed-0".into()).await);
    }

    #[tokio::test]
    async fn explicit_stage_tickets_keep_delayed_text_ahead_of_newer_ready_media() {
        let directory = tempfile::tempdir().unwrap();
        let core = AppCore::load(directory.path().join("config.json")).unwrap();
        let mut events = core.relay_tx.subscribe();

        let video_ticket = core
            .register_stage_output(21_590, "100", 200, StageLane::Media)
            .await;
        let mut video = media(MediaKind::Video, "video");
        video.timestamp = 21_590;
        core.complete_media(video_ticket, video).await;
        assert!(
            matches!(events.recv().await.unwrap(), RelayEvent::Media(event) if event.message_id == "video")
        );
        core.stage_scheduler.stage_state(true, false, false).await;

        let text_ticket = core
            .register_stage_output(22_000, "101", 0, StageLane::Tts)
            .await;
        let image_ticket = core
            .register_stage_output(22_010, "102", 200, StageLane::Media)
            .await;
        let mut image = media(MediaKind::Image, "image");
        image.timestamp = 22_010;
        core.complete_media(image_ticket, image).await;
        core.complete_tts(
            text_ticket,
            TtsEvent {
                id: "text".into(),
                text: "older text".into(),
                author: crate::model::AuthorIdentity {
                    username: "tester".into(),
                    display_avatar_url: String::new(),
                },
                guild_tag: None,
                content_type: String::new(),
                timestamp: 22_000,
                visual_only: true,
                segments: Vec::new(),
            },
        )
        .await;

        core.stage_scheduler.stage_state(false, false, false).await;
        assert!(
            matches!(events.recv().await.unwrap(), RelayEvent::Tts(event) if event.id == "text")
        );
        core.stage_scheduler.stage_state(false, false, true).await;
        core.stage_scheduler.stage_state(false, false, false).await;
        assert!(
            matches!(events.recv().await.unwrap(), RelayEvent::Media(event) if event.message_id == "image")
        );
    }
}
