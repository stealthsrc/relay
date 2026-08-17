const assert = require("node:assert/strict");
const fs = require("node:fs");
const test = require("node:test");
const vm = require("node:vm");

test("TTS Browser Source plays FIFO and handles skip without double advancing", async () => {
  const sockets = [];
  const timers = new Map();
  let nextTimerId = 1;
  const audio = {
    src: "",
    volume: 1,
    onended: null,
    onerror: null,
    pause() {},
    load() {},
    play() { return Promise.resolve(); },
    removeAttribute(name) {
      if (name === "src") this.src = "";
    },
  };

  class MockWebSocket {
    constructor(url) {
      this.url = url;
      this.readyState = 1;
      this.listeners = new Map();
      this.sent = [];
      sockets.push(this);
    }

    addEventListener(type, listener) {
      this.listeners.set(type, listener);
    }

    emit(type, data) {
      this.listeners.get(type)?.({ data });
    }

    send(data) {
      this.sent.push(JSON.parse(data));
    }

    close() {}
  }

  const location = {
    protocol: "http:",
    host: "127.0.0.1:4590",
    href: "http://127.0.0.1:4590/tts?secret=private",
    search: "?secret=private",
    replace() {},
  };
  const window = {
    location,
    addEventListener() {},
    clearTimeout(id) { timers.delete(id); },
    setTimeout(callback) {
      const id = nextTimerId++;
      timers.set(id, callback);
      return id;
    },
  };
  const context = vm.createContext({
    URL,
    URLSearchParams,
    WebSocket: MockWebSocket,
    document: { querySelector: () => audio },
    encodeURIComponent,
    window,
  });
  const source = fs.readFileSync(__dirname + "/tts.js", "utf8");
  vm.runInContext(source, context);

  const socket = sockets[0];
  assert.match(socket.url, /role=tts&source=tts&client=obs&secret=private$/);
  socket.emit("message", JSON.stringify({ type: "config", payload: { mediaVolume: 25 } }));
  assert.equal(audio.volume, 0.25);

  socket.emit("message", JSON.stringify({ type: "tts", payload: { id: "visual", visualOnly: true } }));
  assert.equal(audio.src, "");

  socket.emit("message", JSON.stringify({ type: "tts", payload: { id: "1" } }));
  assert.equal(audio.src, "");
  socket.emit("message", JSON.stringify({
    type: "stageClock",
    payload: { mediaBusy: false, musicBusy: false, ttsBusy: true },
  }));
  const firstEnded = audio.onended;
  assert.match(audio.src, /\/tts-audio\/1\?secret=private$/);
  socket.emit("message", JSON.stringify({ type: "tts", payload: { id: "2" } }));
  assert.match(audio.src, /\/tts-audio\/1\?secret=private$/);

  firstEnded();
  assert.equal(audio.src, "");
  socket.emit("message", JSON.stringify({
    type: "stageClock",
    payload: { mediaBusy: false, musicBusy: false, ttsBusy: true },
  }));
  assert.match(audio.src, /\/tts-audio\/2\?secret=private$/);
  firstEnded();
  assert.match(audio.src, /\/tts-audio\/2\?secret=private$/);

  socket.emit("message", JSON.stringify({ type: "tts", payload: { id: "3" } }));
  socket.emit("message", JSON.stringify({ type: "skip" }));
  socket.emit("message", JSON.stringify({
    type: "stageClock",
    payload: { mediaBusy: false, musicBusy: false, ttsBusy: true },
  }));
  assert.match(audio.src, /\/tts-audio\/3\?secret=private$/);
  socket.emit("message", JSON.stringify({ type: "clear" }));
  assert.equal(audio.src, "");
  assert.equal(audio.onended, null);

  socket.emit("message", JSON.stringify({ type: "tts", payload: { id: "4" } }));
  socket.emit("message", JSON.stringify({
    type: "stageClock",
    payload: { mediaBusy: false, musicBusy: false, ttsBusy: true },
  }));
  socket.emit("message", JSON.stringify({ type: "tts", payload: { id: "5" } }));
  await new Promise((resolve) => setImmediate(resolve));
  const watchdog = timers.values().next().value;
  watchdog();
  socket.emit("message", JSON.stringify({
    type: "stageClock",
    payload: { mediaBusy: false, musicBusy: false, ttsBusy: true },
  }));
  assert.match(audio.src, /\/tts-audio\/5\?secret=private$/);

  socket.emit("message", JSON.stringify({ type: "clear" }));
  socket.emit("message", JSON.stringify({
    type: "testOutput",
    payload: { target: "tts", tts: { id: "999999999999999999" } },
  }));
  socket.emit("message", JSON.stringify({
    type: "stageClock",
    payload: { mediaBusy: false, musicBusy: false, ttsBusy: true },
  }));
  assert.match(audio.src, /\/tts-audio\/999999999999999999\?secret=private$/);
  socket.emit("close");
  assert.equal(audio.src, "");
});

test("TTS recovers when stageClock clears a missed musicIdle", () => {
  const sockets = [];
  const audio = {
    src: "",
    volume: 1,
    onended: null,
    onerror: null,
    pause() {},
    load() {},
    play() { return Promise.resolve(); },
    removeAttribute(name) {
      if (name === "src") this.src = "";
    },
  };

  class MockWebSocket {
    constructor(url) {
      this.url = url;
      this.readyState = 1;
      this.listeners = new Map();
      sockets.push(this);
    }

    addEventListener(type, listener) {
      this.listeners.set(type, listener);
    }

    emit(type, data) {
      this.listeners.get(type)?.({ data });
    }

    send() {}
    close() {}
  }

  const location = {
    protocol: "http:",
    host: "127.0.0.1:4590",
    href: "http://127.0.0.1:4590/tts?secret=private",
    search: "?secret=private",
    replace() {},
  };
  const context = vm.createContext({
    URL,
    URLSearchParams,
    WebSocket: MockWebSocket,
    document: { querySelector: () => audio },
    encodeURIComponent,
    window: {
      location,
      addEventListener() {},
      clearTimeout() {},
      setTimeout() { return 1; },
    },
  });
  vm.runInContext(fs.readFileSync(__dirname + "/tts.js", "utf8"), context);
  const socket = sockets[0];

  socket.emit("message", JSON.stringify({
    type: "musicPlay",
    payload: { playbackId: "stuck" },
  }));
  socket.emit("message", JSON.stringify({ type: "tts", payload: { id: "blocked" } }));
  assert.equal(audio.src, "");

  socket.emit("message", JSON.stringify({
    type: "stageClock",
    payload: { mediaBusy: false, musicBusy: false, ttsBusy: false },
  }));
  assert.equal(audio.src, "");
  socket.emit("message", JSON.stringify({
    type: "stageClock",
    payload: { mediaBusy: false, musicBusy: false, ttsBusy: true },
  }));
  assert.match(audio.src, /\/tts-audio\/blocked\?secret=private$/);
});

test("TTS reconnect keeps pending speech and probes a moved server", () => {
  const source = fs.readFileSync(__dirname + "/tts.js", "utf8");
  assert.match(
    source,
    /function interruptTtsPlayback\(\)\s*\{(?![^}]*queue\.length\s*=\s*0)[^}]*currentTts\s*=\s*undefined/s,
  );
  assert.match(source, /socket\.addEventListener\("close"[\s\S]*interruptTtsPlayback\(\)/);
  assert.match(source, /function moveToPendingPort\(\)[\s\S]*new WebSocket[\s\S]*probe\.addEventListener\("open"/);
  assert.match(source, /probeWatchdog[\s\S]*probe\.close\(\)/);
});
