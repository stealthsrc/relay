const assert = require("node:assert/strict");
const fs = require("node:fs");
const test = require("node:test");
const vm = require("node:vm");

function classList() {
  const values = new Set();
  return {
    add: (...names) => names.forEach((name) => values.add(name)),
    remove: (...names) => names.forEach((name) => values.delete(name)),
    contains: (name) => values.has(name),
    toggle(name, force) {
      if (force === undefined ? !values.has(name) : force) {
        values.add(name);
        return true;
      }
      values.delete(name);
      return false;
    },
  };
}

function createHarness(target = "obs", language = "en", preview = false) {
  const sockets = [];
  const timers = new Map();
  const timerDelays = [];
  let nextTimerId = 1;
  const cssProperties = {};
  const elements = {
    "#notification": {
      classList: classList(),
      attributes: new Map(),
      setAttribute(name, value) { this.attributes.set(name, value); },
    },
    "#notification-avatar": { src: "", onerror: null },
    "#notification-author": { textContent: "" },
    "#notification-message": {
      textContent: "",
      children: [],
      replaceChildren() { this.children = []; this.textContent = ""; },
      append(node) { this.children.push(node); },
    },
    "#notification-move-label": { textContent: "" },
    "#notification-clock": {
      src: "",
      muted: false,
      onended: null,
      onerror: null,
      pause() {},
      load() {},
      play() { return Promise.resolve(); },
      removeAttribute(name) {
        if (name === "src") this.src = "";
      },
    },
  };

  class MockWebSocket {
    constructor(url) {
      this.url = url;
      this.listeners = new Map();
      sockets.push(this);
    }

    addEventListener(type, listener) {
      this.listeners.set(type, listener);
    }

    emit(type, data) {
      this.listeners.get(type)?.({ data });
    }

    close() {}
  }

  const search = `?secret=private&target=${target}&locked=0&lang=${language}${preview ? "&preview=1" : ""}`;
  const location = {
    protocol: "http:",
    host: "127.0.0.1:4590",
    href: `http://127.0.0.1:4590/notifications${search}`,
    search,
    replace() {},
  };
  const window = {
    location,
    addEventListener() {},
    clearTimeout(id) { timers.delete(id); },
    setTimeout(callback, delay) {
      const id = nextTimerId++;
      timers.set(id, callback);
      timerDelays.push(delay);
      return id;
    },
  };
  const pings = [];
  class MockAudio {
    constructor() {
      this.src = "";
      this.volume = 1;
      this.playCount = 0;
      pings.push(this);
    }

    play() {
      this.playCount += 1;
      return Promise.resolve();
    }
  }

  const context = vm.createContext({
    URL,
    URLSearchParams,
    WebSocket: MockWebSocket,
    Audio: MockAudio,
    document: {
      documentElement: {
        classList: classList(),
        dataset: {},
        lang: "en",
        style: { setProperty(name, value) { cssProperties[name] = value; } },
      },
      querySelector: (selector) => elements[selector],
      createElement: (tagName) => ({
        tagName,
        className: "",
        src: "",
        alt: "",
        onerror: null,
        replaceWith() {},
      }),
      createTextNode: (textContent) => ({ textContent }),
    },
    encodeURIComponent,
    window,
  });
  const source = fs.readFileSync(__dirname + "/notifications.js", "utf8");
  vm.runInContext(source, context);

  return { cssProperties, elements, pings, socket: sockets[0], timers, timerDelays };
}

test("notification geometry previews stay visible without consuming live TTS or playing sound", () => {
  const preview = createHarness("widget", "fr", true);
  const card = preview.elements["#notification"];

  assert.equal(card.classList.contains("is-visible"), true);
  assert.equal(preview.elements["#notification-author"].textContent, "Aperçu en direct");
  assert.equal(preview.elements["#notification-message"].textContent, "Votre notification apparaîtra ici.");

  preview.socket.emit("message", JSON.stringify({
    type: "config",
    payload: {
      notificationSoundEnabled: true,
      notificationWidgetGeometry: { cropLeft: 15, contentScale: 125 },
    },
  }));
  preview.socket.emit("message", JSON.stringify({ type: "tts", payload: notification("ignored") }));
  preview.socket.emit("message", JSON.stringify({ type: "clear" }));

  assert.equal(preview.cssProperties["--crop-left"], "15%");
  assert.equal(preview.cssProperties["--content-scale"], "1.25");
  assert.equal(preview.elements["#notification-author"].textContent, "Aperçu en direct");
  assert.equal(card.classList.contains("is-visible"), true);
  assert.equal(preview.elements["#notification-clock"].src, "");
  assert.equal(preview.pings.length, 0);

  const panelSource = fs.readFileSync(__dirname + "/../gui/panel.js", "utf8");
  assert.match(panelSource, /notificationUrl[\s\S]*searchParams\.set\("preview", "1"\)/);
});

test("notification output applies live crop and scale for OBS and widgets", () => {
  const obs = createHarness("obs");
  obs.socket.emit("message", JSON.stringify({
    type: "config",
    payload: {
      notificationObsGeometry: {
        cropTop: 3, cropRight: 6, cropBottom: 9, cropLeft: 12, contentScale: 150,
      },
    },
  }));
  assert.equal(obs.cssProperties["--crop-left"], "12%");
  assert.equal(obs.cssProperties["--content-scale"], "1.5");

  const widget = createHarness("widget");
  widget.socket.emit("message", JSON.stringify({
    type: "config",
    payload: {
      notificationObsGeometry: { contentScale: 60 },
      notificationWidgetGeometry: { cropBottom: 20, contentScale: 90 },
    },
  }));
  assert.equal(widget.cssProperties["--crop-bottom"], "20%");
  assert.equal(widget.cssProperties["--content-scale"], "0.9");

  const css = fs.readFileSync(__dirname + "/notifications.css", "utf8");
  assert.match(css, /clip-path: inset\(var\(--crop-top\)/);
  assert.match(css, /\.notification-card[\s\S]*var\(--content-scale\)/);
});

test("Windows widget plays the configured notification sound per message", () => {
  const { pings, socket } = createHarness("widget");

  socket.emit("message", JSON.stringify({
    type: "config",
    payload: { notificationSoundEnabled: true, mediaVolume: 80 },
  }));
  socket.emit("message", JSON.stringify({ type: "tts", payload: notification("1") }));

  assert.equal(pings.length, 1);
  assert.equal(pings[0].playCount, 1);
  assert.equal(pings[0].volume, 0.8);
  assert.match(pings[0].src, /^\/notification-sound\?secret=private$/);

  const disabled = createHarness("widget");
  disabled.socket.emit("message", JSON.stringify({ type: "tts", payload: notification("1") }));
  assert.equal(disabled.pings.length, 0);
});

test("OBS notification page plays the sound only with its own toggle", () => {
  const widgetOnly = createHarness("obs");
  widgetOnly.socket.emit("message", JSON.stringify({
    type: "config",
    payload: { ttsNotificationsObsEnabled: true, notificationSoundEnabled: true },
  }));
  widgetOnly.socket.emit("message", JSON.stringify({ type: "tts", payload: notification("1") }));
  assert.equal(widgetOnly.pings.length, 0);

  const obsEnabled = createHarness("obs");
  obsEnabled.socket.emit("message", JSON.stringify({
    type: "config",
    payload: { ttsNotificationsObsEnabled: true, notificationSoundObsEnabled: true },
  }));
  obsEnabled.socket.emit("message", JSON.stringify({ type: "tts", payload: notification("1") }));
  assert.equal(obsEnabled.pings.length, 1);
  assert.equal(obsEnabled.pings[0].playCount, 1);
});

test("notification widget move label follows the Relay language", () => {
  const english = createHarness("widget", "en");
  const french = createHarness("widget", "fr");
  assert.equal(english.elements["#notification-move-label"].textContent, "Move notification");
  assert.equal(french.elements["#notification-move-label"].textContent, "Déplacer la notification");
});

function notification(id, username = `User ${id}`) {
  return {
    id,
    text: `Message ${id}`,
    author: {
      username,
      displayAvatarUrl: `https://cdn.discordapp.com/avatars/${id}.png`,
    },
  };
}

function nextMicrotask() {
  return new Promise((resolve) => setImmediate(resolve));
}

test("OBS notifications follow TTS FIFO, skip, clear, and configured queue limit", async () => {
  const { elements, socket } = createHarness();
  const card = elements["#notification"];
  const audio = elements["#notification-clock"];

  assert.match(socket.url, /role=notification&secret=private$/);
  assert.equal(audio.muted, true);

  socket.emit("message", JSON.stringify({
    type: "tts",
    payload: notification("ignored"),
  }));
  assert.equal(audio.src, "");

  socket.emit("message", JSON.stringify({
    type: "config",
    payload: { ttsNotificationsObsEnabled: true, ttsQueueLimit: 1 },
  }));
  socket.emit("message", JSON.stringify({ type: "tts", payload: notification("1", "Alice") }));
  await nextMicrotask();

  assert.match(audio.src, /\/tts-audio\/1\?secret=private$/);
  assert.equal(elements["#notification-author"].textContent, "Alice");
  assert.equal(elements["#notification-message"].textContent, "Message 1");
  assert.match(elements["#notification-avatar"].src, /avatars\/1\.png$/);
  assert.equal(card.classList.contains("is-visible"), true);

  const firstEnded = audio.onended;
  socket.emit("message", JSON.stringify({ type: "tts", payload: notification("2") }));
  socket.emit("message", JSON.stringify({ type: "tts", payload: notification("dropped") }));
  firstEnded();
  await nextMicrotask();

  assert.match(audio.src, /\/tts-audio\/2\?secret=private$/);
  firstEnded();
  assert.match(audio.src, /\/tts-audio\/2\?secret=private$/);

  socket.emit("message", JSON.stringify({ type: "tts", payload: notification("3") }));
  socket.emit("message", JSON.stringify({ type: "skip" }));
  await nextMicrotask();
  assert.match(audio.src, /\/tts-audio\/3\?secret=private$/);

  socket.emit("message", JSON.stringify({ type: "clear" }));
  assert.equal(audio.src, "");
  assert.equal(audio.onended, null);
  assert.equal(card.classList.contains("is-visible"), false);
});

test("Windows notification widget remains independent from the OBS toggle", async () => {
  const { elements, socket } = createHarness("widget");
  const audio = elements["#notification-clock"];

  socket.emit("message", JSON.stringify({
    type: "config",
    payload: { ttsNotificationsObsEnabled: false, ttsQueueLimit: 50 },
  }));
  socket.emit("message", JSON.stringify({ type: "tts", payload: notification("widget") }));
  await nextMicrotask();

  assert.match(audio.src, /\/tts-audio\/widget\?secret=private$/);
  assert.equal(elements["#notification"].classList.contains("is-visible"), true);
});

test("OBS and Windows outputs keep separate queues while displaying the same TTS event", async () => {
  const obs = createHarness("obs");
  const windows = createHarness("widget");
  const config = JSON.stringify({
    type: "config",
    payload: { ttsNotificationsObsEnabled: true, ttsQueueLimit: 50 },
  });
  obs.socket.emit("message", config);
  windows.socket.emit("message", config);

  const event = JSON.stringify({ type: "tts", payload: notification("shared") });
  obs.socket.emit("message", event);
  windows.socket.emit("message", event);
  await nextMicrotask();

  assert.match(obs.elements["#notification-clock"].src, /\/tts-audio\/shared\?secret=private$/);
  assert.match(windows.elements["#notification-clock"].src, /\/tts-audio\/shared\?secret=private$/);
  assert.equal(obs.elements["#notification"].classList.contains("is-visible"), true);
  assert.equal(windows.elements["#notification"].classList.contains("is-visible"), true);

  obs.socket.emit("message", JSON.stringify({ type: "clear" }));
  assert.equal(obs.elements["#notification"].classList.contains("is-visible"), false);
  assert.equal(obs.elements["#notification-clock"].src, "");
  assert.equal(windows.elements["#notification"].classList.contains("is-visible"), true);
  assert.match(windows.elements["#notification-clock"].src, /\/tts-audio\/shared\?secret=private$/);
});

test("notification playback clears when Relay disconnects", async () => {
  const { elements, socket } = createHarness("widget");
  socket.emit("message", JSON.stringify({ type: "tts", payload: notification("disconnect") }));
  await nextMicrotask();
  socket.emit("close");
  assert.equal(elements["#notification-clock"].src, "");
  assert.equal(elements["#notification"].attributes.get("aria-hidden"), "true");
});

test("emoji messages render visually without requesting TTS audio", () => {
  const { elements, socket } = createHarness("widget");
  socket.emit("message", JSON.stringify({
    type: "tts",
    payload: {
      ...notification("emoji", "Emoji user"),
      visualOnly: true,
      segments: [
        { kind: "text", value: "Hello " },
        { kind: "emoji", value: "👋", url: null },
        { kind: "emoji", value: ":relay:", url: "https://cdn.discordapp.com/emojis/1.webp" },
      ],
    },
  }));

  assert.equal(elements["#notification-clock"].src, "");
  assert.equal(elements["#notification"].classList.contains("is-visible"), true);
  assert.equal(elements["#notification-message"].children.length, 3);
  assert.equal(elements["#notification-message"].children[2].tagName, "img");
});

test("a spoken notification stays visible even when audio playback fails", async () => {
  const { elements, socket, timers } = createHarness("widget");
  const card = elements["#notification"];
  const audio = elements["#notification-clock"];
  audio.play = () => Promise.reject(new Error("playback blocked"));

  socket.emit("message", JSON.stringify({ type: "tts", payload: notification("silent", "Muted") }));
  assert.equal(card.classList.contains("is-visible"), true);
  await nextMicrotask();

  assert.equal(card.classList.contains("is-visible"), true);
  assert.equal(elements["#notification-author"].textContent, "Muted");
  assert.equal(elements["#notification-message"].textContent, "Message silent");

  for (const fire of [...timers.values()]) fire();
  assert.equal(card.classList.contains("is-visible"), false);

  audio.play = () => Promise.resolve();
  socket.emit("message", JSON.stringify({ type: "tts", payload: notification("next", "Speaker") }));
  await nextMicrotask();
  assert.match(audio.src, /\/tts-audio\/next\?secret=private$/);
  assert.equal(card.classList.contains("is-visible"), true);
});

test("emoji then plain text messages both notify and keep the queue moving", async () => {
  const { elements, socket } = createHarness("widget");
  const card = elements["#notification"];
  const audio = elements["#notification-clock"];

  socket.emit("message", JSON.stringify({
    type: "tts",
    payload: {
      ...notification("emoji-first"),
      visualOnly: true,
      segments: [{ kind: "emoji", value: "👋", url: null }],
    },
  }));
  assert.equal(card.classList.contains("is-visible"), true);
  assert.equal(audio.src, "");

  socket.emit("message", JSON.stringify({ type: "tts", payload: notification("test", "Tester") }));
  await nextMicrotask();
  assert.equal(card.classList.contains("is-visible"), true);
  assert.equal(elements["#notification-author"].textContent, "Tester");
  assert.match(audio.src, /\/tts-audio\/test\?secret=private$/);

  socket.emit("message", JSON.stringify({ type: "tts", payload: notification("second", "Second user") }));
  audio.onended();
  await nextMicrotask();
  assert.equal(elements["#notification-author"].textContent, "Second user");
  assert.match(audio.src, /\/tts-audio\/second\?secret=private$/);
  assert.equal(card.classList.contains("is-visible"), true);
});

test("visual notifications stay visible for the configured duration", () => {
  const { elements, socket, timerDelays } = createHarness("widget");
  socket.emit("message", JSON.stringify({
    type: "config",
    payload: { notificationDurationMs: 12000 },
  }));
  socket.emit("message", JSON.stringify({
    type: "tts",
    payload: {
      ...notification("timed"),
      visualOnly: true,
      segments: [{ kind: "emoji", value: "👋", url: null }],
    },
  }));

  assert.equal(elements["#notification"].classList.contains("is-visible"), true);
  assert.equal(timerDelays.at(-1), 12000);
});

test("visual notifications fall back to eight seconds without a configured duration", () => {
  const { socket, timerDelays } = createHarness("widget");
  socket.emit("message", JSON.stringify({
    type: "tts",
    payload: {
      ...notification("default-timed"),
      visualOnly: true,
      segments: [{ kind: "emoji", value: "👋", url: null }],
    },
  }));

  assert.equal(timerDelays.at(-1), 8000);
});

test("a visual emoji notification never blocks the next spoken message", async () => {
  const { elements, socket } = createHarness("widget");
  socket.emit("message", JSON.stringify({
    type: "tts",
    payload: {
      ...notification("emoji"),
      visualOnly: true,
      segments: [{ kind: "emoji", value: "👋", url: null }],
    },
  }));
  socket.emit("message", JSON.stringify({
    type: "tts",
    payload: notification("after-emoji", "Friend"),
  }));
  await nextMicrotask();

  assert.match(elements["#notification-clock"].src, /\/tts-audio\/after-emoji\?secret=private$/);
  assert.equal(elements["#notification-author"].textContent, "Friend");
  assert.equal(elements["#notification-message"].textContent, "Message after-emoji");
});
