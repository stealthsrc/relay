const assert = require("node:assert/strict");
const fs = require("node:fs");
const test = require("node:test");

const styles = fs.readFileSync(__dirname + "/overlay.css", "utf8");

test("compact widget captions preserve their content width", () => {
  assert.match(styles, /\.overlay__caption\s*\{[\s\S]*?width:\s*fit-content;/);

  const mobileCaption = styles.match(
    /@media \(max-width: 640px\)\s*\{\s*\.overlay__caption\s*\{([\s\S]*?)\n\s*\}/,
  );
  assert.ok(mobileCaption);
  assert.doesNotMatch(mobileCaption[1], /\bright\s*:/);
  assert.match(mobileCaption[1], /max-width:\s*min\(34ch, calc\(100% - 2rem\)\)/);
});
