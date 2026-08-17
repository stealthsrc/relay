use std::{
    collections::{BTreeMap, HashMap},
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use tokio::sync::{Mutex, broadcast};

use crate::model::{RelayEvent, TtsEvent};

const READY_TIMEOUT: Duration = Duration::from_secs(20);
const CLAIM_TIMEOUT: Duration = Duration::from_secs(3);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StageLane {
    Media,
    Tts,
    Music,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct StageOrderKey {
    pub timestamp_ms: u64,
    pub message_id: u64,
    pub part: u16,
    pub insertion: u64,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct StageTicket(u64);

#[derive(Clone)]
pub struct StageScheduler {
    inner: Arc<Inner>,
}

struct Inner {
    relay_tx: broadcast::Sender<RelayEvent>,
    next_id: AtomicU64,
    next_insertion: AtomicU64,
    ready_timeout: Duration,
    claim_timeout: Duration,
    state: Mutex<SchedulerState>,
}

#[derive(Default)]
struct SchedulerState {
    ordered: BTreeMap<StageOrderKey, StageTicket>,
    entries: HashMap<StageTicket, TicketEntry>,
    active: Option<ActiveTicket>,
    media_busy: bool,
    music_busy: bool,
    tts_busy: bool,
}

struct TicketEntry {
    original_key: StageOrderKey,
    key: Option<StageOrderKey>,
    lane: StageLane,
    event: Option<RelayEvent>,
}

#[derive(Clone, Copy)]
struct ActiveTicket {
    ticket: StageTicket,
    lane: StageLane,
    saw_busy: bool,
}

impl StageScheduler {
    pub fn new(relay_tx: broadcast::Sender<RelayEvent>) -> Self {
        Self::with_timeouts(relay_tx, READY_TIMEOUT, CLAIM_TIMEOUT)
    }

    fn with_timeouts(
        relay_tx: broadcast::Sender<RelayEvent>,
        ready_timeout: Duration,
        claim_timeout: Duration,
    ) -> Self {
        Self {
            inner: Arc::new(Inner {
                relay_tx,
                next_id: AtomicU64::new(1),
                next_insertion: AtomicU64::new(1),
                ready_timeout,
                claim_timeout,
                state: Mutex::new(SchedulerState::default()),
            }),
        }
    }

    pub async fn reserve(
        &self,
        timestamp_ms: u64,
        message_id: &str,
        part: u16,
        lane: StageLane,
    ) -> StageTicket {
        let ticket = StageTicket(self.inner.next_id.fetch_add(1, Ordering::Relaxed));
        let key = StageOrderKey {
            timestamp_ms,
            message_id: message_id.parse().unwrap_or(u64::MAX),
            part,
            insertion: self.inner.next_insertion.fetch_add(1, Ordering::Relaxed),
        };
        {
            let mut state = self.inner.state.lock().await;
            state.ordered.insert(key, ticket);
            state.entries.insert(
                ticket,
                TicketEntry {
                    original_key: key,
                    key: Some(key),
                    lane,
                    event: None,
                },
            );
        }
        self.spawn_ready_timeout(ticket);
        self.try_dispatch().await;
        ticket
    }

    pub async fn enqueue(&self, event: RelayEvent, lane: StageLane) -> StageTicket {
        let (timestamp, message_id, part) = event_order(&event);
        let numeric_message_id = message_id.parse().unwrap_or(u64::MAX);
        let reserved = {
            let state = self.inner.state.lock().await;
            state.ordered.iter().find_map(|(key, ticket)| {
                let entry = state.entries.get(ticket)?;
                (key.timestamp_ms == timestamp
                    && key.message_id == numeric_message_id
                    && entry.original_key.part == part
                    && entry.lane == lane
                    && entry.event.is_none())
                .then_some(*ticket)
            })
        };
        if let Some(ticket) = reserved {
            self.ready(ticket, event).await;
            return ticket;
        }
        let ticket = self.reserve(timestamp, &message_id, part, lane).await;
        self.ready(ticket, event).await;
        ticket
    }

    pub async fn ready(&self, ticket: StageTicket, event: RelayEvent) {
        {
            let mut state = self.inner.state.lock().await;
            let demoted = match state.entries.get_mut(&ticket) {
                Some(entry) => {
                    entry.event = Some(event);
                    entry.key.is_none()
                }
                None => return,
            };
            if demoted {
                self.reinsert_ready_group(&mut state, ticket);
            }
        }
        self.try_dispatch().await;
    }

    pub async fn cancel(&self, ticket: StageTicket) {
        {
            let mut state = self.inner.state.lock().await;
            if let Some(entry) = state.entries.remove(&ticket) {
                if let Some(key) = entry.key {
                    state.ordered.remove(&key);
                }
                self.reinsert_ready_group_for_key(&mut state, entry.original_key);
            }
            if state.active.is_some_and(|active| active.ticket == ticket) {
                state.active = None;
            }
        }
        self.try_dispatch().await;
    }

    pub async fn stage_state(&self, media_busy: bool, music_busy: bool, tts_busy: bool) {
        let should_dispatch = {
            let mut state = self.inner.state.lock().await;
            state.media_busy = media_busy;
            state.music_busy = music_busy;
            state.tts_busy = tts_busy;
            let mut completed = None;
            if let Some(active) = state.active.as_mut() {
                let busy = lane_busy(active.lane, media_busy, music_busy, tts_busy);
                if busy {
                    active.saw_busy = true;
                } else if active.saw_busy {
                    completed = Some(active.ticket);
                }
            }
            if let Some(ticket) = completed {
                state.active = None;
                state.entries.remove(&ticket);
                true
            } else {
                state.active.is_none()
            }
        };
        if should_dispatch {
            self.try_dispatch().await;
        }
    }

    pub async fn clear(&self) {
        let mut state = self.inner.state.lock().await;
        state.ordered.clear();
        state.entries.clear();
        state.active = None;
    }

    pub async fn skip_active(&self) {
        let should_dispatch = {
            let mut state = self.inner.state.lock().await;
            let Some(active) = state.active.take() else {
                return;
            };
            state.entries.remove(&active.ticket);
            true
        };
        if should_dispatch {
            self.try_dispatch().await;
        }
    }

    async fn try_dispatch(&self) {
        let dispatched = {
            let mut state = self.inner.state.lock().await;
            if state.active.is_some() || state.media_busy || state.music_busy || state.tts_busy {
                return;
            }
            let Some((&key, &ticket)) = state.ordered.first_key_value() else {
                return;
            };
            let Some(entry) = state.entries.get_mut(&ticket) else {
                state.ordered.remove(&key);
                return;
            };
            let Some(event) = entry.event.take() else {
                return;
            };
            let lane = entry.lane;
            state.ordered.remove(&key);
            state.active = Some(ActiveTicket {
                ticket,
                lane,
                saw_busy: false,
            });
            Some((ticket, event))
        };
        if let Some((ticket, event)) = dispatched {
            let _ = self.inner.relay_tx.send(event);
            self.spawn_claim_timeout(ticket);
        }
    }

    fn spawn_ready_timeout(&self, ticket: StageTicket) {
        let scheduler = self.clone();
        let ready_timeout = self.inner.ready_timeout;
        tokio::spawn(async move {
            tokio::time::sleep(ready_timeout).await;
            let demoted = {
                let mut state = scheduler.inner.state.lock().await;
                let active_ticket = state.active.is_some_and(|active| active.ticket == ticket);
                let original_key = match state.entries.get(&ticket) {
                    Some(entry) if entry.event.is_none() && !active_ticket => entry.original_key,
                    Some(_) => return,
                    None => return,
                };
                scheduler.demote_group(&mut state, original_key)
            };
            if demoted {
                scheduler.try_dispatch().await;
            }
        });
    }

    fn spawn_claim_timeout(&self, ticket: StageTicket) {
        let scheduler = self.clone();
        let claim_timeout = self.inner.claim_timeout;
        tokio::spawn(async move {
            tokio::time::sleep(claim_timeout).await;
            let released = {
                let mut state = scheduler.inner.state.lock().await;
                if !state
                    .active
                    .is_some_and(|active| active.ticket == ticket && !active.saw_busy)
                {
                    return;
                }
                state.active = None;
                state.entries.remove(&ticket);
                true
            };
            if released {
                scheduler.try_dispatch().await;
            }
        });
    }

    fn demote_group(&self, state: &mut SchedulerState, original_key: StageOrderKey) -> bool {
        let group = message_group(original_key);
        let tickets = state
            .entries
            .iter()
            .filter_map(|(ticket, entry)| {
                (message_group(entry.original_key) == group).then_some(*ticket)
            })
            .collect::<Vec<_>>();
        let mut demoted = false;
        for ticket in tickets {
            let Some(entry) = state.entries.get_mut(&ticket) else {
                continue;
            };
            if let Some(key) = entry.key.take() {
                state.ordered.remove(&key);
                demoted = true;
            }
        }
        demoted
    }

    fn reinsert_ready_group(&self, state: &mut SchedulerState, ticket: StageTicket) {
        let Some(entry) = state.entries.get(&ticket) else {
            return;
        };
        self.reinsert_ready_group_for_key(state, entry.original_key);
    }

    fn reinsert_ready_group_for_key(
        &self,
        state: &mut SchedulerState,
        original_key: StageOrderKey,
    ) {
        let group = message_group(original_key);
        let mut tickets = state
            .entries
            .iter()
            .filter_map(|(ticket, entry)| {
                (message_group(entry.original_key) == group).then_some(*ticket)
            })
            .collect::<Vec<_>>();
        if tickets.is_empty()
            || !tickets.iter().all(|ticket| {
                state
                    .entries
                    .get(ticket)
                    .is_some_and(|entry| entry.event.is_some())
            })
        {
            return;
        }
        tickets.sort_by_key(|ticket| {
            state
                .entries
                .get(ticket)
                .expect("registered stage ticket")
                .original_key
        });
        let readiness_timestamp = now_ms();
        for ticket in tickets {
            let original_key = state
                .entries
                .get(&ticket)
                .expect("registered stage ticket")
                .original_key;
            let key = StageOrderKey {
                timestamp_ms: readiness_timestamp,
                message_id: original_key.message_id,
                part: original_key.part,
                insertion: self.inner.next_insertion.fetch_add(1, Ordering::Relaxed),
            };
            let entry = state
                .entries
                .get_mut(&ticket)
                .expect("registered stage ticket");
            if entry.key.is_none() {
                entry.key = Some(key);
                state.ordered.insert(key, ticket);
            }
        }
    }
}

fn event_order(event: &RelayEvent) -> (u64, String, u16) {
    match event {
        RelayEvent::Tts(TtsEvent { timestamp, id, .. }) => (*timestamp, id.clone(), 0),
        RelayEvent::Sticker(event) => (event.timestamp, event.message_id.clone(), 100),
        RelayEvent::Media(event) => (event.timestamp, event.message_id.clone(), 200),
        RelayEvent::MusicPlay(event) => (now_ms(), event.playback_id.clone(), 300),
        _ => (now_ms(), String::new(), u16::MAX),
    }
}

fn message_group(key: StageOrderKey) -> (u64, u64) {
    (key.timestamp_ms, key.message_id)
}

fn lane_busy(lane: StageLane, media_busy: bool, music_busy: bool, tts_busy: bool) -> bool {
    match lane {
        StageLane::Media => media_busy,
        StageLane::Tts => tts_busy,
        StageLane::Music => music_busy,
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .try_into()
        .unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{AuthorIdentity, MediaEvent, MediaKind, StickerEvent};

    fn media(timestamp: u64, message_id: &str) -> RelayEvent {
        RelayEvent::Media(MediaEvent {
            kind: MediaKind::Image,
            url: format!("https://cdn.discordapp.com/{message_id}.png"),
            proxy_url: String::new(),
            filename: format!("{message_id}.png"),
            content_type: "image/png".into(),
            artwork_id: None,
            audio_id: None,
            cached_media_id: None,
            title: None,
            artist: None,
            text: None,
            author: AuthorIdentity {
                username: "user".into(),
                display_avatar_url: String::new(),
            },
            timestamp,
            message_id: message_id.into(),
        })
    }

    #[tokio::test]
    async fn older_pending_ticket_blocks_newer_ready_media() {
        let (tx, mut rx) = broadcast::channel(16);
        let scheduler = StageScheduler::new(tx);
        let older = scheduler.reserve(22_000, "100", 0, StageLane::Tts).await;
        scheduler
            .ready(
                scheduler
                    .reserve(22_001, "101", 200, StageLane::Media)
                    .await,
                media(22_001, "101"),
            )
            .await;
        assert!(rx.try_recv().is_err());
        scheduler
            .ready(
                older,
                RelayEvent::Tts(TtsEvent {
                    id: "100".into(),
                    text: "older".into(),
                    author: AuthorIdentity {
                        username: "user".into(),
                        display_avatar_url: String::new(),
                    },
                    guild_tag: None,
                    content_type: String::new(),
                    timestamp: 22_000,
                    visual_only: true,
                    segments: Vec::new(),
                }),
            )
            .await;
        assert!(matches!(rx.recv().await.unwrap(), RelayEvent::Tts(_)));
        scheduler.stage_state(false, false, true).await;
        scheduler.stage_state(false, false, false).await;
        assert!(matches!(rx.recv().await.unwrap(), RelayEvent::Media(_)));
    }

    #[tokio::test]
    async fn active_event_is_never_preempted_by_an_older_late_arrival() {
        let (tx, mut rx) = broadcast::channel(16);
        let scheduler = StageScheduler::new(tx);
        scheduler
            .enqueue(media(22_001, "101"), StageLane::Media)
            .await;
        assert!(matches!(rx.recv().await.unwrap(), RelayEvent::Media(_)));
        scheduler.stage_state(true, false, false).await;
        scheduler
            .enqueue(media(22_000, "100"), StageLane::Media)
            .await;
        assert!(rx.try_recv().is_err());
        scheduler.stage_state(false, false, false).await;
        assert!(matches!(rx.recv().await.unwrap(), RelayEvent::Media(_)));
    }

    #[tokio::test]
    async fn slow_head_is_demoted_then_reinserted_without_being_lost() {
        let (tx, mut rx) = broadcast::channel(16);
        let scheduler =
            StageScheduler::with_timeouts(tx, Duration::from_millis(20), Duration::from_millis(20));
        let slow_text = scheduler.reserve(22_000, "100", 0, StageLane::Tts).await;
        let image = scheduler
            .reserve(22_001, "101", 200, StageLane::Media)
            .await;
        scheduler.ready(image, media(22_001, "101")).await;

        assert!(
            tokio::time::timeout(Duration::from_millis(10), rx.recv())
                .await
                .is_err()
        );
        assert!(matches!(rx.recv().await.unwrap(), RelayEvent::Media(_)));
        scheduler.stage_state(true, false, false).await;
        scheduler.stage_state(false, false, false).await;

        scheduler
            .ready(
                slow_text,
                RelayEvent::Tts(TtsEvent {
                    id: "100".into(),
                    text: "slow text".into(),
                    author: AuthorIdentity {
                        username: "user".into(),
                        display_avatar_url: String::new(),
                    },
                    guild_tag: None,
                    content_type: String::new(),
                    timestamp: 22_000,
                    visual_only: true,
                    segments: Vec::new(),
                }),
            )
            .await;
        assert!(matches!(rx.recv().await.unwrap(), RelayEvent::Tts(_)));
    }

    #[tokio::test]
    async fn reinserts_a_demoted_message_in_its_original_part_order() {
        let (tx, mut rx) = broadcast::channel(16);
        let scheduler =
            StageScheduler::with_timeouts(tx, Duration::from_millis(20), Duration::from_millis(20));
        let text = scheduler.reserve(22_000, "100", 0, StageLane::Tts).await;
        let sticker = scheduler
            .reserve(22_000, "100", 100, StageLane::Media)
            .await;
        let newer_image = scheduler
            .reserve(22_001, "101", 200, StageLane::Media)
            .await;
        scheduler
            .ready(
                sticker,
                RelayEvent::Sticker(StickerEvent {
                    id: "sticker".into(),
                    name: "sticker".into(),
                    format: "png".into(),
                    url: "https://cdn.discordapp.com/stickers/sticker.png".into(),
                    cached_media_id: None,
                    author: AuthorIdentity {
                        username: "user".into(),
                        display_avatar_url: String::new(),
                    },
                    timestamp: 22_000,
                    message_id: "100".into(),
                }),
            )
            .await;
        scheduler.ready(newer_image, media(22_001, "101")).await;

        assert!(
            matches!(rx.recv().await.unwrap(), RelayEvent::Media(event) if event.message_id == "101")
        );
        scheduler.stage_state(true, false, false).await;
        scheduler.stage_state(false, false, false).await;

        scheduler
            .ready(
                text,
                RelayEvent::Tts(TtsEvent {
                    id: "100".into(),
                    text: "slow text".into(),
                    author: AuthorIdentity {
                        username: "user".into(),
                        display_avatar_url: String::new(),
                    },
                    guild_tag: None,
                    content_type: String::new(),
                    timestamp: 22_000,
                    visual_only: true,
                    segments: Vec::new(),
                }),
            )
            .await;
        assert!(matches!(rx.recv().await.unwrap(), RelayEvent::Tts(event) if event.id == "100"));
        scheduler.stage_state(false, false, true).await;
        scheduler.stage_state(false, false, false).await;
        assert!(
            matches!(rx.recv().await.unwrap(), RelayEvent::Sticker(event) if event.id == "sticker")
        );
    }
}
