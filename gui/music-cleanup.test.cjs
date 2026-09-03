const assert = require("node:assert/strict");
const fs = require("node:fs");
const test = require("node:test");
const vm = require("node:vm");

function setup() {
  const elements = new Map();
  const element = (id) => {
    if (!elements.has(id)) elements.set(id, { hidden: true, disabled: false, textContent: "",
      handlers: {}, addEventListener(name, callback) { this.handlers[name] = callback; } });
    return elements.get(id);
  };
  const calls = [];
  const musicForm = element("form");
  const dirtyForms = new Set();
  const source = fs.readFileSync(__dirname + "/panel.js", "utf8");
  const block = source.slice(source.indexOf("let musicCleanupToken;"), source.indexOf('refreshChannelsButton.addEventListener("click"'));
  const context = vm.createContext({ $: element, musicForm, dirtyForms,
    t: (key) => key,
    invoke: async (command, args) => {
      calls.push({ command, args });
      return command === "preview_music_cleanup" ? { token: "snapshot-7", count: 3 } : { deleted: 3, failed: 0, skipped: 0 };
    },
  });
  vm.runInContext(block, context);
  return { element, calls, musicForm, dirtyForms };
}

test("preview requires a separate confirmation and sends only its snapshot token", async () => {
  const ui = setup();
  await ui.element("#music-cleanup-preview").handlers.click();
  assert.equal(ui.calls.length, 1);
  assert.equal(ui.element("#music-cleanup-confirmation").hidden, false);
  await ui.element("#music-cleanup-confirm").handlers.click();
  assert.equal(ui.calls[1].command, "confirm_music_cleanup");
  assert.equal(ui.calls[1].args.token, "snapshot-7");
  await ui.element("#music-cleanup-confirm").handlers.click();
  assert.equal(ui.calls.length, 2);
});

test("cancel and settings edits invalidate the cleanup confirmation", async () => {
  for (const cancel of [true, false]) {
    const ui = setup();
    await ui.element("#music-cleanup-preview").handlers.click();
    if (cancel) ui.element("#music-cleanup-cancel").handlers.click();
    else ui.musicForm.handlers.input();
    await ui.element("#music-cleanup-confirm").handlers.click();
    assert.equal(ui.calls.length, 1);
  }
});

test("unsaved music settings prevent cleanup previews", async () => {
  const ui = setup();
  ui.dirtyForms.add(ui.musicForm);
  await ui.element("#music-cleanup-preview").handlers.click();
  assert.equal(ui.calls.length, 0);
});
