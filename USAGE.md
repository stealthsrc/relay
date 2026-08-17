<p align="center">
  <img src="assets/Relay.png" alt="Relay logo" width="120" />
</p>

# Relay — User Guide

This guide walks through a complete, concrete setup: from creating the Discord bot to seeing media and text-to-speech live in OBS. For a general overview, see the [README](README.md).

## Table of contents

1. [Create the Discord bot](#1-create-the-discord-bot)
2. [First launch](#2-first-launch)
3. [YouTube Data API key (music)](#3-youtube-data-api-key-music)
4. [Choose the channels](#4-choose-the-channels)
5. [Add the sources in OBS](#5-add-the-sources-in-obs)
6. [Everyday use](#6-everyday-use)
7. [Text-to-speech](#7-text-to-speech)
8. [Moderation](#8-moderation)
9. [On-screen widgets](#9-on-screen-widgets)
10. [Tray & shortcuts](#10-tray--shortcuts)
11. [The /relay command](#11-the-relay-command)
12. [Personalization](#12-personalization)
13. [Troubleshooting](#13-troubleshooting)

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

## 3. YouTube Data API key (music)

Music search in Discord needs a **Google Cloud API key** for [YouTube Data API v3](https://developers.google.com/youtube/v3). Relay does **not** use OAuth: you do **not** create a client ID, client secret, or authorized redirect URI.

A shorter English walkthrough also lives in [`docs/youtube-api-setup.md`](docs/youtube-api-setup.md) and in the Relay **Music** panel.

### Create the Google Cloud project

1. Open the [Google Cloud Console](https://console.cloud.google.com/).
2. Sign in with a Google account.
3. Top bar → **Select a project** → **New project**.
4. Name it (e.g. `Relay Music`) → **Create**.
5. Select that project so the top bar shows its name (every later step must stay in this project).

### Enable YouTube Data API v3

1. Open the navigation menu (☰) → **APIs & Services** → **Library**  
   (direct: [API Library](https://console.cloud.google.com/apis/library)).
2. Search for **YouTube Data API v3**.
3. Open it → **Enable**.
4. Wait until the console confirms the API is enabled (you may be redirected to the API overview).

### Create and restrict the API key

1. **APIs & Services** → **Credentials**  
   (direct: [Credentials](https://console.cloud.google.com/apis/credentials)).
2. **+ Create credentials** → **API key**.
3. Copy the key once, then open **Edit API key** (or **Restrict key**).
4. Under **API restrictions**, choose **Restrict key** and select only **YouTube Data API v3**.
5. Under **Application restrictions**, leave **None** for a simple home setup, or use **IP addresses** if you have a stable public IP. Do **not** set HTTP referrers for Relay (it calls Google from the desktop app, not from a website).
6. **Save**.

> Treat the key like a password. Never commit it, paste it in Discord, or share screenshots that show the full value.

### Paste the key into Relay

1. In Relay → **Music**.
2. Paste the key into **YouTube API key** and choose a **Music channel**.
3. Click **Save music settings**. The key field clears afterward: the key is stored in **Windows Credential Manager** and is never shown again.
4. In that Discord channel, type a search (e.g. `jennie seoul city`), pick a result, then choose preview (~30 s) or full track. Relay mixes **relevance** with **newest uploads** (up to 15 tracks ≤ **3 minutes**).

### Quotas and common errors

| Symptom | What to check |
|---|---|
| Bot says the YouTube key is missing / invalid | Key saved in Relay; length/format OK; bot restarted after save |
| Search returns API / quota errors | YouTube Data API v3 enabled on the **same** project as the key; quota not exhausted ([Quotas](https://console.cloud.google.com/apis/api/youtube.googleapis.com/quotas)) |
| `403` / accessNotConfigured | API not enabled, or key restricted to the wrong APIs |
| Empty results for a real song | Query too vague, or only videos longer than 3 minutes matched |

Default free quota is usually enough for personal streaming. Create a **new** key (and delete the old one) if it leaks.

## 4. Choose the channels

In the **Configuration** section, pick:

- **Media channel** — the channel viewers post images/GIFs/videos/audio into.
- **TTS channel** *(optional)* — a *different* channel whose text messages are read aloud.
- **Music channel** *(optional)* — configured on the **Music** page with the YouTube API key ([§3](#3-youtube-data-api-key-music)).

Changes apply immediately, no restart needed. Alternatively, a server administrator can run [`/relay channel`](#11-the-relay-command) in Discord.

## 5. Add the sources in OBS

The panel shows **two** ready-to-copy **Browser Source URLs** (Overview → OBS Browser Sources). They replace the older separate medias / audios / stickers / TTS / notifications / YouTube sources.

| Source | URL | Suggested size | Purpose |
|---|---|---|---|
| **Relay Visual** | `http://localhost:4590/obs/visual` | 1920×1080 | Images, GIFs, videos, stickers, TTS notification cards, YouTube jukebox |
| **Relay Audio** | `http://127.0.0.1:4590/obs/audio` | 1920×1080 | Discord audio files + TTS voice |

Use **`localhost`** for Visual (not `127.0.0.1`) so YouTube embeds accept the page Referer. Relay redirects `127.0.0.1/obs/visual` to `localhost` automatically.

In OBS: **Sources → + → Browser**, paste each URL, set width/height, and enable **"Control audio via OBS"** on **both** sources so you can mix Visual (YouTube) and Audio (Discord/TTS) separately.

### Migration from older setups

1. Add the two new sources above.
2. Remove the old separate Browser Sources (`/medias`, `/audios`, `/stickers`, `/tts`, `/notifications`, `/youtube`) to avoid double video/audio.
3. Legacy URLs still work if you need them temporarily.

> **Windows widgets** (media floating widget + notification / Now Playing) stay separate from OBS and are unchanged.

The pages have a transparent background and reconnect automatically if Relay restarts (backoff 1 s → 10 s).

## 6. Everyday use

- **Post media** in the watched channel: up to **3 attachments per message** are relayed. Supported: images (PNG/JPG/WebP), **GIFs** (attachments *and* Tenor/Giphy/KLIPY links), videos, and audio files.
- **Display timing**: static images stay for the *Image duration*, GIFs loop for the *GIF duration*, videos and audio play to the end at the configured *Media volume*.
- **Queue**: media arriving while another is displayed waits in a FIFO queue.
- **Now playing**: audio files with embedded tags show a card with cover art, title and artist.
- **Author badge**: the poster's avatar and name appear with the media (toggleable with *Show author*).
- **Media message**: optionally show up to **180 characters** from the Discord message in OBS, the Windows widget, or both. Standalone links are omitted.
- **History**: the panel lists the last **50** media with **Replay**, and global **Skip** / **Clear overlay** buttons.
- **Skip anywhere**: press **`Ctrl+Alt+S`** even when Relay is not focused.

## 7. Text-to-speech

When a TTS channel is set, every human (non-bot) message in it is synthesized with Windows voices and played through the **Relay Audio** Browser Source (`/obs/audio`, which includes `/tts`).

- **Automatic language detection**: Relay explicitly detects French and English. French messages prefer a French voice (Hortense when installed), English messages prefer an English voice, and other text falls back to the available Windows default voice. Interface language does not change TTS detection. Install voices via *Windows Settings → Time & language → Speech*.
- **Character limit**: optionally truncate long messages.
- **Queue limit**: 1–50 pending messages; extra messages are dropped.
- **Notification card**: enable *Enable OBS overlay* to show a card (avatar, name, message text) inside **Relay Visual** while each message plays. The same card is available as an [on-screen widget](#9-on-screen-widgets).

## 8. Moderation

Enable **Moderation** in the panel to hold every incoming media for review before it reaches the stream:

- Pending media (up to 50) appear in the panel's moderation queue with **Approve** / **Reject** buttons.
- **Per-type filters** let you auto-block categories entirely: images (incl. GIFs), videos, audio.
- Disabling moderation clears the pending queue.

Recommended for public channels — nothing goes on stream without your click.

## 9. On-screen widgets

Widgets are transparent, borderless, **always-on-top** windows that show the overlay or the TTS notifications directly on your desktop — no OBS required (handy for previews or single-PC setups where OBS captures the screen).

- **Media widget** (640×360) and **Notification widget** (640×176), toggled from the panel or the tray.
- **Drag** them anywhere; their position is remembered.
- **Lock** makes a widget click-through (mouse events pass to the window below) — unlock from the panel or tray to move it again.
- Widgets are muted; sound comes from the OBS sources.
- Visible widgets are restored on the next launch.

## 10. Tray & shortcuts

- Closing the main window **does not quit** Relay — it keeps running in the system tray.
- Click the tray icon to open a quick panel: bot/server status, open control panel, show/lock both widgets, quit.
- Only one instance of Relay can run at a time; launching it again focuses the existing one.
- Global shortcut: **`Ctrl+Alt+S`** — skip the currently displayed media.

## 11. The /relay command

Server **administrators** can manage Relay from Discord (replies are ephemeral):

| Command | Effect |
|---|---|
| `/relay channel <#channel>` | Set the watched media channel |
| `/relay show` | Show the current configuration |
| `/relay status` | Show live OBS output, queue, and Windows widget status |
| `/relay test <media\|audio\|tts\|notification\|sticker>` | Send an isolated local test to a connected output |
| `/relay url` | Get the overlay URL (with secret) |
| `/relay regenerate` | Regenerate the overlay secret (old URLs stop working) |

## 12. Personalization

In the panel's appearance settings:

- **Theme**: light or dark (true-black, OLED-friendly) — also applied to the window title bar.
- **Accent color**: any RGB color.
- **Font scale**: 80–140 %.
- **Language**: English (US, UK, and India), Français, Deutsch, Español, Español (Latinoamérica), Русский, 简体中文, 한국어, 日本語, and Bahasa Indonesia.

Appearance changes are broadcast live to connected overlays.

## 13. Troubleshooting

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
- The `/overlay` page requires the secret. Prefer the panel’s **Relay Visual** / **Relay Audio** URLs (`/obs/visual`, `/obs/audio`). Legacy short URLs (`/medias`, `/audios`, `/youtube`, `/tts`, `/notifications`, `/stickers`) still work.

**A media is stuck on screen**
- Press **`Ctrl+Alt+S`** or click **Skip** / **Clear overlay** in the panel.

**Music search fails in Discord**
- Confirm a **YouTube API key** is saved under **Music** ([§3](#3-youtube-data-api-key-music)).
- Confirm **YouTube Data API v3** is enabled on the same Google Cloud project as that key.
- Confirm a **Music channel** is selected and you are typing in that channel.
- English step-by-step: [`docs/youtube-api-setup.md`](docs/youtube-api-setup.md).
