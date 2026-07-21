const assert = require("node:assert/strict");
const fs = require("node:fs");
const test = require("node:test");

const panelHtml = fs.readFileSync(__dirname + "/panel.html", "utf8");
const panelSource = fs.readFileSync(__dirname + "/panel.js", "utf8");
const panelCss = fs.readFileSync(__dirname + "/panel.css", "utf8");
const traySource = fs.readFileSync(__dirname + "/tray.js", "utf8");
const trayCss = fs.readFileSync(__dirname + "/tray.css", "utf8");
const designs = ["openai", "anthropic", "neo-brutalism"];

test("personalization exposes the three accessible design choices", () => {
  assert.match(panelHtml, /<fieldset class="design-picker">/);
  assert.match(panelHtml, /<html[^>]+data-design="openai"/);
  for (const design of designs) {
    assert.match(panelHtml, new RegExp(`name="interface-design" value="${design}"`));
  }
});

test("the selected design is persisted and applied without changing the native preference schema", () => {
  assert.match(panelSource, /const supportedDesigns = \["openai", "anthropic", "neo-brutalism"\]/);
  assert.match(panelSource, /localStorage\.getItem\("relay-design"\)/);
  assert.match(panelSource, /localStorage\.setItem\("relay-design", design\)/);
  assert.match(panelSource, /document\.documentElement\.dataset\.design = design/);
  assert.match(panelSource, /language, theme, accentRgb, fontScale,/);
});

test("each art direction covers light, dark, focus, responsive and reduced-motion states", () => {
  for (const design of designs.slice(1)) {
    assert.match(panelCss, new RegExp(`data-design="${design}"`));
    assert.match(trayCss, new RegExp(`data-design="${design}"`));
  }
  assert.match(panelCss, /data-design="anthropic"\]\[data-theme="dark"\]/);
  assert.match(panelCss, /data-design="neo-brutalism"\]\[data-theme="dark"\]/);
  assert.match(panelCss, /design-choice:has\(input:focus-visible\)/);
  assert.match(panelCss, /@media \(max-width: 700px\)/);
  assert.match(panelCss, /@media \(prefers-reduced-motion: reduce\)/);
});

test("tray refresh reads the design and theme shared by personalization", () => {
  assert.match(traySource, /localStorage\.getItem\("relay-design"\)/);
  assert.match(traySource, /localStorage\.getItem\("relay-theme"\)/);
  assert.match(traySource, /document\.documentElement\.dataset\.design/);
  assert.match(traySource, /document\.documentElement\.dataset\.theme/);
});
