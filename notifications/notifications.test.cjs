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

function createHarness(target = "obs", language = "en") {
  const sockets = [];
  const timers = new Map();
  let nextTimerId = 1;
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

  const search = `?secret=private&target=${target}&locked=0&lang=${language}`;
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
    document: {
      documentElement: {
        classList: classList(),
        dataset: {},
        lang: "en",
        style: { setProperty() {} },
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

  return { elements, socket: sockets[0], timers };
}

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
