# TTS notification widget design

## Understanding

- Messages from the dedicated TTS Discord channel also feed a notification queue.
- A notification appears when its TTS playback starts and remains visible until that playback ends.
- The card shows the Discord avatar, author name, and at most three lines of message text.
- OBS notifications and the Windows desktop widget have independent enable switches.
- The desktop widget is transparent, movable across monitors, lockable, always on top, and remembers its position.
- Queue capacity defaults to 50 and is configurable from 1 through 50.
- Visual notifications never disable TTS, have no extra chime, and are not persisted across restarts.

## Assumptions

- OBS receives notifications through a separate permanent authenticated Browser Source URL.
- Both visual outputs use the same local WAV as the TTS source, muted, to synchronize start, natural end, and skip without adding an inbound overlay command channel.
- Notification data remains in memory and is served only from `127.0.0.1`.
- If an avatar cannot load, the existing radar mark is used.
- The first desktop position is the top-right of the primary monitor; every later valid position is restored.

## Approaches considered

1. **Muted playback clock (selected).** Each visual client follows the same FIFO and muted WAV lifecycle as TTS. This keeps the security model read-only and handles natural media endings.
2. **Server duration timers.** Parsing WAV duration in Rust would avoid duplicate fetches but can drift from actual browser playback.
3. **Playback acknowledgements.** The TTS Browser Source could report start/end to the server, but that expands the trusted WebSocket command surface and fails when OBS is closed.

## Final design

- Extend `TtsEvent` with the prepared text and `AuthorIdentity`.
- Add `ttsNotificationObsEnabled`, `ttsQueueLimit`, and notification widget geometry/visibility/lock fields to local configuration.
- Serve `/notifications` plus dedicated CSS and JavaScript assets.
- Reuse `/tts-audio/{id}` for a muted timing element in notification clients.
- Add a dedicated Tauri `notification-widget` window sized for a compact console-style card.
- Expose notification URL, OBS switch, Windows switch, queue limit, and lock control in the bilingual panel.
- `skip` and `clear` advance or clear TTS and notification clients together.

## Decision log

1. Notifications are sourced only from the TTS channel.
2. OBS and Windows visibility are controlled independently.
3. Cards are synchronized to TTS playback rather than Discord receipt time.
4. Cards remain visible for the full spoken message.
5. Message text is clamped to three lines; speech remains unabridged except for the existing configured character limit.
6. Queue capacity is configurable but never exceeds 50.
7. No notification history or additional notification sound is introduced.
