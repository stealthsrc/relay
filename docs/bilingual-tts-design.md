# Automatic French and English TTS

## Understanding

- Detect French or English independently for every Discord TTS message.
- Use an installed French Windows voice for French text and an English Windows voice for English text.
- Keep synthesis entirely local and offline.
- Preserve the existing FIFO, OBS Browser Source, notification overlay, skip behavior, and credentials.
- Fall back to the Windows default voice when the language is ambiguous or a selected voice fails.

## Assumptions

- French and English are the only detected languages in this iteration.
- Microsoft Hortense Desktop is the preferred French voice when available.
- The current Windows default English voice is preferred for English when it is tagged as English.
- Short or language-neutral messages may intentionally use the default voice.
- No new dependency is introduced.

## Approaches considered

1. **Deterministic local scoring (selected).** Scores common language-specific words and French characters. Fast, private, testable, and dependency-free.
2. **Windows language detection.** Adds WinRT complexity and depends on optional OS components.
3. **Manual prefixes or channel selection.** Reliable but conflicts with automatic per-message adaptation.

## Final design

- Normalize text to lowercase alphabetic tokens while preserving accented characters.
- Add weighted French and English scores using compact, curated word lists.
- Strongly favor French when French-specific characters are present.
- Select the installed SAPI voice whose language attribute matches the detected language.
- Prefer Hortense for French and the current default voice for English when compatible.
- Retry with the default voice, then enumerate remaining installed voices if synthesis fails.

## Decision log

1. Use local scoring because privacy, speed, and no dependency are explicit constraints.
2. Score each message independently so mixed queues can alternate voices safely.
3. Use SAPI language metadata rather than voice names for compatibility, with Hortense only as a preference.
4. Keep the previous fallback chain so a missing language pack never blocks the queue.

## Validation

- Unit-test French, English, mixed, and ambiguous detection.
- Synthesize real French and English WAV files with installed Windows voices.
- Run the complete Rust and Browser Source suites plus strict Clippy.
- Rebuild the portable executable and NSIS installer without committing.
