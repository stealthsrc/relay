use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use crate::{
    model::{MusicPlaybackEvent, MusicPlaybackMode},
    youtube::YouTubeTrack,
};

const SEARCH_TTL: Duration = Duration::from_secs(120);
const SELECTION_TTL: Duration = Duration::from_secs(120);
const PREVIEW_DURATION_SECONDS: u64 = 30;

struct PendingSearch {
    owner_id: u64,
    channel_id: u64,
    query: String,
    results: Vec<YouTubeTrack>,
    expires_at: Instant,
}

struct PendingSelection {
    owner_id: u64,
    channel_id: u64,
    track: YouTubeTrack,
    expires_at: Instant,
}

struct CurrentMusic {
    playback: MusicPlaybackEvent,
    owner_id: u64,
    now_playing_message_id: Option<u64>,
}

#[derive(Default)]
pub struct MusicState {
    searches: HashMap<String, PendingSearch>,
    selections: HashMap<String, PendingSelection>,
    current: Option<CurrentMusic>,
    next_id: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MusicSelection {
    pub owner_id: u64,
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
        self.selections.insert(
            selection_id.clone(),
            PendingSelection {
                owner_id: search.owner_id,
                channel_id: search.channel_id,
                track,
                expires_at: Instant::now() + SELECTION_TTL,
            },
        );
        SearchSelection::Selected(selection_id)
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
            channel_id: selection.channel_id,
            track: selection.track,
        })
    }

    pub fn cancel_selection(&mut self, selection_id: &str, user_id: u64) -> SelectionTake {
        self.take_selection(selection_id, user_id)
    }

    pub fn start(
        &mut self,
        selection: MusicSelection,
        mode: MusicPlaybackMode,
    ) -> (Option<MusicPlaybackEvent>, MusicPlaybackEvent) {
        let playback_id = self.next_id("p");
        let end_seconds = match mode {
            MusicPlaybackMode::Preview => Some(
                selection
                    .track
                    .duration_seconds
                    .min(PREVIEW_DURATION_SECONDS),
            ),
            MusicPlaybackMode::Full => None,
        };
        let playback = MusicPlaybackEvent {
            playback_id,
            video_id: selection.track.video_id,
            title: selection.track.title,
            channel_title: selection.track.channel_title,
            thumbnail: selection.track.thumbnail,
            duration_seconds: selection.track.duration_seconds,
            mode,
            start_seconds: 0,
            end_seconds,
        };
        let previous = self.current.replace(CurrentMusic {
            playback: playback.clone(),
            owner_id: selection.owner_id,
            now_playing_message_id: None,
        });
        (previous.map(|current| current.playback), playback)
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

    pub fn stop_current(&mut self) -> Option<MusicPlaybackEvent> {
        self.current.take().map(|current| current.playback)
    }

    pub fn stop_if_current(&mut self, playback_id: &str) -> Option<MusicPlaybackEvent> {
        if self
            .current
            .as_ref()
            .is_some_and(|current| current.playback.playback_id == playback_id)
        {
            return self.stop_current();
        }
        None
    }

    pub fn skip_allowed(&self, playback_id: &str, user_id: u64, is_admin: bool) -> bool {
        self.current.as_ref().is_some_and(|current| {
            current.playback.playback_id == playback_id && (is_admin || current.owner_id == user_id)
        })
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
    }
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
            channel_id: 9,
            track: track("video-1", 90),
        }
    }

    #[test]
    fn only_the_requester_can_select_and_take_a_track() {
        let mut state = MusicState::default();
        let search_id = state.insert_search(7, 9, "test".into(), vec![track("video-1", 90)]);
        assert_eq!(
            state.select_search(&search_id, 8, "video-1"),
            SearchSelection::NotOwner
        );
        let SearchSelection::Selected(selection_id) = state.select_search(&search_id, 7, "video-1")
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
    fn preview_is_cut_at_thirty_seconds_and_full_has_no_cutoff() {
        let mut state = MusicState::default();
        let (_, preview) = state.start(selection(), MusicPlaybackMode::Preview);
        assert_eq!(preview.end_seconds, Some(30));
        let (_, full) = state.start(selection(), MusicPlaybackMode::Full);
        assert_eq!(full.end_seconds, None);
    }

    #[test]
    fn an_old_playback_id_cannot_stop_the_replacement() {
        let mut state = MusicState::default();
        let (_, first) = state.start(selection(), MusicPlaybackMode::Full);
        let (_, second) = state.start(selection(), MusicPlaybackMode::Full);
        assert!(state.stop_if_current(&first.playback_id).is_none());
        assert_eq!(state.current_event(), Some(second));
    }

    #[test]
    fn skip_requires_the_requester_or_an_admin() {
        let mut state = MusicState::default();
        let (_, playback) = state.start(selection(), MusicPlaybackMode::Full);
        assert!(!state.skip_allowed(&playback.playback_id, 8, false));
        assert!(state.skip_allowed(&playback.playback_id, 7, false));
        assert!(state.skip_allowed(&playback.playback_id, 8, true));
    }
}
