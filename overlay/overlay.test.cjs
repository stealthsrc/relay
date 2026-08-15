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

test("media outputs identify their source and local client context", () => {
  const obs = createHarness("?secret=private", "visual");
  assert.match(obs.socket.url, /role=overlay&source=visual&client=obs&secret=private$/);

  const widget = createHarness("?secret=private&widget=1", "audio");
  assert.match(widget.socket.url, /role=overlay&source=audio&client=widget&secret=private$/);

  const preview = createHarness("?secret=private&preview=1", "visual");
  assert.match(preview.socket.url, /role=overlay&source=visual&client=preview&secret=private$/);
});

test("local visual tests reach matching outputs without affecting previews", () => {
  const visual = createHarness("?secret=private", "visual");
  sendMediaClock(visual);
  visual.socket.emit("message", JSON.stringify({
    type: "testOutput",
    payload: {
      target: "visual",
      media: { kind: "image", url: "/overlay-assets/relay-radar.png" },
    },
  }));
  sendMediaGrant(visual, true, { videoBusy: true });
  assert.equal(visual.elements["#image"].src, "/overlay-assets/relay-radar.png");

  const audio = createHarness("?secret=private", "audio");
  audio.socket.emit("message", JSON.stringify({
    type: "testOutput",
    payload: {
      target: "visual",
      media: { kind: "image", url: "/overlay-assets/relay-radar.png" },
    },
  }));
  assert.equal(audio.elements["#image"].src, "");

  const preview = createHarness("?secret=private&preview=1", "visual");
  preview.socket.emit("message", JSON.stringify({
    type: "testOutput",
    payload: {
      target: "visual",
      media: { kind: "image", url: "https://cdn.discordapp.com/ignored.png" },
    },
  }));
  assert.equal(preview.elements["#image"].src, "/overlay-assets/relay-radar.png");
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
    style: { width: "", height: "", left: "", bottom: "", maxWidth: "" },
    getBoundingClientRect() {
      return this.rect || { left: 0, top: 0, right: 0, bottom: 0, width: 0, height: 0 };
    },
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
    "#image", "#video", "#audio", "#audio-card", "#audio-artwork", "#audio-title", "#audio-artist", "#audio-media-text",
    "#author", "#author-avatar", "#author-name", "#media-text", "#youtube-player",
    "#widget-move-label",
  ];
  const elements = Object.fromEntries(selectors.map((selector) => [selector, element()]));
  const sockets = [];
  const youtubePlayers = [];
  const timers = [];
  const cssProperties = {};

  class MockWebSocket {
    constructor(url) {
      this.url = url;
      this.listeners = new Map();
      this.readyState = 1;
      this.sent = [];
      sockets.push(this);
    }
    addEventListener(type, listener) { this.listeners.set(type, listener); }
    emit(type, data) { this.listeners.get(type)?.({ data }); }
    send(data) { this.sent.push(JSON.parse(data)); }
    close() { this.readyState = 3; }
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
  class FakeYoutubePlayer {
    constructor(id, options) {
      this.id = id;
      this.options = options;
      this.loaded = undefined;
      this.stopCalls = 0;
      youtubePlayers.push(this);
    }

    ready() { this.options.events.onReady({ target: this }); }
    loadVideoById(options) { this.loaded = options; }
    playVideo() {}
    stopVideo() { this.stopCalls += 1; }
    emitState(data) { this.options.events.onStateChange({ data }); }
  }

  const window = {
    location,
    innerWidth: 640,
    innerHeight: 360,
    addEventListener() {},
    clearTimeout() {},
    setTimeout(callback, delay) { timers.push({ callback, delay }); return timers.length; },
    YT: { Player: FakeYoutubePlayer, PlayerState: { ENDED: 0 } },
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
    runTimerByDelay(delay) {
      const index = timers.findIndex((timer) => timer.delay === delay);
      if (index < 0) return false;
      timers.splice(index, 1)[0].callback();
      return true;
    },
    timerDelays: () => timers.map((timer) => timer.delay),
    socket: sockets[0],
    sockets,
    youtubePlayers,
  };
}

function sendMediaClock(harness, payload = {}, socket = harness.sockets.at(-1)) {
  socket.emit("message", JSON.stringify({
    type: "mediaClock",
    payload: {
      videoBusy: false,
      audioBusy: false,
      ...payload,
    },
  }));
}

function sendMediaGrant(
  harness,
  granted,
  clock = {},
  socket = harness.sockets.at(-1),
) {
  socket.emit("message", JSON.stringify({
    type: "mediaGrant",
    payload: {
      granted,
      clock: { videoBusy: false, audioBusy: false, ...clock },
    },
  }));
}

function sendMedia(harnesses, media) {
  const targets = Array.isArray(harnesses) ? harnesses : [harnesses];
  for (const harness of targets) {
    harness.socket.emit("message", JSON.stringify({ type: "media", payload: media }));
  }
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

test("video fitting preserves its aspect ratio without touching the WebView2 viewport edge", () => {
  const { context, elements } = createHarness();
  vm.runInContext("fitVisualToViewport(videoElement, 408, 720, VIDEO_COMPOSITOR_INSET_PX)", context);
  assert.equal(elements["#video"].style.width, "auto");
  assert.equal(elements["#video"].style.height, "calc(100% - 4px)");

  vm.runInContext("fitVisualToViewport(videoElement, 1280, 720, VIDEO_COMPOSITOR_INSET_PX)", context);
  assert.equal(elements["#video"].style.width, "calc(100% - 4px)");
  assert.equal(elements["#video"].style.height, "auto");

  vm.runInContext("fitVisualToViewport(imageElement, 408, 720)", context);
  assert.equal(elements["#image"].style.height, "100%");
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

test("media messages are bounded upstream and independently visible in OBS and widgets", () => {
  const obs = createHarness();
  obs.socket.emit("message", JSON.stringify({
    type: "config",
    payload: { showMediaTextObs: true, showMediaTextWidget: false },
  }));
  obs.socket.emit("message", JSON.stringify({
    type: "media",
    payload: {
      kind: "image",
      url: "https://cdn.discordapp.com/setup.png",
      text: "<strong>Regardez mon setup</strong>",
    },
  }));
  obs.elements["#image"].naturalWidth = 1280;
  obs.elements["#image"].naturalHeight = 720;
  obs.elements["#image"].rect = {
    left: 100,
    top: 10,
    right: 500,
    bottom: 340,
    width: 400,
    height: 330,
  };
  obs.elements["#image"].emit("load");

  assert.equal(
    obs.elements["#media-text"].textContent,
    "<strong>Regardez mon setup</strong>",
  );
  assert.equal(obs.elements["#media-text"].hidden, false);
  assert.equal(obs.elements["#media-text"].classList.contains("is-visible"), true);
  assert.equal(obs.elements["#media-text"].style.left, "113px");
  assert.equal(obs.elements["#media-text"].style.bottom, "33px");
  assert.equal(obs.elements["#media-text"].style.maxWidth, "374px");

  const widget = createHarness("?secret=private&widget=1");
  widget.socket.emit("message", JSON.stringify({
    type: "config",
    payload: { showMediaTextObs: true, showMediaTextWidget: false },
  }));
  widget.socket.emit("message", JSON.stringify({
    type: "media",
    payload: {
      kind: "image",
      url: "https://cdn.discordapp.com/setup.png",
      text: "Widget caption",
    },
  }));
  assert.equal(widget.elements["#media-text"].hidden, true);

  widget.socket.emit("message", JSON.stringify({
    type: "config",
    payload: { showMediaTextWidget: true },
  }));
  assert.equal(widget.elements["#media-text"].textContent, "Widget caption");
  assert.equal(widget.elements["#media-text"].hidden, false);
});

test("audio captions stay in the Now Playing card instead of covering the artwork", () => {
  const harness = createHarness();
  harness.socket.emit("message", JSON.stringify({
    type: "config",
    payload: { showMediaTextObs: true },
  }));
  harness.socket.emit("message", JSON.stringify({
    type: "media",
    payload: {
      kind: "audio",
      url: "https://cdn.discordapp.com/voice.ogg",
      title: "Stay With Me",
      text: "Message attached to the audio",
    },
  }));

  assert.equal(harness.elements["#audio-media-text"].textContent, "Message attached to the audio");
  assert.equal(harness.elements["#audio-media-text"].hidden, false);
  assert.equal(harness.elements["#media-text"].hidden, true);
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

test("YouTube music uses the official IFrame player and rejects stale stops", async () => {
  const audio = createHarness("?secret=private", "audio");
  audio.socket.emit("message", JSON.stringify({
    type: "musicPlay",
    payload: {
      playbackId: "p1",
      videoId: "dQw4w9WgXcQ",
      startSeconds: 0,
      endSeconds: 30,
    },
  }));
  await Promise.resolve();
  const player = audio.youtubePlayers[0];
  assert.ok(player);
  player.ready();
  assert.equal(JSON.stringify(player.loaded), JSON.stringify({
    videoId: "dQw4w9WgXcQ",
    startSeconds: 0,
    endSeconds: 30,
  }));

  audio.socket.emit("message", JSON.stringify({
    type: "musicPlay",
    payload: { playbackId: "p2", videoId: "9bZkp7q19f0", startSeconds: 0 },
  }));
  await Promise.resolve();
  assert.equal(player.stopCalls, 1);
  assert.equal(vm.runInContext("youtubePlaybackId", audio.context), "p2");

  audio.socket.emit("message", JSON.stringify({
    type: "musicStop",
    payload: { playbackId: "p1" },
  }));
  assert.equal(vm.runInContext("youtubePlaybackId", audio.context), "p2");
  player.emitState(0);
  assert.deepEqual(audio.socket.sent.at(-1), {
    type: "musicEnded",
    payload: { playbackId: "p2" },
  });
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

test("desktop media widget keeps video visible when WebView2 blocks audible autoplay", async () => {
  const { elements, socket } = createHarness("?secret=private&widget=1");
  const video = elements["#video"];
  let playCalls = 0;
  video.play = () => {
    playCalls += 1;
    return playCalls === 1
      ? Promise.reject({ name: "NotAllowedError" })
      : Promise.resolve();
  };

  socket.emit("message", JSON.stringify({
    type: "config",
    payload: { widgetSoundEnabled: true, mediaVolume: 69 },
  }));
  socket.emit("message", JSON.stringify({
    type: "media",
    payload: { kind: "video", url: "https://cdn.discordapp.com/clip.mp4" },
  }));
  video.emit("loadeddata");
  await Promise.resolve();

  assert.equal(playCalls, 2);
  assert.equal(video.muted, true);
  assert.equal(video.volume, 0);
  assert.equal(video.classList.contains("is-visible"), true);
});

test("desktop media widget restores active images, GIFs, video and audio after hide and show", () => {
  for (const [selector, media] of [
    ["#image", { kind: "image", url: "https://cdn.discordapp.com/photo.png" }],
    ["#video", { kind: "gif", contentType: "video/mp4", url: "https://cdn.discordapp.com/loop.mp4" }],
    ["#video", { kind: "video", url: "https://cdn.discordapp.com/clip.mp4" }],
    ["#audio", { kind: "audio", url: "https://cdn.discordapp.com/track.mp3" }],
  ]) {
    const widget = createHarness("?secret=private&widget=1");
    widget.socket.emit("message", JSON.stringify({ type: "media", payload: media }));
    vm.runInContext("window.setWidgetVisible(false)", widget.context);
    assert.equal(widget.elements[selector].src, "");
    assert.equal(widget.socket.readyState, 3);
    vm.runInContext("window.setWidgetVisible(true)", widget.context);
    assert.equal(widget.elements[selector].src, media.url);
    assert.equal(widget.sockets.length, 2);
    widget.socket.emit("close");
    assert.equal(widget.elements[selector].src, media.url);
  }
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

test("audio playback reports live state and follows player controls", async () => {
  const harness = createHarness("?secret=private", "audio");
  const { elements, socket } = harness;
  sendMediaClock(harness);
  const media = {
    kind: "audio", url: "https://cdn.discordapp.com/track.mp3", filename: "track.mp3",
  };
  socket.emit("message", JSON.stringify({ type: "media", payload: media }));
  sendMediaGrant(harness, true, { audioBusy: true });
  elements["#audio"].emit("loadeddata");
  elements["#audio"].emit("playing");
  assert.equal(socket.sent.at(-1).type, "audioPlayback");
  assert.equal(socket.sent.at(-1).payload.status, "playing");

  socket.emit("message", JSON.stringify({ type: "audioControl", payload: { action: "pause" } }));
  assert.equal(socket.sent.at(-1).payload.status, "paused");
  assert.ok(elements["#audio"].pauseCalls > 0);

  socket.emit("message", JSON.stringify({ type: "audioControl", payload: { action: "resume" } }));
  await Promise.resolve();
  assert.equal(socket.sent.at(-1).payload.status, "playing");

  socket.emit("message", JSON.stringify({ type: "audioControl", payload: { action: "skip" } }));
  assert.equal(socket.sent.at(-1).payload.status, "idle");
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

test("images, GIFs and videos wait for the exclusive OBS visual lease", () => {
  const cases = [
    {
      label: "image",
      media: { kind: "image", url: "https://cdn.discordapp.com/still.png" },
      selector: "#image",
    },
    {
      label: "GIF",
      media: { kind: "gif", contentType: "image/gif", url: "https://cdn.discordapp.com/loop.gif" },
      selector: "#image",
    },
    {
      label: "video GIF",
      media: { kind: "gif", contentType: "video/mp4", url: "https://media.klipy.com/loop.mp4" },
      selector: "#video",
    },
    {
      label: "video",
      media: { kind: "video", url: "https://cdn.discordapp.com/clip.mp4" },
      selector: "#video",
    },
  ];

  for (const { label, media, selector } of cases) {
    const visual = createHarness("", "visual");
    sendMediaClock(visual, { audioBusy: true });
    sendMedia(visual, media);
    assert.equal(visual.elements[selector].src, "", `${label} started over active OBS audio`);

    sendMediaClock(visual, { audioBusy: false });
    assert.deepEqual(
      visual.socket.sent.at(-1),
      { type: "mediaClock", payload: { busy: true } },
      `${label} did not request the OBS visual lease`,
    );
    assert.equal(visual.elements[selector].src, "");

    sendMediaGrant(visual, true, { videoBusy: true });
    assert.equal(visual.elements[selector].src, media.url);
  }
});

test("audio waits for the active video lease before starting", () => {
  const visual = createHarness("", "visual");
  const audio = createHarness("", "audio");
  const video = { kind: "video", url: "https://cdn.discordapp.com/clip.mp4" };
  const track = { kind: "audio", url: "https://cdn.discordapp.com/track.mp3" };

  sendMediaClock(visual);
  sendMediaClock(audio);
  sendMedia([visual, audio], video);
  assert.equal(visual.elements["#video"].src, "");
  assert.deepEqual(visual.socket.sent.at(-1), { type: "mediaClock", payload: { busy: true } });

  sendMediaGrant(visual, true, { videoBusy: true });
  sendMediaClock(audio, { videoBusy: true });
  sendMedia([visual, audio], track);
  visual.elements["#video"].emit("loadeddata");
  assert.equal(visual.elements["#video"].src, video.url);
  assert.equal(audio.elements["#audio"].src, "");

  visual.elements["#video"].emit("ended");
  assert.equal(visual.runTimerByDelay(320), true);
  assert.deepEqual(visual.socket.sent.at(-1), { type: "mediaClock", payload: { busy: false } });
  sendMediaClock(audio, { videoBusy: false });
  assert.deepEqual(audio.socket.sent.at(-1), { type: "mediaClock", payload: { busy: true } });
  assert.equal(audio.elements["#audio"].src, "");

  sendMediaGrant(audio, true, { audioBusy: true });
  assert.equal(audio.elements["#audio"].src, track.url);
  assert.equal(visual.elements["#video"].src, "");
});

test("video waits for the active audio lease before starting", () => {
  const visual = createHarness("", "visual");
  const audio = createHarness("", "audio");
  const track = { kind: "audio", url: "https://cdn.discordapp.com/track.mp3" };
  const video = { kind: "video", url: "https://cdn.discordapp.com/clip.mp4" };

  sendMediaClock(visual);
  sendMediaClock(audio);
  sendMedia([visual, audio], track);
  sendMediaGrant(audio, true, { audioBusy: true });
  sendMediaClock(visual, { audioBusy: true });
  sendMedia([visual, audio], video);

  assert.equal(audio.elements["#audio"].src, track.url);
  assert.equal(visual.elements["#video"].src, "");

  audio.elements["#audio"].emit("ended");
  assert.equal(audio.runTimerByDelay(320), true);
  sendMediaClock(visual, { audioBusy: false });
  assert.deepEqual(visual.socket.sent.at(-1), { type: "mediaClock", payload: { busy: true } });

  sendMediaGrant(visual, true, { videoBusy: true });
  assert.equal(visual.elements["#video"].src, video.url);
  assert.equal(audio.elements["#audio"].src, "");
});

test("reconnecting split outputs require one exclusive lease before resuming", () => {
  const visual = createHarness("", "visual");
  const audio = createHarness("", "audio");

  sendMediaClock(visual);
  sendMediaClock(audio);
  sendMedia(visual, { kind: "video", url: "https://cdn.discordapp.com/lost.mp4" });
  sendMedia(visual, { kind: "video", url: "https://cdn.discordapp.com/resume.mp4" });
  sendMedia(audio, { kind: "audio", url: "https://cdn.discordapp.com/lost.mp3" });
  sendMedia(audio, { kind: "audio", url: "https://cdn.discordapp.com/resume.mp3" });
  visual.socket.emit("close");
  audio.socket.emit("close");
  assert.equal(visual.runTimerByDelay(1000), true);
  assert.equal(audio.runTimerByDelay(1000), true);

  const visualSocket = visual.sockets.at(-1);
  const audioSocket = audio.sockets.at(-1);
  visualSocket.emit("open");
  audioSocket.emit("open");
  sendMediaClock(visual, {}, visualSocket);
  sendMediaClock(audio, {}, audioSocket);
  assert.equal(visual.elements["#video"].src, "");
  assert.equal(audio.elements["#audio"].src, "");

  sendMediaGrant(visual, true, { videoBusy: true }, visualSocket);
  sendMediaGrant(audio, false, { videoBusy: true }, audioSocket);
  assert.equal(visual.elements["#video"].src, "https://cdn.discordapp.com/resume.mp4");
  assert.equal(audio.elements["#audio"].src, "");
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
