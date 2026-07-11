const cardElement = document.querySelector("#notification");
const avatarElement = document.querySelector("#notification-avatar");
const authorElement = document.querySelector("#notification-author");
const messageElement = document.querySelector("#notification-message");
const audioElement = document.querySelector("#notification-clock");
const parameters = new URLSearchParams(window.location.search);
const relaySecret = parameters.get("secret") || "";
const target = parameters.get("target") === "widget" ? "widget" : "obs";
let interfaceLanguage = parameters.get("lang") || "en";
const moveLabelElement = document.querySelector("#notification-move-label");
const moveLabels = { en: "Move notification", fr: "Déplacer la notification", es: "Mover notificación", de: "Benachrichtigung verschieben" };
const fallbackAvatar = "/overlay-assets/relay-radar.png";
const queue = [];

let config = { ttsNotificationsObsEnabled: false, ttsQueueLimit: 50 };
let currentNotification;
let socket;
let reconnectTimer;
let reconnectDelayMs = 1000;
let pendingPort;
let isUnloading = false;
let playbackGeneration = 0;
let playbackWatchdog;

audioElement.muted = true;
document.documentElement.classList.toggle("notification-widget", target === "widget");

function isEnabled() {
  return target === "widget" || Boolean(config.ttsNotificationsObsEnabled);
}

function queueLimit() {
  return Math.min(50, Math.max(1, Number(config.ttsQueueLimit) || 50));
}

function audioUrl(ttsEvent) {
  return `/tts-audio/${encodeURIComponent(ttsEvent.id)}?secret=${encodeURIComponent(relaySecret)}`;
}

function setCardContent(notification) {
  authorElement.textContent = notification.author?.username || "Discord";
  messageElement.textContent = notification.text || "";
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
  audioElement.onended = () => finishCurrent(generation);
  audioElement.onerror = () => finishCurrent(generation);
  const armWatchdog = (delay = 20000) => {
    window.clearTimeout(playbackWatchdog);
    playbackWatchdog = window.setTimeout(() => finishCurrent(generation), delay);
  };
  audioElement.onplaying = () => armWatchdog();
  audioElement.ontimeupdate = () => armWatchdog();
  audioElement.onwaiting = () => armWatchdog(15000);
  audioElement.onstalled = () => armWatchdog(15000);
  audioElement.onabort = () => finishCurrent(generation);
  audioElement.onemptied = () => finishCurrent(generation);
  audioElement.src = audioUrl(currentNotification);
  audioElement.load();
  armWatchdog(15000);
  audioElement.play()
    .then(() => {
      if (currentNotification && generation === playbackGeneration) {
        showCard();
      }
    })
    .catch(() => finishCurrent(generation));
}

function enqueue(notification) {
  if (!isEnabled() || queue.length >= queueLimit()) {
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
    const configuredPort = Number(message.payload?.port);
    if (
      Number.isInteger(configuredPort)
      && configuredPort > 0
      && configuredPort <= 65535
      && String(configuredPort) !== window.location.port
    ) {
      pendingPort = configuredPort;
    }
    queue.length = Math.min(queue.length, queueLimit());
    if (!isEnabled()) {
      clearNotifications();
    }
  } else if (message.type === "tts") {
    enqueue(message.payload);
  } else if (message.type === "skip") {
    finishCurrent();
  } else if (message.type === "clear") {
    clearNotifications();
  } else if (message.type === "serverMove") {
    pendingPort = message.payload.port;
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
}

applyAppearance({ language: interfaceLanguage, fontScale: 100, accentRgb: [88, 185, 137] });

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
  window.setTimeout(() => window.location.replace(nextUrl), 500);
}

function connect() {
  const protocol = window.location.protocol === "https:" ? "wss:" : "ws:";
  socket = new WebSocket(
    `${protocol}//${window.location.host}/ws?role=notification&secret=${encodeURIComponent(relaySecret)}`,
  );
  socket.addEventListener("open", () => {
    reconnectDelayMs = 1000;
  });
  socket.addEventListener("message", handleMessage);
  socket.addEventListener("close", () => {
    clearNotifications();
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
