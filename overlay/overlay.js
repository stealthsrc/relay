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
const mediaTextElement = document.querySelector("#media-text");
const widgetParameters = new URLSearchParams(window.location.search);
const relaySecret = widgetParameters.get("secret") || "";
const isWidgetWindow = widgetParameters.get("widget") === "1";
const isPreview = widgetParameters.get("preview") === "1";
let interfaceLanguage = widgetParameters.get("lang") || "en";
const relayMode = document.querySelector('meta[name="relay-mode"]')?.content || "all";
const outputClient = isPreview ? "preview" : isWidgetWindow ? "widget" : "obs";
const coordinatesSplitOutputs = outputClient === "obs"
  && (relayMode === "visual" || relayMode === "audio");
const moveLabelElement = document.querySelector("#widget-move-label");
const moveLabels = { en: "Move overlay", fr: "Déplacer l’overlay", es: "Mover overlay", de: "Overlay verschieben" };
const previewLabels = { en: "Live preview", fr: "Aperçu en direct", es: "Vista previa", de: "Live-Vorschau" };
const previewCaptionLabels = {
  en: "Discord message shown with the media",
  fr: "Message Discord affiché avec le média",
  es: "Mensaje de Discord mostrado con el medio",
  de: "Discord-Nachricht zum Medium",
};

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
  showMediaTextObs: false,
  showMediaTextWidget: false,
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
let widgetPlaybackVisible = true;
let lastAudioPlaybackReport = "";
let mediaClockReady = !coordinatesSplitOutputs;
let videoOutputBusy = false;
let audioOutputBusy = false;
let outputLeaseHeld = false;
let outputLeasePending = false;
let waitingForOutputLease = false;

function outputSocketUrl(
  host,
  client = outputClient,
  protocol = window.location.protocol === "https:" ? "wss:" : "ws:",
) {
  return `${protocol}//${host}/ws?role=overlay&source=${encodeURIComponent(relayMode)}&client=${encodeURIComponent(client)}&secret=${encodeURIComponent(relaySecret)}`;
}

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

function sendOutputLeaseState(busy) {
  if (
    !coordinatesSplitOutputs
    || socket?.readyState !== 1
  ) return;
  socket.send(JSON.stringify({ type: "mediaClock", payload: { busy } }));
}

function isCoordinatedMedia(media) {
  return (relayMode === "visual" && media?.kind === "video")
    || (relayMode === "audio" && media?.kind === "audio");
}

function opposingOutputBusy() {
  return relayMode === "visual" ? audioOutputBusy : videoOutputBusy;
}

function requestOutputLease() {
  if (
    !waitingForOutputLease
    || outputLeaseHeld
    || outputLeasePending
    || !mediaClockReady
    || opposingOutputBusy()
    || socket?.readyState !== 1
  ) return;
  outputLeasePending = true;
  sendOutputLeaseState(true);
}

function releaseOutputLease() {
  const shouldNotifyServer = outputLeaseHeld || outputLeasePending;
  outputLeaseHeld = false;
  outputLeasePending = false;
  if (shouldNotifyServer) sendOutputLeaseState(false);
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

function setMediaText(media) {
  const enabled = isWidgetWindow ? config.showMediaTextWidget : config.showMediaTextObs;
  const text = enabled && typeof media?.text === "string" ? media.text.trim() : "";
  mediaTextElement.textContent = text;
  mediaTextElement.hidden = !text;
  mediaTextElement.classList.toggle(
    "is-visible",
    Boolean(text) && (isPreview || activeVisual?.classList.contains("is-visible")),
  );
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
  setMediaText({
    text: previewCaptionLabels[interfaceLanguage] || previewCaptionLabels.en,
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
  mediaTextElement.classList.remove("is-visible");
  mediaTextElement.hidden = true;
  mediaTextElement.textContent = "";
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
  if (!mediaTextElement.hidden) {
    mediaTextElement.classList.add("is-visible");
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
  const finishedMedia = currentMedia;
  resetElements();
  currentMedia = undefined;
  waitingForOutputLease = false;
  showNextMedia();
  if (isCoordinatedMedia(finishedMedia) && !isCoordinatedMedia(currentMedia)) {
    releaseOutputLease();
  }
  if (finishedMedia.kind === "audio" && currentMedia?.kind !== "audio") {
    reportAudioPlayback("idle", finishedMedia);
  }
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
  mediaTextElement.classList.remove("is-visible");
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

function startCurrentMedia(generation = playbackGeneration) {
  if (!currentMedia || generation !== playbackGeneration) return;
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

function showNextMedia() {
  if ((isWidgetWindow && !widgetPlaybackVisible) || currentMedia || queue.length === 0 || !mediaClockReady) return;
  currentMedia = queue.shift();
  const generation = ++playbackGeneration;
  setAuthor(currentMedia);
  setMediaText(currentMedia);
  if (isCoordinatedMedia(currentMedia) && !outputLeaseHeld) {
    waitingForOutputLease = true;
    requestOutputLease();
    return;
  }
  startCurrentMedia(generation);
}

function clearOverlay() {
  queue.length = 0;
  finishCurrentMedia();
  waitingForOutputLease = false;
  releaseOutputLease();
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
    waitingForOutputLease = false;
    releaseOutputLease();
    return;
  }
  const interruptedMedia = currentMedia;
  reportAudioPlayback("idle", interruptedMedia);
  waitingForOutputLease = false;
  if (isCoordinatedMedia(interruptedMedia)) releaseOutputLease();
  playbackGeneration += 1;
  resetElements();
  currentMedia = undefined;
}

function controlAudio(control = {}) {
  if (control.action === "previous" && control.media?.kind === "audio") {
    queue.unshift(control.media);
    if (currentMedia?.kind === "audio") finishCurrentMedia();
    else showNextMedia();
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

function applyMediaClock(payload = {}) {
  mediaClockReady = true;
  videoOutputBusy = Boolean(payload.videoBusy);
  audioOutputBusy = Boolean(payload.audioBusy);
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
    else setMediaText(currentMedia);
  } else if (message.type === "media") {
    if (isPreview) return;
    if (message.payload) enqueueMedia(message.payload);
  } else if (message.type === "testOutput") {
    if (isPreview) return;
    const outputTest = message.payload;
    if (
      (outputTest?.target === "visual" || outputTest?.target === "audio")
      && outputTest.media
    ) {
      enqueueMedia(outputTest.media);
    }
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
  } else if (message.type === "mediaClock") {
    applyMediaClock(message.payload);
    showNextMedia();
    requestOutputLease();
  } else if (message.type === "mediaGrant") {
    outputLeasePending = false;
    if (message.payload?.clock) applyMediaClock(message.payload.clock);
    if (message.payload?.granted && waitingForOutputLease && isCoordinatedMedia(currentMedia)) {
      outputLeaseHeld = true;
      waitingForOutputLease = false;
      startCurrentMedia();
    } else if (message.payload?.granted) {
      outputLeaseHeld = true;
      releaseOutputLease();
    } else {
      requestOutputLease();
    }
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
  if (isUnloading || reconnectTimer || (isWidgetWindow && !widgetPlaybackVisible)) {
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
    outputSocketUrl(`${window.location.hostname}:${pendingPort}`, "probe", "ws:"),
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
  const nextSocket = new WebSocket(outputSocketUrl(window.location.host));
  socket = nextSocket;
  nextSocket.addEventListener("open", () => {
    reconnectDelayMs = 1000;
    showNextMedia();
  });
  nextSocket.addEventListener("message", handleMessage);
  nextSocket.addEventListener("close", () => {
    if (socket !== nextSocket) return;
    if (isWidgetWindow && !widgetPlaybackVisible) return;
    // Stop the current playback but keep the queue: the server does not
    // rebroadcast queued media after a transient reconnect.
    interruptPlayback();
    outputLeaseHeld = false;
    outputLeasePending = false;
    waitingForOutputLease = false;
    mediaClockReady = !coordinatesSplitOutputs;
    if (pendingPort) {
      moveToPendingPort();
      return;
    }
    scheduleReconnect();
  });
  nextSocket.addEventListener("error", () => nextSocket.close());
}

window.setWidgetVisible = (visible) => {
  if (!isWidgetWindow || widgetPlaybackVisible === visible) return;
  widgetPlaybackVisible = visible;
  if (!visible) {
    reportAudioPlayback("idle");
    playbackGeneration += 1;
    resetElements();
    window.clearTimeout(reconnectTimer);
    reconnectTimer = undefined;
    socket?.close();
    return;
  }
  if (!socket || socket.readyState >= 2) connect();
  if (currentMedia) {
    setAuthor(currentMedia);
    setMediaText(currentMedia);
    startCurrentMedia();
  } else {
    showNextMedia();
  }
};

window.addEventListener("beforeunload", () => {
  isUnloading = true;
  clearOverlay();
  window.clearTimeout(reconnectTimer);
  socket?.close();
});

connect();
