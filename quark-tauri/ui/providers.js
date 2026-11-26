"use strict";
// Mes jeux : bibliothèques Steam / Epic / GOG / itch.io, comptes et outils.

Object.assign(app, {
  pv: {
    data: null,
    loading: false,
    error: null,
    q: "",
    providers: new Set(),
    state: "all",
    sort: "title",
    shown: 24,
    coverTried: new Set(),
    coverFailures: new Map(),
    coverQueue: new Set(),
    imgObs: null,
    prefsLoaded: false,
    searchT: null,
  },
});

const PV_LABEL = { steam: "Steam", epic: "Epic Games", gog: "GOG", itch: "itch.io" };
const PV_COLOR = { steam: "#66c0f4", epic: "#d8d8d8", gog: "#b56cff", itch: "#fa5c5c" };
const PROVIDER_DETAILS = {
  "Clé Web API et SteamID64 facultatifs, pour la bibliothèque possédée et les succès.": "pv.steamHint",
  "Legendary n'est pas installé": "pv.legendaryMissing",
  "Legendary installé · compte non connecté": "pv.legendaryInstalled",
  "Session Legendary": "pv.legendarySession",
  "gogdl n'est pas installé": "pv.gogMissing",
  "gogdl installé · compte non connecté": "pv.gogInstalled",
  "Session gogdl": "pv.gogSession",
  "Clé API personnelle enregistrée": "pv.itchSaved",
  "Clé API personnelle, pour la bibliothèque possédée": "pv.itchHint",
};
const pd = (detail) => {
  if (app.lang === "fr") return detail;
  const key = PROVIDER_DETAILS[detail];
  if (key) return t(key);
  if (detail.startsWith("Connexion directe")) return t("pv.epicDirect") + detail.slice(18);
  return detail;
};
const pvStateLabel = (s) => t("pv.state." + s);
const PV_PAGE = 24;
const PV_FILTERS_KEY = "quark.mygames.filters.v1";
const pvInstalled = (g) => g.installState === "installed";
const pvIsLocal = (g) => g.coverCached || String(g.coverUrl || "").startsWith("quarkimg://");

function pvLoadPrefs() {
  if (app.pv.prefsLoaded) return;
  app.pv.prefsLoaded = true;
  try {
    const raw = JSON.parse(localStorage.getItem(PV_FILTERS_KEY) || "{}");
    const providers = Array.isArray(raw.providers) ? raw.providers.filter((id) => PV_LABEL[id]) : [];
    app.pv.providers = new Set(providers);
    if (["all", "installed", "missing"].includes(raw.state)) app.pv.state = raw.state;
    if (["title", "installed", "synced"].includes(raw.sort)) app.pv.sort = raw.sort;
  } catch {}
}

function pvSavePrefs() {
  try {
    localStorage.setItem(PV_FILTERS_KEY, JSON.stringify({
      providers: [...app.pv.providers],
      state: app.pv.state,
      sort: app.pv.sort,
    }));
  } catch {}
}

function pvLayout() {
  return app.cfg.my_games_list ? "list" : "grid";
}

async function providersLoad(force) {
  if (app.pv.loading || (app.pv.data && !force)) return app.pv.data;
  app.pv.loading = true;
  if (app.view === "mygames") repaintMyGamesResults();
  try {
    app.pv.data = await invoke("providers_data");
    app.pv.error = null;
    app.pv.shown = PV_PAGE;
    pvLoadPrefs();
  } catch (e) {
    app.pv.error = String(e);
    app.pv.data ||= { games: [], accounts: [], tools: [], error: String(e) };
  }
  app.pv.loading = false;
  if (app.view === "mygames") {
    if ($("#pv-results")) repaintMyGamesResults();
    else renderSheetKeepScroll();
  }
  return app.pv.data;
}

function pvMatches(g, q) {
  if (!q) return true;
  return norm([g.title, PV_LABEL[g.provider], g.installPath].join(" ")).includes(q);
}

function pvSort(a, b) {
  if (app.pv.sort === "installed") {
    const delta = Number(pvInstalled(b)) - Number(pvInstalled(a));
    if (delta) return delta;
  }
  if (app.pv.sort === "synced") {
    const delta = (b.lastSynced || 0) - (a.lastSynced || 0);
    if (delta) return delta;
  }
  return a.title.localeCompare(b.title, locale(), { sensitivity: "base" });
}

function pvGames() {
  const q = norm(app.pv.q.trim());
  return (app.pv.data?.games || [])
    .filter((g) =>
      (!app.pv.providers.size || app.pv.providers.has(g.provider)) &&
      (app.pv.state === "all" || (app.pv.state === "installed" ? pvInstalled(g) : !pvInstalled(g))) &&
      pvMatches(g, q))
    .sort(pvSort);
}

function pvButton(label, action, cls = "sec", attrs = "") {
  return `<button class="btn ${cls} sm" data-pv="${action}" ${attrs}>${esc(label)}</button>`;
}

function pvActions(g) {
  const installed = pvInstalled(g);
  const linked = g.quarkGameId;
  const actions = [];
  if (g.installState === "installing") {
    actions.push(`<span class="pvwait">${esc(t("pv.installing"))}</span>`);
  } else if (installed) {
    actions.push(linked
      ? pvButton(t("pv.open"), "open", "pri", `data-game="${esc(linked)}"`)
      : pvButton(t("pv.adopt"), "adopt", "pri", `data-provider="${g.provider}" data-product="${esc(g.productId)}"`));
    if (g.provider !== "steam") {
      actions.push(pvButton(t("pv.uninstall"), "uninstall", "sec", `data-provider="${g.provider}" data-product="${esc(g.productId)}"`));
    }
  } else if (g.provider === "steam") {
    actions.push(pvButton(t("pv.installSteam"), "install", "pri", `data-provider="${g.provider}" data-product="${esc(g.productId)}"`));
  } else {
    actions.push(pvButton(t("pv.install"), "install", "pri", `data-provider="${g.provider}" data-product="${esc(g.productId)}"`));
  }
  return actions.join("");
}

function pvCover(g, cls = "") {
  const cached = pvIsLocal(g);
  const mono = `<span class="pvmono">${esc((g.title || "?").slice(0, 1).toUpperCase())}</span>`;
  const img = g.coverUrl
    ? `<img src="${esc(g.coverUrl)}" loading="lazy" decoding="async" referrerpolicy="no-referrer" alt="" ${cached ? 'data-cover-cached="1"' : ""}>`
    : "";
  return `<div class="pcover ${cls}" data-pv-cover data-provider="${esc(g.provider)}" data-product="${esc(g.productId)}" ${cached ? 'data-cover-cached="1"' : ""}>${mono}${img}</div>`;
}

function providerCard(g) {
  const installed = pvInstalled(g);
  return `<article class="pcard" style="--pc:${PV_COLOR[g.provider] || "var(--accent)"}" data-provider="${esc(g.provider)}" data-product="${esc(g.productId)}">
    ${pvCover(g)}
    <span class="pvsrc">${esc(PV_LABEL[g.provider] || g.provider)}</span>
    <span class="pvstate ${installed ? "on" : ""}">${esc(pvStateLabel(g.installState))}</span>
    <div class="pmeta"><b title="${esc(g.title)}">${esc(g.title)}</b>
      <small>${installed ? esc(t("pv.installedAt", { path: g.installPath || "—" })) : esc(t("pv.owned"))}</small>
      <div class="pactions">${pvActions(g)}</div>
    </div>
  </article>`;
}

function providerRow(g) {
  const installed = pvInstalled(g);
  return `<article class="pvrow" data-provider="${esc(g.provider)}" data-product="${esc(g.productId)}">
    ${pvCover(g, "sm")}
    <div class="pvrow-main"><b title="${esc(g.title)}">${esc(g.title)}</b>
      <small>${esc(PV_LABEL[g.provider] || g.provider)} · ${installed ? esc(g.installPath || t("pv.state.installed")) : esc(t("pv.owned"))}</small>
    </div>
    <span class="pvstate ${installed ? "on" : ""}">${esc(pvStateLabel(g.installState))}</span>
    <div class="pactions">${pvActions(g)}</div>
  </article>`;
}

function pvCount(games) {
  return t("pv.count", { n: games.length });
}

function renderMyGamesResults() {
  const games = pvGames();
  const shown = Math.min(app.pv.shown || PV_PAGE, games.length);
  const visible = games.slice(0, shown);
  if (!games.length) {
    return `<div class="none">${esc(app.pv.data?.games?.length ? t("pv.noMatch") : t("pv.empty"))}</div>`;
  }
  const layout = pvLayout();
  const body = layout === "list" ? visible.map(providerRow).join("") : visible.map(providerCard).join("");
  const more = games.length > shown ? `<button class="pvmore" id="pv-more" type="button">${esc(t("pv.more"))}</button>` : "";
  return `<div class="${layout === "list" ? "pvlist" : "pvgrid"}">${body}${more}</div>`;
}

function repaintMyGamesResults() {
  const count = $("#pv-count"), results = $("#pv-results");
  if (!count || !results) return;
  const games = pvGames();
  count.textContent = pvCount(games);
  results.innerHTML = renderMyGamesResults();
  bindMyGamesResults();
  pvLazyImages();
  pvQueueCovers();
}

function renderMyGames() {
  const data = app.pv.data;
  if (!data && app.pv.loading) return `<div class="pv-skeleton">${Array.from({ length: 8 }, () => `<div class="pcard sk"></div>`).join("")}</div>`;
  if (!data) return `<div class="empty"><h2>${esc(t("pv.title"))}</h2><p>${esc(app.pv.error || t("pv.emptyHint"))}</p><button class="btn" data-pv="reload">${esc(t("pv.reload"))}</button></div>`;
  pvLoadPrefs();
  const chips = [["", t("pv.all")], ["steam", "Steam"], ["epic", "Epic Games"], ["gog", "GOG"], ["itch", "itch.io"]];
  return `<div class="mygames">
    <div class="pvhero"><div><span class="pill">${esc(t("pv.mygames"))}</span><h2>${esc(t("pv.title"))}</h2><p>${esc(t("pv.subtitle"))}</p></div>
      <div class="pvactions"><button class="btn sec" data-pv="accounts">${ic("gear")}${esc(t("pv.accounts"))}</button><button class="btn sec" data-pv="sync-all">${ic("refresh")}${esc(t("pv.syncAll"))}</button><button class="btn" data-pv="steam-import">${ic("plus")}${esc(t("pv.importSteam"))}</button></div></div>
    ${data.error ? `<div class="sline err">${esc(data.error)}</div>` : ""}
    <div class="pvtoolbar">
      <input type="search" id="pv-q" value="${esc(app.pv.q)}" placeholder="${esc(t("pv.search"))}" autocomplete="off">
      <div class="pchips" id="pv-providers">${chips.map(([id, label]) => {
        const active = id ? app.pv.providers.has(id) : !app.pv.providers.size;
        return `<button data-pvprov="${id}" class="${active ? "on" : ""}">${esc(label)}</button>`;
      }).join("")}</div>
      <select id="pv-state">${["all", "installed", "missing"].map((id) => `<option value="${id}" ${app.pv.state === id ? "selected" : ""}>${esc(t("pv.filter." + id))}</option>`).join("")}</select>
      <select id="pv-sort">${["title", "installed", "synced"].map((id) => `<option value="${id}" ${app.pv.sort === id ? "selected" : ""}>${esc(t("pv.sort." + id))}</option>`).join("")}</select>
      <button class="tb" id="pv-layout" title="${esc(t(pvLayout() === "list" ? "pv.layout.grid" : "pv.layout.list"))}">${ic(pvLayout() === "list" ? "grid" : "list")}</button>
      <button class="tb" data-pv="reload" title="${esc(t("pv.reload"))}">${ic("refresh")}</button>
    </div>
    <div class="pvcount" id="pv-count">${esc(pvCount(pvGames()))}</div>
    <div id="pv-results">${renderMyGamesResults()}</div>
  </div>`;
}

function stopMyGames() {
  clearTimeout(app.pv.searchT);
  app.pv.imgObs?.disconnect();
  app.pv.imgObs = null;
}

function bindMyGamesResults() {
  const more = $("#pv-more");
  if (more) more.onclick = pvMore;
  const results = $("#pv-results");
  if (results && !results.dataset.bound) {
    results.dataset.bound = "1";
    results.addEventListener("error", (e) => {
      const img = e.target.closest?.("img");
      if (!img) return;
      img.remove();
    }, true);
  }
}

function bindMyGames() {
  const reset = () => { app.pv.shown = PV_PAGE; };
  const q = $("#pv-q");
  if (q) q.oninput = () => {
    app.pv.q = q.value;
    reset();
    clearTimeout(app.pv.searchT);
    app.pv.searchT = setTimeout(repaintMyGamesResults, 120);
  };
  const state = $("#pv-state");
  if (state) state.onchange = () => { app.pv.state = state.value; reset(); pvSavePrefs(); repaintMyGamesResults(); };
  const sort = $("#pv-sort");
  if (sort) sort.onchange = () => { app.pv.sort = sort.value; reset(); pvSavePrefs(); repaintMyGamesResults(); };
  $$("[data-pvprov]").forEach((button) => (button.onclick = () => {
    const id = button.dataset.pvprov;
    if (!id) app.pv.providers.clear();
    else if (app.pv.providers.has(id)) app.pv.providers.delete(id);
    else app.pv.providers.add(id);
    reset();
    pvSavePrefs();
    repaintMyGamesResults();
    $$("[data-pvprov]").forEach((chip) => {
      const key = chip.dataset.pvprov;
      chip.classList.toggle("on", key ? app.pv.providers.has(key) : !app.pv.providers.size);
    });
  }));
  const layout = $("#pv-layout");
  if (layout) layout.onclick = () => run(async () => {
    const next = pvLayout() === "list" ? "grid" : "list";
    await cfgSet({ my_games_list: next === "list" });
    const button = $("#pv-layout");
    if (button) {
      button.innerHTML = ic(next === "list" ? "grid" : "list");
      button.title = t(next === "list" ? "pv.layout.grid" : "pv.layout.list");
    }
    repaintMyGamesResults();
  });
  bindMyGamesResults();
  pvLazyImages();
  pvQueueCovers();
}

function pvLazyImages() {
  const images = $$("#pv-results img[data-src]");
  if (!images.length) return;
  if (!("IntersectionObserver" in window)) {
    images.forEach((img) => { img.src = img.dataset.src; });
    return;
  }
  app.pv.imgObs?.disconnect();
  app.pv.imgObs = new IntersectionObserver((entries) => {
    for (const entry of entries) {
      if (!entry.isIntersecting) continue;
      const img = entry.target;
      if (!img.getAttribute("src") && img.dataset.src) img.src = img.dataset.src;
      app.pv.imgObs.unobserve(img);
    }
  }, { root: $("#sheet"), rootMargin: "300px" });
  images.forEach((img) => app.pv.imgObs.observe(img));
}

function pvQueueCovers() {
  const nodes = $$("#pv-results [data-pv-cover]");
  const keys = [];
  for (const node of nodes) {
    if (node.dataset.coverCached === "1") continue;
    const provider = node.dataset.provider;
    const productId = node.dataset.product;
    const key = `${provider}:${productId}`;
    if (app.pv.coverTried.has(key)) continue;
    if ((app.pv.coverFailures.get(key) || 0) >= 3) continue;
    if (app.pv.coverQueue.has(key)) continue;
    app.pv.coverQueue.add(key);
    keys.push({ provider, productId });
    if (keys.length >= 8) break;
  }
  if (!keys.length) return;
  invoke("provider_ensure_covers", { keys })
    .then((updates) => { pvApplyCoverUpdates(updates); pvQueueCovers(); })
    .catch(() => keys.forEach((key) => app.pv.coverQueue.delete(`${key.provider}:${key.productId}`)));
}

function pvApplyCoverUpdates(updates) {
  for (const update of updates) {
    const key = `${update.provider}:${update.productId}`;
    app.pv.coverQueue.delete(key);
    if (!update.coverUrl) {
      app.pv.coverFailures.set(key, (app.pv.coverFailures.get(key) || 0) + 1);
      continue;
    }
    app.pv.coverTried.add(key);
    app.pv.coverFailures.delete(key);
    const node = $(`#pv-results [data-pv-cover][data-provider="${update.provider}"][data-product="${update.productId}"]`);
    if (!node) continue;
    node.dataset.coverCached = "1";
    let img = node.querySelector("img");
    if (!img) {
      img = document.createElement("img");
      img.alt = "";
      img.loading = "lazy";
      img.decoding = "async";
      node.prepend(img);
    }
    img.src = update.coverUrl;
    img.hidden = false;
    node.querySelector(".pvmono")?.remove();
  }
}

function pvMore() {
  const games = pvGames();
  const from = app.pv.shown || PV_PAGE;
  const next = games.slice(from, from + PV_PAGE);
  const more = $("#pv-more");
  if (!next.length) { more?.remove(); return; }
  const layout = pvLayout();
  more?.insertAdjacentHTML("beforebegin", next.map(layout === "list" ? providerRow : providerCard).join(""));
  app.pv.shown = from + next.length;
  if (app.pv.shown >= games.length) more?.remove();
  pvLazyImages();
  pvQueueCovers();
}

function providersSettings() {
  const data = app.pv.data;
  if (!data) return `<div class="card"><h3>${esc(t("pv.accounts"))}</h3><p class="dim">${esc(t("pv.loading"))}</p></div>`;
  return `<div class="card"><h3>${esc(t("pv.accounts"))}</h3><p class="hint">${esc(t("pv.accountsHint"))}</p><div class="accounts col">${data.accounts.map(accountBadge).join("")}</div>
      <div class="foot l"><button class="btn sec sm" data-pv="steam-creds">${esc(t("pv.steamCreds"))}</button><button class="btn sec sm" data-pv="itch-key">${esc(t("pv.itchKey"))}</button></div></div>
    <div class="card"><h3>${esc(t("pv.sync"))}</h3><p class="hint">${esc(t("pv.syncHint"))}</p><div class="chips">${["steam", "epic", "gog", "itch"].map((p) => `<button class="chipbtn" data-pv="sync" data-provider="${p}">${ic("refresh")}${esc(PV_LABEL[p])}</button>`).join("")}</div>
      <div class="foot l"><button class="btn sec sm" data-pv="steam-import">${ic("plus")}${esc(t("pv.importSteam"))}</button></div></div>
    <div class="card"><h3>${esc(t("pv.tools"))}</h3><p class="hint">${esc(t("pv.toolsHint"))}</p><div class="tools">${data.tools.map(toolRow).join("")}</div></div>`;
}

async function providerAction(action, data) {
  const p = data.provider || data.pvprov || "";
  if (action === "reload") return providersLoad(true);
  if (action === "accounts") { app.stab = "providers"; app.view = "settings"; renderAll(); return; }
  if (action === "sync-all") return run(async () => { for (const k of ["steam", "epic", "gog", "itch"]) { const a = app.pv.data?.accounts.find((x) => x.id === k); if (a?.connected || k === "steam") await invoke("provider_sync", { provider: k }).catch(() => null); } await providersLoad(true); toast(t("pv.synced")); });
  if (action === "sync") return run(async () => { app.pv.loading = true; renderSheet(); await invoke("provider_sync", { provider: p }); await providersLoad(true); toast(t("pv.synced")); });
  if (action === "connect") return providerLogin(p);
  if (action === "disconnect") return confirmProviderDisconnect(p);
  if (action === "tool") return run(async () => { toast(t("pv.toolInstalling")); await invoke("provider_tool_install", { tool: data.tool }); await providersLoad(true); toast(t("pv.toolDone")); });
  if (action === "steam-creds") return steamCredentialsModal();
  if (action === "itch-key") return itchKeyModal();
  if (action === "steam-import") return steamImportModal();
  if (action === "open") { const g = game(data.game); if (g) { app.view = "library"; app.open = true; renderAll(); select(g.id); } return; }
  if (action === "adopt") return run(async () => { const g = await invoke("provider_adopt", { provider: data.provider, productId: data.product }); setGame(g); app.view = "library"; app.open = true; renderAll(); select(g.id); toast(t("pv.adopted", { name: g.name })); });
  if (action === "install") return installProviderModal(p, data.product);
  if (action === "uninstall") return confirmUninstall(p, data.product);
}

function providerLogin(provider) {
  const epic = provider === "epic";
  const link = epic ? "https://legendary.gl/epiclogin" : "https://gog.com/auth";
  modal(`<h2>${esc(t("pv.connect") + " · " + PV_LABEL[provider])}</h2>
    <p class="desc">${esc(epic ? t("pv.epicLoginHint") : t("pv.gogLoginHint"))}</p>
    <p><button class="btn sec sm" data-open="${link}">${ic("ext")}${esc(t("pv.openLogin"))}</button></p>
    <label class="fl"><span>${esc(t("pv.authCode"))}</span><input type="text" id="pv-code" autocomplete="off" spellcheck="false" placeholder="${esc(t("pv.authCode"))}"></label>
    ${foot(t("pv.connect"))}`);
  $("#ok").onclick = () => run(async () => {
    const code = $("#pv-code").value.trim();
    if (!code) return toast(t("pv.codeRequired"), true);
    $("#ok").disabled = true; $("#ok").textContent = t("pv.connecting");
    try { toast(await invoke("provider_login", { provider, code })); closeModal(); await providersLoad(true); renderAll(); }
    catch (e) { $("#ok").disabled = false; $("#ok").textContent = t("pv.connect"); toast(String(e), true); }
  });
}

function confirmProviderDisconnect(provider) {
  modal(`<h2>${esc(t("pv.disconnect") + " · " + PV_LABEL[provider])}</h2><p class="desc">${esc(t("pv.disconnectBody"))}</p>${foot(t("pv.disconnect"), "red")}`);
  $("#ok").onclick = () => run(async () => { await invoke("provider_disconnect", { provider }); closeModal(); await providersLoad(true); renderAll(); toast(t("pv.disconnected")); });
}

function steamCredentialsModal() {
  const data = app.pv.data || {};
  modal(`<h2>Steam</h2><p class="desc">${esc(t("pv.steamHint"))}</p>
    <label class="fl"><span>Steam Web API key</span><input type="text" id="st-key" autocomplete="off" spellcheck="false" placeholder="XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX"></label>
    <label class="fl"><span>SteamID64</span><input type="text" id="st-id" value="${esc(data.steamId || "")}" autocomplete="off" spellcheck="false" placeholder="7656119…"></label>
    ${foot(t("x.save"))}`);
  $("#ok").onclick = () => run(async () => { await invoke("provider_save_credentials", { provider: "steam", key: $("#st-key").value.trim() || null, account: $("#st-id").value.trim() || null }); closeModal(); await providersLoad(true); toast(t("toast.saved")); });
}

function itchKeyModal() {
  modal(`<h2>itch.io</h2><p class="desc">${esc(t("pv.itchHint"))}</p>
    <label class="fl"><span>${esc(t("pv.itchKey"))}</span><input type="text" id="itch-key" autocomplete="off" spellcheck="false"></label>
    ${foot(t("x.save"))}`);
  $("#ok").onclick = () => run(async () => { await invoke("provider_save_credentials", { provider: "itch", key: $("#itch-key").value.trim(), account: null }); closeModal(); await providersLoad(true); toast(t("toast.saved")); });
}

function installProviderModal(provider, productId) {
  const rec = app.pv.data?.games.find((g) => g.provider === provider && g.productId === productId);
  if (!rec) return;
  const steam = provider === "steam";
  modal(`<h2>${esc(t("pv.install") + " · " + rec.title)}</h2>
    ${steam ? `<p class="desc">${esc(t("pv.steamInstallHint"))}</p>` : `<label class="fl"><span>${esc(t("pv.destination"))}</span><input type="text" id="pv-dest" value="${esc(app.cfg.default_install_dir || "~/Games")}" placeholder="~/Games"></label>`}
    ${foot(steam ? t("pv.openSteam") : t("pv.install"))}`);
  $("#ok").onclick = () => {
    const destination = steam ? null : ($("#pv-dest")?.value || null);
    closeModal(); toast(t("pv.installStarted"));
    invoke("provider_install", { provider, productId, destination })
      .then(() => providersLoad(true))
      .then(() => { if (app.view === "mygames") renderSheetKeepScroll(); })
      .catch((e) => toast(String(e), true));
  };
}

function confirmUninstall(provider, productId) {
  const rec = app.pv.data?.games.find((g) => g.provider === provider && g.productId === productId);
  if (!rec) return;
  modal(`<h2>${esc(t("pv.uninstall"))}</h2><p class="desc">${esc(t("pv.uninstallBody", { name: rec.title }))}<br><code>${esc(rec.installPath || "")}</code></p>${foot(t("pv.uninstall"), "red")}`);
  $("#ok").onclick = () => run(async () => { closeModal(); toast(t("pv.uninstalling")); await invoke("provider_uninstall", { provider, productId }); await providersLoad(true); renderAll(); toast(t("pv.uninstalled")); });
}

async function steamImportModal() {
  modal(`<h2>${esc(t("pv.importSteam"))}</h2><p class="desc">${esc(t("pv.importSteamHint"))}</p><div id="st-list" class="sline">${esc(t("log.loading"))}</div>${foot(t("pv.import"))}`, true);
  try {
    const list = await invoke("steam_candidates");
    const box = $("#st-list");
    if (!list.length) { box.textContent = t("pv.noSteam"); return; }
    box.innerHTML = `<div class="steamlist">${list.map((g) => `<label class="ck"><input type="checkbox" data-st="${g.appId}" ${g.owned ? "" : "checked"} ${g.owned ? "disabled" : ""}><span><b>${esc(g.name)}</b><small>${esc(g.exe || g.dir)}</small></span>${g.owned ? `<em>${esc(t("pv.already"))}</em>` : ""}</label>`).join("")}</div>`;
    $("#ok").onclick = () => run(async () => {
      const ids = $$("[data-st]:checked").map((el) => +el.dataset.st);
      if (!ids.length) return toast(t("pv.noneSelected"), true);
      const added = await invoke("steam_import", { appIds: ids });
      for (const g of added) setGame(g);
      closeModal(); app.view = "library"; renderAll(); if (added[0]) select(added[0].id); toast(t("pv.imported", { n: added.length }));
    });
  } catch (e) { $("#st-list").textContent = String(e); }
}

// ---------- transferts ----------
function providerProgress(p) {
  if (!p?.key) return;
  app.transfers.set(p.key, { ...p, at: Date.now() });
  renderTop();
  if (app.view === "mygames" && (p.state === "installed" || p.state === "failed")) {
    providersLoad(true).then(() => { if (app.view === "mygames") renderSheetKeepScroll(); });
  }
  if (p.state === "installed" || p.state === "failed") {
    setTimeout(() => {
      app.transfers.delete(p.key);
      renderTop();
      if (app.view === "mygames") renderSheetKeepScroll();
    }, 4500);
  }
}

function transferRow(p) {
  const pct = p.percent == null ? null : Math.max(0, Math.min(100, p.percent));
  const size = p.written != null && p.total ? `${bytes(p.written)} / ${bytes(p.total)}` : "";
  const meta = [p.stage || "", p.speed, p.eta ? `ETA ${p.eta}` : "", size].filter(Boolean).join(" · ");
  const state = p.state === "installed" ? "done" : p.state === "failed" ? "failed" : "";
  return `<div class="transfer ${state}">
    <div class="t-top"><b>${esc(p.title || p.productId)}</b><small>${esc(PV_LABEL[p.provider] || p.provider)}</small></div>
    <div class="t-bar${pct == null ? " indeterminate" : ""}"><i style="width:${pct == null ? 35 : pct}%"></i></div>
    <div class="t-meta"><span>${esc(meta || t("pv.preparing"))}</span>${pct == null ? "" : `<b>${Math.round(pct)} %</b>`}</div>
    ${p.state === "failed" ? `<div class="t-error">${esc(p.error || t("pv.installFailed"))}</div>` : ""}
    ${p.state === "installed" ? `<div class="t-ok">${esc(t("pv.installDone"))}</div>` : ""}
    ${p.state === "installing" ? `<div class="foot"><button class="btn sec sm" data-transfer-cancel="${esc(p.key)}" data-provider="${esc(p.provider)}" data-product="${esc(p.productId)}">${esc(t("pv.cancel"))}</button></div>` : ""}
  </div>`;
}

function transfersModal() {
  const rows = [...app.transfers.values()];
  modal(`<h2>${esc(t("pv.transfers"))}</h2>
    <div class="transfers">${rows.length ? rows.map(transferRow).join("") : `<div class="dim">${esc(t("pv.noTransfers"))}</div>`}</div>
    <div class="foot"><button class="btn sec" data-act="close">${esc(t("x.close"))}</button></div>`, true);
  $$("[data-transfer-cancel]").forEach((button) => (button.onclick = () => run(async () => {
    button.disabled = true;
    button.textContent = t("pv.cancelling");
    await invoke("provider_cancel", { provider: button.dataset.provider, productId: button.dataset.product });
    toast(t("pv.cancelled"));
  })));
}

document.addEventListener("click", (e) => {
  const el = e.target.closest("[data-pv]");
  if (el) { e.preventDefault(); providerAction(el.dataset.pv, el.dataset); }
});
