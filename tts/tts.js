const audioElement = document.querySelector("#tts-audio");
const parameters = new URLSearchParams(window.location.search);
const relaySecret =
  parameters.get("secret")
  || document.querySelector('meta[name="relay-secret"]')?.content
  || "";
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
let mediaBusy = false;
let musicActive = false;
let ttsBusy = false;
let reportedTtsStageBusy = false;
let ttsStageClaimPending = false;
let lastStageClockPayload = {};

function audioUrl(ttsEvent) {
  return `/tts-audio/${encodeURIComponent(ttsEvent.id)}?secret=${encodeURIComponent(relaySecret)}`;
}

function stageBlocked() {
  return mediaBusy || musicActive;
}

function syncTtsStageBusy(busy) {
  if (socket?.readyState !== 1) return;
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
  if (Object.prototype.hasOwnProperty.call(payload, "musicBusy")) {
    musicActive = Boolean(payload.musicBusy);
  }
  ttsBusy = Boolean(payload.ttsBusy);
  resolveTtsStageClaim();
  if (!stageBlocked() && !currentTts && !ttsStageClaimPending) playNextTts();
}

function setMusicActive(active, { resume = false } = {}) {
  musicActive = Boolean(active);
  if (resume && !stageBlocked() && !ttsStageClaimPending) playNextTts();
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
  beginCurrentTts();
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
  syncTtsStageBusy(false);
  playNextTts();
}

function beginCurrentTts() {
  if (currentTts || queue.length === 0) {
    syncTtsStageBusy(false);
    return;
  }
  currentTts = queue.shift();
  const generation = playbackGeneration;
  const keepGoing = () => finishCurrentTts(generation);
  audioElement.onended = keepGoing;
  audioElement.onerror = keepGoing;
  const armWatchdog = (delay = 20000) => {
    window.clearTimeout(playbackWatchdog);
    playbackWatchdog = window.setTimeout(keepGoing, delay);
  };
  audioElement.onplaying = () => armWatchdog();
  audioElement.ontimeupdate = () => armWatchdog();
  audioElement.onwaiting = () => armWatchdog(15000);
  audioElement.onstalled = () => armWatchdog(15000);
  audioElement.onabort = keepGoing;
  audioElement.onemptied = keepGoing;
  audioElement.volume = Math.min(1, Math.max(0, (Number(config.mediaVolume) || 50) / 100));
  audioElement.src = audioUrl(currentTts);
  audioElement.load();
  armWatchdog(15000);
  audioElement.play().catch(keepGoing);
}

function playNextTts() {
  if (currentTts || queue.length === 0 || stageBlocked() || ttsStageClaimPending) return;
  ttsStageClaimPending = true;
  syncTtsStageBusy(true);
}

function enqueue(ttsEvent) {
  const limit = Math.min(50, Math.max(1, Number(config.ttsQueueLimit) || 50));
  if (!ttsEvent || ttsEvent.visualOnly || queue.length >= limit) return;
  queue.push(ttsEvent);
  playNextTts();
}

function clearTts() {
  queue.length = 0;
  resetAudio();
  currentTts = undefined;
  ttsStageClaimPending = false;
  syncTtsStageBusy(false);
}

function interruptTtsPlayback() {
  resetAudio();
  currentTts = undefined;
  ttsStageClaimPending = false;
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
    audioElement.volume = Math.min(1, Math.max(0, (Number(config.mediaVolume) || 50) / 100));
  } else if (message.type === "tts") {
    enqueue(message.payload);
  } else if (message.type === "testOutput") {
    if (message.payload?.target === "tts" && message.payload.tts) {
      enqueue(message.payload.tts);
    }
  } else if (message.type === "musicPlay") {
    setMusicActive(true);
  } else if (message.type === "musicStop") {
    setMusicActive(false);
  } else if (message.type === "musicIdle") {
    setMusicActive(false, { resume: true });
  } else if (message.type === "stageClock") {
    applyStageClock(message.payload);
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
  if (isUnloading || reconnectTimer) return;
  reconnectTimer = window.setTimeout(() => {
    reconnectTimer = undefined;
    connect();
  }, reconnectDelayMs);
  reconnectDelayMs = Math.min(reconnectDelayMs * 2, 10000);
}

function moveToPendingPort() {
  const nextUrl = new URL(window.location.href);
  nextUrl.port = String(pendingPort);
  const probe = new WebSocket(
    ttsSocketUrl(`${window.location.hostname}:${pendingPort}`, "probe", "ws:"),
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
  socket = new WebSocket(ttsSocketUrl(window.location.host));
  socket.addEventListener("open", () => {
    reconnectDelayMs = 1000;
    playNextTts();
  });
  socket.addEventListener("message", handleMessage);
  socket.addEventListener("close", () => {
    interruptTtsPlayback();
    mediaBusy = false;
    musicActive = false;
    ttsBusy = false;
    ttsStageClaimPending = false;
    reportedTtsStageBusy = false;
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
  clearTts();
  window.clearTimeout(reconnectTimer);
  socket?.close();
});

connect();
