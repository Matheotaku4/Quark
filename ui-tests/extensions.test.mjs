import test from "node:test";
import assert from "node:assert/strict";
import { createRequire } from "node:module";
const require = createRequire(import.meta.url);
const { extensionSites, extensionSearchUrl } = require("../quark-tauri/ui/extensions.js");

test("liste vide sans extension", () => {
  assert.deepEqual(extensionSites({}), []);
  assert.deepEqual(extensionSites({ searchSites: [] }), []);
});

test("lit les sites fournis par le manifeste", () => {
  const sites = [{ uid: "quark-sites/gamebounty", label: "GameBounty" }];
  assert.deepEqual(extensionSites({ searchSites: sites }), sites);
});

test("construit l'URL de recherche avec encodage", () => {
  const site = { searchUrl: "https://example.com/search?q={query}" };
  assert.equal(
    extensionSearchUrl(site, "Portal 2 & co"),
    "https://example.com/search?q=Portal%202%20%26%20co",
  );
});

test("une adresse absente ne casse pas le bouton", () => {
  assert.equal(extensionSearchUrl({}, "Portal"), "");
});
