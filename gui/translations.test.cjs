const assert = require("node:assert/strict");
const fs = require("node:fs");
const test = require("node:test");
const vm = require("node:vm");

const panelSource = fs.readFileSync(__dirname + "/panel.js", "utf8");
const dictionarySource = panelSource.slice(
  panelSource.indexOf("const translations ="),
  panelSource.indexOf("const pageMetadata ="),
);
const context = vm.createContext({});
vm.runInContext(`${dictionarySource}\nglobalThis.translationsForTest = translations;`, context);
const translations = context.translationsForTest;

test("Spanish and German provide every interface translation key", () => {
  const expectedKeys = Object.keys(translations.en).sort();
  assert.deepEqual(Object.keys(translations.es).sort(), expectedKeys);
  assert.deepEqual(Object.keys(translations.de).sort(), expectedKeys);
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
