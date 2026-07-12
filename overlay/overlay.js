const imageElement = document.querySelector("#image");
const videoElement = document.querySelector("#video");
const audioElement = document.querySelector("#audio");
const audioCardElement = document.querySelector("#audio-card");
const audioArtworkElement = document.querySelector("#audio-artwork");
const audioTitleElement = document.querySelector("#audio-title");
const audioArtistElement = document.querySelector("#audio-artist");
const authorElement = document.querySelector("#author");
const authorAvatarElement = document.querySelector("#author-avatar");
const authorNameElement = document.querySelector("#author-name");
const widgetParameters = new URLSearchParams(window.location.search);
const relaySecret = widgetParameters.get("secret") || "";
const isWidgetWindow = widgetParameters.get("widget") === "1";
const isPreview = widgetParameters.get("preview") === "1";
let interfaceLanguage = widgetParameters.get("lang") || "en";
const relayMode = document.querySelector('meta[name="relay-mode"]')?.content || "all";
const moveLabelElement = document.querySelector("#widget-move-label");
const moveLabels = { en: "Move overlay", fr: "Déplacer l’overlay", es: "Mover overlay", de: "Overlay verschieben" };
const previewLabels = { en: "Live preview", fr: "Aperçu en direct", es: "Vista previa", de: "Live-Vorschau" };

window.setWidgetLocked = (locked) => {
  document.documentElement.classList.toggle("widget-window", isWidgetWindow);
  document.documentElement.classList.toggle("widget-edit", isWidgetWindow && !locked);
};

window.setWidgetLocked(widgetParameters.get("locked") === "1");

const FADE_DURATION_MS = 320;
const FALLBACK_AVATAR = "/overlay-assets/relay-radar.png";
const queue = [];

let config = {
  displayDurationMs: 8000,
  gifDurationMs: 8000,
  mediaVolume: 50,
  showAuthor: true,
  widgetSoundEnabled: false,
  mediaObsGeometry: {},
  mediaWidgetGeometry: {},
};
let currentMedia;
let activeVisual;
let activePlayback;
let clearLoadListeners = () => {};
let displayTimer;
let fadeTimer;
let mediaWatchdog;
let playbackGeneration = 0;
let reconnectTimer;
let reconnectDelayMs = 1000;
let socket;
let pendingPort;
let isUnloading = false;
let lastAudioPlaybackReport = "";

function reportAudioPlayback(status, media = currentMedia) {
  if (isPreview || media?.kind !== "audio" || socket?.readyState !== 1) return;
  const reportKey = `${status}:${media.url}`;
  if (reportKey === lastAudioPlaybackReport) return;
  socket.send(JSON.stringify({
    type: "audioPlayback",
    payload: { status, target: isWidgetWindow ? "widget" : "obs", media },
  }));
  lastAudioPlaybackReport = reportKey;
}

function mediaSources(media) {
  if (media.cachedMediaId) {
    return [`/media-cache/${encodeURIComponent(media.cachedMediaId)}?secret=${encodeURIComponent(relaySecret)}`];
  }
  if (media.kind === "audio" && media.audioId) {
    return [`/media-audio/${encodeURIComponent(media.audioId)}?secret=${encodeURIComponent(relaySecret)}`];
  }
  return [media.url, media.proxyUrl].filter(
    (source, index, sources) => source && sources.indexOf(source) === index,
  );
}

function setAudioArtwork(media) {
  audioArtworkElement.onerror = () => {
    audioArtworkElement.onerror = null;
    audioArtworkElement.src = FALLBACK_AVATAR;
  };
  audioArtworkElement.src = media.artworkId
    ? `/media-artwork/${encodeURIComponent(media.artworkId)}?secret=${encodeURIComponent(relaySecret)}`
    : FALLBACK_AVATAR;
}

function setAuthor(media) {
  if (config.showAuthor && media.author) {
    authorAvatarElement.onerror = () => {
      authorAvatarElement.onerror = null;
      authorAvatarElement.src = FALLBACK_AVATAR;
    };
    authorAvatarElement.src = media.author.displayAvatarUrl || FALLBACK_AVATAR;
    authorNameElement.textContent = media.author.username;
    authorElement.hidden = false;
  } else {
    authorElement.hidden = true;
  }
}

function showPreview() {
  imageElement.src = FALLBACK_AVATAR;
  imageElement.alt = "Relay preview";
  imageElement.style.width = "auto";
  imageElement.style.height = "100%";
  imageElement.classList.add("is-visible");
  setAuthor({
    author: {
      username: previewLabels[interfaceLanguage] || previewLabels.en,
      displayAvatarUrl: FALLBACK_AVATAR,
    },
  });
  if (!authorElement.hidden) authorElement.classList.add("is-visible");
}

function fitVisualToViewport(element, width, height) {
  if (!width || !height || !window.innerWidth || !window.innerHeight) return;
  const fitByWidth = width / height >= window.innerWidth / window.innerHeight;
  element.style.width = fitByWidth ? "100%" : "auto";
  element.style.height = fitByWidth ? "auto" : "100%";
}

function applyOutputGeometry() {
  const geometry = isWidgetWindow ? config.mediaWidgetGeometry : config.mediaObsGeometry;
  const crop = (value) => Math.min(40, Math.max(0, Number(value) || 0));
  const scale = Math.min(200, Math.max(50, Number(geometry?.contentScale) || 100));
  const rootStyle = document.documentElement.style;
  rootStyle.setProperty("--crop-top", `${crop(geometry?.cropTop)}%`);
  rootStyle.setProperty("--crop-right", `${crop(geometry?.cropRight)}%`);
  rootStyle.setProperty("--crop-bottom", `${crop(geometry?.cropBottom)}%`);
  rootStyle.setProperty("--crop-left", `${crop(geometry?.cropLeft)}%`);
  rootStyle.setProperty("--content-scale", String(scale / 100));
}

function resetElements() {
  clearLoadListeners();
  clearLoadListeners = () => {};
  window.clearTimeout(displayTimer);
  window.clearTimeout(fadeTimer);
  window.clearTimeout(mediaWatchdog);

  for (const element of [imageElement, videoElement]) {
    element.classList.remove("is-visible", "is-hiding");
    element.removeAttribute("src");
    element.style.width = "";
    element.style.height = "";
  }
  for (const element of [videoElement, audioElement]) {
    element.pause();
    element.removeAttribute("src");
    element.load();
  }
  videoElement.loop = false;
  audioCardElement.classList.remove("is-visible", "is-hiding");
  audioCardElement.hidden = true;
  audioTitleElement.textContent = "";
  audioArtistElement.textContent = "";
  audioArtistElement.hidden = true;
  setAudioArtwork({});
  authorElement.classList.remove("is-visible");
  authorElement.hidden = true;
  authorAvatarElement.removeAttribute("src");
  activeVisual = undefined;
  activePlayback = undefined;
}

function revealMedia({ timed = false } = {}) {
  if (!currentMedia || !activeVisual) {
    return;
  }
  activeVisual.classList.add("is-visible");
  if (!authorElement.hidden) {
    authorElement.classList.add("is-visible");
  }
  if (timed) {
    const durationMs = currentMedia.kind === "gif"
      ? config.gifDurationMs
      : config.displayDurationMs;
    displayTimer = window.setTimeout(hideCurrentMedia, durationMs);
  }
}

function finishCurrentMedia(expectedGeneration = playbackGeneration) {
  if (!currentMedia || expectedGeneration !== playbackGeneration) return;
  reportAudioPlayback("idle");
  resetElements();
  currentMedia = undefined;
  showNextMedia();
}

function hideCurrentMedia(expectedGeneration = playbackGeneration) {
  if (!currentMedia || expectedGeneration !== playbackGeneration) {
    return;
  }
  window.clearTimeout(displayTimer);
  activePlayback?.pause();
  activeVisual?.classList.remove("is-visible");
  activeVisual?.classList.add("is-hiding");
  authorElement.classList.remove("is-visible");
  fadeTimer = window.setTimeout(() => finishCurrentMedia(expectedGeneration), FADE_DURATION_MS);
}

function skipCurrentMedia() {
  if (currentMedia) {
    finishCurrentMedia();
  } else {
    showNextMedia();
  }
}

function loadImage(media, generation) {
  activeVisual = imageElement;
  const sources = mediaSources(media);
  let sourceIndex = 0;
  let revealed = false;

  const clear = () => {
    imageElement.removeEventListener("load", onLoad);
    imageElement.removeEventListener("error", onError);
  };
  const onLoad = () => {
    if (revealed || generation !== playbackGeneration) {
      return;
    }
    revealed = true;
    window.clearTimeout(mediaWatchdog);
    fitVisualToViewport(imageElement, imageElement.naturalWidth, imageElement.naturalHeight);
    clear();
    revealMedia({ timed: true });
  };
  const onError = () => {
    if (generation !== playbackGeneration) return;
    clear();
    sourceIndex += 1;
    if (sourceIndex < sources.length) {
      trySource();
    } else {
      hideCurrentMedia(generation);
    }
  };
  const trySource = () => {
    imageElement.addEventListener("load", onLoad, { once: true });
    imageElement.addEventListener("error", onError, { once: true });
    imageElement.src = sources[sourceIndex];
  };

  clearLoadListeners = clear;
  mediaWatchdog = window.setTimeout(() => hideCurrentMedia(generation), 15000);
  if (sources.length === 0) {
    hideCurrentMedia();
    return;
  }
  trySource();
  if (imageElement.complete && imageElement.naturalWidth > 0) {
    onLoad();
  }
}

function loadPlayback(media, playbackElement, visualElement, generation) {
  activePlayback = playbackElement;
  activeVisual = visualElement;
  const isTimedGif = media.kind === "gif";
  const mustMute = (isWidgetWindow && !config.widgetSoundEnabled) || isTimedGif;
  playbackElement.loop = isTimedGif;
  playbackElement.muted = mustMute;
  playbackElement.volume = mustMute
    ? 0
    : Math.min(1, Math.max(0, config.mediaVolume / 100));
  const sources = mediaSources(media);
  let sourceIndex = 0;

  const clearSourceListeners = () => {
    playbackElement.removeEventListener("loadeddata", onLoaded);
    playbackElement.removeEventListener("error", onError);
  };
  const clear = () => {
    clearSourceListeners();
    playbackElement.removeEventListener("ended", onEnded);
    for (const eventName of ["playing", "timeupdate", "waiting", "stalled", "abort", "emptied"]) {
      playbackElement.removeEventListener(eventName, playbackStateChanged);
    }
  };
  const armWatchdog = (delay = 20000) => {
    window.clearTimeout(mediaWatchdog);
    mediaWatchdog = window.setTimeout(() => hideCurrentMedia(generation), delay);
  };
  const playbackStateChanged = (event) => {
    if (generation !== playbackGeneration) return;
    if (event.type === "playing" || event.type === "timeupdate") {
      armWatchdog(20000);
      reportAudioPlayback("playing");
    }
    else armWatchdog(15000);
  };
  const onEnded = () => {
    if (!isTimedGif) hideCurrentMedia(generation);
  };
  const onLoaded = () => {
    if (generation !== playbackGeneration) return;
    if (visualElement === videoElement) {
      fitVisualToViewport(videoElement, videoElement.videoWidth, videoElement.videoHeight);
    }
    clearSourceListeners();
    revealMedia({ timed: isTimedGif });
    armWatchdog();
    playbackElement.play().catch(() => hideCurrentMedia(generation));
  };
  const onError = () => {
    clearSourceListeners();
    sourceIndex += 1;
    if (sourceIndex < sources.length) {
      trySource();
    } else {
      hideCurrentMedia(generation);
    }
  };
  const trySource = () => {
    playbackElement.addEventListener("loadeddata", onLoaded, { once: true });
    playbackElement.addEventListener("error", onError, { once: true });
    playbackElement.src = sources[sourceIndex];
    playbackElement.load();
  };

  playbackElement.addEventListener("ended", onEnded, { once: true });
  for (const eventName of ["playing", "timeupdate", "waiting", "stalled", "abort", "emptied"]) {
    playbackElement.addEventListener(eventName, playbackStateChanged);
  }
  clearLoadListeners = clear;
  armWatchdog(15000);
  if (sources.length === 0) {
    hideCurrentMedia();
    return;
  }
  trySource();
}

function showNextMedia() {
  if (currentMedia || queue.length === 0) {
    return;
  }
  currentMedia = queue.shift();
  const generation = ++playbackGeneration;
  setAuthor(currentMedia);

  if (
    currentMedia.kind === "video"
    || (currentMedia.kind === "gif" && currentMedia.contentType?.startsWith("video/"))
  ) {
    loadPlayback(currentMedia, videoElement, videoElement, generation);
  } else if (currentMedia.kind === "audio") {
    audioTitleElement.textContent = currentMedia.title || currentMedia.filename || "Discord audio";
    audioArtistElement.textContent = currentMedia.artist || "";
    audioArtistElement.hidden = !currentMedia.artist;
    setAudioArtwork(currentMedia);
    audioCardElement.hidden = false;
    loadPlayback(currentMedia, audioElement, audioCardElement, generation);
  } else {
    loadImage(currentMedia, generation);
  }
}

function clearOverlay() {
  queue.length = 0;
  finishCurrentMedia();
}

const MEDIA_QUEUE_LIMIT = 50;

function enqueueMedia(mediaEvent) {
  if (
    (relayMode === "visual" && mediaEvent.kind === "audio")
    || (relayMode === "audio" && mediaEvent.kind !== "audio")
    || queue.length >= MEDIA_QUEUE_LIMIT
  ) {
    return;
  }
  queue.push(mediaEvent);
  showNextMedia();
}

function interruptPlayback() {
  if (!currentMedia) {
    return;
  }
  reportAudioPlayback("idle");
  playbackGeneration += 1;
  resetElements();
  currentMedia = undefined;
}

function controlAudio(control = {}) {
  if (control.action === "previous" && control.media?.kind === "audio") {
    if (currentMedia?.kind === "audio") interruptPlayback();
    queue.unshift(control.media);
    showNextMedia();
    return;
  }
  if (currentMedia?.kind !== "audio") return;
  if (control.action === "pause") {
    window.clearTimeout(mediaWatchdog);
    activePlayback?.pause();
    reportAudioPlayback("paused");
  } else if (control.action === "resume") {
    activePlayback?.play().then(() => reportAudioPlayback("playing")).catch(() => hideCurrentMedia());
  } else if (control.action === "skip") {
    finishCurrentMedia();
  }
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
    const volume = Math.min(1, Math.max(0, config.mediaVolume / 100));
    const widgetMuted = isWidgetWindow && !config.widgetSoundEnabled;
    videoElement.muted = widgetMuted || currentMedia?.kind === "gif";
    audioElement.muted = widgetMuted;
    videoElement.volume = videoElement.muted ? 0 : volume;
    audioElement.volume = audioElement.muted ? 0 : volume;
    if (!config.showAuthor) {
      authorElement.classList.remove("is-visible");
      authorElement.hidden = true;
    }
    if (isPreview) showPreview();
  } else if (message.type === "media") {
    if (isPreview) return;
    if (message.payload) enqueueMedia(message.payload);
  } else if (message.type === "image") {
    if (isPreview) return;
    if (message.payload) enqueueMedia({ kind: "image", ...message.payload });
  } else if (message.type === "skip") {
    if (isPreview) return;
    skipCurrentMedia();
  } else if (message.type === "audioControl") {
    if (isPreview) return;
    controlAudio(message.payload);
  } else if (message.type === "clear") {
    if (isPreview) return;
    clearOverlay();
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
  interfaceLanguage = ["en", "fr", "es", "de"].includes(preferences.language)
    ? preferences.language : interfaceLanguage;
  document.documentElement.lang = interfaceLanguage;
  document.documentElement.dataset.theme = preferences.theme || "dark";
  const rgb = Array.isArray(preferences.accentRgb) ? preferences.accentRgb : [88, 185, 137];
  document.documentElement.style.setProperty("--accent", `rgb(${rgb.join(" ")})`);
  document.documentElement.style.setProperty("--font-scale", String((preferences.fontScale || 100) / 100));
  if (moveLabelElement) moveLabelElement.textContent = moveLabels[interfaceLanguage] || moveLabels.en;
  if (isPreview) showPreview();
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
    `ws://${window.location.hostname}:${pendingPort}/ws?role=overlay&secret=${encodeURIComponent(relaySecret)}`,
  );
  let ready = false;
  probe.addEventListener("open", () => {
    ready = true;
    probe.close();
    window.location.replace(nextUrl);
  });
  probe.addEventListener("close", () => {
    if (!ready && !isUnloading) {
      window.setTimeout(moveToPendingPort, 1000);
    }
  });
}

function connect() {
  const protocol = window.location.protocol === "https:" ? "wss:" : "ws:";
  socket = new WebSocket(
    `${protocol}//${window.location.host}/ws?role=overlay&secret=${encodeURIComponent(relaySecret)}`,
  );
  socket.addEventListener("open", () => {
    reconnectDelayMs = 1000;
    showNextMedia();
  });
  socket.addEventListener("message", handleMessage);
  socket.addEventListener("close", () => {
    // Stop the current playback but keep the queue: the server does not
    // rebroadcast queued media after a transient reconnect.
    interruptPlayback();
    if (pendingPort) {
      moveToPendingPort();
      return;
    }
    scheduleReconnect();
  });
  socket.addEventListener("error", () => socket.close());
}

window.addEventListener("beforeunload", () => {
  isUnloading = true;
  clearOverlay();
  window.clearTimeout(reconnectTimer);
  socket?.close();
});

connect();
