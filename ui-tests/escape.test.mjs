import test from "node:test";
import assert from "node:assert/strict";
import { createRequire } from "node:module";
const require = createRequire(import.meta.url);
const { html, raw, cssColor } = require("../quark-tauri/ui/html.js");

test("échappe une valeur distante", () => {
  assert.equal(
    html`<b>${"<img src=x onerror=alert(1)>"}</b>`,
    "<b>&lt;img src=x onerror=alert(1)&gt;</b>",
  );
});

test("raw conserve le HTML de confiance", () => {
  assert.equal(html`<i>${raw("<svg></svg>")}</i>`, "<i><svg></svg></i>");
});

test("refuse une couleur injectée", () => {
  assert.equal(cssColor("red; background:url(//evil)"), "transparent");
  assert.equal(cssColor("#ff0000"), "#ff0000");
});
