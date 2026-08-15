# Windows smoke-test matrix

Run the automated checks in [CONTRIBUTING.md](../CONTRIBUTING.md) first. Use this matrix only for behavior that needs Windows, Discord, OBS, configured media codecs, or a signed release artifact.

Use a disposable Discord server and synthetic local media. Do not use tokens, private Browser Source URLs, real addresses, real contact details, or personal image metadata.

## Before testing

1. Start Relay with `cargo tauri dev` from `src-tauri` or install a locally built package.
2. Use a disposable bot and test channel. Grant only the permissions needed for the scenario.
3. Open the relevant OBS Browser Source or Windows widget before checking Output readiness.
4. Record the Relay version, Windows version, test result, and any sanitized error message.

## Runtime and outputs

| Scenario | Steps | Expected result |
| --- | --- | --- |
| Start without credentials | Launch Relay before configuring Discord credentials. | The interface remains usable and shows the bot as offline. |
| Bot connection | Save valid credentials and select a test media channel. | The bot status becomes online and visible text channels can be refreshed. |
| Local server | Open the Overlay page. | The server status is online and the preview can connect. |
| OBS source | Add one generated Browser Source URL to OBS. | Output readiness lists an OBS client for that output. |
| Windows widgets | Show, move, and lock each widget. | The widget follows its visibility and lock settings without blocking Relay. |
| Local output test | Connect an OBS source or widget, then use **Test output**. | The test appears only on the selected local output and no Discord message is sent. |

## Discord media and moderation

| Scenario | Steps | Expected result |
| --- | --- | --- |
| Image, GIF, audio, video, and sticker | Post one synthetic sample of each supported type in the watched channel. | Each accepted item follows its configured output route and duration. |
| H.264 and HEVC video | Post short synthetic H.264 and HEVC files separately. | Relay stays responsive. Record the observed initial delay and any FFmpeg fallback result. |
| Manual moderation | Enable review for the tested media type and post a synthetic sample. | The item enters the local review queue and reaches outputs only after approval. |
| Automatic filter | Add `RELAY-PRIVACY-SMOKE-ONLY` as a temporary filter word, then post that exact text. | The configured block or review action occurs before public Relay outputs. Remove the test word afterwards. |
| Privacy scanner | Add the same temporary value as a protected private string and repeat the test. | Relay records only the classification and category. The test value is not copied into logs or visible history. |
| Delete blocked messages | Enable deletion, grant Manage Messages in the disposable channel, and repeat a known blocking test. | The blocked test message is deleted only when Relay has the required Discord permission. |

## Image metadata and OCR

| Scenario | Steps | Expected result |
| --- | --- | --- |
| EXIF/GPS | Use a locally created image with synthetic metadata only. | The configured privacy policy handles detected metadata without exposing its values in Relay errors or logs. |
| Local OCR | Use a locally created image containing `RELAY-PRIVACY-SMOKE-ONLY`. | OCR failure does not stop Relay. A confirmed match follows the configured privacy policy. |
| Large or malformed file | Attach a safe test file near the configured limit and a malformed image. | Relay rejects unsafe input cleanly and continues receiving later media. |

## Release-only checks

| Scenario | Steps | Expected result |
| --- | --- | --- |
| Signed update | Run against an official signed installer and release feed. | Relay accepts only the expected signed update path. |
| Missing speech packs | Test an English or French TTS message on a Windows installation without the matching pack. | Relay reports a recoverable local error; the application remains usable. |

Do not mark a pull request as fully smoke-tested when a required integration is unavailable. State the skipped row and why in the pull request instead.
