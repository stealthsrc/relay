# Music channel cleanup and Loop

Available in Relay 1.3.3 on Windows.

## Protect the welcome message

1. Open **Music** in Relay and select the Discord music channel.
2. Copy the welcome message link from Discord into **Protected welcome message**.
   A message ID also works. Relay verifies that the message exists in the selected channel.
3. Enable **Automatically clean music messages** and save the music settings.

The bot needs **View Channel**, **Read Message History**, and **Manage Messages**.
Cleanup is disabled by default and requires a protected message. Replacing the music
channel through Relay's channel recreation command disables cleanup until a new
welcome message is configured.

Requests disappear after processing. Search results disappear after a selection or
after 120 seconds without a selection. Error replies expire after 120 seconds.
Mode choices and confirmations are private Discord responses. Playback cards remain
while a track is queued, playing, or waiting to repeat. They disappear when removed
from the queue, skipped, cleared, or completed with Loop off.

## Clean existing history

1. Save the music settings.
2. Select **Preview channel cleanup**.
3. Review the count, then choose **Delete these messages** or **Cancel**.

The preview examines up to the latest 1,000 messages. It excludes the welcome
message and tracked active interactions. Confirmation uses that exact snapshot,
expires after 120 seconds, and cannot be reused. Messages arriving after the
preview are not added to it. Changing the channel or protected message invalidates
the preview. Deletion stops on the first Discord error; the panel reports the result.
Repeat the preview if older messages remain. No cleanup confirmation is posted to Discord.

## Repeat a track

The requester can toggle **Loop: OFF / ON** beside **Skip**. Loop starts disabled.
It repeats the selected full track, preview, or custom range without another YouTube
search. Each repetition receives a fresh playback ID and joins the shared scheduler
behind waiting work, while keeping the same Discord card.

Turning Loop off lets the current pass finish. If a repetition is still queued,
turning Loop off removes it. Skip removes the track immediately, including a queued
repetition. Stale or duplicated playback-end notifications cannot repeat an old track.

## Limits

- Cleanup runs while Relay is running. An interrupted cleanup can leave messages;
  use the history preview after restarting.
- The existing volume control applies to YouTube, media audio/video, and TTS.
- Discord permission failures are reported in Relay. They do not create a public
  cleanup message.
- Actual Discord deletion, permissions, and OBS playback should be checked in a
  disposable channel before enabling cleanup in a shared channel.
