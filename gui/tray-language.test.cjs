const assert = require("node:assert/strict");
const fs = require("node:fs");
const test = require("node:test");
const vm = require("node:vm");

const traySource = fs.readFileSync(__dirname + "/tray.js", "utf8");
const dictionarySource = traySource.slice(
  traySource.indexOf("const trayTranslations ="),
  traySource.indexOf("let language ="),
);
const context = vm.createContext({});
vm.runInContext(`${dictionarySource}\nglobalThis.trayTranslationsForTest = trayTranslations;`, context);
const translations = context.trayTranslationsForTest;

test("tray translations cover every interface language", () => {
  const expectedKeys = Object.keys(translations.en).sort();
  for (const language of ["fr", "es", "de"]) {
    assert.deepEqual(Object.keys(translations[language]).sort(), expectedKeys, language);
  }
});

test("tray refresh reads the language shared by the control panel", () => {
  assert.match(traySource, /localStorage\.getItem\("relay-language"\)/);
  assert.match(traySource, /async function refreshTray\(\) \{\s+applyTrayLanguage\(\)/);
  assert.match(traySource, /invoke\("get_start_with_windows"\)[\s\S]*"set_start_with_windows", \{ enabled \}/);
});
