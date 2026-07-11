<p align="center">
  <img src="assets/Relay.png" alt="Relay logo" width="120" />
</p>

# Relay — User Guide

This guide walks through a complete, concrete setup: from creating the Discord bot to seeing media and text-to-speech live in OBS. For a general overview, see the [README](README.md).

## Table of contents

1. [Create the Discord bot](#1-create-the-discord-bot)
2. [First launch](#2-first-launch)
3. [Choose the channels](#3-choose-the-channels)
4. [Add the sources in OBS](#4-add-the-sources-in-obs)
5. [Everyday use](#5-everyday-use)
6. [Text-to-speech](#6-text-to-speech)
7. [Moderation](#7-moderation)
8. [On-screen widgets](#8-on-screen-widgets)
9. [Tray & shortcuts](#9-tray--shortcuts)
10. [The /relay command](#10-the-relay-command)
11. [Personalization](#11-personalization)
12. [Troubleshooting](#12-troubleshooting)

---

## 1. Create the Discord bot

1. Open the [Discord Developer Portal](https://discord.com/developers/applications) and click **New Application**. Name it (e.g. *Relay*).
2. In **Bot**, click **Reset Token** and copy the **Token** — you will paste it into Relay. Treat it like a password.
3. Still in **Bot**, enable **Message Content Intent** (under *Privileged Gateway Intents*). Without it the bot cannot read messages.
4. In **General Information**, copy the **Application ID** (this is the *Client ID*).

> The bot only needs to *read* the watched channels. No admin permission, no send-message permission.

## 2. First launch

1. Start **Relay**. The control panel opens.
2. In the **Credentials** section, paste the **Client ID** and the **Token**, then save. Both are stored in the **Windows Credential Manager** — they never touch the disk in plain text.
3. The panel shows an **invite link**: open it to add the bot to your Discord server. It requests only *View Channel* and *Read Message History*.
4. Once the bot connects, the **Status** section shows its name and avatar, the local server state, and how many overlay clients (OBS sources) are connected.

## 3. Choose the channels

In the **Configuration** section, pick:

- **Media channel** — the channel viewers post images/GIFs/videos/audio into.
- **TTS channel** *(optional)* — a *different* channel whose text messages are read aloud.

Changes apply immediately, no restart needed. Alternatively, a server administrator can run [`/relay channel`](#10-the-relay-command) in Discord.

## 4. Add the sources in OBS

The panel shows ready-to-copy **Browser Source URLs**. All of them point at your machine only (`127.0.0.1`).

| Source | URL | Suggested size | Purpose |
|---|---|---|---|
| Media | `http://127.0.0.1:4590/medias` | canvas size (e.g. 1920×1080) | Images, GIFs, videos |
| Audio | `http://127.0.0.1:4590/audios` | 1920×1080 | Audio files + "Now playing" card |
| TTS | `http://127.0.0.1:4590/tts` | 300×100 | Text-to-speech playback |
| Notifications | `http://127.0.0.1:4590/notifications` | 560×160 | TTS notification cards |

In OBS: **Sources → + → Browser**, paste the URL, set width/height, and enable **"Control audio via OBS"** for the *Audio* and *TTS* sources so you can mix their volume.

> **Tip — split outputs:** using separate *Media* and *Audio* sources lets you route visuals and sound independently (different scenes, audio filters, etc.). The `/overlay` page combines both, but requires the secret in the URL — use the URLs from the panel instead.

The pages have a transparent background and reconnect automatically if Relay restarts (backoff 1 s → 10 s).

## 5. Everyday use

- **Post media** in the watched channel: up to **3 attachments per message** are relayed. Supported: images (PNG/JPG/WebP), **GIFs** (attachments *and* Tenor/Giphy/KLIPY links), videos, and audio files.
- **Display timing**: static images stay for the *Image duration*, GIFs loop for the *GIF duration*, videos and audio play to the end at the configured *Media volume*.
- **Queue**: media arriving while another is displayed waits in a FIFO queue.
- **Now playing**: audio files with embedded tags show a card with cover art, title and artist.
- **Author badge**: the poster's avatar and name appear with the media (toggleable with *Show author*).
- **History**: the panel lists the last **50** media with **Replay**, and global **Skip** / **Clear overlay** buttons.
- **Skip anywhere**: press **`Ctrl+Alt+S`** even when Relay is not focused.

## 6. Text-to-speech

When a TTS channel is set, every human (non-bot) message in it is synthesized with Windows voices and played through the `/tts` browser source.

- **Automatic language detection**: French messages are read by a French voice (Hortense preferred if installed), everything else by an English voice. Install voices via *Windows Settings → Time & language → Speech*.
- **Character limit**: optionally truncate long messages.
- **Queue limit**: 1–50 pending messages; extra messages are dropped.
- **Notification card**: enable *TTS notifications (OBS)* to show a card (avatar, name, message text) in the `/notifications` source while each message plays. The same card is available as an [on-screen widget](#8-on-screen-widgets).

## 7. Moderation

Enable **Moderation** in the panel to hold every incoming media for review before it reaches the stream:

- Pending media (up to 50) appear in the panel's moderation queue with **Approve** / **Reject** buttons.
- **Per-type filters** let you auto-block categories entirely: images (incl. GIFs), videos, audio.
- Disabling moderation clears the pending queue.

Recommended for public channels — nothing goes on stream without your click.

## 8. On-screen widgets

Widgets are transparent, borderless, **always-on-top** windows that show the overlay or the TTS notifications directly on your desktop — no OBS required (handy for previews or single-PC setups where OBS captures the screen).

- **Media widget** (640×360) and **Notification widget** (510×130), toggled from the panel or the tray.
- **Drag** them anywhere; their position is remembered.
- **Lock** makes a widget click-through (mouse events pass to the window below) — unlock from the panel or tray to move it again.
- Widgets are muted; sound comes from the OBS sources.
- Visible widgets are restored on the next launch.

## 9. Tray & shortcuts

- Closing the main window **does not quit** Relay — it keeps running in the system tray.
- Click the tray icon to open a quick panel: bot/server status, open control panel, show/lock both widgets, quit.
- Only one instance of Relay can run at a time; launching it again focuses the existing one.
- Global shortcut: **`Ctrl+Alt+S`** — skip the currently displayed media.

## 10. The /relay command

Server **administrators** can manage Relay from Discord (replies are ephemeral):

| Command | Effect |
|---|---|
| `/relay channel <#channel>` | Set the watched media channel |
| `/relay show` | Show the current configuration |
| `/relay url` | Get the overlay URL (with secret) |
| `/relay regenerate` | Regenerate the overlay secret (old URLs stop working) |

## 11. Personalization

In the panel's appearance settings:

- **Theme**: light or dark (true-black, OLED-friendly) — also applied to the window title bar.
- **Accent color**: any RGB color.
- **Font scale**: 80–140 %.
- **Language**: English, Français, Español, Deutsch.

Appearance changes are broadcast live to connected overlays.

## 12. Troubleshooting

**Nothing appears in OBS**
- Check the panel's *Status*: is the bot connected? Is at least one overlay client connected?
- Verify the Browser Source URL matches the one shown in the panel (port included).
- In OBS, right-click the source → *Refresh cache of current page*.

**The bot is online but ignores messages**
- Make sure the **Message Content Intent** is enabled in the Developer Portal.
- Confirm the watched channel is the one you post in, and the bot can see it (*View Channel* + *Read Message History*).
- Messages from bots are ignored by design.

**TTS is silent**
- The TTS channel must be set and *different* from the media channel.
- Check the `/tts` browser source exists and is unmuted in the OBS audio mixer.
- Verify Windows voices are installed (*Settings → Speech*).

**Port already in use**
- Change the port in the panel (≥ 1024). Connected overlays follow the move automatically; update your OBS URLs if they don't reconnect.

**Overlay URL shows "401"**
- The `/overlay` page requires the secret. Use the `/medias`, `/audios`, `/tts` and `/notifications` URLs from the panel, or get a full URL via `/relay url`.

**A media is stuck on screen**
- Press **`Ctrl+Alt+S`** or click **Skip** / **Clear overlay** in the panel.
