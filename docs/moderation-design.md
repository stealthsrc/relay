# Relay media moderation

## Understanding

- Moderation is optional and disabled by default.
- When disabled, media keeps the existing immediate relay behavior.
- When enabled, only allowed media types may enter the pending queue.
- Allowed media still requires local manual approval before reaching OBS.
- Images and GIFs share one type toggle; video and audio have separate toggles.
- TTS is never moderated and keeps its existing behavior.
- Approval and rejection are available only from the local Relay application.

## Assumptions

- The pending queue is FIFO, memory-only, and limited to 50 items.
- Pending media is discarded on restart.
- Media is rejected when the queue is full or its type is disabled.
- Disabling moderation clears the pending queue instead of releasing it.
- Disabling a media type removes pending items of that type.
- Discord users are not notified about moderation decisions.

## Design

The Rust application core owns the pending queue and applies moderation before
history insertion or WebSocket broadcast. Each pending item receives a local
numeric identifier so attachments from the same Discord message can be decided
independently. Tauri commands approve, reject, or clear pending items. Approval
uses the existing media publication path; rejection only removes the item.

The control panel exposes a Moderation page with the master switch, three media
type switches, the pending count, and per-item Approve and Reject actions. The
panel refreshes pending state through the existing local runtime-status polling.

## Decision log

- Chosen: central Rust queue. Rejected: frontend-only interception, because OBS
  could receive media before the panel processes it.
- Chosen: volatile queue. Rejected: disk persistence, to prevent unexpected
  delayed publication and unnecessary retention.
- Chosen: local application decisions only. Rejected: Discord moderation
  commands, to keep approval private and under the broadcaster's control.
