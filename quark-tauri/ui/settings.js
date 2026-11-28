"use strict";
// Page Paramètres : 10 onglets, recherche, enregistrement automatique (aucun bouton « Appliquer »).

const L = (fr, en) => (app.lang === "fr" ? fr : en);
const swk = (k, label, hint) => `<div class="sw" data-s="${esc(label + " " + (hint || ""))}"><span>${esc(label)}${hint ? `<small>${esc(hint)}</small>` : ""}</span><input type="checkbox" data-k="${k}" ${getPath(app.cfg, k) ? "checked" : ""}></div>`;
const selk = (k, label, opts, hint, extra = "") => `<div class="sw" data-s="${esc(label + " " + (hint || ""))}"><span>${esc(label)}${hint ? `<small>${esc(hint)}</small>` : ""}</span><select data-k="${k}" ${extra}>${opts.map(([v, l]) => `<option value="${esc(v)}" ${String(getPath(app.cfg, k) ?? "") === String(v) ? "selected" : ""}>${esc(l)}</option>`).join("")}</select></div>`;
const rangek = (k, label, min, max, step, hint) => `<div class="sw" data-s="${esc(label + " " + (hint || ""))}"><span>${esc(label)}${hint ? `<small>${esc(hint)}</small>` : ""}</span><span class="rg"><input type="range" data-k="${k}" min="${min}" max="${max}" step="${step}" value="${getPath(app.cfg, k) ?? min}"><output>${getPath(app.cfg, k) ?? min}</output></span></div>`;
const textk = (k, label, hint, ph = "", nullable = true, wide = true) => `<div class="fl2" data-s="${esc(label + " " + (hint || ""))}"><span>${esc(label)}${hint ? `<small>${esc(hint)}</small>` : ""}</span><input type="text" data-k="${k}" ${/gamescope_(width|height|fps)/.test(k) ? "data-num" : ""} ${nullable ? "data-nullable" : ""} value="${esc(getPath(app.cfg, k) ?? "")}" placeholder="${esc(ph)}"></div>`;
const card = (title, ...rows) => `<div class="card"><h3>${esc(title)}</h3>${rows.join("")}</div>`;
const STEAM_LANGS = [["english", "English"], ["french", "Français"], ["german", "Deutsch"], ["spanish", "Español"], ["portuguese", "Português"], ["italian", "Italiano"], ["russian", "Русский"], ["japanese", "日本語"], ["korean", "한국어"], ["schinese", "简体中文"], ["polish", "Polski"], ["turkish", "Türkçe"]];

function tabsDef() {
  const c = app.cfg, runtimes = [["", L("Automatique", "Automatic")], ...(app.runtimes || []).map((r) => [r.key, r.name])];
  return {
    appearance: () => [
      card(L("Langue et thème", "Language and theme"), selk("ui_language", L("Langue de l'interface", "Interface language"), app.data.languages.map((l) => [l.id, l.label])),
        `<div class="sw col" data-s="theme thème"><span>${esc(t("settings.theme"))}</span><div class="themes">${app.data.themes.map((th) => `<button class="th${th.id === c.theme ? " on" : ""}" data-theme="${th.id}" style="background:${th.vars.bg};color:${th.vars.text}"><i style="background:${th.vars.accent}"></i>${esc(th.label)}</button>`).join("")}</div></div>`),
      card(L("Démarrage", "Startup"), selk("start_page", L("Page d'ouverture", "Startup page"), [["home", t("x.home")], ["library", t("nav.library")], ["store", t("x.store")], ["my_games", t("x.mygames")]])),
      card(t("nav.library"), selk("view", L("Affichage", "Layout"), [["list", L("Liste", "List")], ["grid", L("Grille", "Grid")]]), swk("ui_compact", t("settings.compact"), t("settings.compactHint")),
        rangek("sidebar_width", L("Largeur de la liste", "List width"), 240, 420, 10), rangek("grid_tile_width", L("Largeur des vignettes", "Tile width"), 120, 260, 10),
        selk("game_actions", L("Boutons d'action du jeu", "Game action buttons"), [["top_right", L("En haut à droite", "Top right")], ["beside_play", L("À côté de Jouer", "Beside Play")]])),
      card(L("Mode Deck", "Deck mode"), selk("deck_mode", L("Manette et plein écran", "Gamepad and fullscreen"), [["auto", L("Automatique", "Automatic")], ["on", L("Toujours", "Always")], ["off", L("Désactivé", "Disabled")]], L("Auto : Steam Deck, gamescope ou première manette connectée.", "Auto: Steam Deck, gamescope or the first connected gamepad."))),
      card(L("Fiche de jeu", "Game page"), selk("detail_tab", L("Onglet par défaut", "Default tab"), [["overview", t("x.overview")], ["media", t("x.media")], ["ach", t("x.ach")], ["news", t("x.news")], ["hist", t("x.hist")], ["notes", t("tab.notes")], ["launch", t("tab.launch")], ["log", t("tab.log")]], L("Onglet ouvert quand tu sélectionnes un jeu.", "Tab opened when you select a game."))),
      card(t("x.home"), selk("home_sort", L("Tri du carrousel", "Carousel sort"), [["recent", t("sort.recent")], ["added", t("sort.added")], ["playtime", t("sort.playtime")], ["favorites", t("detail.favorite")]]), swk("carousel_auto", L("Défilement automatique", "Auto advance")), rangek("carousel_secs", L("Intervalle (s)", "Interval (s)"), 3, 30, 1)),
      card(L("Zone de notification", "Notification area"), swk("tray_icon", L("Garder Quark dans la zone de notification", "Keep Quark in the tray"), L("Quark reste accessible dans la zone de notification.", "Keep Quark available from the notification area."))),
    ],
    ingame: () => [
      card(L("Captures d'écran", "Screenshots"), textk("screenshot_key", L("Raccourci de capture", "Screenshot shortcut"), L("Uniquement pendant une partie.", "Only while playing."), "F12", false)),

      card("Notifications", swk("notify_install_done", L("Fin d'une installation", "Installation finished")), swk("notify_save_done", L("Sauvegarde Ludusavi terminée", "Ludusavi backup done")), swk("notify_session_done", L("Fin d'une session de jeu", "Game session ended"))),
      card(L("Sauvegardes cloud", "Cloud saves"), swk("cloud_saves", L("Synchronisation Epic/GOG", "Epic/GOG sync"), L("Après l'installation et à la sortie du jeu.", "After install and on exit."))),
    ],
    defaults: () => [
      card(L("Valeurs copiées dans chaque nouveau jeu", "Copied into each new game"), `<p class="hint">${esc(L("Ces valeurs ne sont pas globales : chaque jeu garde les siennes une fois ajouté.", "These are not global: each game keeps its own once added."))}</p>`,
        textk("default_prefix", L("Préfixe Proton", "Proton prefix"), L("Vide = préfixe automatique par jeu.", "Empty = automatic prefix per game."), "~/.local/share/quark/prefixes/<jeu>"),
        textk("default_install_dir", L("Dossier d'installation des boutiques", "Store install folder"), L("Vide = ~/Games.", "Empty = ~/Games."), "~/Games"), textk("default_args", L("Arguments", "Arguments"), "", "", false),
        selk("default_proton", L("Runtime Proton", "Proton runtime"), runtimes, "", "data-nullable"),
        selk("default_anticheat", t("launch.anticheat"), ANTICHEAT)),
      card(t("launch.options"), swk("default_launch.gamemode", "GameMode"), swk("default_launch.mangohud", "MangoHud"), swk("default_launch.prime", "PRIME"), swk("default_launch.gamescope", "gamescope"), swk("default_launch.gamescope_fsr", "gamescope FSR"),
        textk("default_launch.gamescope_width", "gamescope · L", "", "0", false), textk("default_launch.gamescope_height", "gamescope · H", "", "0", false), textk("default_launch.wrapper", t("launch.wrapper"), "", "", false), swk("default_proton_log", "PROTON_LOG"), swk("default_umu", "umu")),
      card(t("launch.env"), `<div class="envs" id="denv">${(c.default_env || []).map((e, i) => `<div class="row2"><input type="text" data-de="name" data-i="${i}" value="${esc(e.name)}" placeholder="NAME"><input type="text" data-de="value" data-i="${i}" value="${esc(e.value)}" placeholder="value"><button class="ib sm" data-dedel="${i}">${ic("x")}</button></div>`).join("") || `<div class="dim">${esc(t("x.noEnv"))}</div>`}</div><button class="btn sec sm" id="denv-add">${ic("plus")}${esc(t("x.addVar"))}</button>`),
      card(L("Runtimes détectés", "Detected runtimes"), `<div class="chips">${(app.runtimes || []).map((r) => `<span>${esc(r.name)}</span>`).join("") || `<span class="dim">—</span>`}</div><button class="btn sec sm" id="rt-redo">${ic("refresh")}${esc(L("Détecter à nouveau", "Detect again"))}</button>`),
    ],
    artworks: () => [card(L("Sources d'illustrations", "Artwork sources"), textk("sgdb_api_key", L("Clé SteamGridDB (facultative)", "SteamGridDB key (optional)"), L("Stockée dans la configuration locale.", "Stored in the local configuration."), "", false),
      selk("language", L("Langue des métadonnées", "Metadata language"), STEAM_LANGS), textk("country", L("Pays de la boutique", "Store country"), "", "FR", false)), card(L("Cache", "Cache"), `<button class="btn sec sm" data-open="${esc(app.data.dataDir)}">${ic("folder")}${esc(t("settings.openFolder"))}</button>`)],
    saves: () => [
      card(L("Ludusavi", "Ludusavi"),
        `<p class="hint">${esc(L("Choisis où Quark range les sauvegardes et les dossiers supplémentaires que Ludusavi doit analyser pour chaque jeu.", "Choose where Quark stores backups and the extra folders Ludusavi should scan for every game."))}</p>`,
        textk("save_backup_dir", L("Dossier des sauvegardes", "Backup folder"), L("Vide = ~/.local/share/quark/save-backups", "Empty = ~/.local/share/quark/save-backups"), "~/.local/share/quark/save-backups"),
        `<div class="fl2" data-s="${esc(L("Dossiers de recherche Ludusavi", "Ludusavi search folders"))}"><span>${esc(L("Dossiers de recherche Ludusavi", "Ludusavi search folders"))}<small>${esc(L("Un chemin absolu par ligne. Ils s'ajoutent aux dossiers propres à chaque jeu.", "One absolute path per line. Added to each game's own folders."))}</small></span><textarea id="sv-search-dirs" rows="4" spellcheck="false" placeholder="~/.config/MyGame/Saves">${esc((app.cfg.save_search_dirs || []).join("\n"))}</textarea></div>`),
    ],
    providers: () => [providersSettings()],
    extensions: () => [extensionsSettings()],
    stats: () => [renderStats()],
    disk: () => [card(L("Espace disque", "Disk usage"), `<div id="disk"><button class="btn sm" id="disk-go">${esc(L("Analyser", "Analyze"))}</button></div>`)],
    about: () => [card(t("settings.about"), `<div class="kv"><span>${esc(t("settings.version"))}</span><b>${esc(app.data.version)}</b></div>${app.data.memoryMb ? `<div class="kv"><span>${esc(t("settings.memory"))}</span><b>${app.data.memoryMb} MB</b></div>` : ""}<div class="kv"><span>${esc(t("settings.configPath"))}</span><b class="path">${esc(app.data.configPath)}</b></div><div class="kv"><span>${esc(t("settings.dataDir"))}</span><b class="path">${esc(app.data.dataDir)}</b></div>${app.data.demo ? `<div class="kv"><span>${esc(t("settings.demo"))}</span><b>✓</b></div>` : ""}`,
      swk("auto_update", L("Vérifier les mises à jour au démarrage", "Check for updates at startup")),
      `<div class="foot l"><button class="btn sec sm" id="ab-upd">${esc(t("x.updates"))}</button><button class="btn sec sm" data-open="${esc(app.data.configPath)}">${esc(t("settings.openFolder"))}</button><button class="btn sec sm" id="ab-reload">${esc(t("settings.reload"))}</button><button class="btn sec sm" id="ab-guide">${esc(L("Revoir le guide", "Review the guide"))}</button><button class="btn sec sm" data-open="https://github.com/Matheotaku4/Quark">GitHub</button></div>`)],
  };
}
const STABS = [["appearance", "Apparence", "Appearance"], ["ingame", "En jeu", "In game"], ["defaults", "Nouveaux jeux", "New games"], ["artworks", "Illustrations", "Artwork"], ["saves", "Sauvegardes", "Saves"], ["providers", "Boutiques & outils", "Stores & tools"], ["extensions", "Extensions", "Extensions"], ["stats", "Statistiques", "Statistics"], ["disk", "Disque", "Disk"], ["about", "À propos", "About"]];

function extensionDate(ts) {
  if (!ts) return "";
  try { return new Date(ts * 1000).toLocaleString(locale()); } catch { return ""; }
}

function extensionsSettings() {
  const list = app.data.extensions || [];
  const cards = list.map((ext) => {
    const name = ext.name || ext.id || t("ext.unknown");
    const meta = [ext.version, t("ext.sites", { n: ext.siteCount || 0 })].filter(Boolean).join(" · ");
    const updated = ext.fetchedAt ? t("ext.updated", { date: extensionDate(ext.fetchedAt) }) : "";
    return `<div class="extcard">
      <div class="exthead"><div><b>${esc(name)}</b><small>${esc(meta)}</small></div>
        <div class="extactions"><button class="btn sec sm" data-ext-refresh="${esc(ext.url)}">${ic("refresh")}${esc(t("ext.refresh"))}</button><button class="btn sec sm" data-ext-remove="${esc(ext.url)}">${ic("trash")}${esc(t("ext.remove"))}</button></div></div>
      <div class="exturl">${esc(ext.url)}</div>
      <div class="extmeta">${esc(updated)}${ext.stale ? ` · <b>${esc(t("ext.stale"))}</b>` : ""}</div>
      ${ext.error ? `<p class="weberr">${esc(ext.error)}</p>` : ""}
    </div>`;
  }).join("");
  return card(t("settings.extensions"), `<p class="hint">${esc(t("ext.hint"))}</p>
    <div class="row2"><input type="text" id="ext-url" placeholder="${esc(t("ext.urlPlaceholder"))}" spellcheck="false" autocomplete="off"><button class="btn" id="ext-add">${ic("plus")}${esc(t("ext.add"))}</button></div>
    <p class="hint">${esc(t("ext.template"))}</p>
    <div class="extlist">${cards || `<div class="extempty">${esc(t("ext.empty"))}</div>`}</div>`);
}

function syncDependent(root, masterKey, selector) {
  const master = root.querySelector(`[data-k="${masterKey}"]`);
  if (!master) return;
  const apply = () => {
    $$(selector, root).forEach((el) => {
      el.disabled = !master.checked;
      const row = el.closest(".sw,.fl2,.fl");
      row && row.classList.toggle("dep-off", !master.checked);
    });
  };
  master.addEventListener("change", apply);
  apply();
}

function syncLaunchDeps(root, prefix) {
  syncDependent(root, `${prefix}gamescope`, `[data-k^="${prefix}gamescope_"]`);
  syncDependent(root, "umu", '[data-k="umu_game_id"]');
}

function renderStats() {
  const day = (ts) => Math.floor(ts / 86400), today = day(Date.now() / 1000), per = new Map();
  let total = 0;
  for (const g of app.data.games) { total += g.playtimeSecs; for (const s of g.sessions) per.set(day(s.start), (per.get(day(s.start)) || 0) + s.secs); }
  const wk = (n) => { let s = 0; for (let i = 0; i < 7; i++) s += per.get(today - n * 7 - i) || 0; return s; }, weeks = Array.from({ length: 12 }, (_, i) => wk(11 - i)), max = Math.max(1, ...weeks);
  let streak = 0; for (let d = per.has(today) ? today : today - 1; per.has(d); d--) streak++;
  const cells = []; const hv = [...per.values()].sort((a, b) => a - b), q = (p) => hv[Math.floor(p * (hv.length - 1))] || 1;
  for (let c = 25; c >= 0; c--) for (let r = 6; r >= 0; r--) { const v = per.get(today - c * 7 - r) || 0; cells.push(`<i class="hc l${!v ? 0 : v <= q(.33) ? 1 : v <= q(.66) ? 2 : 3}" title="${hm(v)}"></i>`); }
  const top = app.data.games.slice().sort((a, b) => b.playtimeSecs - a.playtimeSecs).slice(0, 8), tm = Math.max(1, top[0]?.playtimeSecs || 1);
  return `<div class="card"><h3>${esc(L("Vue d'ensemble", "Overview"))}</h3><div class="stats3"><div><b>${hm(total)}</b><span>${esc(L("Temps de jeu total", "Total playtime"))}</span></div><div><b>${hm(wk(0))}</b><span>${esc(L("Cette semaine", "This week"))}</span></div><div><b>${streak} ${esc(L("j", "d"))}</b><span>${esc(L("Série de jours", "Day streak"))}</span></div></div></div>
    <div class="card"><h3>${esc(t("x.weeks"))}</h3><div class="bars">${weeks.map((v, i) => `<i class="${i === 11 ? "now " : ""}${v ? "v" : ""}" style="height:${Math.max(6, (v / max) * 100)}%" title="${hm(v)}"></i>`).join("")}</div></div>
    <div class="card"><h3>${esc(L("26 dernières semaines", "Last 26 weeks"))}</h3><div class="heat">${cells.join("")}</div></div>
    <div class="card"><h3>${esc(L("Jeux les plus joués", "Most played"))}</h3>${top.map((g) => `<div class="topg"><span>${esc(g.name)}</span><i style="width:${(g.playtimeSecs / tm) * 60}%"></i><b>${hm(g.playtimeSecs)}</b></div>`).join("")}</div>`;
}

function renderSettings() {
  const defs = tabsDef(), q = norm(app.sq.trim());
  let body;
  if (q) {
    body = ""; for (const [id, fr, en] of STABS) { if (id === "stats" || id === "disk" || id === "providers" || id === "extensions") continue; const html = defs[id]().join(""), d = document.createElement("div"); d.innerHTML = html;
      const hits = $$("[data-s]", d).filter((n) => norm(n.dataset.s).includes(q)); if (hits.length) body += `<div class="card"><h3>${esc(L(fr, en))}</h3>${hits.map((n) => n.outerHTML).join("")}</div>`; }
    body ||= `<div class="none">${esc(t("x.none"))}</div>`;
  } else body = defs[app.stab]().join("");
  return `<div class="settings"><nav class="snav"><input type="search" id="sq" placeholder="${esc(L("Rechercher un réglage…", "Search settings…"))}" value="${esc(app.sq)}">
    ${STABS.map(([id, fr, en]) => `<button data-stab="${id}" class="${!q && app.stab === id ? "on" : ""}">${esc(L(fr, en))}</button>`).join("")}</nav><div class="sbody"><h2>${esc(q ? t("x.search") : L(...STABS.find((x) => x[0] === app.stab).slice(1)))}</h2>${body}</div></div>`;
}
function bindSettings() {
  const root = $("#sheet");
  bindModel(root, app.cfg, (k) => {
    const top = k.split(".")[0], patch = { [top]: app.cfg[top] };
    run(() => cfgSet(patch)).then(() => { if (k === "tray_icon" || k === "ui_language") invoke("tray_visibility", { visible: !!app.cfg.tray_icon }).catch(() => toast(t("x.trayUnavailable"), true)); if (k === "deck_mode") { QuarkDeck.mode(app.cfg.deck_mode); renderTop(); } else if (k === "ui_language" || k === "ui_compact" || k === "game_actions") { renderAllKeepScroll(); } else if (k === "view") renderTop(); });
  });
  $$('[data-k="screenshot_key"]', root).forEach((el) =>
    el.addEventListener("change", () => run(async () => {
      await cfgSet({ screenshot_key: app.cfg.screenshot_key });
      await invoke("refresh_shortcuts");
    })),
  );
  syncLaunchDeps(root, "default_launch.");
  const sqEl = $("#sq"); sqEl.oninput = () => { app.sq = sqEl.value; const pos = sqEl.selectionStart, top = $("#sheet").scrollTop; $("#sheet").innerHTML = renderSettings(); bindSettings(); $("#sheet").scrollTop = top; const n = $("#sq"); n.focus(); n.setSelectionRange(pos, pos); };
  const commitEnv = () => run(() => cfgSet({ default_env: app.cfg.default_env }));
  $$("[data-de]", root).forEach((el) => (el.oninput = () => { app.cfg.default_env[+el.dataset.i][el.dataset.de] = el.value; })); $$("[data-de]", root).forEach((el) => (el.onchange = commitEnv));
  $$("[data-dedel]", root).forEach((b) => (b.onclick = async () => { app.cfg.default_env.splice(+b.dataset.dedel, 1); await commitEnv(); renderSheetKeepScroll(); }));
  const ea = $("#denv-add"); ea && (ea.onclick = () => { (app.cfg.default_env ||= []).push({ name: "", value: "" }); renderSheetKeepScroll(); });
  const sdirs = $("#sv-search-dirs");
  sdirs && (sdirs.onchange = () => run(async () => {
    const dirs = sdirs.value.split("\n").map((line) => line.trim()).filter(Boolean);
    await cfgSet({ save_search_dirs: dirs });
    toast(t("toast.saved"));
  }));
  const rr = $("#rt-redo"); rr && (rr.onclick = () => run(async () => { app.runtimes = await invoke("list_runtimes"); renderSheetKeepScroll(); }));
  const dg = $("#disk-go"); dg && (dg.onclick = () => run(async () => { dg.disabled = true; dg.textContent = t("log.loading"); const d = await invoke("disk_report"), m = Math.max(1, ...d.rows.map((r) => r.bytes));
    $("#disk").innerHTML = `<div class="kv"><span>Total</span><b>${esc(d.human)}</b></div>` + d.rows.map((r) => `<div class="drow"><div><b>${esc(r.label)}</b><small>${esc(r.detail || r.path)}</small></div><i style="width:${(r.bytes / m) * 140}px"></i><span>${esc(r.human)}</span><button class="ib sm" data-open="${esc(r.path)}">${ic("folder")}</button></div>`).join(""); }));
  const refreshExtensionData = async () => {
    const data = await invoke("bootstrap_data");
    app.data.extensions = data.extensions || [];
    app.data.searchSites = data.searchSites || [];
  };
  const extAdd = $("#ext-add");
  extAdd && (extAdd.onclick = () => run(async () => {
    const url = $("#ext-url").value.trim();
    if (!url) return toast(t("ext.urlRequired"), true);
    extAdd.disabled = true;
    try {
      app.data.extensions = await invoke("extensions_add", { url });
      await refreshExtensionData();
      renderSheetKeepScroll();
      toast(t("ext.added"));
    } finally { extAdd.disabled = false; }
  }));
  $$("[data-ext-refresh]", root).forEach((button) => (button.onclick = () => run(async () => {
    button.disabled = true;
    try {
      app.data.extensions = await invoke("extensions_refresh", { url: button.dataset.extRefresh });
      await refreshExtensionData();
      renderSheetKeepScroll();
      toast(t("ext.refreshed"));
    } finally { button.disabled = false; }
  })));
  $$("[data-ext-remove]", root).forEach((button) => (button.onclick = () => run(async () => {
    if (!confirm(t("ext.removeConfirm"))) return;
    app.data.extensions = await invoke("extensions_remove", { url: button.dataset.extRemove });
    await refreshExtensionData();
    renderSheetKeepScroll();
    toast(t("ext.removed"));
  })));
  const au = $("#ab-upd"); au && (au.onclick = updatesModal);
  const ar = $("#ab-reload"); ar && (ar.onclick = () => run(async () => { app.data = await invoke("reload_library"); setRunning(app.data.running); app.cfg = Object.assign({}, await invoke("get_settings_raw")); applySettings(); renderAll(); toast(t("toast.saved")); }));
  const ag = $("#ab-guide"); ag && (ag.onclick = onboardingModal);
}
