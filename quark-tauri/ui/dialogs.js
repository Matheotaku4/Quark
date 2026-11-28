"use strict";
// Ajout, personnalisation, réglages d'un jeu (Réglages / DLL overrides), journal, retrait.

const getPath = (o, p) => p.split(".").reduce((x, k) => (x == null ? x : x[k]), o);
function setPath(o, p, v) { const ks = p.split("."); const last = ks.pop(); ks.reduce((x, k) => (x[k] ??= {}), o)[last] = v; }
/** Lie les champs [data-k] de `root` à l'objet `model` ; `onChange(chemin)` est appelé après chaque modification. */
function bindModel(root, model, onChange) {
  $$("[data-k]", root).forEach((el) => {
    const k = el.dataset.k, ev = el.type === "checkbox" || el.tagName === "SELECT" ? "change" : "input";
    el.addEventListener(ev, () => {
      let v = el.type === "checkbox" ? el.checked : el.value;
      if (el.dataset.num != null) v = v === "" ? (el.dataset.nullable != null ? null : 0) : Number(v);
      else if (el.dataset.nullable != null && v === "") v = null;
      if (el.type === "range") { v = Number(v); const o = el.nextElementSibling; o && (o.textContent = v); }
      setPath(model, k, v); onChange && onChange(k, el);
    });
  });
}
const bind = (k, extra = "") => `data-k="${k}" ${extra}`;
const fieldsFor = (g = {}) => `${lab(t("add.name"), inp("f-name", g.name))}
  ${lab(t("add.exe"), `<div class="row2">${inp("f-exe", g.exe, t("add.exePlaceholder"))}<button class="btn sec sm" id="browse">${esc(t("add.browse"))}</button></div>`)}${lab(t("add.args"), inp("f-args", g.args))}`;
const wireBrowse = () => { const b = $("#browse"); b && (b.onclick = () => run(async () => { const p = await invoke("pick_executable"); if (p) $("#f-exe").value = p; })); };

function addModal(initialName = "") {
  modal(`<h2>${esc(t("add.title"))}</h2>${fieldsFor({ name: initialName })}<div class="row2"><button class="btn sec sm" id="scan-open">${ic("search")}${esc(t("scan.title"))}</button><button class="btn sec sm" id="web-open">${ic("ext")}${esc(t("web.title"))}</button></div>${foot(t("add.submit"))}`); wireBrowse();
  const scan = $("#scan-open"); scan && (scan.onclick = scanModal);
  const web = $("#web-open"); web && (web.onclick = () => webSearchModal());
  $("#ok").onclick = () => run(async () => {
    const exe = $("#f-exe").value.trim(); if (!exe) return toast(t("add.exeRequired"), true);
    let g = await invoke("add_game", { name: $("#f-name").value.trim(), exe });
    if ($("#f-args").value.trim()) g = await invoke("update_game", { id: g.id, args: $("#f-args").value });
    setGame(g); closeModal(); app.f = { status: "", genre: "", tag: "", fav: false }; app.query = ""; $("#q").value = ""; app.view = "library"; renderAll(); select(g.id); toast(t("toast.added"));
  });
}

// ---------- Scan d'un dossier de jeux ----------
function scanModal() {
  modal(`<h2>${esc(t("scan.title"))}</h2>
    <div class="row2"><input type="text" id="scan-folder" placeholder="${esc(t("scan.placeholder"))}"><button class="btn sec sm" id="scan-pick">${ic("folder")}${esc(t("add.browse"))}</button></div>
    <div class="dim" id="scan-state">${esc(t("scan.hint"))}</div><div class="scanlist" id="scan-list"></div>
    <div class="foot"><button class="btn sec" data-act="close">${esc(t("x.close"))}</button><button class="btn" id="ok">${esc(t("scan.add"))}</button></div>`, true);
  const input = $("#scan-folder"), list = $("#scan-list"), state = $("#scan-state");
  const load = (folder) => run(async () => {
    state.textContent = t("scan.scanning");
    const hits = await invoke("scan_folder", { folder, maxDepth: 3, limit: 50 });
    state.textContent = hits.length ? t("scan.found", { n: hits.length }) : t("scan.none");
    list.innerHTML = hits.map((hit, index) => `<label class="scanrow"><input type="checkbox" data-scan="${index}" checked><span><b>${esc(hit.title)}</b><small>${esc(hit.exe)}</small></span></label>`).join("");
  });
  $("#scan-pick").onclick = () => run(async () => { const folder = await invoke("pick_folder"); if (!folder) return; input.value = folder; await load(folder); });
  $("#ok").onclick = () => run(async () => {
    const folder = input.value.trim(); if (!folder) return toast(t("scan.pickFirst"), true);
    let hits = list.querySelectorAll("[data-scan]:checked"); if (!hits.length) return load(folder);
    const ids = [...hits].map((el) => +el.dataset.scan);
    const result = await invoke("scan_folder", { folder, maxDepth: 3, limit: 50 });
    let added = 0, lastId = null;
    for (const index of ids) {
      const hit = result[index]; if (!hit) continue;
      try { const game = await invoke("add_game", { name: hit.title, exe: hit.exe }); lastId = game.id; added++; } catch {}
    }
    app.data = await invoke("bootstrap_data"); closeModal(); renderAll(); if (lastId) select(lastId); toast(t("scan.added", { n: added }));
  });
}
let webSearchSeq = 0;
// Les sites viennent uniquement des manifestes ajoutés par l'utilisateur.
// Aucun site de secours n'est proposé par le launcher.
function siteMeta() {
  const sites = app.data?.searchSites;
  if (!Array.isArray(sites)) return [];
  const seen = new Set();
  return sites
    .filter((site) => {
      const uid = site?.uid || site?.id;
      return uid && !seen.has(uid) && seen.add(uid);
    })
    .map((site) => ({
      uid: site.uid || site.id,
      label: site.label,
      host: site.host,
      search: (q) => String(site.searchUrl || "").replaceAll("{query}", encodeURIComponent(q)),
    }));
}

function siteColor(id) {
  let hash = 0;
  for (const char of String(id)) hash = (hash * 31 + char.codePointAt(0)) | 0;
  return `hsl(${Math.abs(hash) % 360} 65% 60%)`;
}

function webError(message) {
  const text = String(message || "");
  const http = text.match(/HTTP (\d+)/);
  if (http) return t("web.httpError", { code: http[1] });
  if (/expir/i.test(text)) return t("web.expired");
  if (/injoignable|unreachable|timeout/i.test(text)) return t("web.unreachable");
  return t("web.error");
}

function renderWebResults(state) {
  const visible = (hit) => !state.filter || hit.siteId === state.filter;
  const groups = siteMeta()
    .map((site) => ({ site, hits: state.hits.filter((hit) => hit.siteId === site.uid && visible(hit)), errors: state.errors.filter((error) => error.siteId === site.uid) }))
    .filter((group) => !state.filter || group.site.uid === state.filter)
    .filter((group) => group.hits.length || group.errors.length);
  if (!groups.length) return `<div class="webempty">${esc(t(state.loading ? "log.loading" : "web.none"))}</div>`;
  return groups.map(({ site, hits, errors }) => `<section class="webgroup">
    <div class="webhead"><span class="webdot" style="background:${siteColor(site.uid)}"></span><b>${esc(site.label)}</b><small>${esc(site.host)}</small><em>${hits.length} ${esc(t("web.results"))}</em><button class="websearch" data-open="${esc(site.search(state.q))}">${ic("ext")}${esc(t("web.siteSearch"))}</button></div>
    ${errors.map((error) => `<p class="weberr">${esc(webError(error.message))}</p>`).join("")}
    ${hits.map((hit) => `<div class="webhit"><div class="webcover">${hit.cover ? `<img src="${esc(hit.cover)}" alt="" loading="lazy">` : `<span>${esc((hit.title || "?").slice(0, 1).toUpperCase())}</span>`}</div>
      <div class="webmeta"><b>${esc(hit.title)}</b><small>${esc(hit.detail || site.label)}</small></div>
      <button class="btn sec sm" data-web-use="${esc(hit.title)}">${esc(t("web.use"))}</button>
      <button class="btn sm" data-open="${esc(hit.url)}">${ic("ext")}${esc(t("web.open"))}</button></div>`).join("")}
  </section>`).join("");
}

function webSearchModal(query = "") {
  if (typeof query !== "string") query = "";
  if (!siteMeta().length) {
    modal(`<h2>${esc(t("web.title"))}</h2><p class="desc">${esc(t("web.disabled"))}</p>
      <div class="foot"><button class="btn sec" data-act="close">${esc(t("x.close"))}</button><button class="btn" id="web-settings">${ic("gear")}${esc(t("settings.extensions"))}</button></div>`, true);
    const settings = $("#web-settings");
    if (settings) settings.onclick = () => { closeModal(); app.sq = ""; app.stab = "extensions"; app.view = "settings"; renderAll(); };
    return;
  }
  const seq = ++webSearchSeq;
  const state = { q: query, loading: false, hits: [], errors: [], filter: null };
  modal(`<h2>${esc(t("web.title"))}</h2>
    <div class="websearchrow"><input type="text" id="web-q" value="${esc(query)}" placeholder="${esc(t("web.placeholder"))}" autocomplete="off" spellcheck="false"><button class="btn" id="web-go">${ic("search")}${esc(t("web.search"))}</button></div>
    <div class="webchips" id="web-chips"></div>
    <p class="webnote">${esc(t("web.hint"))}</p>
    <div id="web-res"></div>
    <div class="foot"><button class="btn sec" data-act="close">${esc(t("x.close"))}</button></div>`, true);
  const drawChips = () => {
    const root = $("#web-chips");
    if (!root) return;
    const chips = [{ uid: null, label: t("web.filterAll") }, ...siteMeta()];
    root.innerHTML = chips.map((site) => `<button class="webchip${state.filter === site.uid ? " on" : ""}" data-webfilter="${esc(site.uid || "")}">${esc(site.label)}</button>`).join("");
    $$("[data-webfilter]", root).forEach((button) => (button.onclick = () => { state.filter = button.dataset.webfilter || null; draw(); }));
  };
  const bindResults = () => {
    $$("[data-web-use]").forEach((button) => (button.onclick = () => { closeModal(); addModal(button.dataset.webUse); }));
  };
  const draw = () => {
    const list = $("#web-res");
    if (seq !== webSearchSeq || !list) return;
    drawChips();
    if (state.loading) list.innerHTML = `<div class="webempty">${esc(t("log.loading"))}</div>`;
    else if (!state.hits.length && !state.errors.length) list.innerHTML = `<div class="webempty">${esc(t("web.none"))}</div>`;
    else list.innerHTML = renderWebResults(state);
    bindResults();
  };
  const search = () => run(async () => {
    const input = $("#web-q");
    if (!input) return;
    const q = input.value.trim();
    if (q.length < 2) return;
    state.q = q; state.loading = true; state.filter = null; draw();
    try {
      const result = await invoke("site_search", { query: q });
      state.hits = result.hits || []; state.errors = result.errors || [];
    } catch (error) {
      state.hits = []; state.errors = siteMeta().map((site) => ({ siteId: site.uid, site: site.label, message: String(error) }));
    }
    state.loading = false; draw();
  });
  $("#web-go").onclick = search;
  $("#web-q").onkeydown = (event) => { if (event.key === "Enter") { event.preventDefault(); search(); } };
  draw();
  if (query.trim().length >= 2) search();
}

function removeModal() {
  const g = game(app.sel); modal(`<h2>${esc(t("remove.title"))}</h2><p class="desc">${esc(g.name)}<br>${esc(t("remove.body"))}</p>${foot(t("remove.confirm"), "red")}`);
  $("#ok").onclick = () => run(async () => {
    await invoke("remove_game", { id: g.id }); app.data.games = app.data.games.filter((x) => x.id !== g.id); closeModal();
    app.sel = filtered()[0]?.id ?? null; renderAll(); toast(t("toast.removed"));
  });
}
function logModal() {
  const g = game(app.sel); modal(`<h2>${esc(t("x.logBtn"))}</h2><pre class="logbox" id="log-out">${esc(t("log.loading"))}</pre><div class="foot"><button class="btn sec" id="rl">${esc(t("log.refresh"))}</button><button class="btn" data-act="close">${esc(t("settings.close"))}</button></div>`, true);
  const load = () => run(async () => { const txt = await invoke("read_log", { id: g.id }); const o = $("#log-out"); o.textContent = txt.trim() ? txt.split("\n").slice(-300).join("\n") : t("log.empty"); o.scrollTop = o.scrollHeight; });
  $("#rl").onclick = load; load();
}

// ---------- Personnaliser : illustrations, notes, tags ----------
function customizeModal() {
  const id = app.sel;
  let tab = "meta";
  const art = (g, k, l, url) => `<div class="art"><div class="thumb${url ? " has" : ""}" id="th-${k}" style="${url ? `background-image:url('${esc(url)}')` : ""}">${url ? "" : `<span class="artph">${ic("image")}</span>`}</div><button class="btn sec sm" data-art="${k}">${esc(l)}</button></div>`;
  const draw = () => {
    const g = game(id); if (!g) return;
    const tabs = `<div class="tabs sm"><button data-cust="meta" class="${tab === "meta" ? "on" : ""}">${esc(t("customize.metadata"))}</button><button data-cust="images" class="${tab === "images" ? "on" : ""}">${esc(t("customize.images"))}</button></div>`;
    const body = tab === "images"
      ? `<div class="arts">${art(g, "cover", t("detail.changeCover"), g.coverUrl)}${art(g, "hero", t("detail.changeHero"), g.heroUrl)}${art(g, "logo", t("x.logo"), g.logoUrl)}${art(g, "icon", t("x.icon"), g.iconUrl)}</div>
        ${lab(t("art.search"), `<div class="row2"><input type="text" id="art-q" value="${esc(g.name)}"><button class="btn sec sm" id="art-auto">${ic("search")}${esc(t("art.auto"))}</button><button class="btn sec sm" id="art-find">${esc(t("art.choose"))}</button></div>`)}
        <p class="hint">${esc(t("customize.imagesHint"))}</p><div class="artgrid" id="art-grid"></div>`
      : `${lab(t("add.name"), inp("c-name", g.name))}<div class="row2">${lab(t("overview.developer"), inp("c-dev", g.developer))}${lab(t("overview.released"), inp("c-rel", g.released))}</div>
        ${lab(t("overview.genres"), inp("c-gen", g.genres.join(", ")))}${lab(t("overview.tags"), inp("c-tags", g.tags.join(", ")))}
        ${lab(t("overview.about"), `<textarea id="c-desc">${esc(g.description)}</textarea>`)}${lab(t("tab.notes"), `<textarea id="c-notes" placeholder="${esc(t("notes.placeholder"))}">${esc(g.notes)}</textarea>`)}`;
    modal(`<h2>${esc(t("x.customize"))}</h2>${tabs}${body}${foot(tab === "images" ? t("x.close") : t("x.save"))}`, true);
    const root = $("#modal");
    $$("[data-cust]", root).forEach((b) => (b.onclick = () => { tab = b.dataset.cust; draw(); }));
    const list = (s) => s.split(",").map((x) => x.trim()).filter(Boolean);
    if (tab === "meta") {
      $("#ok").onclick = () => run(async () => {
        await gameSet(id, { name: $("#c-name").value.trim() || g.name, developer: $("#c-dev").value, released: $("#c-rel").value, genres: list($("#c-gen").value), tags: list($("#c-tags").value), description: $("#c-desc").value, notes: $("#c-notes").value });
        closeModal(); renderAll(); toast(t("toast.saved"));
      });
      return;
    }
    $("#ok").onclick = closeModal;
    const auto = $("#art-auto"); auto && (auto.onclick = () => run(async () => {
      auto.disabled = true;
      try { setGame(await invoke("artwork_fetch", { id, query: $("#art-q").value.trim() || null })); draw(); renderAll(); toast(t("toast.saved")); }
      finally { auto.disabled = false; }
    }));
    const find = $("#art-find"); find && (find.onclick = () => run(async () => {
      const grid = $("#art-grid"); grid.innerHTML = `<div class="dim">${esc(t("art.loading"))}</div>`;
      const result = await invoke("artwork_candidates", { id, query: $("#art-q").value.trim() || null });
      grid.innerHTML = result.items.length ? result.items.map((item, index) => `<button class="artcand" data-cand="${index}" title="${esc(item.source)} · ${esc(item.title)}"><img src="${esc(item.preview || item.url)}" loading="lazy" alt=""><span>${esc(t("art." + item.kind))}</span></button>`).join("") : `<div class="dim">${esc(t("art.none"))}</div>`;
      $$("[data-cand]", grid).forEach((button) => (button.onclick = () => run(async () => {
        const item = result.items[+button.dataset.cand];
        setGame(await invoke("artwork_choose", { id, kind: item.kind, url: item.url }));
        draw(); renderAll(); toast(t("toast.saved"));
      })));
    }));
    $$("[data-art]", root).forEach((b) => (b.onclick = () => run(async () => { setGame(await invoke("pick_artwork", { id, kind: b.dataset.art })); draw(); renderAll(); })));
  };
  draw();
}

// ---------- Paramètres du jeu ----------
const ANTICHEAT = [["none", "—"], ["battle_eye", "BattlEye"], ["easy_anti_cheat", "EasyAntiCheat"], ["both", "BattlEye + EAC"]];
function parseDll(value) { // « winmm=n,b;steam_api64=b » -> [{name,native,builtin}]
  const out = [];
  for (const seg of (value || "").split(";")) { const [names, modes = ""] = seg.split("="); if (!names.trim()) continue;
    for (const name of names.split(",")) out.push({ name: name.trim(), native: modes.includes("n"), builtin: modes.includes("b") }); }
  return out;
}
const dllValue = (l) => l.map((d) => `${d.name}=${[d.native && "n", d.builtin && "b"].filter(Boolean).join(",")}`).join(";");
const dllEq = (left, right) => left.replace(/\.dll$/i, "").toLowerCase() === right.replace(/\.dll$/i, "").toLowerCase();
const dirOf = (path) => path.replace(/[/\\][^/\\]*$/, "");
async function gameSettingsModal() {
  const id = app.sel, raw = await run(() => invoke("get_game_raw", { id })); if (!raw) return;
  app.runtimes = await invoke("list_runtimes").catch(() => app.runtimes || []);
  const w = raw, st = { tab: "settings", sec: "general", dllFolder: w.dll_source || dirOf(w.exe), dlls: null, dllBusy: false, dllError: "" };
  const mode = () => (w.proton?.mode === "runtime" ? w.proton.runtime : w.proton?.mode || "default");
  const envList = () => (w.env ||= []);
  const sections = {
    general: () => `${lab(t("add.name"), `<input type="text" ${bind("name")} value="${esc(w.name)}">`)}${lab(t("add.exe"), `<div class="row2"><input type="text" ${bind("exe")} id="f-exe" value="${esc(w.exe)}"><button class="btn sec sm" id="browse">${esc(t("add.browse"))}</button></div>`)}${lab(t("add.args"), `<input type="text" ${bind("args")} value="${esc(w.args)}">`)}`,
    proton: () => `${lab("Proton", sel("pm", [["default", "Auto"], ["native", t("launch.native")], ...(app.runtimes || []).map((r) => [r.key, r.name])], mode()))}
      ${lab(t("launch.prefix"), `<input type="text" ${bind("prefix", "data-nullable")} value="${esc(w.prefix ?? "")}" placeholder="${esc(t("launch.automatic"))}">`)}
      ${lab(t("launch.anticheat"), `<select ${bind("anticheat")}>${ANTICHEAT.map(([v, l]) => `<option value="${v}" ${w.anticheat === v ? "selected" : ""}>${l}</option>`).join("")}</select>`)}
      <div class="sw2">${["umu", "proton_log", "launch.gamemode", "launch.mangohud", "launch.prime", "launch.gamescope", "launch.gamescope_fsr"].map((k) => `<label class="ck"><input type="checkbox" ${bind(k)} ${getPath(w, k) ? "checked" : ""}>${({ umu: "umu", proton_log: "PROTON_LOG", "launch.gamemode": "GameMode", "launch.mangohud": "MangoHud", "launch.prime": "PRIME", "launch.gamescope": "gamescope", "launch.gamescope_fsr": "FSR" })[k]}</label>`).join("")}</div>
      <div class="row2">${lab("gamescope L", `<input type="text" ${bind("launch.gamescope_width", "data-num")} value="${w.launch?.gamescope_width ?? 0}">`)}${lab("H", `<input type="text" ${bind("launch.gamescope_height", "data-num")} value="${w.launch?.gamescope_height ?? 0}">`)}${lab("FPS", `<input type="text" ${bind("launch.gamescope_fps", "data-num")} value="${w.launch?.gamescope_fps ?? 0}">`)}</div>
      ${lab(t("launch.wrapper"), `<input type="text" ${bind("launch.wrapper")} value="${esc(w.launch?.wrapper ?? "")}" placeholder="wrapper %command%">`)}${lab("umu GAMEID", `<input type="text" ${bind("umu_game_id")} value="${esc(w.umu_game_id ?? "")}">`)}`,
    env: () => `<div class="envs">${envList().filter((e) => e.name !== "WINEDLLOVERRIDES").map((e) => `<div class="row2"><input type="text" data-ev="name" data-i="${envList().indexOf(e)}" value="${esc(e.name)}" placeholder="NAME"><input type="text" data-ev="value" data-i="${envList().indexOf(e)}" value="${esc(e.value)}" placeholder="value"><button class="ib sm" data-evdel="${envList().indexOf(e)}">${ic("x")}</button></div>`).join("") || `<div class="dim">${esc(t("x.noEnv"))}</div>`}</div><button class="btn sec sm" id="env-add">${ic("plus")}${esc(t("x.addVar"))}</button>`,
    data: () => `${lab("Steam AppID", `<input type="text" ${bind("steam_app_id", "data-num data-nullable")} value="${w.steam_app_id ?? ""}">`)}${lab("SteamGridDB ID", `<input type="text" ${bind("sgdb_id", "data-num data-nullable")} value="${w.sgdb_id ?? ""}">`)}
      <div class="foot l"><button class="btn sec sm" id="open-px" ${game(id).prefix ? "" : "disabled"}>${ic("folder")}${esc(t("detail.openFolder"))} · ${esc(t("launch.prefix"))}</button></div>`,
  };
  const dllView = () => {
    const l = parseDll(w.env?.find((e) => e.name === "WINEDLLOVERRIDES")?.value);
    const found = st.dlls || [];
    return `<div class="dllbar"><div><b>${esc(t("x.dllFolder"))}</b><small class="path">${esc(st.dllFolder || "—")}</small></div>
        <button class="btn sec sm" id="dll-pick">${ic("folder")}${esc(t("x.dllChange"))}</button>
        <button class="btn sec sm" id="dll-rescan">${ic("refresh")}${esc(t("x.dllScan"))}</button></div>
      ${st.dllBusy ? `<div class="dim">${esc(t("log.loading"))}</div>` : st.dllError ? `<div class="sline err">${esc(st.dllError)}</div>` : found.length ? `<div class="dllgrid">${found.map((d) => {
        const on = l.some((row) => dllEq(row.name, d.name));
        return `<label class="dllrow"><input type="checkbox" data-dll="${esc(d.name)}" ${on ? "checked" : ""}><span><b>${esc(d.name)}.dll</b><small>${esc(d.relative)}</small></span></label>`;
      }).join("")}</div>` : `<div class="dim">${esc(t("x.dllNone"))}</div>`}
      <h4 class="dlltitle">${esc(t("x.dllActive"))}</h4>
      <div class="envs">${l.map((d, i) => `<div class="row2 dll"><b>${esc(d.name)}</b><label class="ck"><input type="checkbox" data-dn="${i}" ${d.native ? "checked" : ""}>${t("x.native")}</label><label class="ck"><input type="checkbox" data-db="${i}" ${d.builtin ? "checked" : ""}>${t("x.builtin")}</label><button class="ib sm" data-ddel="${i}">${ic("x")}</button></div>`).join("") || `<div class="dim">${esc(t("x.noDll"))}</div>`}</div>
      <div class="row2"><input type="text" id="dll-new" placeholder="winmm, d3d11…"><button class="btn sec sm" id="dll-add">${ic("plus")}${esc(t("x.addDll"))}</button></div>`; };
  const setDll = (l) => { const e = envList(), i = e.findIndex((x) => x.name === "WINEDLLOVERRIDES"), v = dllValue(l); if (!l.length) { if (i >= 0) e.splice(i, 1); } else i >= 0 ? (e[i].value = v) : e.push({ name: "WINEDLLOVERRIDES", value: v }); };
  const setDllChecked = (name, checked) => {
    const l = parseDll(w.env?.find((e) => e.name === "WINEDLLOVERRIDES")?.value);
    const index = l.findIndex((row) => dllEq(row.name, name));
    if (checked && index < 0) l.push({ name, native: true, builtin: true });
    if (!checked && index >= 0) l.splice(index, 1);
    setDll(l);
  };
  async function refreshDlls() {
    if (st.dllBusy) return;
    st.dllBusy = true; st.dllError = "";
    if (st.tab === "dll") draw();
    try { st.dlls = await invoke("detect_game_dlls", { id, folder: st.dllFolder || null }); }
    catch (e) { st.dlls = []; st.dllError = String(e); }
    st.dllBusy = false;
    if (st.tab === "dll") draw();
  }
  const draw = () => {
    const secs = [["general", "x.general"], ["proton", "x.proton"], ["env", "x.env"], ["data", "x.data"]];
    modal(`<h2>${esc(w.name)}</h2><div class="tabs sm"><button data-t="settings" class="${st.tab === "settings" ? "on" : ""}">${esc(t("x.settingsTab"))}</button><button data-t="dll" class="${st.tab === "dll" ? "on" : ""}">${esc(t("x.dll"))}</button></div>
      ${st.tab === "settings" ? `<div class="pills">${secs.map(([k, l]) => `<button data-s="${k}" class="${st.sec === k ? "on" : ""}">${esc(t(l))}</button>`).join("")}</div><div class="gsbody">${sections[st.sec]()}</div>` : `<div class="gsbody">${dllView()}</div>`}${foot(t("x.save"))}`, true);
    const root = $("#modal"); wireBrowse(); bindModel(root, w); syncLaunchDeps(root, "launch.");
    $$("[data-t]", root).forEach((b) => (b.onclick = () => { st.tab = b.dataset.t; draw(); $("#modal .modal").scrollTop = 0; })); $$("[data-s]", root).forEach((b) => (b.onclick = () => { st.sec = b.dataset.s; draw(); $("#modal .modal").scrollTop = 0; }));
    const pm = $("#pm"); pm && (pm.onchange = () => { w.proton = pm.value === "default" || pm.value === "native" ? { mode: pm.value } : { mode: "runtime", runtime: pm.value }; });
    $$("[data-ev]", root).forEach((el) => (el.oninput = () => (envList()[+el.dataset.i][el.dataset.ev] = el.value))); $$("[data-evdel]", root).forEach((b) => (b.onclick = () => { envList().splice(+b.dataset.evdel, 1); draw(); }));
    const ea = $("#env-add"); ea && (ea.onclick = () => { envList().push({ name: "", value: "" }); draw(); });
    const l = parseDll(w.env?.find((e) => e.name === "WINEDLLOVERRIDES")?.value);
    $$("[data-dn]", root).forEach((c) => (c.onchange = () => { l[+c.dataset.dn].native = c.checked; setDll(l); })); $$("[data-db]", root).forEach((c) => (c.onchange = () => { l[+c.dataset.db].builtin = c.checked; setDll(l); }));
    $$("[data-ddel]", root).forEach((b) => (b.onclick = () => { l.splice(+b.dataset.ddel, 1); setDll(l); draw(); }));
    $$("[data-dll]", root).forEach((c) => (c.onchange = () => { setDllChecked(c.dataset.dll, c.checked); draw(); }));
    const dp = $("#dll-pick"); dp && (dp.onclick = () => run(async () => {
      const picked = await invoke("pick_folder");
      if (!picked) return;
      st.dllFolder = picked; st.dlls = null; w.dll_source = picked;
      await gameSet(id, { dll_source: picked });
      await refreshDlls();
    }));
    const dr = $("#dll-rescan"); dr && (dr.onclick = () => refreshDlls());
    const da = $("#dll-add"); da && (da.onclick = () => { const n = $("#dll-new").value.split(",").map((x) => x.trim()).filter(Boolean); n.forEach((x) => l.push({ name: x, native: true, builtin: true })); setDll(l); draw(); });
    if (st.tab === "dll" && st.dlls === null && !st.dllBusy) refreshDlls();
    const op = $("#open-px"); op && (op.onclick = () => run(() => invoke("open_url", { target: game(id).prefix })));
    $("#ok").onclick = () => run(async () => {
      envList(); w.env = w.env.filter((e) => e.name.trim());
      await gameSet(id, { name: w.name, exe: w.exe, args: w.args, proton: w.proton, prefix: w.prefix, umu: w.umu, umu_game_id: w.umu_game_id, proton_log: w.proton_log, anticheat: w.anticheat, launch: w.launch, env: w.env, steam_app_id: w.steam_app_id, sgdb_id: w.sgdb_id, dll_source: w.dll_source });
      closeModal(); renderAll(); toast(t("toast.saved"));
    });
  };
  draw();
}

async function updatesModal() {
  modal(`<h2>${esc(t("x.updates"))}</h2><p class="desc" id="upd">${esc(t("log.loading"))}</p>${foot(t("settings.close"))}`); $("#ok").onclick = closeModal;
  const u = await run(() => invoke("check_update")); if (!u) return; app.upd = u; renderTop();
  const el = $("#upd"); if (!el) return;
  if (!u.newer) { el.innerHTML = `Quark ${esc(u.current)} ✓`; return; }
  el.innerHTML = `${esc(u.name || u.tag)} <b>(${esc(u.tag)})</b><div class="foot l"><button class="btn sm" id="upd-install">${ic("down")}${esc(t("x.updateInstall"))}</button><button class="btn sec sm" data-open="${esc(u.url)}">${ic("ext")}${esc(t("x.updates"))}</button></div>`;
  $("#upd-install").onclick = () => run(async () => {
    const b = $("#upd-install"); if (!b) return;
    b.disabled = true; b.textContent = t("x.updateInstalling");
    await invoke("install_update");
  });
}
