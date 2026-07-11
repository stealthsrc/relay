# TTS Browser Source design

## Understanding

- Relay watches a second Discord text channel dedicated to TTS.
- Every non-empty message from a human user is accepted without a command or prefix.
- Only the message content is spoken; the author name is not announced.
- TTS uses its own FIFO queue and its own OBS Browser Source.
- The Windows default voice synthesizes the audio locally; no cloud service or additional secret is used.
- Messages are unlimited by default. An optional character limit can truncate future messages.
- Skip and Clear apply to both media and TTS outputs.

## Assumptions

- Media and TTS queues are independent because OBS receives them through separate Browser Sources.
- The TTS queue keeps up to 50 prepared messages in memory for predictable resource use.
- A character limit of `0` means unlimited; a positive value truncates by Unicode characters.
- Bots, empty messages, and attachment-only messages are ignored by the TTS channel.
- The voice and pronunciation follow the Windows default speech engine and installed language voices.
- Audio is generated once by the desktop app and played only by the dedicated OBS Browser Source, avoiding duplicate speech from the preview or floating widget.

## Decision log

1. **Separate Discord channels.** Media routing and TTS routing are configured independently.
2. **Plain-message trigger.** The dedicated channel is the explicit opt-in, so no command syntax is needed.
3. **Native Windows synthesis.** This preserves the Windows default voice and keeps text local.
4. **Separate OBS output.** `/tts` carries audio only; `/overlay` remains the visual media source.
5. **Independent FIFO.** TTS never blocks a long video, while TTS messages remain ordered among themselves.
6. **Bounded prepared queue.** The last 50 synthesized items are retained; this prevents unbounded cached audio growth.
