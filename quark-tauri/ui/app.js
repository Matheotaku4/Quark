"use strict";
// Actions, événements globaux, palette de commandes et démarrage.

const acts = {
  play: () => run(async () => {
    const g = game(app.sel); if (!g) return;
    if (app.running.has(g.id)) { const r = await invoke("stop_game", { id: g.id }); app.running.set(g.id, { ...app.running.get(g.id), ...r, at: Date.now(), stopping: true }); }
    else { setGame(await invoke("launch_game", { id: g.id })); app.running.set(g.id, { id: g.id, elapsed: 0, at: Date.now(), label: "" }); toast(t("toast.launched", { name: g.name })); }
    renderList(); renderSheet();
  }),
  fav: () => run(async () => { const g = game(app.sel); setGame(await invoke("set_favorite", { id: g.id, favorite: !g.favorite })); renderList(); renderSheet(); }),
  gsettings: () => gameSettingsModal(), customize: () => customizeModal(), log: () => logModal(), remove: () => removeModal(), add: () => addModal(),
  sites: () => webSearchModal(game(app.sel)?.name || ""),
  settings: () => setView(app.view === "settings" ? "library" : "settings"), close: () => closeModal(), palette: () => palette(), updates: () => updatesModal(), deck: () => QuarkDeck.toggle(), transfers: () => transfersModal(),
  back: () => { app.open = false; renderSheet(); },
  "toggle-view": () => run(async () => { await cfgSet({ view: app.cfg.view === "grid" ? "list" : "grid" }); app.open = false; app.view = "library"; renderAll(); }),
  "news-refresh": () => { const g = game(app.sel); g && loadNews(g, true); },
  "reset-filters": () => { app.f = { status: "", genre: "", tag: "", fav: false }; renderSide(); renderList(); renderSheet(); },
  "open-folder": () => run(() => invoke("open_folder", { id: app.sel })),
  views: () => viewsModal(), "batch-remove": () => batchRemoveModal(), "batch-status": () => batchStatusModal(), "batch-fav": () => batchFavorite(), "compare-ach": () => achievementsCompareModal(),
  "batch-clear": () => { app.marked.clear(); renderList(); },
};

let ctxGame = null;
function closeContextMenu() { const m = $("#ctxmenu"); ctxGame = null; if (m) { m.hidden = true; m.innerHTML = ""; } }
function showContextMenu(event, g) {
  event.preventDefault();
  ctxGame = g.id;
  app.sel = g.id; app.open = true; updateSelection();
  const item = (action, icn, label) => `<button data-ctx="${action}">${ic(icn)}<span>${esc(label)}</span></button>`;
  const m = $("#ctxmenu");
  m.innerHTML = `<div class="ctxhead">${esc(g.name)}</div>
    ${item("play", app.running.has(g.id) ? "stop" : "play", t(app.running.has(g.id) ? "detail.stop" : "detail.play"))}
    ${item("gsettings", "gear", t("x.settingsGame"))}${item("customize", "edit", t("x.customize"))}
    ${item("log", "log", t("x.logBtn"))}${item("open-folder", "folder", t("detail.openFolder"))}
    <div class="ctxsep"></div>${item("fav", "star", t(g.favorite ? "detail.unfavorite" : "detail.favorite"))}
    <div class="ctxsep"></div><small class="ctxlabel">${esc(t("status.label"))}</small>
    ${ST_ORDER.map((s) => `<button data-ctx="status:${s}" class="${g.status === s ? "on" : ""}"><i class="ctxdot" style="background:${s === "none" ? "var(--text-faint)" : g.statusColor}"></i><span>${esc(t("status." + s))}</span></button>`).join("")}
    <div class="ctxsep"></div>${item("remove", "trash", t("x.remove"))}`;
  m.hidden = false;
  const r = m.getBoundingClientRect();
  m.style.left = `${Math.min(event.clientX, innerWidth - r.width - 8)}px`;
  m.style.top = `${Math.min(event.clientY, innerHeight - r.height - 8)}px`;
}

// ---------- palette (Ctrl+K) ----------
function palette() {
  app.pal = { q: "", i: 0 };
  $("#modal").innerHTML = `<div class="scrim" data-act="close-scrim"><div class="pal"><input id="pq" placeholder="${esc(t("x.palette"))}" autocomplete="off" spellcheck="false"><div class="res" id="pres"></div></div></div>`;
  const input = $("#pq"); input.focus(); input.oninput = () => { app.pal = { q: input.value, i: 0 }; palList(); }; palList();
}
function palItems() {
  const q = norm(app.pal.q.trim());
  const games = app.data.games.filter((g) => !q || norm(g.name).includes(q)).sort((a, b) => (b.lastPlayed || 0) - (a.lastPlayed || 0)).slice(0, 8).map((g) => ({ label: g.name, hint: rel(g.lastPlayed), go: () => { app.view = "library"; renderAll(); select(g.id); }, play: g.id }));
  const providerGames = (app.pv?.data?.games || []).filter((g) => !q || norm(g.title).includes(q)).slice(0, 6).map((g) => ({ label: g.title, hint: PV_LABEL[g.provider] || g.provider, icon: "grid", action: true, go: () => { app.pv.q = g.title; app.pv.provider = ""; app.view = "mygames"; renderAll(); } }));
  const actions = [[t("add.title"), addModal, "plus"], [t("settings.title"), () => setView("settings"), "gear"], [t("x.home"), () => setView("home"), "home"], [t("nav.library"), () => setView("library"), "grid"], ...(app.pal.q.trim().length >= 2 ? [[t("web.title"), () => webSearchModal(app.pal.q.trim()), "search"]] : [])].filter(([l]) => !q || norm(l).includes(q)).map(([label, go, icon]) => ({ label, hint: "", go, icon, action: true }));
  return [...games, ...providerGames, ...actions];
}
function palList() {
  const items = palItems(); app.pal.items = items;
  $("#pres").innerHTML = items.length ? items.map((x, n) => `<button data-n="${n}" class="${n === app.pal.i ? "on" : ""}">${ic(x.action ? x.icon : "play")}<span>${esc(x.label)}</span><small>${esc(x.hint)}</small></button>`).join("") : `<div class="none">${esc(t("x.none"))}</div>`;
}
function palGo(n, play) { const x = app.pal.items[n]; if (!x) return; closeModal(); x.go(); if (play && x.play) setTimeout(acts.play, 0); }

// ---------- événements ----------
document.addEventListener("click", (e) => {
  const trailer = e.target.closest("[data-trailer]");
  if (trailer) { playTrailer(trailer); return; }
  const ctx = e.target.closest("[data-ctx]");
  if (ctx) {
    const action = ctx.dataset.ctx, id = ctxGame;
    closeContextMenu();
    const g = game(id);
    if (!g) return;
    if (action.startsWith("status:")) return run(async () => { setGame(await invoke("set_status", { id: g.id, status: action.slice(7) })); renderList(); renderSheet(); });
    if (action === "open-folder") return run(() => invoke("open_folder", { id: g.id }));
    app.sel = g.id;
    acts[action]?.();
    return;
  }
  if (!e.target.closest("#ctxmenu")) closeContextMenu();
  const row = e.target.closest("#list .row, #sheet .tile");
  if (row && row.dataset.id) {
    const now = Date.now(), prev = app.lastClick;
    if (prev && prev.id === row.dataset.id && now - prev.at < 360) {
      app.lastClick = null;
      const id = row.dataset.id;
      if (app.sel !== id) select(id);
      setTimeout(acts.play, 0);
      return;
    }
    app.lastClick = { id: row.dataset.id, at: now };
  } else app.lastClick = null;
  const a = e.target.closest("[data-act]");
  if (a && a.dataset.act === "close-scrim") { if (e.target === a) closeModal(); }
  else if (a) { const d = a.closest("details"); d && (d.open = false); acts[a.dataset.act]?.(); }
  // Le body porte aussi data-view : sans ce test, chaque clic relance un rendu complet.
  const v = e.target.closest("[data-view]"); if (v && v.dataset.view !== app.view) setView(v.dataset.view);
  const inst = e.target.closest("[data-installation]");
  if (inst) { e.preventDefault(); return run(async () => { const g = await invoke("activate_installation", { id: app.sel, installationId: inst.dataset.installation }); setGame(g); renderAll(); toast(t("toast.saved")); }); }
  const r = e.target.closest(".row, .tile"); if (r) {
    if (e.ctrlKey || e.metaKey) { app.marked.has(r.dataset.id) ? app.marked.delete(r.dataset.id) : app.marked.add(r.dataset.id); renderList(); if (document.body.classList.contains("grid")) renderSheet(); return; }
    if (r.dataset.go) { app.view = "library"; app.open = true; renderAll(); select(r.dataset.id); if (r.dataset.launch) setTimeout(acts.play, 0); } else select(r.dataset.id);
  }
  const gr = e.target.closest("[data-group]"); if (gr) { const k = gr.dataset.group; app.collapsed.has(k) ? app.collapsed.delete(k) : app.collapsed.add(k); renderList(); }
  const tb = e.target.closest("[data-tab]"); if (tb) { app.tab = tb.dataset.tab; $$(".tabs [data-tab]").forEach((b) => b.classList.toggle("on", b === tb)); const g = game(app.sel); g && renderTab(g); }
  const s = e.target.closest("[data-status]"); if (s) { const d = s.closest("details"); d && (d.open = false); run(async () => { setGame(await invoke("set_status", { id: app.sel, status: s.dataset.status })); renderList(); renderSheet(); }); }
  const st = e.target.closest("[data-stab]"); if (st) { app.stab = st.dataset.stab; app.sq = ""; renderSheet(); $("#sheet").scrollTop = 0; }
  const th = e.target.closest("[data-theme]"); if (th) run(async () => { await cfgSet({ theme: th.dataset.theme }); renderSheet(); });
  const o = e.target.closest("[data-open]"); if (o && o.dataset.open) run(() => invoke("open_url", { target: o.dataset.open }));
  const hi = e.target.closest("[data-hi]"); if (hi) { app.hi = +hi.dataset.hi; app.hTick = 0; renderSheet(); }
  const w = e.target.closest("[data-win]"); if (w) invoke(w.dataset.win, {});
  const pn = e.target.closest("#pres [data-n]"); if (pn) palGo(+pn.dataset.n);
  $$("details.menu[open]").forEach((d) => d.contains(e.target) || (d.open = false));
});
document.addEventListener("contextmenu", (e) => {
  const row = e.target.closest("#list .row, #sheet .tile");
  if (!row || !row.dataset.id) return;
  const g = game(row.dataset.id); if (g) showContextMenu(e, g);
});
let qTimer;
$("#q").addEventListener("input", (e) => {
  app.query = e.target.value;
  clearTimeout(qTimer);
  qTimer = setTimeout(() => { renderList(); if (document.body.classList.contains("grid")) renderSheet(); }, 90);
});
document.addEventListener("change", (e) => {
  const id = e.target.id; if (!id) return;
  if (id === "sort") run(() => cfgSet({ library_sort: e.target.value })).then(renderList);
  else if (id === "group") run(() => cfgSet({ group_by: e.target.value })).then(renderList);
  else if (id.startsWith("f-")) { app.f = { status: $("#f-status").value, genre: $("#f-genre").value, tag: $("#f-tag").value, fav: $("#f-fav").checked }; renderList(); if (document.body.classList.contains("grid")) renderSheet(); }
});
document.addEventListener("keydown", (e) => {
  if (app.pal) {
    const n = app.pal.items.length;
    if (e.key === "ArrowDown" || e.key === "ArrowUp") { e.preventDefault(); app.pal.i = (app.pal.i + (e.key === "ArrowDown" ? 1 : n - 1)) % Math.max(1, n); palList(); }
    else if (e.key === "Enter") palGo(app.pal.i, e.shiftKey); else if (e.key === "Escape") closeModal();
    return;
  }
  const typing = /INPUT|TEXTAREA|SELECT/.test(document.activeElement.tagName), lib = app.view === "library" && !$("#modal").firstChild;
  if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "k") { e.preventDefault(); palette(); }
  else if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "a" && lib) { e.preventDefault(); app.marked = new Set(filtered().map((g) => g.id)); renderList(); }
  else if (e.key === "Escape") { if (!$("#ctxmenu")?.hidden) closeContextMenu(); else if ($("#modal").firstChild) closeModal(); else if (app.query) { app.query = $("#q").value = ""; $("#q").blur(); renderList(); } else if (app.view === "settings") setView("library"); }
  else if (e.key === "/" && !typing && lib) { e.preventDefault(); $("#q").focus(); }
  else if (!typing && lib && !document.body.classList.contains("grid") && (e.key === "ArrowDown" || e.key === "ArrowUp")) {
    e.preventDefault(); const l = filtered(), i = l.findIndex((g) => g.id === app.sel), n = l[Math.max(0, Math.min(l.length - 1, i + (e.key === "ArrowDown" ? 1 : -1)))]; n && select(n.id);
  } else if (!typing && lib && e.key === "Enter") { e.preventDefault(); acts.play(); }
});

function tick() {
  $$("[data-live]").forEach((el) => (el.textContent = clock(elapsed(el.dataset.live))));
  if (app.view === "home" && app.cfg.carousel_auto && !$("#modal").firstChild) { app.hTick = (app.hTick || 0) + 1; if (app.hTick >= (app.cfg.carousel_secs || 8)) { app.hTick = 0; app.hi = (app.hi || 0) + 1; renderSheet(); } }
}
function wireWindow() {
  const d = $("#top");
  d.addEventListener("mousedown", (e) => e.button === 0 && !e.target.closest("button,input") && invoke("win_start_drag", {}));
  d.addEventListener("dblclick", (e) => !e.target.closest("button,input") && invoke("win_toggle_maximize", {}));
  const E = { n: "top:0;left:6px;right:6px;height:4px;cursor:ns-resize", s: "bottom:0;left:6px;right:6px;height:4px;cursor:ns-resize", e: "right:0;top:6px;bottom:6px;width:4px;cursor:ew-resize", w: "left:0;top:6px;bottom:6px;width:4px;cursor:ew-resize", ne: "top:0;right:0;width:8px;height:8px;cursor:nesw-resize", nw: "top:0;left:0;width:8px;height:8px;cursor:nwse-resize", se: "bottom:0;right:0;width:8px;height:8px;cursor:nwse-resize", sw: "bottom:0;left:0;width:8px;height:8px;cursor:nesw-resize" };
  for (const [dir, css] of Object.entries(E)) { const h = Object.assign(document.createElement("div"), { className: "rz" }); h.style.cssText = css; h.onmousedown = (e) => { e.preventDefault(); invoke("win_start_resize", { direction: dir }); }; document.body.append(h); }
}

async function boot() {
  wireWindow();
  app.data = await invoke("bootstrap_data");
  Object.assign(app.cfg, await invoke("get_settings_raw"));
  setRunning(app.data.running);
  applySettings();
  app.view = ({ home: "home", library: "library", store: "store", my_games: "mygames" })[app.cfg.start_page] || "library";
  let saved = null; try { saved = localStorage.getItem("quark.sel"); } catch {}
  app.sel = game(saved) ? saved : filtered()[0]?.id ?? null; app.open = app.cfg.view !== "grid";
  app.tab = app.cfg.detail_tab || "overview";
  renderAll();
  providersLoad().catch(() => {});
  await QuarkDeck.init();
  if (!app.cfg.onboarding_done) onboardingModal();
  else if (app.cfg.last_seen_version !== app.data.version) whatsNewModal();
  invoke("list_runtimes").then((r) => (app.runtimes = r)).catch(() => (app.runtimes = []));
  if (app.cfg.auto_update) invoke("check_update").then((u) => { app.upd = u; renderTop(); }).catch(() => {});
  await listen("quark://session-started", (e) => { app.running.set(e.payload.id, { id: e.payload.id, elapsed: 0, at: Date.now(), label: e.payload.label }); app.view === "library" && renderList(); app.sel === e.payload.id && renderSheet(); });
  await listen("quark://session-stopping", (e) => { const r = app.running.get(e.payload.id); r && (r.stopping = true); app.view === "library" && app.sel === e.payload.id && renderSheet(); });
  await listen("quark://session-ended", (e) => {
    const { id, secs, game: g } = e.payload; app.running.delete(id); g && setGame(g);
    const n = game(id); n && toast(t("toast.stopped", { name: n.name }) + ` · ${hm(secs)}`); app.view === "library" && renderList(); app.sel === id && app.view === "library" && renderSheet();
  });
  await listen("quark://gamepad", (e) => QuarkDeck.handle(e.payload.action, e.payload.pressed));
  await listen("quark://provider-install-progress", (e) => providerProgress(e.payload));
  await listen("quark://notice", (e) => toast(e.payload.message, !!e.payload.error));
  await listen("quark://screenshot-saved", (e) => toast(t("toast.screenshot", { path: e.payload.path })));
  await listen("quark://extensions-updated", async () => {
    try {
      app.data = await invoke("bootstrap_data");
      if (app.view === "settings" && app.stab === "extensions") renderSheetKeepScroll();
    } catch {}
  });
  setInterval(tick, 1000);
  setInterval(() => run(async () => { setRunning(await invoke("running_games")); if (app.view === "library") renderList(); }), 15000);
}
boot().catch((e) => { document.body.textContent = String(e); });
