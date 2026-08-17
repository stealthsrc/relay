(function notificationOverlayApp() {
const cardElement = document.querySelector("#notification");
const avatarElement = document.querySelector("#notification-avatar");
const authorElement = document.querySelector("#notification-author");
const guildTagElement = document.querySelector("#notification-guild-tag");
const guildTagBadgeElement = document.querySelector("#notification-guild-tag-badge");
const guildTagNameElement = document.querySelector("#notification-guild-tag-name");
const messageElement = document.querySelector("#notification-message");
const musicCardElement = document.querySelector("#music");
const musicArtworkElement = document.querySelector("#music-artwork");
const musicLabelElement = document.querySelector("#music-label");
const musicTitleElement = document.querySelector("#music-title");
const musicArtistElement = document.querySelector("#music-artist");
const musicMetaElement = document.querySelector("#music-meta");
const musicTimeElement = document.querySelector("#music-time");
const musicProgressElement = document.querySelector("#music-progress");
const musicProgressFillElement = document.querySelector("#music-progress-fill");
const audioElement = document.querySelector("#notification-clock");
const parameters = new URLSearchParams(window.location.search);
const relaySecret =
  parameters.get("secret")
  || document.querySelector('meta[name="relay-secret"]')?.content
  || "";
const target = parameters.get("target") === "widget" ? "widget" : "obs";
const isPreview = parameters.get("preview") === "1";
const outputClient = isPreview ? "preview" : target === "widget" ? "widget" : "obs";
let interfaceLanguage = parameters.get("lang") || "en";
const moveLabelElement = document.querySelector("#notification-move-label");
const moveLabels = { en: "Move notification", fr: "Déplacer la notification", es: "Mover notificación", de: "Benachrichtigung verschieben" };
const previewCopy = {
  en: { author: "Live preview", message: "Your notification will appear here." },
  fr: { author: "Aperçu en direct", message: "Votre notification apparaîtra ici." },
  es: { author: "Vista previa", message: "Tu notificación aparecerá aquí." },
  de: { author: "Live-Vorschau", message: "Deine Benachrichtigung erscheint hier." },
};
const musicLabels = {
  en: "NOW PLAYING",
  fr: "EN LECTURE",
  es: "REPRODUCIENDO",
  de: "WIRD ABGESPIELT",
  ru: "СЕЙЧАС ИГРАЕТ",
  zh: "正在播放",
  ko: "지금 재생 중",
  ja: "再生中",
  id: "SEDANG DIPUTAR",
};
const musicAddedByLabels = {
  en: "Added by",
  fr: "Ajouté par",
  es: "Añadido por",
  de: "Hinzugefügt von",
  ru: "Добавил(а)",
  zh: "添加者",
  ko: "추가한 사람",
  ja: "追加した人",
  id: "Ditambahkan oleh",
};
const fallbackAvatar = "/overlay-assets/relay-radar.png";
const queue = [];

let config = {
  ttsNotificationsObsEnabled: false,
  ttsQueueLimit: 50,
  notificationDurationMs: 8000,
  notificationSoundEnabled: false,
  notificationSoundObsEnabled: false,
  mediaVolume: 50,
  widgetSoundEnabled: false,
  notificationObsGeometry: {},
  notificationWidgetGeometry: {},
};
let pingElement;
let currentNotification;
let socket;
let reconnectTimer;
let reconnectDelayMs = 1000;
let pendingPort;
let isUnloading = false;
let playbackGeneration = 0;
let playbackWatchdog;
let visualTimer;
let musicPlaybackId = "";
let musicHideTimer;
let lastMusicPlayback;
/** Latest musicPlay payload waiting for an exclusive free stage (TTS/media). */
let pendingMusicPlayback;
let musicYoutubePlayer;
let musicYoutubePlayerElement = document.querySelector("#music-youtube-player");
let musicYoutubeReady = false;
let musicYoutubeApiPromise;
let musicYoutubePending;
let musicYoutubeGeneration = 0;
let musicYoutubeActiveGeneration = 0;
let musicYoutubeTeardownSilent = false;
let musicYoutubeHasStarted = false;
let musicTimePollTimer;
let musicEndSeconds = 0;
let reportedMusicMediaBusy = false;
/** True while overlay media or YouTube holds the shared stage. */
let mediaBusy = false;
/** True while server reports an active YouTube track (may be deferred locally on overlay). */
let musicActive = false;
/** True while this notification client (or a peer TTS lane) holds the stage. */
let ttsBusy = false;
let reportedTtsStageBusy = false;
let ttsStageClaimPending = false;
let lastStageClockPayload = {};

function notificationSocketUrl(
  host,
  client = outputClient,
  protocol = window.location.protocol === "https:" ? "wss:" : "ws:",
) {
  return `${protocol}//${host}/ws?role=notification&source=notification&client=${encodeURIComponent(client)}&secret=${encodeURIComponent(relaySecret)}`;
}

audioElement.muted = true;
document.documentElement.classList.toggle("notification-widget", target === "widget");

function isEnabled(notification) {
  return target === "widget"
    || Boolean(config.ttsNotificationsObsEnabled)
    || Boolean(notification?.relayTest);
}

function queueLimit() {
  return Math.min(50, Math.max(1, Number(config.ttsQueueLimit) || 50));
}

function displayDuration() {
  return Math.min(60000, Math.max(1000, Number(config.notificationDurationMs) || 8000));
}

function applyOutputGeometry() {
  const geometry = target === "widget"
    ? config.notificationWidgetGeometry
    : config.notificationObsGeometry;
  const crop = (value) => Math.min(40, Math.max(0, Number(value) || 0));
  const scale = Math.min(200, Math.max(50, Number(geometry?.contentScale) || 100));
  const rootStyle = document.documentElement.style;
  rootStyle.setProperty("--crop-top", `${crop(geometry?.cropTop)}%`);
  rootStyle.setProperty("--crop-right", `${crop(geometry?.cropRight)}%`);
  rootStyle.setProperty("--crop-bottom", `${crop(geometry?.cropBottom)}%`);
  rootStyle.setProperty("--crop-left", `${crop(geometry?.cropLeft)}%`);
  rootStyle.setProperty("--content-scale", String(scale / 100));
}

function audioUrl(ttsEvent) {
  return `/tts-audio/${encodeURIComponent(ttsEvent.id)}?secret=${encodeURIComponent(relaySecret)}`;
}

function setGuildTag(guildTag) {
  const name = typeof guildTag?.name === "string" ? guildTag.name.trim() : "";
  if (!name) {
    guildTagElement.hidden = true;
    guildTagNameElement.textContent = "";
    guildTagBadgeElement.hidden = true;
    guildTagBadgeElement.onerror = null;
    guildTagBadgeElement.removeAttribute("src");
    return;
  }

  guildTagNameElement.textContent = name;
  const badgeUrl = typeof guildTag.badgeUrl === "string" ? guildTag.badgeUrl : "";
  if (badgeUrl) {
    guildTagBadgeElement.onerror = () => {
      guildTagBadgeElement.onerror = null;
      guildTagBadgeElement.hidden = true;
    };
    guildTagBadgeElement.src = badgeUrl;
    guildTagBadgeElement.hidden = false;
  } else {
    guildTagBadgeElement.hidden = true;
    guildTagBadgeElement.onerror = null;
    guildTagBadgeElement.removeAttribute("src");
  }
  guildTagElement.hidden = false;
}

function setCardContent(notification) {
  authorElement.textContent = notification.author?.username || "Discord";
  setGuildTag(notification.guildTag);
  messageElement.replaceChildren();
  if (notification.visualOnly && Array.isArray(notification.segments)) {
    for (const segment of notification.segments) {
      if ((segment.kind === "emoji" || segment.kind === "sticker") && segment.url) {
        const image = document.createElement("img");
        image.className = segment.kind === "sticker"
          ? "notification-card__sticker"
          : "notification-card__emoji";
        image.src = segment.url;
        image.alt = segment.value || segment.kind;
        image.onerror = () => image.replaceWith(document.createTextNode(segment.value || ""));
        messageElement.append(image);
      } else {
        messageElement.append(document.createTextNode(segment.value || ""));
      }
    }
  } else {
    messageElement.textContent = notification.text || "";
  }
  avatarElement.onerror = () => {
    avatarElement.onerror = null;
    avatarElement.src = fallbackAvatar;
  };
  avatarElement.src = notification.author?.displayAvatarUrl || fallbackAvatar;
}

function showCard() {
  hideMusicNowPlaying();
  cardElement.classList.add("is-visible");
  cardElement.setAttribute("aria-hidden", "false");
}

function clearPendingMusicPlayback(playbackId) {
  if (!pendingMusicPlayback) return;
  if (
    playbackId
    && pendingMusicPlayback.playbackId
    && playbackId !== pendingMusicPlayback.playbackId
  ) {
    return;
  }
  pendingMusicPlayback = undefined;
}

function hasLegacyMusicActivity() {
  return Boolean(
    musicPlaybackId
    || lastMusicPlayback
    || pendingMusicPlayback
    || musicYoutubePlayer
    || musicYoutubePending
    || musicCardElement.classList.contains("is-visible")
  );
}

function hideMusicNowPlaying(playbackId) {
  if (!hasLegacyMusicActivity()) return;
  if (
    playbackId
    && musicPlaybackId
    && playbackId !== musicPlaybackId
  ) return;
  window.clearTimeout(musicHideTimer);
  stopMusicTimePoll();
  const endingId = musicPlaybackId;
  musicPlaybackId = "";
  lastMusicPlayback = undefined;
  musicEndSeconds = 0;
  // Local teardown must not report musicEnded (YouTube stop/destroy can fire onError).
  unloadMusicYoutubePlayer({ silent: true });
  syncMusicMediaBusy(false);
  musicCardElement.classList.remove("is-visible", "is-playing");
  musicCardElement.setAttribute("aria-hidden", "true");
  musicTitleElement.textContent = "";
  musicArtistElement.textContent = "";
  if (musicMetaElement) {
    musicMetaElement.textContent = "";
    musicMetaElement.hidden = true;
  }
  if (musicTimeElement) {
    musicTimeElement.textContent = "";
    musicTimeElement.hidden = true;
  }
  if (musicProgressFillElement) musicProgressFillElement.style.width = "0%";
  if (musicProgressElement) {
    musicProgressElement.hidden = true;
    musicProgressElement.setAttribute("aria-valuenow", "0");
  }
  if (musicArtworkElement) {
    musicArtworkElement.removeAttribute("src");
    musicArtworkElement.src = fallbackAvatar;
    musicArtworkElement.alt = "";
  }
  return endingId;
}

function formatMusicClock(seconds) {
  const total = Math.max(0, Math.floor(Number(seconds) || 0));
  return `${Math.floor(total / 60)}:${String(total % 60).padStart(2, "0")}`;
}

function musicEndClock(playback = {}) {
  const startSeconds = Math.max(0, Number(playback.startSeconds) || 0);
  const endSeconds = Number(playback.endSeconds);
  const durationSeconds = Math.max(0, Number(playback.durationSeconds) || 0);
  if (Number.isFinite(endSeconds) && endSeconds > startSeconds) return endSeconds;
  if (durationSeconds > 0) return durationSeconds;
  return 0;
}

function musicPlaybackRange(playback = {}, currentSeconds) {
  const end = musicEndClock(playback);
  if (end <= 0) return "";
  const startSeconds = Math.max(0, Number(playback.startSeconds) || 0);
  const current = Number.isFinite(Number(currentSeconds))
    ? Math.min(end, Math.max(startSeconds, Number(currentSeconds)))
    : startSeconds;
  return `${formatMusicClock(current)} → ${formatMusicClock(end)}`;
}

function normalizeInterfaceLanguage(value) {
  const primary = String(value || "en").split(/[-_]/)[0].toLowerCase();
  return Object.prototype.hasOwnProperty.call(musicLabels, primary) ? primary : "en";
}

function decodeBasicHtmlEntities(value) {
  return String(value ?? "")
    .replace(/&#x([0-9a-f]+);/gi, (match, hex) => {
      const code = Number.parseInt(hex, 16);
      try {
        return Number.isFinite(code) ? String.fromCodePoint(code) : match;
      } catch {
        return match;
      }
    })
    .replace(/&#(\d+);/g, (match, dec) => {
      const code = Number(dec);
      try {
        return Number.isFinite(code) ? String.fromCodePoint(code) : match;
      } catch {
        return match;
      }
    })
    .replace(/&quot;/g, "\"")
    .replace(/&apos;/g, "'")
    .replace(/&lt;/g, "<")
    .replace(/&gt;/g, ">")
    .replace(/&amp;/g, "&");
}

function fillMusicCardMeta(playback = {}) {
  const language = normalizeInterfaceLanguage(interfaceLanguage);
  musicLabelElement.textContent = musicLabels[language] || musicLabels.en;
  musicTitleElement.textContent = decodeBasicHtmlEntities(playback.title || "Relay").trim();
  musicArtistElement.textContent = decodeBasicHtmlEntities(playback.channelTitle || "YouTube").trim();
  const requester = decodeBasicHtmlEntities(playback.requestedBy || "").trim();
  const addedBy = musicAddedByLabels[language] || musicAddedByLabels.en;
  if (musicMetaElement) {
    musicMetaElement.textContent = requester ? `${addedBy} ${requester}` : "";
    musicMetaElement.hidden = !requester;
  }
  musicEndSeconds = musicEndClock(playback);
  updateMusicLiveTime(Number(playback.startSeconds) || 0);
  const thumbnail = typeof playback.thumbnail === "string" && /^https:\/\/i\.ytimg\.com\//i.test(playback.thumbnail)
    ? playback.thumbnail.trim()
    : "";
  musicArtworkElement.onerror = () => {
    musicArtworkElement.onerror = null;
    musicArtworkElement.src = fallbackAvatar;
  };
  musicArtworkElement.src = thumbnail || fallbackAvatar;
  musicArtworkElement.alt = musicTitleElement.textContent;
}

function updateMusicLiveTime(currentSeconds) {
  if (!musicTimeElement) return;
  const playback = lastMusicPlayback || {};
  const range = musicPlaybackRange(playback, currentSeconds);
  musicTimeElement.textContent = range;
  musicTimeElement.hidden = !range;
  const end = musicEndClock(playback);
  const startSeconds = Math.max(0, Number(playback.startSeconds) || 0);
  const span = Math.max(0, end - startSeconds);
  if (musicProgressElement && musicProgressFillElement && span > 0 && range) {
    const current = Number.isFinite(Number(currentSeconds))
      ? Math.min(end, Math.max(startSeconds, Number(currentSeconds)))
      : startSeconds;
    const percent = Math.min(100, Math.max(0, ((current - startSeconds) / span) * 100));
    musicProgressFillElement.style.width = `${percent}%`;
    musicProgressElement.hidden = false;
    musicProgressElement.setAttribute("aria-valuenow", String(Math.round(percent)));
  } else if (musicProgressElement && musicProgressFillElement) {
    musicProgressFillElement.style.width = "0%";
    musicProgressElement.hidden = true;
    musicProgressElement.setAttribute("aria-valuenow", "0");
  }
}

function stopMusicTimePoll() {
  window.clearTimeout(musicTimePollTimer);
  musicTimePollTimer = undefined;
}

function startMusicTimePoll() {
  stopMusicTimePoll();
  const tick = () => {
    let current;
    try {
      current = musicYoutubePlayer?.getCurrentTime?.();
    } catch {
      current = undefined;
    }
    if (Number.isFinite(current)) updateMusicLiveTime(current);
    musicTimePollTimer = window.setTimeout(tick, 250);
  };
  tick();
}

function syncMusicMediaBusy(busy) {
  // Music occupancy is authoritative via server musicBusy (MusicPlay/Idle).
  // Claiming lane:"media" while musicBusy=true is always rejected and races the
  // exclusive stage clock (flash-and-die on cold start). Keep the helper as a
  // no-op so hide/show paths stay call-compatible.
  if (!busy) reportedMusicMediaBusy = false;
}

function loadMusicYoutubeApi() {
  if (!musicYoutubePlayerElement) {
    return Promise.reject(new Error("YouTube player element is unavailable"));
  }
  if (window.YT?.Player) return Promise.resolve(window.YT);
  if (musicYoutubeApiPromise) return musicYoutubeApiPromise;

  musicYoutubeApiPromise = new Promise((resolve, reject) => {
    const previousReady = window.onYouTubeIframeAPIReady;
    window.onYouTubeIframeAPIReady = () => {
      try {
        if (typeof previousReady === "function") previousReady();
      } finally {
        if (window.YT?.Player) resolve(window.YT);
        else reject(new Error("YouTube player API did not initialize"));
      }
    };
    const script = document.createElement("script");
    script.src = "https://www.youtube.com/iframe_api";
    script.async = true;
    script.addEventListener("error", () => reject(new Error("YouTube player API failed to load")), { once: true });
    (document.head || document.documentElement).appendChild(script);
  });
  return musicYoutubeApiPromise;
}

function applyMusicYoutubeAudioSettings() {
  if (!musicYoutubePlayer) return;
  // WebView2 only guarantees autoplay while the iframe is muted. Keep it
  // muted through loadVideoById(), then apply the user's sound preference
  // after YouTube confirms PLAYING.
  const muted = !config.widgetSoundEnabled || !musicYoutubeHasStarted;
  if (muted) musicYoutubePlayer.mute?.();
  else musicYoutubePlayer.unMute?.();
  musicYoutubePlayer.setVolume?.(
    muted ? 0 : Math.min(100, Math.max(0, Number(config.mediaVolume) || 0)),
  );
}

function allowMusicYoutubeAutoplay() {
  const frame = musicYoutubePlayer?.getIframe?.();
  if (!frame?.setAttribute) return;
  frame.setAttribute("allow", "autoplay; encrypted-media; picture-in-picture");
  frame.setAttribute("referrerpolicy", "strict-origin-when-cross-origin");
}

function disableMusicYoutubeCaptions() {
  if (!musicYoutubePlayer) return;
  try {
    musicYoutubePlayer.unloadModule?.("captions");
  } catch {
    // Optional API; ignore when unavailable.
  }
  try {
    musicYoutubePlayer.setOption?.("captions", "track", {});
  } catch {
    // Optional API; ignore when unavailable.
  }
}

function resetMusicYoutubeHost() {
  const host = musicYoutubePlayerElement || document.querySelector("#music-youtube-player");
  if (host?.parentNode) {
    const next = document.createElement("div");
    next.id = "music-youtube-player";
    next.className = "music-card__player";
    next.setAttribute("aria-hidden", "true");
    host.parentNode.replaceChild(next, host);
    musicYoutubePlayerElement = next;
  } else if (host) {
    host.className = "music-card__player";
    host.setAttribute("aria-hidden", "true");
    if (typeof host.replaceChildren === "function") host.replaceChildren();
    else host.innerHTML = "";
    musicYoutubePlayerElement = host;
  }
  musicCardElement.classList.remove("is-playing");
}

function unloadMusicYoutubePlayer({ silent = false } = {}) {
  stopMusicTimePoll();
  musicYoutubeTeardownSilent = Boolean(silent);
  if (musicYoutubePlayer) {
    try {
      if (typeof musicYoutubePlayer.stopVideo === "function") musicYoutubePlayer.stopVideo();
    } catch {
      // Ignore stop failures during teardown.
    }
    try {
      const frame = musicYoutubePlayer.getIframe?.();
      if (frame) {
        try {
          frame.src = "about:blank";
        } catch {
          // Ignore navigation failures on a tearing-down frame.
        }
        frame.removeAttribute("src");
        frame.style.display = "none";
        frame.setAttribute("aria-hidden", "true");
      }
    } catch {
      // Optional API; continue with destroy.
    }
    try {
      if (typeof musicYoutubePlayer.destroy === "function") musicYoutubePlayer.destroy();
    } catch {
      // Ignore destroy failures; still clear local state below.
    }
  }
  musicYoutubePlayer = undefined;
  musicYoutubeReady = false;
  musicYoutubeHasStarted = false;
  musicYoutubePending = undefined;
  musicYoutubeTeardownSilent = false;
  resetMusicYoutubeHost();
}

function loadMusicYoutubePendingPlayback() {
  if (
    !musicYoutubeReady
    || !musicYoutubePlayer
    || !musicYoutubePending
    || musicYoutubePending.generation !== musicYoutubeGeneration
    || typeof musicYoutubePlayer.loadVideoById !== "function"
  ) return;
  const pending = musicYoutubePending;
  musicYoutubePending = undefined;
  musicYoutubeActiveGeneration = pending.generation;
  const options = {
    videoId: pending.videoId,
    startSeconds: Math.max(0, Number(pending.startSeconds) || 0),
  };
  if (Number.isFinite(Number(pending.endSeconds)) && Number(pending.endSeconds) > options.startSeconds) {
    options.endSeconds = Number(pending.endSeconds);
  }
  musicYoutubePlayer.loadVideoById(options);
  if (typeof musicYoutubePlayer.playVideo === "function") musicYoutubePlayer.playVideo();
  startMusicTimePoll();
}

function finishMusicYoutubePlayback(playbackId, generation, notifyServer = true) {
  if (!playbackId || playbackId !== musicPlaybackId || generation !== musicYoutubeGeneration) {
    return false;
  }
  if (musicYoutubeTeardownSilent) {
    notifyServer = false;
  }
  if (
    notifyServer
    && !isPreview
    && target === "widget"
    && socket?.readyState === 1
  ) {
    socket.send(JSON.stringify({ type: "musicEnded", payload: { playbackId } }));
  }
  hideMusicNowPlaying(playbackId);
  return true;
}

function createMusicYoutubePlayer() {
  if (!musicYoutubePlayerElement || musicYoutubePlayer || !window.YT?.Player) return;
  musicYoutubeReady = false;
  musicYoutubePlayerElement.style.display = "";
  musicYoutubePlayerElement.setAttribute("aria-hidden", "false");
  musicYoutubePlayer = new window.YT.Player(musicYoutubePlayerElement.id || "music-youtube-player", {
    host: "https://www.youtube-nocookie.com",
    width: "100%",
    height: "100%",
    playerVars: {
      autoplay: 1,
      cc_load_policy: 0,
      controls: 0,
      disablekb: 1,
      enablejsapi: 1,
      fs: 0,
      iv_load_policy: 3,
      modestbranding: 1,
      mute: 1,
      origin: window.location.origin,
      playsinline: 1,
      rel: 0,
    },
    events: {
      onReady: () => {
        musicYoutubeReady = true;
        allowMusicYoutubeAutoplay();
        disableMusicYoutubeCaptions();
        musicYoutubePlayer.mute?.();
        loadMusicYoutubePendingPlayback();
        applyMusicYoutubeAudioSettings();
      },
      onStateChange: (event) => {
        if (event.data === (window.YT?.PlayerState?.PLAYING ?? 1)) {
          musicYoutubeHasStarted = true;
          musicCardElement.classList.add("is-playing");
          allowMusicYoutubeAutoplay();
          disableMusicYoutubeCaptions();
          applyMusicYoutubeAudioSettings();
          return;
        }
        if (event.data === window.YT?.PlayerState?.ENDED) {
          finishMusicYoutubePlayback(musicPlaybackId, musicYoutubeActiveGeneration, true);
        }
      },
      onError: () => {
        // Match OBS overlay: embed failures (150, autoplay, cold-start) must not
        // clear server jukebox state or tear down the Now Playing chrome.
        musicYoutubeHasStarted = false;
        musicCardElement.classList.remove("is-playing");
      },
    },
  });
}

function beginMusicYoutube(playback = {}) {
  const videoId = typeof playback.videoId === "string" ? playback.videoId : "";
  const playbackId = typeof playback.playbackId === "string" ? playback.playbackId : "";
  if (!/^[A-Za-z0-9_-]{11}$/.test(videoId) || !playbackId) return;

  if (musicYoutubePlayer || musicYoutubePending) {
    unloadMusicYoutubePlayer({ silent: true });
  }

  const generation = ++musicYoutubeGeneration;
  musicYoutubeHasStarted = false;
  musicYoutubePending = { ...playback, videoId, playbackId, generation };
  syncMusicMediaBusy(true);
  loadMusicYoutubeApi()
    .then(() => {
      if (generation !== musicYoutubeGeneration || !musicYoutubePending) return;
      createMusicYoutubePlayer();
      loadMusicYoutubePendingPlayback();
    })
    .catch(() => {
      musicYoutubeApiPromise = undefined;
      // Keep the card + server playback; a later musicPlay/eval can retry the embed.
      musicYoutubePending = undefined;
      musicCardElement.classList.remove("is-playing");
    });
}

function musicUiBlocked() {
  // Overlay/media holds the stage only when this client is not already the music owner.
  const foreignMediaBusy = mediaBusy && !musicPlaybackId && !musicYoutubePending;
  return Boolean(
    currentNotification
    || ttsStageClaimPending
    || ttsBusy
    || foreignMediaBusy
  );
}

function flushPendingMusic() {
  if (!pendingMusicPlayback || musicUiBlocked()) return;
  const playback = pendingMusicPlayback;
  pendingMusicPlayback = undefined;
  setMusicActive(true);
  showMusicNowPlaying(playback);
}

function requestMusicNowPlaying(playback = {}) {
  if (target !== "widget" || !playback?.playbackId) return;
  pendingMusicPlayback = playback;
  if (musicUiBlocked()) return;
  flushPendingMusic();
}

function showMusicNowPlaying(playback = {}) {
  if (target !== "widget" || !playback.playbackId) return;
  window.clearTimeout(musicHideTimer);
  const playbackId = String(playback.playbackId);
  const resumeSame = musicPlaybackId === playbackId
    && Boolean(musicYoutubePlayer || musicYoutubePending);
  pendingMusicPlayback = undefined;
  lastMusicPlayback = playback;
  musicPlaybackId = playbackId;
  fillMusicCardMeta(playback);
  musicCardElement.classList.add("is-visible");
  musicCardElement.setAttribute("aria-hidden", "false");
  hideCard();
  syncMusicMediaBusy(true);

  if (!resumeSame) {
    beginMusicYoutube(playback);
  }

  const endSeconds = musicEndClock(playback);
  const startSeconds = Math.max(0, Number(playback.startSeconds) || 0);
  const durationSeconds = endSeconds > startSeconds ? endSeconds - startSeconds : endSeconds;
  if (durationSeconds > 0) {
    const visiblePlaybackId = musicPlaybackId;
    const hideGeneration = musicYoutubeGeneration;
    musicHideTimer = window.setTimeout(
      () => finishMusicYoutubePlayback(visiblePlaybackId, hideGeneration, true),
      Math.min(600000, Math.max(1000, (durationSeconds + 1) * 1000)),
    );
  }
}

function showPreview() {
  const copy = previewCopy[interfaceLanguage] || previewCopy.en;
  setCardContent({ author: { username: copy.author }, text: copy.message });
  showCard();
}

function playNotificationPing() {
  if (isPreview) {
    return;
  }
  const enabled = target === "widget"
    ? Boolean(config.notificationSoundEnabled)
    : Boolean(config.notificationSoundObsEnabled);
  if (!enabled) {
    return;
  }
  if (!pingElement) {
    if (typeof Audio !== "function") {
      return;
    }
    pingElement = new Audio();
  }
  pingElement.volume = Math.min(1, Math.max(0, (Number(config.mediaVolume) || 50) / 100));
  pingElement.src = `/notification-sound?secret=${encodeURIComponent(relaySecret)}`;
  pingElement.play().catch(() => {});
}

function hideCard() {
  cardElement.classList.remove("is-visible");
  cardElement.setAttribute("aria-hidden", "true");
}

function resetAudio() {
  playbackGeneration += 1;
  audioElement.onended = null;
  audioElement.onerror = null;
  audioElement.onplaying = null;
  audioElement.ontimeupdate = null;
  audioElement.onwaiting = null;
  audioElement.onstalled = null;
  audioElement.onabort = null;
  audioElement.onemptied = null;
  window.clearTimeout(playbackWatchdog);
  window.clearTimeout(visualTimer);
  audioElement.pause();
  audioElement.removeAttribute("src");
  audioElement.load();
}

function stageBlocked() {
  return mediaBusy || musicActive;
}

function syncTtsStageBusy(busy) {
  if (isPreview || socket?.readyState !== 1) return;
  if (busy === reportedTtsStageBusy) return;
  reportedTtsStageBusy = busy;
  if (!busy) ttsBusy = false;
  socket.send(JSON.stringify({
    type: "stageClock",
    payload: { lane: "tts", busy },
  }));
}

function applyStageClock(payload = {}) {
  lastStageClockPayload = payload && typeof payload === "object" ? payload : {};
  mediaBusy = Boolean(payload.mediaBusy);
  // Prefer server musicBusy when present so a missed musicIdle cannot strand
  // the OBS notification lane while /tts keeps speaking.
  if (Object.prototype.hasOwnProperty.call(payload, "musicBusy")) {
    musicActive = Boolean(payload.musicBusy);
  }
  ttsBusy = Boolean(payload.ttsBusy);
  resolveTtsStageClaim();
  if (!stageBlocked() && !currentNotification && !ttsStageClaimPending) playNext();
  flushPendingMusic();
}

function setMusicActive(active, { resume = false } = {}) {
  musicActive = Boolean(active);
  if (resume && !stageBlocked() && !ttsStageClaimPending) playNext();
  if (resume) flushPendingMusic();
}

function resolveTtsStageClaim() {
  if (!ttsStageClaimPending) return;
  if (
    (lastStageClockPayload.granted === false && lastStageClockPayload.lane === "tts")
    || mediaBusy
    || musicActive
  ) {
    ttsStageClaimPending = false;
    reportedTtsStageBusy = false;
    return;
  }
  if (!ttsBusy) return;
  ttsStageClaimPending = false;
  beginCurrentNotification();
}

function beginCurrentNotification() {
  if (currentNotification || queue.length === 0 || !isEnabled(queue[0])) {
    syncTtsStageBusy(false);
    return;
  }
  currentNotification = queue.shift();
  const generation = playbackGeneration;
  setCardContent(currentNotification);
  // Exclusive stage: never leave a music card visible under TTS.
  hideMusicNowPlaying();
  showCard();
  playNotificationPing();
  syncTtsStageBusy(true);
  if (currentNotification.visualOnly) {
    visualTimer = window.setTimeout(() => finishCurrent(generation), displayDuration());
    return;
  }
  const keepVisibleWithoutAudio = () => {
    if (!currentNotification || generation !== playbackGeneration) {
      return;
    }
    window.clearTimeout(playbackWatchdog);
    window.clearTimeout(visualTimer);
    visualTimer = window.setTimeout(() => finishCurrent(generation), displayDuration());
  };
  audioElement.onended = () => finishCurrent(generation);
  audioElement.onerror = keepVisibleWithoutAudio;
  const armWatchdog = (delay = 20000) => {
    window.clearTimeout(playbackWatchdog);
    playbackWatchdog = window.setTimeout(() => finishCurrent(generation), delay);
  };
  audioElement.onplaying = () => armWatchdog();
  audioElement.ontimeupdate = () => armWatchdog();
  audioElement.onwaiting = () => armWatchdog(15000);
  audioElement.onstalled = () => armWatchdog(15000);
  audioElement.onabort = keepVisibleWithoutAudio;
  audioElement.onemptied = keepVisibleWithoutAudio;
  audioElement.src = audioUrl(currentNotification);
  audioElement.load();
  armWatchdog(15000);
  audioElement.play().catch(keepVisibleWithoutAudio);
}

function finishCurrent(expectedGeneration = playbackGeneration) {
  if (!currentNotification || expectedGeneration !== playbackGeneration) {
    return;
  }
  resetAudio();
  currentNotification = undefined;
  hideCard();
  syncTtsStageBusy(false);
  playNext();
  flushPendingMusic();
}

function playNext() {
  if (
    currentNotification
    || queue.length === 0
    || !isEnabled(queue[0])
    || stageBlocked()
    || ttsStageClaimPending
  ) {
    return;
  }
  ttsStageClaimPending = true;
  syncTtsStageBusy(true);
}

function enqueue(notification) {
  if (!isEnabled(notification)) {
    return;
  }
  if (currentNotification?.visualOnly && !notification?.visualOnly) {
    resetAudio();
    currentNotification = undefined;
    syncTtsStageBusy(false);
    hideCard();
    queue.unshift(notification);
    if (queue.length > queueLimit()) {
      queue.length = queueLimit();
    }
    playNext();
    return;
  }
  if (queue.length >= queueLimit()) {
    return;
  }
  queue.push(notification);
  playNext();
}

function clearNotifications() {
  queue.length = 0;
  if (currentNotification) {
    resetAudio();
    currentNotification = undefined;
  }
  ttsStageClaimPending = false;
  syncTtsStageBusy(false);
  pendingMusicPlayback = undefined;
  hideCard();
  hideMusicNowPlaying();
}

function handleMessage(event) {
  let message;
  try {
    message = JSON.parse(event.data);
  } catch {
    return;
  }
  if (message.type === "config") {
    config = { ...config, ...message.payload };
    applyOutputGeometry();
    applyMusicYoutubeAudioSettings();
    const configuredPort = Number(message.payload?.port);
    if (
      Number.isInteger(configuredPort)
      && configuredPort > 0
      && configuredPort <= 65535
    ) {
      pendingPort = String(configuredPort) !== window.location.port
        ? configuredPort
        : undefined;
    }
    queue.length = Math.min(queue.length, queueLimit());
    if (!isEnabled()) {
      clearNotifications();
    }
  } else if (message.type === "tts") {
    if (isPreview) return;
    if (message.payload) enqueue(message.payload);
  } else if (message.type === "musicPlay") {
    if (isPreview) return;
    // YouTube renders in the media overlay. Notifications only track the
    // authoritative music occupancy so TTS waits on every output target.
    setMusicActive(true);
  } else if (message.type === "musicStop") {
    if (isPreview) return;
    clearPendingMusicPlayback(message.payload?.playbackId);
    hideMusicNowPlaying(message.payload?.playbackId);
  } else if (message.type === "musicIdle") {
    if (isPreview) return;
    pendingMusicPlayback = undefined;
    setMusicActive(false, { resume: true });
    hideMusicNowPlaying();
  } else if (message.type === "stageClock") {
    if (isPreview) return;
    applyStageClock(message.payload);
  } else if (message.type === "testOutput") {
    if (isPreview) return;
    const outputTest = message.payload;
    if (outputTest?.target === "notification" && outputTest.tts) {
      enqueue({ ...outputTest.tts, relayTest: true });
    }
  } else if (message.type === "skip") {
    if (isPreview) return;
    finishCurrent();
  } else if (message.type === "clear") {
    if (isPreview) return;
    clearNotifications();
  } else if (message.type === "serverMove") {
    const movedPort = Number(message.payload?.port);
    if (Number.isInteger(movedPort) && movedPort > 0 && movedPort <= 65535) {
      pendingPort = movedPort;
    }
  } else if (message.type === "appearance") {
    applyAppearance(message.payload);
  }
}

function applyAppearance(preferences = {}) {
  interfaceLanguage = normalizeInterfaceLanguage(preferences.language || interfaceLanguage);
  document.documentElement.lang = interfaceLanguage;
  document.documentElement.dataset.theme = preferences.theme || "dark";
  const rgb = Array.isArray(preferences.accentRgb) ? preferences.accentRgb : [88, 185, 137];
  document.documentElement.style.setProperty("--accent", `rgb(${rgb.join(" ")})`);
  document.documentElement.style.setProperty("--font-scale", String((preferences.fontScale || 100) / 100));
  if (moveLabelElement) moveLabelElement.textContent = moveLabels[interfaceLanguage] || moveLabels.en;
  if (musicLabelElement) musicLabelElement.textContent = musicLabels[interfaceLanguage] || musicLabels.en;
  if (musicPlaybackId && lastMusicPlayback) {
    showMusicNowPlaying(lastMusicPlayback);
  } else if (pendingMusicPlayback && !musicUiBlocked()) {
    flushPendingMusic();
  } else if (isPreview) {
    showPreview();
  }
}

applyAppearance({ language: interfaceLanguage, fontScale: 100, accentRgb: [88, 185, 137] });
applyOutputGeometry();

function scheduleReconnect() {
  if (isUnloading || reconnectTimer) {
    return;
  }
  reconnectTimer = window.setTimeout(() => {
    reconnectTimer = undefined;
    connect();
  }, reconnectDelayMs);
  reconnectDelayMs = Math.min(reconnectDelayMs * 2, 10000);
}

function moveToPendingPort() {
  const nextUrl = new URL(window.location.href);
  nextUrl.port = String(pendingPort);
  // Probe the moved server before navigating: OBS browser sources never
  // retry a failed page load, so a blind navigation can leave them dead.
  const probe = new WebSocket(
    notificationSocketUrl(`${window.location.hostname}:${pendingPort}`, "probe", "ws:"),
  );
  let ready = false;
  const probeWatchdog = window.setTimeout(() => {
    if (!ready) probe.close();
  }, 5000);
  probe.addEventListener("open", () => {
    ready = true;
    window.clearTimeout(probeWatchdog);
    probe.close();
    window.location.replace(nextUrl);
  });
  probe.addEventListener("close", () => {
    window.clearTimeout(probeWatchdog);
    if (!ready && !isUnloading) {
      window.setTimeout(moveToPendingPort, 1000);
    }
  });
}

function connect() {
  socket = new WebSocket(notificationSocketUrl(window.location.host));
  socket.addEventListener("open", () => {
    reconnectDelayMs = 1000;
    playNext();
  });
  socket.addEventListener("message", handleMessage);
  socket.addEventListener("close", () => {
    // Stop the current notification but keep the queue for after the reconnect.
    if (currentNotification) {
      resetAudio();
      currentNotification = undefined;
    }
    reportedTtsStageBusy = false;
    ttsStageClaimPending = false;
    ttsBusy = false;
    reportedMusicMediaBusy = false;
    mediaBusy = false;
    // Do not clear musicActive / tear down YouTube on a transient WS drop —
    // unload+onError previously reported musicEnded and killed cold-start play.
    if (!isPreview) {
      hideCard();
    }
    if (pendingPort) {
      moveToPendingPort();
      return;
    }
    scheduleReconnect();
  });
  socket.addEventListener("error", () => socket.close());
}

window.setWidgetLocked = (locked) => {
  document.documentElement.classList.toggle("widget-edit", !locked);
};

if (target === "widget") {
  const locked = parameters.get("locked") === "1";
  window.setWidgetLocked(locked);
  const moveLayer = document.querySelector("#notification-move-layer");
  moveLayer?.addEventListener("pointerdown", (event) => {
    if (event.button !== 0) {
      return;
    }
    const tauri = window.__TAURI__;
    const current =
      tauri?.webviewWindow?.getCurrentWebviewWindow?.() ||
      tauri?.window?.getCurrentWindow?.();
    current?.startDragging?.().catch(() => {});
  });
}

window.addEventListener("beforeunload", () => {
  isUnloading = true;
  window.clearTimeout(reconnectTimer);
  window.clearTimeout(playbackWatchdog);
  window.clearTimeout(musicHideTimer);
  stopMusicTimePoll();
  if (hasLegacyMusicActivity()) unloadMusicYoutubePlayer();
  socket?.close();
});

connect();
}());
