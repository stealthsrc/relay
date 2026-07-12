const stickerElement = document.querySelector("#sticker");
const fallbackElement = document.querySelector("#sticker-fallback");
const parameters = new URLSearchParams(window.location.search);
const relaySecret = parameters.get("secret") || "";
const queue = [];

let config = { stickerDurationMs: 8000 };
let currentSticker;
let displayTimer;
let loadWatchdog;
let reconnectTimer;
let reconnectDelayMs = 1000;
let socket;
let pendingPort;
let isUnloading = false;

function queueLimit() {
  return 50;
}

function stickerSource(sticker) {
  if (sticker.cachedMediaId) {
    return `/media-cache/${encodeURIComponent(sticker.cachedMediaId)}?secret=${encodeURIComponent(relaySecret)}`;
  }
  return sticker.url || "";
}

function resetSticker() {
  window.clearTimeout(displayTimer);
  window.clearTimeout(loadWatchdog);
  stickerElement.onload = null;
  stickerElement.onerror = null;
  stickerElement.classList.remove("is-visible");
  fallbackElement.classList.remove("is-visible");
  stickerElement.removeAttribute("src");
  fallbackElement.hidden = true;
}

function finishCurrent() {
  if (!currentSticker) return;
  resetSticker();
  currentSticker = undefined;
  playNext();
}

function reveal(element) {
  window.requestAnimationFrame(() => element.classList.add("is-visible"));
  const duration = Math.min(60000, Math.max(1000, Number(config.stickerDurationMs) || 8000));
  displayTimer = window.setTimeout(finishCurrent, duration);
}

function showFallback() {
  stickerElement.removeAttribute("src");
  fallbackElement.hidden = false;
  reveal(fallbackElement);
}

function playNext() {
  if (currentSticker || queue.length === 0) return;
  currentSticker = queue.shift();
  resetSticker();
  if (currentSticker.format === "lottie" || currentSticker.format === "unknown") {
    showFallback();
    return;
  }
  stickerElement.onload = () => {
    window.clearTimeout(loadWatchdog);
    reveal(stickerElement);
  };
  stickerElement.onerror = () => {
    window.clearTimeout(loadWatchdog);
    showFallback();
  };
  stickerElement.alt = currentSticker.name || "Discord sticker";
  stickerElement.src = stickerSource(currentSticker);
  loadWatchdog = window.setTimeout(showFallback, 12000);
}

function enqueue(sticker) {
  if (!sticker || queue.length >= queueLimit()) return;
  queue.push(sticker);
  playNext();
}

function clearStickers() {
  queue.length = 0;
  resetSticker();
  currentSticker = undefined;
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
    if (Number.isInteger(configuredPort) && configuredPort > 0 && configuredPort <= 65535
      && String(configuredPort) !== window.location.port) {
      pendingPort = configuredPort;
    }
  } else if (message.type === "sticker") {
    enqueue(message.payload);
  } else if (message.type === "clear") {
    clearStickers();
  } else if (message.type === "serverMove") {
    pendingPort = message.payload.port;
  }
}

function scheduleReconnect() {
  if (isUnloading || reconnectTimer) return;
  reconnectTimer = window.setTimeout(() => {
    reconnectTimer = undefined;
    connect();
  }, reconnectDelayMs);
  reconnectDelayMs = Math.min(reconnectDelayMs * 2, 10000);
}

function connect() {
  const protocol = window.location.protocol === "https:" ? "wss:" : "ws:";
  socket = new WebSocket(
    `${protocol}//${window.location.host}/ws?role=sticker&secret=${encodeURIComponent(relaySecret)}`,
  );
  socket.addEventListener("open", () => { reconnectDelayMs = 1000; });
  socket.addEventListener("message", handleMessage);
  socket.addEventListener("close", () => {
    clearStickers();
    if (pendingPort) {
      const nextUrl = new URL(window.location.href);
      nextUrl.port = String(pendingPort);
      window.setTimeout(() => window.location.replace(nextUrl), 500);
      return;
    }
    scheduleReconnect();
  });
  socket.addEventListener("error", () => socket.close());
}

window.addEventListener("beforeunload", () => {
  isUnloading = true;
  clearStickers();
  window.clearTimeout(reconnectTimer);
  socket?.close();
});

connect();
