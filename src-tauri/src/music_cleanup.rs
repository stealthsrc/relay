use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use serde::Serialize;
use serenity::{
    all::{ChannelId, GetMessages, MessageId},
    http::Http,
};
use tauri::State;

use crate::state::AppCore;

const PREVIEW_LIMIT: usize = 1_000;
const TTL: Duration = Duration::from_secs(120);

#[derive(Default)]
pub struct MusicCleanup {
    active: HashMap<(u64, u64), Instant>,
    preview: Option<CleanupSnapshot>,
    generation: u64,
}

struct CleanupSnapshot {
    token: String,
    channel: u64,
    protected: u64,
    messages: Vec<u64>,
    expires: Instant,
}

impl MusicCleanup {
    fn take_preview(
        &mut self,
        token: &str,
        scope: (u64, u64),
        now: Instant,
    ) -> Result<CleanupSnapshot, String> {
        let snapshot = self
            .preview
            .take()
            .ok_or("Preview the cleanup again before confirming.")?;
        if snapshot.token != token || snapshot.expires <= now {
            return Err("The cleanup preview expired. Preview it again.".into());
        }
        if scope != (snapshot.channel, snapshot.protected) {
            return Err("Music settings changed. Preview the cleanup again.".into());
        }
        Ok(snapshot)
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanupPreview {
    token: String,
    count: usize,
    limit_reached: bool,
}

#[derive(Serialize)]
pub struct CleanupResult {
    deleted: usize,
    failed: usize,
    skipped: usize,
}

pub fn protected_message_id(value: &str, channel: &str) -> Result<String, String> {
    let value = value.trim();
    if value.is_empty() {
        return Ok(String::new());
    }
    let id = if value.starts_with("https://discord.com/channels/") {
        let parts: Vec<_> = value.trim_end_matches('/').split('/').collect();
        if parts.len() != 7 || parts[5] != channel || parts[4].parse::<u64>().is_err() {
            return Err(
                "The welcome message link must belong to the selected music channel.".into(),
            );
        }
        parts[6]
    } else {
        value
    };
    if id.parse::<u64>().ok().filter(|id| *id != 0).is_none() {
        return Err("Enter a valid Discord welcome message ID or link.".into());
    }
    Ok(id.to_owned())
}

fn may_delete(id: u64, protected: u64, active: bool) -> bool {
    id != protected && !active
}

async fn http(core: &AppCore) -> Result<Arc<Http>, String> {
    core.bot_runtime
        .lock()
        .await
        .as_ref()
        .map(|runtime| runtime.http.clone())
        .ok_or_else(|| "Connect the Discord bot first.".into())
}

async fn configured(core: &AppCore) -> Result<(u64, u64), String> {
    let config = core.config.read().await;
    if !config.music_cleanup_enabled {
        return Err("Enable music channel cleanup first.".into());
    }
    let channel = config
        .music_channel_id
        .parse::<u64>()
        .map_err(|_| "Select a music channel first.")?;
    let protected = config
        .music_welcome_message_id
        .parse::<u64>()
        .map_err(|_| "Set the welcome message first.")?;
    Ok((channel, protected))
}

async fn active(core: &AppCore, channel: u64, message: u64) -> bool {
    if core
        .music_cleanup
        .lock()
        .await
        .active
        .get(&(channel, message))
        .is_some_and(|expires| *expires > Instant::now())
    {
        return true;
    }
    core.music
        .lock()
        .await
        .active_message_ids()
        .contains(&(channel, message))
}

pub async fn delete(core: &AppCore, http: &Http, channel: u64, message: u64) {
    let Ok((configured_channel, protected)) = configured(core).await else {
        return;
    };
    if channel != configured_channel || !may_delete(message, protected, false) {
        return;
    }
    core.music_cleanup
        .lock()
        .await
        .active
        .remove(&(channel, message));
    if ChannelId::new(channel)
        .delete_message(http, MessageId::new(message))
        .await
        .is_err()
    {
        core.bot_status.write().await.error = Some(
            "Music message cleanup failed. Check View Channel, Read Message History and Manage Messages permissions.".into());
    }
}

pub async fn expire_message(core: &Arc<AppCore>, http: &Arc<Http>, channel: u64, message: u64) {
    let Ok((configured_channel, protected)) = configured(core).await else {
        return;
    };
    if channel != configured_channel || message == protected {
        return;
    }
    core.music_cleanup
        .lock()
        .await
        .active
        .insert((channel, message), Instant::now() + TTL);
    let weak = Arc::downgrade(core);
    let http = http.clone();
    tokio::spawn(async move {
        tokio::time::sleep(TTL).await;
        if let Some(core) = weak.upgrade() {
            let tracked = core
                .music_cleanup
                .lock()
                .await
                .active
                .remove(&(channel, message))
                .is_some();
            if tracked {
                delete(&core, &http, channel, message).await;
            }
        }
    });
}

#[tauri::command]
pub async fn preview_music_cleanup(
    core: State<'_, Arc<AppCore>>,
) -> Result<CleanupPreview, String> {
    let (channel, protected) = configured(&core).await?;
    let http = http(&core).await?;
    ChannelId::new(channel)
        .message(&http, MessageId::new(protected))
        .await
        .map_err(
            |_| "The welcome message could not be found in this channel. Nothing was deleted.",
        )?;
    let mut messages = Vec::new();
    let mut cursor = None;
    let mut scanned = 0;
    while scanned < PREVIEW_LIMIT {
        let mut request = GetMessages::new().limit(100);
        if let Some(before) = cursor {
            request = request.before(before);
        }
        let page = ChannelId::new(channel)
            .messages(&http, request)
            .await
            .map_err(|_| "Unable to read music channel history.")?;
        if page.is_empty() {
            break;
        }
        cursor = page.iter().map(|message| message.id).min();
        scanned += page.len();
        for message in &page {
            if may_delete(
                message.id.get(),
                protected,
                active(&core, channel, message.id.get()).await,
            ) {
                messages.push(message.id.get());
            }
        }
        if page.len() < 100 {
            break;
        }
    }
    let mut cleanup = core.music_cleanup.lock().await;
    cleanup.generation = cleanup.generation.wrapping_add(1);
    let token = format!("cleanup-{}", cleanup.generation);
    let count = messages.len();
    cleanup.preview = Some(CleanupSnapshot {
        token: token.clone(),
        channel,
        protected,
        messages,
        expires: Instant::now() + TTL,
    });
    Ok(CleanupPreview {
        token,
        count,
        limit_reached: scanned >= PREVIEW_LIMIT,
    })
}

#[tauri::command]
pub async fn confirm_music_cleanup(
    core: State<'_, Arc<AppCore>>,
    token: String,
) -> Result<CleanupResult, String> {
    let scope = configured(&core).await?;
    let snapshot = core
        .music_cleanup
        .lock()
        .await
        .take_preview(&token, scope, Instant::now())?;
    let http = http(&core).await?;
    ChannelId::new(snapshot.channel)
        .message(&http, MessageId::new(snapshot.protected))
        .await
        .map_err(|_| "The protected welcome message is no longer available. Cleanup cancelled.")?;
    let mut result = CleanupResult {
        deleted: 0,
        failed: 0,
        skipped: 0,
    };
    for id in snapshot.messages {
        if configured(&core).await? != (snapshot.channel, snapshot.protected) {
            return Err("Music settings changed; cleanup stopped.".into());
        }
        if !may_delete(
            id,
            snapshot.protected,
            active(&core, snapshot.channel, id).await,
        ) {
            result.skipped += 1;
            continue;
        }
        match ChannelId::new(snapshot.channel)
            .delete_message(&http, MessageId::new(id))
            .await
        {
            Ok(()) => result.deleted += 1,
            Err(_) => {
                result.failed += 1;
                break;
            }
        }
        tokio::time::sleep(Duration::from_millis(250)).await;
    }
    if result.failed > 0 {
        core.bot_status.write().await.error = Some(
            "Music cleanup stopped after a Discord deletion error. Check bot permissions.".into(),
        );
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn welcome_message_links_must_match_the_music_channel() {
        assert_eq!(
            protected_message_id("https://discord.com/channels/1/2/3", "2"),
            Ok("3".into())
        );
        assert!(protected_message_id("https://discord.com/channels/1/4/3", "2").is_err());
        assert!(protected_message_id("0", "2").is_err());
        assert!(protected_message_id("not-an-id", "2").is_err());
        assert_eq!(protected_message_id("3", "2"), Ok("3".into()));
    }

    #[test]
    fn cleanup_preserves_welcome_and_active_messages() {
        assert!(!may_delete(7, 7, false));
        assert!(!may_delete(8, 7, true));
        assert!(may_delete(8, 7, false));
    }

    #[test]
    fn confirmation_is_single_use_and_bound_to_channel_message_and_expiration() {
        let now = Instant::now();
        for (token, scope, time, valid) in [
            ("one", (2, 7), now, true),
            ("wrong", (2, 7), now, false),
            ("one", (3, 7), now, false),
            ("one", (2, 8), now, false),
            ("one", (2, 7), now + TTL, false),
        ] {
            let mut cleanup = MusicCleanup {
                preview: Some(CleanupSnapshot {
                    token: "one".into(),
                    channel: 2,
                    protected: 7,
                    messages: vec![8, 9],
                    expires: now + TTL,
                }),
                ..Default::default()
            };
            let result = cleanup.take_preview(token, scope, time);
            assert_eq!(result.is_ok(), valid);
            if let Ok(snapshot) = result {
                assert_eq!(snapshot.messages, vec![8, 9]);
            }
            assert!(cleanup.take_preview("one", (2, 7), now).is_err());
        }
    }

    #[tokio::test]
    async fn automatic_cleanup_does_not_request_deletion_for_protected_or_foreign_messages() {
        let directory = tempfile::tempdir().unwrap();
        let core = AppCore::load(directory.path().join("config.json")).unwrap();
        {
            let mut config = core.config.write().await;
            config.music_cleanup_enabled = true;
            config.music_channel_id = "2".into();
            config.music_welcome_message_id = "7".into();
        }
        let http = Http::new("unused-test-token");
        delete(&core, &http, 2, 7).await;
        delete(&core, &http, 3, 8).await;
        assert!(core.bot_status.read().await.error.is_none());
    }
}
