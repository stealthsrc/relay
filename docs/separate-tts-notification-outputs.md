# Separate TTS notification outputs

## Understanding

- Keep one Discord TTS message stream but expose two independent visual outputs.
- The OBS notification overlay has its own enable switch and permanent Browser Source URL.
- The Windows notification widget has its own visibility, position, and lock state.
- Both outputs may display the same message simultaneously.
- Disabling, hiding, moving, or locking one output must not affect the other.

## Assumptions

- Audio TTS remains a third, separate OBS Browser Source.
- Skip and clear remain global transport actions because both visuals represent the same spoken item.
- Each notification client keeps its own in-memory FIFO and playback clock.
- Existing URLs, credentials, ports, and persisted widget position remain unchanged.

## Approaches considered

1. **Separate controls over independent existing clients (selected).** The current OBS and Windows clients already own separate queues; the UI now exposes that architecture accurately.
2. **Duplicate the notification server and assets.** More code and maintenance with no behavioral benefit.
3. **One shared visibility toggle.** Rejected because it prevents simultaneous but independently controlled outputs.

## Final design

- Move the OBS notification switch out of Playback settings and into a dedicated output card.
- Move the notification Browser Source URL into the same OBS card.
- Keep the Windows output in a separate card with show/hide and lock controls.
- Save the OBS switch immediately and show its own save state.
- Keep Windows changes immediate through their existing Tauri commands.

## Decision log

1. Preserve the two existing WebSocket clients because their queues and DOM state are already isolated.
2. Separate control surfaces rather than duplicate runtime code.
3. Keep the same visual component so OBS and Windows notifications remain recognizably related.
4. Add a regression test proving simultaneous playback and one-client clearing do not affect the other client.

## Validation

- Verify both outputs can be active at the same time.
- Disable OBS while the Windows widget continues to receive messages.
- Clear one client harness while the second remains visible.
- Validate bilingual UI, JavaScript syntax, Rust suites, Clippy, and release startup.
- Rebuild the portable executable and NSIS installer without committing.
