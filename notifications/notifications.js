const cardElement = document.querySelector("#notification");
const avatarElement = document.querySelector("#notification-avatar");
const authorElement = document.querySelector("#notification-author");
const messageElement = document.querySelector("#notification-message");
const audioElement = document.querySelector("#notification-clock");
const parameters = new URLSearchParams(window.location.search);
const relaySecret = parameters.get("secret") || "";
const target = parameters.get("target") === "widget" ? "widget" : "obs";
const isPreview = parameters.get("preview") === "1";
let interfaceLanguage = parameters.get("lang") || "en";
const moveLabelElement = document.querySelector("#notification-move-label");
const moveLabels = { en: "Move notification", fr: "Déplacer la notification", es: "Mover notificación", de: "Benachrichtigung verschieben" };
const previewCopy = {
  en: { author: "Live preview", message: "Your notification will appear here." },
  fr: { author: "Aperçu en direct", message: "Votre notification apparaîtra ici." },
  es: { author: "Vista previa", message: "Tu notificación aparecerá aquí." },
  de: { author: "Live-Vorschau", message: "Deine Benachrichtigung erscheint hier." },
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

audioElement.muted = true;
document.documentElement.classList.toggle("notification-widget", target === "widget");

function isEnabled() {
  return target === "widget" || Boolean(config.ttsNotificationsObsEnabled);
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

function setCardContent(notification) {
  authorElement.textContent = notification.author?.username || "Discord";
  messageElement.replaceChildren();
  if (notification.visualOnly && Array.isArray(notification.segments)) {
    for (const segment of notification.segments) {
      if (segment.kind === "emoji" && segment.url) {
        const image = document.createElement("img");
        image.className = "notification-card__emoji";
        image.src = segment.url;
        image.alt = segment.value || "emoji";
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
  cardElement.classList.add("is-visible");
  cardElement.setAttribute("aria-hidden", "false");
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

function finishCurrent(expectedGeneration = playbackGeneration) {
  if (!currentNotification || expectedGeneration !== playbackGeneration) {
    return;
  }
  resetAudio();
  currentNotification = undefined;
  if (queue.length === 0) {
    hideCard();
  }
  playNext();
}

function playNext() {
  if (!isEnabled() || currentNotification || queue.length === 0) {
    return;
  }
  currentNotification = queue.shift();
  const generation = playbackGeneration;
  setCardContent(currentNotification);
  showCard();
  playNotificationPing();
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

function enqueue(notification) {
  if (!isEnabled()) {
    return;
  }
  if (currentNotification?.visualOnly && !notification?.visualOnly) {
    resetAudio();
    currentNotification = undefined;
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
  hideCard();
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
    queue.length = Math.min(queue.length, queueLimit());
    if (!isEnabled()) {
      clearNotifications();
    }
  } else if (message.type === "tts") {
    if (isPreview) return;
    if (message.payload) enqueue(message.payload);
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
    `ws://${window.location.hostname}:${pendingPort}/ws?role=notification&secret=${encodeURIComponent(relaySecret)}`,
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
    `${protocol}//${window.location.host}/ws?role=notification&secret=${encodeURIComponent(relaySecret)}`,
  );
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
    if (!isPreview) hideCard();
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
}

window.addEventListener("beforeunload", () => {
  isUnloading = true;
  window.clearTimeout(reconnectTimer);
  window.clearTimeout(playbackWatchdog);
  socket?.close();
});

connect();
