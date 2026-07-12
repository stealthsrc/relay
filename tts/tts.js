const audioElement = document.querySelector("#tts-audio");
const parameters = new URLSearchParams(window.location.search);
const relaySecret = parameters.get("secret") || "";
const queue = [];

function ttsSocketUrl(
  host,
  client = "obs",
  protocol = window.location.protocol === "https:" ? "wss:" : "ws:",
) {
  return `${protocol}//${host}/ws?role=tts&source=tts&client=${encodeURIComponent(client)}&secret=${encodeURIComponent(relaySecret)}`;
}

let config = { mediaVolume: 50, ttsQueueLimit: 50 };
let currentTts;
let socket;
let reconnectTimer;
let reconnectDelayMs = 1000;
let pendingPort;
let isUnloading = false;
let playbackGeneration = 0;
let playbackWatchdog;

function audioUrl(ttsEvent) {
  return `/tts-audio/${encodeURIComponent(ttsEvent.id)}?secret=${encodeURIComponent(relaySecret)}`;
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

function finishCurrentTts(expectedGeneration = playbackGeneration) {
  if (!currentTts || expectedGeneration !== playbackGeneration) {
    return;
  }
  resetAudio();
  currentTts = undefined;
  playNextTts();
}

function playNextTts() {
  if (currentTts || queue.length === 0) {
    return;
  }
  currentTts = queue.shift();
  const generation = playbackGeneration;
  audioElement.onended = () => finishCurrentTts(generation);
  audioElement.onerror = () => finishCurrentTts(generation);
  const armWatchdog = (delay = 20000) => {
    window.clearTimeout(playbackWatchdog);
    playbackWatchdog = window.setTimeout(() => finishCurrentTts(generation), delay);
  };
  audioElement.onplaying = () => armWatchdog();
  audioElement.ontimeupdate = () => armWatchdog();
  audioElement.onwaiting = () => armWatchdog(15000);
  audioElement.onstalled = () => armWatchdog(15000);
  audioElement.onabort = () => finishCurrentTts(generation);
  audioElement.onemptied = () => finishCurrentTts(generation);
  audioElement.volume = Math.min(1, Math.max(0, config.mediaVolume / 100));
  audioElement.src = audioUrl(currentTts);
  audioElement.load();
  armWatchdog(15000);
  audioElement.play().catch(() => finishCurrentTts(generation));
}

function enqueueTts(ttsEvent) {
  if (ttsEvent?.visualOnly) {
    return;
  }
  const queueLimit = Math.min(50, Math.max(1, Number(config.ttsQueueLimit) || 50));
  if (queue.length >= queueLimit) {
    return;
  }
  queue.push(ttsEvent);
  playNextTts();
}

function clearTts() {
  queue.length = 0;
  if (currentTts) {
    finishCurrentTts();
  } else {
    resetAudio();
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
    queue.length = Math.min(
      queue.length,
      Math.min(50, Math.max(1, Number(config.ttsQueueLimit) || 50)),
    );
    audioElement.volume = Math.min(1, Math.max(0, config.mediaVolume / 100));
  } else if (message.type === "tts") {
    if (message.payload) enqueueTts(message.payload);
  } else if (message.type === "testOutput") {
    const outputTest = message.payload;
    if (outputTest?.target === "tts" && outputTest.tts) {
      enqueueTts(outputTest.tts);
    }
  } else if (message.type === "skip") {
    finishCurrentTts();
  } else if (message.type === "clear") {
    clearTts();
  } else if (message.type === "serverMove") {
    const movedPort = Number(message.payload?.port);
    if (Number.isInteger(movedPort) && movedPort > 0 && movedPort <= 65535) {
      pendingPort = movedPort;
    }
  }
}

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
    ttsSocketUrl(`${window.location.hostname}:${pendingPort}`, "probe", "ws:"),
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
  socket = new WebSocket(ttsSocketUrl(window.location.host));
  socket.addEventListener("open", () => {
    reconnectDelayMs = 1000;
    playNextTts();
  });
  socket.addEventListener("message", handleMessage);
  socket.addEventListener("close", () => {
    // Stop the current playback but keep the queue for after the reconnect.
    if (currentTts) {
      resetAudio();
      currentTts = undefined;
    }
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
  window.clearTimeout(reconnectTimer);
  window.clearTimeout(playbackWatchdog);
  socket?.close();
});

connect();
