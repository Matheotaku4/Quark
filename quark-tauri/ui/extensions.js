"use strict";
// Sites fournis par les extensions de recherche. Aucun site n'est actif tant
// que l'utilisateur n'a pas ajouté un manifeste dans Réglages > Extensions.

function extensionSites(data) {
  return Array.isArray(data?.searchSites) ? data.searchSites : [];
}

function extensionSearchUrl(site, query) {
  const template = String(site?.searchUrl || "");
  return template.replace("{query}", encodeURIComponent(String(query ?? "")));
}

function searchSites() {
  return extensionSites(typeof app === "undefined" ? null : app.data);
}

function hasSearchSites() {
  return searchSites().length > 0;
}

if (typeof module !== "undefined" && module.exports) {
  module.exports = { extensionSites, extensionSearchUrl };
}
