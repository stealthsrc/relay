# Relay commands design

## Understanding

- Relay gains a dedicated Commands page in the desktop application.
- Discord controls remain grouped under the `/relay` command.
- Each supported subcommand can be enabled or disabled locally.
- `/relay clear` clears active outputs, waiting queues, and local media history.
- `/relay lock` toggles the configured media channel between locked and unlocked.
- Administrators and moderation roles retain access while the channel is locked.
- Channel permission changes must be reversible and survive an application restart.

## Assumptions

- Relay manages one configured media channel at a time.
- Moderation roles are roles with `Administrator`, `Manage Channels`, or `Manage Messages`.
- The Relay bot has `Manage Roles`, which Discord requires to edit channel permission overwrites.
- Command responses are ephemeral and commands remain unavailable to regular members.
- Relay does not collect telemetry or transmit command configuration outside Discord.

## Design

Command availability is stored in `AppConfig`. The Commands page edits this configuration
through the existing Tauri configuration path. Disabled subcommands stay registered so
Discord clients do not require a slow global-command refresh; the handler rejects them
ephemerally before any action is performed.

`clear` performs one atomic local operation: clear every connected overlay, empty pending
media, clear queued TTS work, and remove local history. It never deletes Discord messages.

`lock` reads the configured channel and toggles a persisted lock record. Locking stores the
channel's relevant permission overwrites, denies `Send Messages` to `@everyone`, and adds an
explicit allow for eligible moderation roles when needed. Unlocking restores the saved
overwrites. A second lock request never overwrites the original snapshot.

Failures are reported ephemerally and leave the saved snapshot intact so an unlock can be
retried. Configuration changes and permission mutations are serialized through Relay's
shared state.

## Decision log

- Use `/relay <subcommand>` instead of global `/clear` and `/lock` to avoid name conflicts.
- Keep disabled commands registered and enforce availability in Relay for predictable UI updates.
- Restore exact previous overwrites rather than guessing an unlocked permission state.
- Treat the lock command as a toggle to match the requested one-command workflow.
- Keep Discord message deletion explicitly out of scope.
