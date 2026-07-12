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
  };
}

function createHarness() {
  const elements = {
    "#sticker": {
      alt: "",
      classList: classList(),
      onerror: null,
      onload: null,
      src: "",
      removeAttribute(name) { if (name === "src") this.src = ""; },
    },
    "#sticker-fallback": {
      classList: classList(),
      hidden: true,
    },
  };
  const sockets = [];
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
  const window = {
    location: {
      protocol: "http:", host: "127.0.0.1:4590", port: "4590",
      href: "http://127.0.0.1:4590/stickers?secret=private", search: "?secret=private",
      replace() {},
    },
    addEventListener() {},
    clearTimeout() {},
    requestAnimationFrame(callback) { callback(); },
    setTimeout() { return 1; },
  };
  const context = vm.createContext({
    URL,
    URLSearchParams,
    WebSocket: MockWebSocket,
    document: { querySelector: (selector) => elements[selector] },
    encodeURIComponent,
    window,
  });
  vm.runInContext(fs.readFileSync(__dirname + "/stickers.js", "utf8"), context);
  return { elements, socket: sockets[0] };
}

test("sticker output identifies itself and accepts isolated tests", () => {
  const { elements, socket } = createHarness();
  assert.match(socket.url, /role=sticker&source=sticker&client=obs&secret=private$/);

  socket.emit("message", JSON.stringify({
    type: "testOutput",
    payload: {
      target: "sticker",
      sticker: {
        name: "Relay sticker test",
        format: "png",
        url: "/overlay-assets/relay-radar.png",
      },
    },
  }));

  assert.equal(elements["#sticker"].src, "/overlay-assets/relay-radar.png");
  assert.equal(elements["#sticker"].alt, "Relay sticker test");
});
