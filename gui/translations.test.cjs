const assert = require("node:assert/strict");
const fs = require("node:fs");
const test = require("node:test");
const vm = require("node:vm");

const panelSource = fs.readFileSync(__dirname + "/panel.js", "utf8");
const panelHtml = fs.readFileSync(__dirname + "/panel.html", "utf8");
const dictionarySource = panelSource.slice(
  panelSource.indexOf("const translations ="),
  panelSource.indexOf("const pageMetadata ="),
);
const context = vm.createContext({});
vm.runInContext(`${dictionarySource}\nglobalThis.translationsForTest = translations;`, context);
const translations = context.translationsForTest;

test("French, Spanish and German provide every interface translation key", () => {
  const expectedKeys = Object.keys(translations.en).sort();
  for (const language of ["fr", "es", "de"]) {
    assert.deepEqual(Object.keys(translations[language]).sort(), expectedKeys, language);
  }
});

test("reported Spanish and German pages no longer use English fallback copy", () => {
  for (const language of ["es", "de"]) {
    for (const key of [
      "overviewTitle", "credentialsTitle", "routingTitle", "moderationTitle",
      "moderationSettings", "historyKicker", "historyTitle", "historyCopy",
    ]) {
      assert.notEqual(translations[language][key], translations.en[key], `${language}.${key}`);
    }
  }
});

test("translation values contain no UTF-8 mojibake", () => {
  for (const language of Object.keys(translations)) {
    assert.doesNotMatch(Object.values(translations[language]).join("\n"), /Ã.|â€|Â./, language);
  }
});

test("the top bar audio player exposes only previous, pause and skip controls", () => {
  assert.match(panelHtml, /id="now-playing"[\s\S]*id="previous-audio"[\s\S]*id="toggle-audio"[\s\S]*id="skip-audio"/);
  assert.doesNotMatch(panelHtml, /shuffle-audio|repeat-audio|next-audio/);
  assert.match(panelSource, /message\.type === "audioPlayback"/);
  assert.match(panelSource, /invoke\("control_audio"/);
});
