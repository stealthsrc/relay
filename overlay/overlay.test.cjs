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
      if (force) values.add(name);
      else values.delete(name);
    },
  };
}

test("media previews stay visible and isolated from the live media queue", () => {
  const preview = createHarness("?secret=private&preview=1&lang=fr");
  const image = preview.elements["#image"];
  const author = preview.elements["#author"];

  assert.equal(image.src, "/overlay-assets/relay-radar.png");
  assert.equal(image.classList.contains("is-visible"), true);
  assert.equal(image.style.height, "100%");
  assert.equal(preview.elements["#author-name"].textContent, "Aperçu en direct");
  assert.equal(author.hidden, false);

  preview.socket.emit("message", JSON.stringify({
    type: "config",
    payload: { mediaObsGeometry: { cropRight: 18, contentScale: 140 } },
  }));
  preview.socket.emit("message", JSON.stringify({
    type: "media",
    payload: { kind: "video", url: "https://cdn.discordapp.com/ignored.mp4" },
  }));
  preview.socket.emit("message", JSON.stringify({ type: "clear" }));

  assert.equal(preview.cssProperties["--crop-right"], "18%");
  assert.equal(preview.cssProperties["--content-scale"], "1.4");
  assert.equal(image.src, "/overlay-assets/relay-radar.png");
  assert.equal(image.classList.contains("is-visible"), true);
  assert.equal(preview.elements["#video"].src, "");
  assert.equal(preview.timerDelays().length, 0);

  const panelSource = fs.readFileSync(__dirname + "/../gui/panel.js", "utf8");
  assert.match(panelSource, /previewUrl\.searchParams\.set\("preview", "1"\)/);
  assert.match(panelSource, /url\.searchParams\.set\("preview", "1"\)/);
});

function element() {
  const listeners = new Map();
  return {
    classList: classList(),
    hidden: false,
    src: "",
    textContent: "",
    complete: false,
    naturalWidth: 0,
    naturalHeight: 0,
    videoWidth: 0,
    videoHeight: 0,
    loop: false,
    style: { width: "", height: "" },
    onerror: null,
    addEventListener(type, listener) {
      if (!listeners.has(type)) listeners.set(type, new Set());
      listeners.get(type).add(listener);
    },
    removeEventListener(type, listener) { listeners.get(type)?.delete(listener); },
    emit(type) {
      for (const listener of [...(listeners.get(type) || [])]) listener({ type });
    },
    removeAttribute(name) {
      if (name === "src") this.src = "";
    },
    load() {},
    pauseCalls: 0,
    pause() { this.pauseCalls += 1; },
    play() { return Promise.resolve(); },
  };
}

function createHarness(search = "?secret=private", mode = "all") {
  const selectors = [
    "#image", "#video", "#audio", "#audio-card", "#audio-artwork", "#audio-title", "#audio-artist",
    "#author", "#author-avatar", "#author-name",
    "#widget-move-label",
  ];
  const elements = Object.fromEntries(selectors.map((selector) => [selector, element()]));
  const sockets = [];
  const timers = [];
  const cssProperties = {};

  class MockWebSocket {
    constructor(url) {
      this.url = url;
      this.listeners = new Map();
      sockets.push(this);
    }
    addEventListener(type, listener) { this.listeners.set(type, listener); }
    emit(type, data) { this.listeners.get(type)?.({ data }); }
    close() {}
  }

  const location = {
    protocol: "http:",
    host: "127.0.0.1:4590",
    port: "4590",
    href: `http://127.0.0.1:4590/overlay${search}`,
    search,
    replaced: "",
    replace(url) { this.replaced = String(url); },
  };
  const window = {
    location,
    innerWidth: 640,
    innerHeight: 360,
    addEventListener() {},
    clearTimeout() {},
    setTimeout(callback, delay) { timers.push({ callback, delay }); return timers.length; },
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
        style: { setProperty(name, value) { cssProperties[name] = value; } },
      },
      querySelector: (selector) => selector === 'meta[name="relay-mode"]'
        ? { content: mode }
        : elements[selector],
    },
    encodeURIComponent,
    window,
  });
  vm.runInContext(fs.readFileSync(__dirname + "/overlay.js", "utf8"), context);
  return {
    context,
    cssProperties,
    elements,
    location,
    queuedAuthors: () => vm.runInContext("queue.map((media) => media.author.username)", context),
    runNextTimer: () => timers.shift()?.callback(),
    timerDelays: () => timers.map((timer) => timer.delay),
    socket: sockets[0],
    sockets,
  };
}

test("media output applies live crop and scale for OBS and widgets", () => {
  const obs = createHarness();
  obs.socket.emit("message", JSON.stringify({
    type: "config",
    payload: {
      mediaObsGeometry: {
        cropTop: 4, cropRight: 8, cropBottom: 12, cropLeft: 16, contentScale: 125,
      },
    },
  }));
  assert.deepEqual(
    Object.fromEntries(Object.entries(obs.cssProperties).filter(([name]) => name.startsWith("--crop"))),
    { "--crop-top": "4%", "--crop-right": "8%", "--crop-bottom": "12%", "--crop-left": "16%" },
  );
  assert.equal(obs.cssProperties["--content-scale"], "1.25");

  const widget = createHarness("?secret=private&widget=1");
  widget.socket.emit("message", JSON.stringify({
    type: "config",
    payload: {
      mediaObsGeometry: { contentScale: 75 },
      mediaWidgetGeometry: { cropTop: 10, contentScale: 180 },
    },
  }));
  assert.equal(widget.cssProperties["--crop-top"], "10%");
  assert.equal(widget.cssProperties["--content-scale"], "1.8");

  const css = fs.readFileSync(__dirname + "/overlay.css", "utf8");
  assert.match(css, /clip-path: inset\(var\(--crop-top\)/);
  assert.match(css, /\.audio-card[\s\S]*var\(--content-scale\)/);
  assert.match(css, /\.overlay__author[\s\S]*var\(--content-scale\)/);
});

test("portrait GIF videos use their native aspect ratio without side letterboxing", () => {
  const { context, elements } = createHarness();
  vm.runInContext("fitVisualToViewport(videoElement, 408, 720)", context);
  assert.equal(elements["#video"].style.width, "auto");
  assert.equal(elements["#video"].style.height, "100%");

  vm.runInContext("fitVisualToViewport(videoElement, 1280, 720)", context);
  assert.equal(elements["#video"].style.width, "100%");
  assert.equal(elements["#video"].style.height, "auto");
});

test("media overlay reads the camelCase Discord avatar and falls back locally", () => {
  const { elements, socket } = createHarness();
  const avatar = elements["#author-avatar"];

  socket.emit("message", JSON.stringify({
    type: "media",
    payload: {
      kind: "image",
      url: "https://cdn.discordapp.com/media.png",
      author: {
        username: "Stealthy",
        displayAvatarUrl: "https://cdn.discordapp.com/avatar.png",
      },
    },
  }));

  assert.equal(avatar.src, "https://cdn.discordapp.com/avatar.png");
  assert.equal(elements["#author-name"].textContent, "Stealthy");
  assert.equal(elements["#author"].hidden, false);

  avatar.onerror();
  assert.equal(avatar.src, "/overlay-assets/relay-radar.png");
  assert.equal(avatar.onerror, null);
});

test("media overlay follows a configured server port once it responds", () => {
  const { location, sockets, socket } = createHarness();

  socket.emit("message", JSON.stringify({
    type: "config",
    payload: { port: 4600 },
  }));
  socket.emit("close");

  const probe = sockets[1];
  assert.ok(probe.url.includes(":4600/ws"));
  assert.equal(location.replaced, "");
  probe.emit("open");
  assert.equal(location.replaced, "http://127.0.0.1:4600/overlay?secret=private");
});

test("audio overlay uses embedded artwork and falls back to the Relay logo", () => {
  const { elements, socket } = createHarness();
  const artwork = elements["#audio-artwork"];

  socket.emit("message", JSON.stringify({
    type: "media",
    payload: {
      kind: "audio",
      url: "https://cdn.discordapp.com/track.mp3",
      artworkId: "123456789012345678",
      title: "BOOMBAYAH (Japanese ver.)",
      artist: "BLACKPINK",
    },
  }));

  assert.equal(
    artwork.src,
    "/media-artwork/123456789012345678?secret=private",
  );
  assert.equal(elements["#audio-title"].textContent, "BOOMBAYAH (Japanese ver.)");
  assert.equal(elements["#audio-artist"].textContent, "BLACKPINK");
  assert.equal(elements["#audio-artist"].hidden, false);
  artwork.onerror();
  assert.equal(artwork.src, "/overlay-assets/relay-radar.png");
});

test("desktop media widget stays muted to avoid duplicate OBS audio", () => {
  const { elements, socket } = createHarness("?secret=private&widget=1");

  socket.emit("message", JSON.stringify({
    type: "media",
    payload: { kind: "audio", url: "https://cdn.discordapp.com/track.mp3" },
  }));

  assert.equal(elements["#audio"].muted, true);
  assert.equal(elements["#audio"].volume, 0);
});

test("desktop media widget plays sound locally when widget sound is enabled", () => {
  const { elements, socket } = createHarness("?secret=private&widget=1");

  socket.emit("message", JSON.stringify({
    type: "config",
    payload: { widgetSoundEnabled: true, mediaVolume: 80 },
  }));
  socket.emit("message", JSON.stringify({
    type: "media",
    payload: { kind: "audio", url: "https://cdn.discordapp.com/track.mp3" },
  }));

  assert.equal(elements["#audio"].muted, false);
  assert.equal(elements["#audio"].volume, 0.8);

  socket.emit("message", JSON.stringify({
    type: "config",
    payload: { widgetSoundEnabled: false, mediaVolume: 80 },
  }));
  assert.equal(elements["#audio"].muted, true);
  assert.equal(elements["#audio"].volume, 0);
});

test("OBS audio uses the unchanged bytes cached by Relay", () => {
  const { elements, socket } = createHarness();
  socket.emit("message", JSON.stringify({
    type: "media",
    payload: {
      kind: "audio",
      audioId: "123456789012345678",
      url: "https://cdn.discordapp.com/track.mp3",
    },
  }));

  assert.equal(
    elements["#audio"].src,
    "/media-audio/123456789012345678?secret=private",
  );
});

test("overlay clears playback and visuals when Relay stops", () => {
  const { elements, socket } = createHarness();
  socket.emit("message", JSON.stringify({
    type: "media",
    payload: { kind: "audio", url: "https://cdn.discordapp.com/track.mp3" },
  }));
  socket.emit("close");

  assert.equal(elements["#audio"].src, "");
  assert.equal(elements["#audio-card"].hidden, true);
  assert.ok(elements["#audio"].pauseCalls > 0);
});

test("short OBS sources keep visual and audio media separate", () => {
  const visual = createHarness("", "visual");
  visual.socket.emit("message", JSON.stringify({
    type: "media",
    payload: { kind: "audio", url: "https://cdn.discordapp.com/track.mp3" },
  }));
  assert.equal(visual.elements["#audio"].src, "");

  const audio = createHarness("", "audio");
  audio.socket.emit("message", JSON.stringify({
    type: "media",
    payload: { kind: "image", url: "https://cdn.discordapp.com/still.png" },
  }));
  assert.equal(audio.elements["#image"].src, "");
});

test("media widget move label follows the Relay language", () => {
  const english = createHarness("?secret=private&widget=1&lang=en");
  const french = createHarness("?secret=private&widget=1&lang=fr");
  assert.equal(english.elements["#widget-move-label"].textContent, "Move overlay");
  assert.equal(french.elements["#widget-move-label"].textContent, "Déplacer l’overlay");
});

test("multi-user bursts keep every image and GIF in FIFO order", () => {
  const { queuedAuthors, socket } = createHarness();
  const authors = Array.from({ length: 25 }, (_, index) => `User ${index + 1}`);
  for (const [index, username] of authors.entries()) {
    socket.emit("message", JSON.stringify({
      type: "media",
      payload: {
        kind: index % 2 === 0 ? "image" : "gif",
        url: `https://cdn.discordapp.com/media-${index}.png`,
        author: { username, displayAvatarUrl: "" },
      },
    }));
  }

  assert.deepEqual([...queuedAuthors()], authors.slice(1));
});

test("Discord gifv embeds render through the muted video player", () => {
  const { elements, socket } = createHarness();
  socket.emit("message", JSON.stringify({
    type: "media",
    payload: {
      kind: "gif",
      contentType: "video/mp4",
      url: "https://media.tenor.com/example.mp4",
      author: { username: "Friend", displayAvatarUrl: "" },
    },
  }));

  assert.equal(elements["#video"].src, "https://media.tenor.com/example.mp4");
  assert.equal(elements["#video"].muted, true);
  assert.equal(elements["#video"].volume, 0);
});

test("video GIFs loop until the configured GIF duration expires", () => {
  const { elements, socket, timerDelays } = createHarness();
  socket.emit("message", JSON.stringify({
    type: "config",
    payload: { displayDurationMs: 11111, gifDurationMs: 12345 },
  }));
  socket.emit("message", JSON.stringify({
    type: "media",
    payload: { kind: "gif", contentType: "video/mp4", url: "https://media.klipy.com/gif.mp4" },
  }));

  elements["#video"].videoWidth = 408;
  elements["#video"].videoHeight = 300;
  elements["#video"].emit("loadeddata");

  assert.equal(elements["#video"].loop, true);
  assert.ok(timerDelays().includes(12345));
  assert.ok(!timerDelays().includes(11111));
});

test("static images keep their independent image duration", () => {
  const { elements, socket, timerDelays } = createHarness();
  socket.emit("message", JSON.stringify({
    type: "config",
    payload: { displayDurationMs: 11111, gifDurationMs: 12345 },
  }));
  socket.emit("message", JSON.stringify({
    type: "media",
    payload: { kind: "image", url: "https://cdn.discordapp.com/still.png" },
  }));

  elements["#image"].naturalWidth = 1280;
  elements["#image"].naturalHeight = 720;
  elements["#image"].emit("load");

  assert.ok(timerDelays().includes(11111));
  assert.ok(!timerDelays().includes(12345));
});

test("Discord picker GIFs prefer Relay's local inline media route", () => {
  const { elements, socket } = createHarness();
  socket.emit("message", JSON.stringify({
    type: "media",
    payload: {
      kind: "gif",
      contentType: "video/mp4",
      cachedMediaId: "123456789012345678-embed-0",
      url: "https://images-ext-1.discordapp.net/external/file.mp4",
    },
  }));
  assert.equal(
    elements["#video"].src,
    "/media-cache/123456789012345678-embed-0?secret=private",
  );
});

test("a stalled image advances to the next queued media", () => {
  const { elements, runNextTimer, socket } = createHarness();
  socket.emit("message", JSON.stringify({
    type: "media", payload: { kind: "image", url: "https://cdn.discordapp.com/stalled.png" },
  }));
  socket.emit("message", JSON.stringify({
    type: "media", payload: { kind: "image", url: "https://cdn.discordapp.com/next.png" },
  }));

  runNextTimer();
  runNextTimer();
  assert.equal(elements["#image"].src, "https://cdn.discordapp.com/next.png");
});
