const assert = require("node:assert/strict");
const fs = require("node:fs");
const test = require("node:test");

const panelHtml = fs.readFileSync(__dirname + "/panel.html", "utf8");
const panelSource = fs.readFileSync(__dirname + "/panel.js", "utf8");
const locales = ["en-US", "en-GB", "en-IN", "fr-FR", "de-DE", "es-ES", "es-419"];
const flags = ["us", "gb", "in", "fr", "de", "es", "mx"];

test("the language picker exposes every supported regional locale", () => {
  for (const locale of locales) {
    assert.match(panelHtml, new RegExp(`data-locale="${locale}"`), locale);
    assert.match(panelSource, new RegExp(`locale: "${locale}"`), locale);
  }
  assert.match(panelSource, /language = activeLanguageOption\(\)\.language/);
  assert.match(panelSource, /localStorage\.setItem\("relay-locale", locale\)/);
});

test("every regional locale uses a bundled SVG flag", () => {
  for (const flag of flags) {
    const path = `${__dirname}/assets/flags/${flag}.svg`;
    assert.ok(fs.existsSync(path), path);
    assert.match(fs.readFileSync(path, "utf8"), /<svg[^>]+viewBox="0 0 24 16"/);
    assert.match(panelHtml, new RegExp(`assets/flags/${flag}\\.svg`), flag);
  }
});

test("regional variants keep the four complete translation dictionaries", () => {
  assert.match(panelSource, /locale: "en-GB", language: "en"/);
  assert.match(panelSource, /locale: "en-IN", language: "en"/);
  assert.match(panelSource, /locale: "es-419", language: "es"/);
  assert.doesNotMatch(panelHtml, /data-locale="(?:ru|ja|ko|zh|pt)-/);
});
