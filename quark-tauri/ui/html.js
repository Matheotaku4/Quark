(function (global) {
  "use strict";

  const ESCAPES = { "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" };
  const esc = (value) => String(value ?? "").replace(/[&<>"']/g, (c) => ESCAPES[c]);
  const RAW = Symbol("raw");

  function raw(value) {
    return { [RAW]: String(value ?? "") };
  }

  function html(strings, ...values) {
    let out = "";
    strings.forEach((part, index) => {
      out += part;
      if (index >= values.length) return;
      const value = values[index];
      if (Array.isArray(value)) out += value.map(render).join("");
      else out += render(value);
    });
    return out;
  }

  function render(value) {
    if (value && typeof value === "object" && RAW in value) return value[RAW];
    return esc(value);
  }

  const NAMED = new Set([
    "transparent", "currentcolor", "black", "white", "red", "orange", "yellow",
    "green", "blue", "purple", "pink", "gray", "grey", "silver", "gold",
    "cyan", "magenta", "lime", "navy", "teal", "maroon", "olive",
  ]);

  function cssColor(value, fallback = "transparent") {
    const text = String(value ?? "").trim().toLowerCase();
    if (NAMED.has(text)) return text;
    if (/^#[0-9a-f]{3,8}$/.test(text)) return text;
    if (/^(rgb|hsl)a?\([0-9.,%\s/]+\)$/.test(text)) return text;
    return fallback;
  }

  const api = { esc, html, raw, cssColor };
  global.QuarkHTML = api;
  if (typeof module !== "undefined") module.exports = api;
})(typeof globalThis !== "undefined" ? globalThis : this);
