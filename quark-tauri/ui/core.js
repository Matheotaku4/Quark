"use strict";
const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

Object.assign(I18N.fr, {
  "x.home": "Accueil", "x.store": "Boutique", "x.mygames": "Mes jeux", "x.inGame": "En jeu", "x.palette": "Aller à un jeu ou une action…",
  "x.none": "Aucun résultat", "x.filters": "Filtres", "x.groupBy": "Grouper", "x.back": "Bibliothèque", "x.news": "Actualités", "x.ach": "Succès", "x.hist": "Historique",
  "x.overview": "Aperçu", "x.refresh": "Actualiser", "x.noNews": "Aucune actualité pour ce jeu.", "x.noAppId": "Ce jeu n'a pas d'identifiant Steam : les actualités et les notes ne sont pas disponibles.",
  "x.settingsGame": "Paramètres du jeu", "x.customize": "Personnaliser", "x.logBtn": "Journal de la session", "x.remove": "Retirer de la bibliothèque", "x.weeks": "12 dernières semaines",
  "x.main": "Histoire principale", "x.extra": "Histoire + extras", "x.compl": "Complétionniste", "x.hltbLink": "Voir sur HowLongToBeat", "x.search": "Rechercher", "x.updates": "Mises à jour",
  "x.view": "Affichage", "x.add": "Ajouter un jeu", "x.soon": "Bientôt", "x.soonBody": "Cet écran arrive dans le prochain lot du portage.", "x.sessions": "Sessions", "x.allStatuses": "Tous les statuts",
  "x.allGenres": "Tous les genres", "x.allTags": "Tous les tags", "x.favOnly": "Favoris uniquement", "x.reset": "Réinitialiser", "x.continue": "Continuer", "x.favorites": "Favoris", "x.backlog": "À jouer",
  "x.save": "Enregistrer", "x.general": "Général", "x.proton": "Proton", "x.env": "Variables", "x.data": "Données", "x.dll": "DLL overrides", "x.settingsTab": "Réglages",
  "x.noEnv": "Aucune variable.", "x.addVar": "Ajouter une variable", "x.addDll": "Ajouter une DLL", "x.noDll": "Aucun override.", "x.native": "native", "x.builtin": "builtin",
});
Object.assign(I18N.en, {
  "x.home": "Home", "x.store": "Store", "x.mygames": "My games", "x.inGame": "Playing", "x.palette": "Jump to a game or action…",
  "x.none": "No results", "x.filters": "Filters", "x.groupBy": "Group by", "x.back": "Library", "x.news": "News", "x.ach": "Achievements", "x.hist": "History",
  "x.overview": "Overview", "x.refresh": "Refresh", "x.noNews": "No news for this game.", "x.noAppId": "This game has no Steam id: news and ratings are unavailable.",
  "x.settingsGame": "Game settings", "x.customize": "Customize", "x.logBtn": "Session log", "x.remove": "Remove from library", "x.weeks": "Last 12 weeks",
  "x.main": "Main story", "x.extra": "Main + extras", "x.compl": "Completionist", "x.hltbLink": "View on HowLongToBeat", "x.search": "Search", "x.updates": "Updates",
  "x.view": "Layout", "x.add": "Add a game", "x.soon": "Coming soon", "x.soonBody": "This screen arrives in the next batch of the port.", "x.sessions": "Sessions", "x.allStatuses": "All statuses",
  "x.allGenres": "All genres", "x.allTags": "All tags", "x.favOnly": "Favorites only", "x.reset": "Reset", "x.continue": "Continue", "x.favorites": "Favorites", "x.backlog": "Backlog",
  "x.save": "Save", "x.general": "General", "x.proton": "Proton", "x.env": "Variables", "x.data": "Data", "x.dll": "DLL overrides", "x.settingsTab": "Settings",
  "x.noEnv": "No variables.", "x.addVar": "Add a variable", "x.addDll": "Add a DLL", "x.noDll": "No overrides.", "x.native": "native", "x.builtin": "builtin",
});

const app = { data: null, cfg: {}, lang: "fr", view: "library", sel: null, query: "", f: { status: "", genre: "", tag: "", fav: false }, running: new Map(), tab: "overview", marked: new Set(),
  cache: {}, open: true, collapsed: new Set(), pal: null, bgGame: null, stab: "appearance", sq: "" };
const $ = (s, r = document) => r.querySelector(s);
const $$ = (s, r = document) => Array.from(r.querySelectorAll(s));
const esc = (v) => String(v ?? "").replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[c]);
const norm = (s) => String(s).normalize("NFD").replace(/[\u0300-\u036f]/g, "").toLowerCase();
const locale = () => LOCALES[app.lang] || "fr-FR";
function t(key, vars) {
  let s = (I18N[app.lang] || {})[key] ?? I18N.fr[key] ?? key;
  for (const [k, v] of Object.entries(vars || {})) s = s.split(`{${k}}`).join(v);
  return s;
}
const IC = {
  play: '<path d="M8 5v14l11-7z" fill="currentColor" stroke="none"/>', stop: '<rect x="6" y="6" width="12" height="12" rx="2" fill="currentColor" stroke="none"/>',
  star: '<path d="M12 3.5l2.7 5.6 6.1.9-4.4 4.3 1 6.1-5.4-2.9-5.4 2.9 1-6.1-4.4-4.3 6.1-.9z"/>', plus: '<path d="M12 5v14M5 12h14"/>',
  gear: '<circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.7 1.7 0 0 0 .3 1.8l.1.1a2 2 0 1 1-2.8 2.8l-.1-.1a1.7 1.7 0 0 0-1.8-.3 1.7 1.7 0 0 0-1 1.5V21a2 2 0 1 1-4 0v-.1a1.7 1.7 0 0 0-1.1-1.5 1.7 1.7 0 0 0-1.8.3l-.1.1a2 2 0 1 1-2.8-2.8l.1-.1a1.7 1.7 0 0 0 .3-1.8 1.7 1.7 0 0 0-1.5-1H3a2 2 0 1 1 0-4h.1a1.7 1.7 0 0 0 1.5-1.1 1.7 1.7 0 0 0-.3-1.8l-.1-.1a2 2 0 1 1 2.8-2.8l.1.1a1.7 1.7 0 0 0 1.8.3H9a1.7 1.7 0 0 0 1-1.5V3a2 2 0 1 1 4 0v.1a1.7 1.7 0 0 0 1 1.5 1.7 1.7 0 0 0 1.8-.3l.1-.1a2 2 0 1 1 2.8 2.8l-.1.1a1.7 1.7 0 0 0-.3 1.8V9a1.7 1.7 0 0 0 1.5 1H21a2 2 0 1 1 0 4h-.1a1.7 1.7 0 0 0-1.5 1z"/>',
  dots: '<circle cx="5" cy="12" r="1.4"/><circle cx="12" cy="12" r="1.4"/><circle cx="19" cy="12" r="1.4"/>', search: '<circle cx="11" cy="11" r="6.5"/><path d="m20 20-4.2-4.2"/>',
  min: '<path d="M6 12h12"/>', max: '<rect x="6" y="6" width="12" height="12" rx="1.5"/>', x: '<path d="M6 6l12 12M18 6 6 18"/>',
  folder: '<path d="M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v9a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/>', edit: '<path d="M16.5 3.5a2.1 2.1 0 0 1 3 3L7 19l-4 1 1-4z"/>',
  image: '<rect x="3" y="4" width="18" height="16" rx="2"/><circle cx="8.5" cy="9.5" r="1.5"/><path d="m21 15-5-5L5 21"/>', trash: '<path d="M3 6h18M8 6V4h8v2M6 6l1 14h10l1-14"/>',
  grid: '<rect x="4" y="4" width="7" height="7" rx="1.5"/><rect x="13" y="4" width="7" height="7" rx="1.5"/><rect x="4" y="13" width="7" height="7" rx="1.5"/><rect x="13" y="13" width="7" height="7" rx="1.5"/>',
  list: '<path d="M8 6h12M8 12h12M8 18h12"/><circle cx="4" cy="6" r="1"/><circle cx="4" cy="12" r="1"/><circle cx="4" cy="18" r="1"/>',
  down: '<path d="M12 4v11m0 0-4-4m4 4 4-4M5 20h14"/>', log: '<path d="M7 4h10a2 2 0 0 1 2 2v12a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2z"/><path d="M9 9h6M9 13h6M9 17h3"/>',
  filter: '<path d="M4 5h16l-6 8v6l-4-2v-4z"/>', caret: '<path d="m7 10 5 5 5-5"/>', refresh: '<path d="M21 12a9 9 0 1 1-2.6-6.4M21 3v6h-6"/>', back: '<path d="M15 6l-6 6 6 6"/>',
  ext: '<path d="M14 4h6v6M20 4l-9 9M10 6H6a2 2 0 0 0-2 2v10a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2v-4"/>',
  eye: '<path d="M2.5 12S6 5.5 12 5.5 21.5 12 21.5 12 18 18.5 12 18.5 2.5 12 2.5 12z"/><circle cx="12" cy="12" r="3"/>',
};
const ic = (n, c = "") => `<svg class="ic ${c}" viewBox="0 0 24 24">${IC[n]}</svg>`;

const hm = (s) => (!s || s < 60 ? "0 min" : s < 3600 ? `${Math.floor(s / 60)} min` : `${new Intl.NumberFormat(locale(), { maximumFractionDigits: 1 }).format(s / 3600)} h`);
const hrs = (v) => (v == null ? "—" : `${new Intl.NumberFormat(locale(), { maximumFractionDigits: 1 }).format(v)} h`);
const clock = (s) => { s = Math.max(0, Math.floor(s)); const p = (n) => String(n).padStart(2, "0"); return s >= 3600 ? `${Math.floor(s / 3600)}:${p(Math.floor(s / 60) % 60)}:${p(s % 60)}` : `${Math.floor(s / 60)}:${p(s % 60)}`; };
function rel(ts) {
  if (!ts) return t("detail.never");
  const d = ts - Date.now() / 1000, a = Math.abs(d), f = new Intl.RelativeTimeFormat(locale(), { numeric: "auto" });
  return a < 3600 ? f.format(Math.round(d / 60), "minute") : a < 86400 ? f.format(Math.round(d / 3600), "hour") : a < 5184000 ? f.format(Math.round(d / 86400), "day") : f.format(Math.round(d / 2592000), "month");
}
const date = (ts) => new Date(ts * 1000).toLocaleDateString(locale(), { day: "numeric", month: "short", year: "numeric" });
const elapsed = (id) => { const r = app.running.get(id); return r ? r.elapsed + (Date.now() - r.at) / 1000 : 0; };
const game = (id) => app.data.games.find((g) => g.id === id);
const accent = (g) => `--h:${g.hue};--g:hsl(${g.hue} 78% 66%)`;
const bytes = (n) => (n < 1024 ? `${n} B` : n < 1048576 ? `${(n / 1024).toFixed(0)} KB` : n < 1073741824 ? `${(n / 1048576).toFixed(1)} MB` : `${(n / 1073741824).toFixed(1)} GB`);

function setGame(g) { const i = app.data.games.findIndex((x) => x.id === g.id); i >= 0 ? (app.data.games[i] = g) : app.data.games.push(g); }
function setRunning(list) { app.running = new Map(list.map((r) => [r.id, { ...r, at: Date.now() }])); }
async function run(fn) { try { return await fn(); } catch (e) { toast(String(e), true); } }
function toast(msg, err) {
  const el = Object.assign(document.createElement("div"), { className: "toast" + (err ? " err" : ""), textContent: msg });
  $("#toasts").append(el); setTimeout(() => el.remove(), err ? 6000 : 2800);
}
/** Écrit un ou plusieurs réglages du cœur (JSON brut) et met à jour le cache local. */
async function cfgSet(patch) { Object.assign(app.cfg, await invoke("patch_settings_raw", { patch })); applySettings(); return app.cfg; }
async function gameSet(id, patch) { const g = await invoke("patch_game_raw", { id, patch }); setGame(g); return g; }

// ---------- fenêtres modales ----------
function modal(html, wide) { $("#modal").innerHTML = `<div class="scrim" data-act="close-scrim"><div class="modal${wide ? " wide" : ""}">${html}</div></div>`; const i = $("#modal input[type=text]"); i && i.focus(); }
function closeModal() { $("#modal").innerHTML = ""; app.pal = null; }
const foot = (ok, cls = "") => `<div class="foot"><button class="btn sec" data-act="close">${esc(t("add.cancel"))}</button><button class="btn ${cls}" id="ok">${esc(ok)}</button></div>`;

// ---------- petits composants de formulaire ----------
const sw = (id, label, hint, on) => `<div class="sw"><span>${esc(label)}${hint ? `<small>${esc(hint)}</small>` : ""}</span><input type="checkbox" id="${id}" ${on ? "checked" : ""}></div>`;
const sel = (id, opts, cur, cls = "") => `<select id="${id}" class="${cls}">${opts.map(([v, l]) => `<option value="${esc(v)}" ${String(v) === String(cur) ? "selected" : ""}>${esc(l)}</option>`).join("")}</select>`;
const inp = (id, v, ph = "", extra = "") => `<input type="text" id="${id}" value="${esc(v ?? "")}" placeholder="${esc(ph)}" ${extra}>`;
const lab = (l, body) => `<label class="fl"><span>${esc(l)}</span>${body}</label>`;
function applySettings() {
  const c = app.cfg, th = app.data.themes.find((x) => x.id === c.theme) || app.data.themes[0], root = document.documentElement.style;
  for (const [k, v] of Object.entries(th.vars)) root.setProperty(`--${k}`, v);
  const lum = (h) => { const n = parseInt(h.slice(1), 16); return (0.299 * (n >> 16) + 0.587 * ((n >> 8) & 255) + 0.114 * (n & 255)) / 255; };
  root.setProperty("--on-accent", lum(th.vars.accent) > 0.6 ? "#05070a" : "#fff");
  root.setProperty("--bgblur", `${Math.round(c.hero_background_blur ?? 18)}px`);
  root.setProperty("--sidew", `${Math.round(c.sidebar_width || 300)}px`);
  document.body.classList.toggle("compact", !!c.ui_compact);
  app.lang = LANGS[c.ui_language] || "fr"; document.documentElement.lang = app.lang;
  const bg = $("#bg"); if (bg) { const g = app.bgGame && game(app.bgGame); bg.style.backgroundImage = c.hero_background && g && g.heroUrl ? `url('${g.heroUrl}')` : "none"; }
}
