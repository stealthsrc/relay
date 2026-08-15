const assert = require("node:assert/strict");
const fs = require("node:fs");
const test = require("node:test");

const panelHtml = fs.readFileSync(__dirname + "/panel.html", "utf8");
const panelSource = fs.readFileSync(__dirname + "/panel.js", "utf8");
const panelCss = fs.readFileSync(__dirname + "/panel.css", "utf8");

test("sidebar layout preference persists fixed, compact and dynamic modes", () => {
  assert.match(panelHtml, /id="sidebar-layout"/);
  assert.match(panelHtml, /option value="fixed" data-i18n="sidebarLayoutFixed"/);
  assert.match(panelHtml, /option value="compact" data-i18n="sidebarLayoutCompact"/);
  assert.match(panelHtml, /option value="dynamic" data-i18n="sidebarLayoutDynamic"/);
  assert.match(panelSource, /const supportedSidebarLayouts = \["fixed", "compact", "dynamic"\]/);
  assert.match(panelSource, /localStorage\.getItem\("relay-sidebar-layout"\)/);
  assert.match(panelSource, /localStorage\.setItem\("relay-sidebar-layout", sidebarLayout\)/);
  assert.match(panelSource, /sidebarLayout === "dynamic"\s*\? \(sidebarExpanded \? "fixed" : "compact"\)/);
  assert.match(panelSource, /document\.documentElement\.dataset\.sidebarBehavior = sidebarLayout/);
  assert.match(panelSource, /sidebarElement\.addEventListener\("pointerenter", \(\) => setDynamicSidebarExpanded\(true\)\)/);
  assert.match(panelSource, /sidebarElement\.addEventListener\("pointerleave", \(\) => setDynamicSidebarExpanded\(false\)\)/);
});

test("compact navigation retains numbered icons and restores labels on mobile", () => {
  for (const target of ["overview", "media", "overlay", "moderation", "commands", "history", "help", "personalization", "about"]) {
    assert.match(panelHtml, new RegExp(`data-page-target="${target}"[\\s\\S]*?navigation__icon`), target);
  }
  assert.match(panelHtml, /class="navigation__label" data-i18n="navOverview"/);
  assert.match(panelCss, /data-sidebar-layout="compact"/);
  assert.match(panelCss, /--sidebar-width: 84px/);
  assert.match(panelCss, /data-sidebar-layout="compact"\] \.navigation__index/);
  assert.match(panelCss, /data-sidebar-layout="compact"\] \.navigation__label/);
  assert.match(panelCss, /data-sidebar-layout="compact"\] #language-value \{\s*display: none;/);
  assert.match(panelCss, /sidebar-language-picker \.sidebar-language-picker__options \{\s*top: auto;/);
  assert.match(panelCss, /data-sidebar-behavior="dynamic"\] \.app-shell/);
  assert.match(panelCss, /@media \(max-width: 700px\)[\s\S]*data-sidebar-layout="compact"\] \.navigation__label/);
  assert.match(panelSource, /languageToggleButton\.title = selected\.label/);
});
