use std::{
    collections::{HashMap, VecDeque},
    time::{Duration, Instant},
};

use crate::{
    model::{MusicPlaybackEvent, MusicPlaybackMode},
    youtube::YouTubeTrack,
};

const SEARCH_TTL: Duration = Duration::from_secs(120);
const SELECTION_TTL: Duration = Duration::from_secs(120);
const PREVIEW_DURATION_SECONDS: u64 = 30;
/// Per Discord user: minimum gap between YouTube Data API searches.
/// Mid-range of 5–8s — slows quota spam without blocking pick/play after results.
pub const MUSIC_SEARCH_COOLDOWN: Duration = Duration::from_secs(6);
pub const CUSTOM_MAX_WINDOW_SECONDS: u64 = 60;
pub const MUSIC_QUEUE_CAP: usize = 20;

struct PendingSearch {
    owner_id: u64,
    channel_id: u64,
    query: String,
    results: Vec<YouTubeTrack>,
    expires_at: Instant,
}

struct PendingSelection {
    owner_id: u64,
    owner_name: String,
    channel_id: u64,
    track: YouTubeTrack,
    expires_at: Instant,
}

struct CurrentMusic {
    playback: MusicPlaybackEvent,
    owner_id: u64,
    channel_id: u64,
    now_playing_message_id: Option<u64>,
}

/// Result of stopping the current track, including Discord announce cleanup.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoppedMusic {
    pub playback: MusicPlaybackEvent,
    pub channel_id: u64,
    pub now_playing_message_id: Option<u64>,
}

#[derive(Default)]
pub struct MusicState {
    searches: HashMap<String, PendingSearch>,
    selections: HashMap<String, PendingSelection>,
    /// Last YouTube search attempt per Discord user id (API quota guard).
    search_cooldowns: HashMap<u64, Instant>,
    current: Option<CurrentMusic>,
    pending: VecDeque<CurrentMusic>,
    next_id: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MusicStartResult {
    Started(MusicPlaybackEvent),
    Queued {
        playback: MusicPlaybackEvent,
        position: usize,
    },
    QueueFull,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MusicSkipDecision {
    Allowed,
    NotOwner,
    NotCurrent,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MusicSelection {
    pub owner_id: u64,
    pub owner_name: String,
    pub channel_id: u64,
    pub track: YouTubeTrack,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SearchSelection {
    Selected(String),
    NotFound,
    NotOwner,
    InvalidVideo,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SelectionTake {
    Taken(MusicSelection),
    NotFound,
    NotOwner,
}

impl MusicState {
    /// Remaining wait before this user may call YouTube search again.
    /// Does not affect select / preview / full / custom / queue play.
    pub fn search_cooldown_remaining(&self, user_id: u64, now: Instant) -> Option<Duration> {
        remaining_search_cooldown(
            self.search_cooldowns.get(&user_id).copied(),
            now,
            MUSIC_SEARCH_COOLDOWN,
        )
    }

    /// Record a search attempt (call when about to hit the YouTube API).
    pub fn mark_search_attempt(&mut self, user_id: u64, now: Instant) {
        self.prune_search_cooldowns(now);
        self.search_cooldowns.insert(user_id, now);
    }

    pub fn insert_search(
        &mut self,
        owner_id: u64,
        channel_id: u64,
        query: String,
        results: Vec<YouTubeTrack>,
    ) -> String {
        self.prune(Instant::now());
        let search_id = self.next_id("s");
        self.searches.insert(
            search_id.clone(),
            PendingSearch {
                owner_id,
                channel_id,
                query,
                results,
                expires_at: Instant::now() + SEARCH_TTL,
            },
        );
        search_id
    }

    pub fn select_search(
        &mut self,
        search_id: &str,
        user_id: u64,
        owner_name: &str,
        video_id: &str,
    ) -> SearchSelection {
        self.prune(Instant::now());
        let Some(search) = self.searches.get(search_id) else {
            return SearchSelection::NotFound;
        };
        if search.owner_id != user_id {
            return SearchSelection::NotOwner;
        }
        let Some(track) = search
            .results
            .iter()
            .find(|track| track.video_id == video_id)
            .cloned()
        else {
            return SearchSelection::InvalidVideo;
        };
        let search = self
            .searches
            .remove(search_id)
            .expect("search was checked immediately before removal");
        let selection_id = self.next_id("t");
        let owner_name = owner_name.trim();
        let owner_name = if owner_name.is_empty() {
            format!("User {}", search.owner_id)
        } else {
            owner_name.to_owned()
        };
        self.selections.insert(
            selection_id.clone(),
            PendingSelection {
                owner_id: search.owner_id,
                owner_name,
                channel_id: search.channel_id,
                track,
                expires_at: Instant::now() + SELECTION_TTL,
            },
        );
        SearchSelection::Selected(selection_id)
    }

    pub fn selection_duration_seconds(&self, selection_id: &str) -> Option<u64> {
        self.selections
            .get(selection_id)
            .map(|selection| selection.track.duration_seconds)
    }

    pub fn peek_selection(&self, selection_id: &str, user_id: u64) -> SelectionTake {
        let Some(selection) = self.selections.get(selection_id) else {
            return SelectionTake::NotFound;
        };
        if selection.owner_id != user_id {
            return SelectionTake::NotOwner;
        }
        SelectionTake::Taken(MusicSelection {
            owner_id: selection.owner_id,
            owner_name: selection.owner_name.clone(),
            channel_id: selection.channel_id,
            track: selection.track.clone(),
        })
    }

    pub fn touch_selection(&mut self, selection_id: &str, user_id: u64) -> bool {
        self.prune(Instant::now());
        let Some(selection) = self.selections.get_mut(selection_id) else {
            return false;
        };
        if selection.owner_id != user_id {
            return false;
        }
        selection.expires_at = Instant::now() + SELECTION_TTL;
        true
    }

    pub fn take_selection(&mut self, selection_id: &str, user_id: u64) -> SelectionTake {
        self.prune(Instant::now());
        let Some(selection) = self.selections.get(selection_id) else {
            return SelectionTake::NotFound;
        };
        if selection.owner_id != user_id {
            return SelectionTake::NotOwner;
        }
        let selection = self
            .selections
            .remove(selection_id)
            .expect("selection was checked immediately before removal");
        SelectionTake::Taken(MusicSelection {
            owner_id: selection.owner_id,
            owner_name: selection.owner_name,
            channel_id: selection.channel_id,
            track: selection.track,
        })
    }

    pub fn restore_selection(&mut self, selection_id: &str, selection: MusicSelection) {
        self.prune(Instant::now());
        self.selections
            .entry(selection_id.to_owned())
            .or_insert(PendingSelection {
                owner_id: selection.owner_id,
                owner_name: selection.owner_name,
                channel_id: selection.channel_id,
                track: selection.track,
                expires_at: Instant::now() + SELECTION_TTL,
            });
    }

    pub fn cancel_selection(&mut self, selection_id: &str, user_id: u64) -> SelectionTake {
        self.take_selection(selection_id, user_id)
    }

    pub fn start(
        &mut self,
        selection: MusicSelection,
        mode: MusicPlaybackMode,
    ) -> MusicStartResult {
        match mode {
            MusicPlaybackMode::Preview => {
                let end = selection
                    .track
                    .duration_seconds
                    .min(PREVIEW_DURATION_SECONDS);
                self.start_range(selection, mode, 0, Some(end))
            }
            MusicPlaybackMode::Full => self.start_range(selection, mode, 0, None),
            MusicPlaybackMode::Custom => {
                unreachable!("custom clips must go through start_custom")
            }
        }
    }

    pub fn start_custom(
        &mut self,
        selection: MusicSelection,
        start_seconds: u64,
        end_seconds: u64,
    ) -> Result<MusicStartResult, CustomRangeError> {
        let (start_seconds, end_seconds) =
            validate_custom_range(selection.track.duration_seconds, start_seconds, end_seconds)?;
        Ok(self.start_range(
            selection,
            MusicPlaybackMode::Custom,
            start_seconds,
            Some(end_seconds),
        ))
    }

    fn start_range(
        &mut self,
        selection: MusicSelection,
        mode: MusicPlaybackMode,
        start_seconds: u64,
        end_seconds: Option<u64>,
    ) -> MusicStartResult {
        if self.current.is_some() && self.pending.len() >= MUSIC_QUEUE_CAP {
            return MusicStartResult::QueueFull;
        }
        let playback_id = self.next_id("p");
        let playback = MusicPlaybackEvent {
            playback_id,
            video_id: selection.track.video_id,
            title: selection.track.title,
            channel_title: selection.track.channel_title,
            thumbnail: selection.track.thumbnail,
            duration_seconds: selection.track.duration_seconds,
            mode,
            start_seconds,
            end_seconds,
            requested_by: selection.owner_name,
        };
        let entry = CurrentMusic {
            playback: playback.clone(),
            owner_id: selection.owner_id,
            channel_id: selection.channel_id,
            now_playing_message_id: None,
        };
        if self.current.is_none() {
            self.current = Some(entry);
            MusicStartResult::Started(playback)
        } else {
            self.pending.push_back(entry);
            MusicStartResult::Queued {
                position: self.pending.len(),
                playback,
            }
        }
    }

    pub fn set_now_playing_message_id(&mut self, playback_id: &str, message_id: u64) -> bool {
        let Some(current) = self.current.as_mut() else {
            return false;
        };
        if current.playback.playback_id != playback_id {
            return false;
        }
        current.now_playing_message_id = Some(message_id);
        true
    }

    pub fn current_event(&self) -> Option<MusicPlaybackEvent> {
        self.current
            .as_ref()
            .map(|current| current.playback.clone())
    }

    /// Clears the current track only (does not touch the pending queue).
    pub fn stop_current(&mut self) -> Option<StoppedMusic> {
        self.current.take().map(|current| StoppedMusic {
            playback: current.playback,
            channel_id: current.channel_id,
            now_playing_message_id: current.now_playing_message_id,
        })
    }

    pub fn stop_if_current(&mut self, playback_id: &str) -> Option<StoppedMusic> {
        if self
            .current
            .as_ref()
            .is_some_and(|current| current.playback.playback_id == playback_id)
        {
            return self.stop_current();
        }
        None
    }

    /// Promotes the next queued track to current, if any.
    pub fn promote_next(&mut self) -> Option<MusicPlaybackEvent> {
        let next = self.pending.pop_front()?;
        let playback = next.playback.clone();
        self.current = Some(next);
        Some(playback)
    }

    /// Stops the current track and drops the entire pending queue.
    pub fn clear_all(&mut self) -> Option<StoppedMusic> {
        self.pending.clear();
        self.stop_current()
    }

    pub fn skip_decision(&self, playback_id: &str, user_id: u64) -> MusicSkipDecision {
        match self.current.as_ref() {
            Some(current) if current.playback.playback_id == playback_id => {
                if current.owner_id == user_id {
                    MusicSkipDecision::Allowed
                } else {
                    MusicSkipDecision::NotOwner
                }
            }
            _ => MusicSkipDecision::NotCurrent,
        }
    }

    fn next_id(&mut self, prefix: &str) -> String {
        self.next_id = self.next_id.wrapping_add(1).max(1);
        format!("{prefix}{:x}", self.next_id)
    }

    fn prune(&mut self, now: Instant) {
        self.searches
            .retain(|_, search| search.expires_at > now && !search.query.is_empty());
        self.selections
            .retain(|_, selection| selection.expires_at > now);
        self.prune_search_cooldowns(now);
    }

    fn prune_search_cooldowns(&mut self, now: Instant) {
        self.search_cooldowns
            .retain(|_, last| now.saturating_duration_since(*last) < MUSIC_SEARCH_COOLDOWN);
    }
}

/// Pure helper: how long until `cooldown` elapses since `last_at`.
pub fn remaining_search_cooldown(
    last_at: Option<Instant>,
    now: Instant,
    cooldown: Duration,
) -> Option<Duration> {
    let last_at = last_at?;
    let elapsed = now.saturating_duration_since(last_at);
    if elapsed >= cooldown {
        None
    } else {
        Some(cooldown - elapsed)
    }
}

/// Whole seconds to show in the user-facing cooldown message (ceil, at least 1).
pub fn cooldown_wait_seconds(remaining: Duration) -> u64 {
    let millis = remaining.as_millis();
    (millis.div_ceil(1000) as u64).max(1)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CustomRangeError {
    EmptyRange,
    WindowTooLong,
    OutsideTrack,
}

pub fn parse_timestamp(value: &str) -> Option<u64> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    if let Ok(seconds) = value.parse::<u64>() {
        return Some(seconds);
    }
    let parts = value.split(':').collect::<Vec<_>>();
    match parts.as_slice() {
        [minutes, seconds] => {
            let minutes = minutes.parse::<u64>().ok()?;
            let seconds = seconds.parse::<u64>().ok()?;
            if seconds >= 60 || minutes > 59 {
                return None;
            }
            Some(minutes.checked_mul(60)?.checked_add(seconds)?)
        }
        [hours, minutes, seconds] => {
            let hours = hours.parse::<u64>().ok()?;
            let minutes = minutes.parse::<u64>().ok()?;
            let seconds = seconds.parse::<u64>().ok()?;
            if seconds >= 60 || minutes >= 60 || hours > 2 {
                return None;
            }
            Some(
                hours
                    .checked_mul(3_600)?
                    .checked_add(minutes.checked_mul(60)?)?
                    .checked_add(seconds)?,
            )
        }
        _ => None,
    }
}

pub fn validate_custom_range(
    track_duration_seconds: u64,
    start_seconds: u64,
    end_seconds: u64,
) -> Result<(u64, u64), CustomRangeError> {
    if end_seconds <= start_seconds {
        return Err(CustomRangeError::EmptyRange);
    }
    if end_seconds - start_seconds > CUSTOM_MAX_WINDOW_SECONDS {
        return Err(CustomRangeError::WindowTooLong);
    }
    if start_seconds >= track_duration_seconds || end_seconds > track_duration_seconds {
        return Err(CustomRangeError::OutsideTrack);
    }
    Ok((start_seconds, end_seconds))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn track(video_id: &str, duration_seconds: u64) -> YouTubeTrack {
        YouTubeTrack {
            video_id: video_id.into(),
            title: "Test track".into(),
            channel_title: "Test channel".into(),
            thumbnail: "https://i.ytimg.com/vi/test/default.jpg".into(),
            duration_seconds,
        }
    }

    fn selection() -> MusicSelection {
        MusicSelection {
            owner_id: 7,
            owner_name: "stealthy".into(),
            channel_id: 9,
            track: track("video-1", 90),
        }
    }

    #[test]
    fn only_the_requester_can_select_and_take_a_track() {
        let mut state = MusicState::default();
        let search_id = state.insert_search(7, 9, "test".into(), vec![track("video-1", 90)]);
        assert_eq!(
            state.select_search(&search_id, 8, "other", "video-1"),
            SearchSelection::NotOwner
        );
        let SearchSelection::Selected(selection_id) =
            state.select_search(&search_id, 7, "stealthy", "video-1")
        else {
            panic!("expected a selection");
        };
        assert!(matches!(
            state.take_selection(&selection_id, 8),
            SelectionTake::NotOwner
        ));
        assert!(matches!(
            state.take_selection(&selection_id, 7),
            SelectionTake::Taken(_)
        ));
        assert_eq!(
            state.take_selection(&selection_id, 7),
            SelectionTake::NotFound
        );
    }

    #[test]
    fn rejected_playback_can_restore_the_pending_selection() {
        let mut state = MusicState::default();
        let search_id = state.insert_search(7, 9, "test".into(), vec![track("video-1", 90)]);
        let SearchSelection::Selected(selection_id) =
            state.select_search(&search_id, 7, "stealthy", "video-1")
        else {
            panic!("expected a selection");
        };
        let SelectionTake::Taken(selection) = state.take_selection(&selection_id, 7) else {
            panic!("expected selection ownership");
        };

        state.restore_selection(&selection_id, selection);

        assert!(matches!(
            state.take_selection(&selection_id, 7),
            SelectionTake::Taken(_)
        ));
    }

    #[test]
    fn preview_is_cut_at_thirty_seconds_and_full_has_no_cutoff() {
        let mut state = MusicState::default();
        let MusicStartResult::Started(preview) =
            state.start(selection(), MusicPlaybackMode::Preview)
        else {
            panic!("expected started");
        };
        assert_eq!(preview.end_seconds, Some(30));
        assert_eq!(preview.requested_by, "stealthy");
        // Second start queues while the first is still current.
        let MusicStartResult::Queued {
            playback: full,
            position,
        } = state.start(selection(), MusicPlaybackMode::Full)
        else {
            panic!("expected queued");
        };
        assert_eq!(position, 1);
        assert_eq!(full.end_seconds, None);
        assert_eq!(full.start_seconds, 0);
        assert_eq!(full.duration_seconds, 90);
    }

    #[test]
    fn a_second_track_queues_instead_of_replacing() {
        let mut state = MusicState::default();
        let MusicStartResult::Started(first) = state.start(selection(), MusicPlaybackMode::Full)
        else {
            panic!("expected started");
        };
        let MusicStartResult::Queued {
            playback: second,
            position,
        } = state.start(selection(), MusicPlaybackMode::Full)
        else {
            panic!("expected queued");
        };
        assert_eq!(position, 1);
        assert_eq!(state.current_event(), Some(first.clone()));
        assert!(state.stop_if_current(&second.playback_id).is_none());
        let stopped = state
            .stop_if_current(&first.playback_id)
            .expect("first stopped");
        assert_eq!(stopped.playback, first);
        assert_eq!(state.promote_next(), Some(second.clone()));
        assert_eq!(state.current_event(), Some(second));
    }

    #[test]
    fn queue_full_rejects_additional_tracks() {
        let mut state = MusicState::default();
        assert!(matches!(
            state.start(selection(), MusicPlaybackMode::Full),
            MusicStartResult::Started(_)
        ));
        for _ in 0..MUSIC_QUEUE_CAP {
            assert!(matches!(
                state.start(selection(), MusicPlaybackMode::Full),
                MusicStartResult::Queued { .. }
            ));
        }
        assert_eq!(
            state.start(selection(), MusicPlaybackMode::Full),
            MusicStartResult::QueueFull
        );
        // Current + MUSIC_QUEUE_CAP pending; stop+promote drains the FIFO.
        assert!(state.stop_current().is_some());
        for _ in 0..MUSIC_QUEUE_CAP {
            assert!(state.promote_next().is_some());
            assert!(state.stop_current().is_some());
        }
        assert!(state.promote_next().is_none());
    }

    #[test]
    fn clear_all_drops_current_and_pending() {
        let mut state = MusicState::default();
        let MusicStartResult::Started(first) = state.start(selection(), MusicPlaybackMode::Full)
        else {
            panic!("expected started");
        };
        assert!(matches!(
            state.start(selection(), MusicPlaybackMode::Full),
            MusicStartResult::Queued { .. }
        ));
        assert_eq!(
            state.clear_all().map(|stopped| stopped.playback),
            Some(first)
        );
        assert!(state.current_event().is_none());
        assert!(state.promote_next().is_none());
    }

    #[test]
    fn stop_returns_now_playing_announce_for_cleanup() {
        let mut state = MusicState::default();
        let MusicStartResult::Started(playback) = state.start(selection(), MusicPlaybackMode::Full)
        else {
            panic!("expected started");
        };
        assert!(state.set_now_playing_message_id(&playback.playback_id, 99));
        let stopped = state
            .stop_if_current(&playback.playback_id)
            .expect("stopped");
        assert_eq!(stopped.playback, playback);
        assert_eq!(stopped.channel_id, 9);
        assert_eq!(stopped.now_playing_message_id, Some(99));
    }

    #[test]
    fn skip_requires_the_requester_only() {
        let mut state = MusicState::default();
        let MusicStartResult::Started(playback) = state.start(selection(), MusicPlaybackMode::Full)
        else {
            panic!("expected started");
        };
        assert_eq!(
            state.skip_decision(&playback.playback_id, 7),
            MusicSkipDecision::Allowed
        );
        assert_eq!(
            state.skip_decision(&playback.playback_id, 8),
            MusicSkipDecision::NotOwner
        );
        assert_eq!(
            state.skip_decision("missing", 7),
            MusicSkipDecision::NotCurrent
        );
    }

    #[test]
    fn parses_seconds_and_m_ss_timestamps() {
        assert_eq!(parse_timestamp("50"), Some(50));
        assert_eq!(parse_timestamp("0:50"), Some(50));
        assert_eq!(parse_timestamp("1:50"), Some(110));
        assert_eq!(parse_timestamp("1:60"), None);
        assert_eq!(parse_timestamp(""), None);
    }

    #[test]
    fn custom_range_allows_a_one_minute_window_inside_the_track() {
        assert_eq!(validate_custom_range(180, 50, 110), Ok((50, 110)));
        assert_eq!(validate_custom_range(90, 0, 60), Ok((0, 60)));
        assert_eq!(
            validate_custom_range(90, 35, 95),
            Err(CustomRangeError::OutsideTrack)
        );
        assert_eq!(
            validate_custom_range(180, 10, 80),
            Err(CustomRangeError::WindowTooLong)
        );
        assert_eq!(
            validate_custom_range(180, 40, 40),
            Err(CustomRangeError::EmptyRange)
        );
    }

    #[test]
    fn start_custom_sets_the_validated_window() {
        let mut state = MusicState::default();
        let selection = MusicSelection {
            owner_id: 7,
            owner_name: "stealthy".into(),
            channel_id: 9,
            track: track("video-1", 180),
        };
        let MusicStartResult::Started(playback) = state.start_custom(selection, 50, 110).unwrap()
        else {
            panic!("expected started");
        };
        assert_eq!(playback.mode, MusicPlaybackMode::Custom);
        assert_eq!(playback.start_seconds, 50);
        assert_eq!(playback.end_seconds, Some(110));
    }

    #[test]
    fn remaining_search_cooldown_is_none_when_fresh_or_elapsed() {
        let now = Instant::now();
        assert!(remaining_search_cooldown(None, now, MUSIC_SEARCH_COOLDOWN).is_none());
        assert!(
            remaining_search_cooldown(
                Some(now - MUSIC_SEARCH_COOLDOWN),
                now,
                MUSIC_SEARCH_COOLDOWN
            )
            .is_none()
        );
        let remaining = remaining_search_cooldown(
            Some(now - Duration::from_secs(2)),
            now,
            MUSIC_SEARCH_COOLDOWN,
        )
        .expect("still cooling down");
        assert!(remaining <= Duration::from_secs(4));
        assert!(remaining >= Duration::from_millis(3_900));
    }

    #[test]
    fn cooldown_wait_seconds_ceils_partial_seconds() {
        assert_eq!(cooldown_wait_seconds(Duration::from_millis(1)), 1);
        assert_eq!(cooldown_wait_seconds(Duration::from_millis(1000)), 1);
        assert_eq!(cooldown_wait_seconds(Duration::from_millis(1001)), 2);
        assert_eq!(cooldown_wait_seconds(Duration::from_secs(6)), 6);
    }

    #[test]
    fn search_cooldown_is_per_user_and_ignores_select_play() {
        let mut state = MusicState::default();
        let now = Instant::now();
        state.mark_search_attempt(7, now);
        assert!(state.search_cooldown_remaining(7, now).is_some());
        assert!(state.search_cooldown_remaining(8, now).is_none());

        let search_id = state.insert_search(7, 9, "test".into(), vec![track("video-1", 90)]);
        let SearchSelection::Selected(selection_id) =
            state.select_search(&search_id, 7, "stealthy", "video-1")
        else {
            panic!("select must work during search cooldown");
        };
        let SelectionTake::Taken(selection) = state.take_selection(&selection_id, 7) else {
            panic!("take must work during search cooldown");
        };
        assert!(matches!(
            state.start(selection, MusicPlaybackMode::Preview),
            MusicStartResult::Started(_)
        ));
        // Cooldown still only keyed by the search attempt, not by play.
        assert!(
            state
                .search_cooldown_remaining(7, now + Duration::from_secs(1))
                .is_some()
        );
        assert!(
            state
                .search_cooldown_remaining(7, now + MUSIC_SEARCH_COOLDOWN)
                .is_none()
        );
    }
}
