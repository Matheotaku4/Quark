"use strict";
// Onboarding, nouveautés, vues enregistrées et sélection multiple.

function onboardingModal() {
  const langOpts = app.data.languages.map((l) => [l.id, l.label]);
  const themeOpts = app.data.themes.map((th) => [th.id, th.label]);
  const pages = [["library", t("nav.library")], ["home", t("x.home")], ["store", t("x.store")], ["my_games", t("x.mygames")]];
  modal(`<h2>${esc(t("onboarding.title"))}</h2><p class="desc">${esc(t("onboarding.subtitle"))}</p>
    <div class="row2">${lab(t("onboarding.language"), sel("ob-lang", langOpts, app.cfg.ui_language))}${lab(t("onboarding.theme"), sel("ob-theme", themeOpts, app.cfg.theme))}</div>
    ${lab(t("onboarding.start"), sel("ob-start", pages, app.cfg.start_page))}
    ${lab(t("onboarding.install"), inp("ob-install", app.cfg.default_install_dir || "~/Games", "~/Games"))}
    <label class="check"><input type="checkbox" id="ob-auto" ${app.cfg.auto_update ? "checked" : ""}> ${esc(t("Vérifier les mises à jour"))}</label>
    <div class="foot"><button class="btn sec" data-act="close">${esc(t("onboarding.skip"))}</button><button class="btn" id="ok">${esc(t("onboarding.finish"))}</button></div>`, true);
  $("#ok").onclick = () => run(async () => {
    await cfgSet({
      ui_language: $("#ob-lang").value, theme: $("#ob-theme").value, start_page: $("#ob-start").value,
      default_install_dir: $("#ob-install").value.trim() || null,
      auto_update: $("#ob-auto").checked,
      onboarding_done: true, last_seen_version: app.data.version,
    });
    closeModal(); applySettings(); app.view = ({ home: "home", library: "library", store: "store", my_games: "mygames" })[app.cfg.start_page] || "library";
    renderAll(); toast(t("toast.saved"));
  });
}

function whatsNewModal() {
  modal(`<h2>${esc(t("whatsnew.title"))}</h2><p class="desc">${esc(t("whatsnew.body"))}</p>
    <div class="card mini"><b>Quark ${esc(app.data.version)}</b><p class="hint">${esc(t("pv.subtitle"))}</p></div>
    ${foot(t("onboarding.finish"))}`);
  $("#ok").onclick = () => run(async () => { await cfgSet({ last_seen_version: app.data.version }); closeModal(); });
}

function viewsModal() {
  const views = app.cfg.saved_views || [];
  modal(`<h2>${esc(t("views.title"))}</h2>
    ${views.length ? `<div class="viewlist">${views.map((v, i) => `<div class="viewrow"><div><b>${esc(v.name)}</b><small>${esc([v.status && t("status." + v.status), v.genre, v.tag, v.query].filter(Boolean).join(" · ") || t("pv.all"))}</small></div>
      <button class="btn sec sm" data-view-apply="${i}">${esc(t("views.apply"))}</button><button class="ib sm" data-view-del="${i}" title="${esc(t("views.delete"))}">${ic("trash")}</button></div>`).join("")}</div>` : `<p class="dim">${esc(t("views.empty"))}</p>`}
    <div class="row2"><input type="text" id="view-name" placeholder="${esc(t("views.name"))}"><button class="btn sm" id="view-save">${esc(t("views.save"))}</button></div>
    <div class="foot"><button class="btn sec" data-act="close">${esc(t("x.close"))}</button></div>`, true);
  const save = () => run(async () => {
    const name = $("#view-name").value.trim(); if (!name) return toast(t("views.name"), true);
    const view = { name, status: app.f.status || "none", tag: app.f.tag || "", genre: app.f.genre || "", query: app.query.trim(), group_by: app.cfg.group_by, view: app.cfg.view, sort: app.cfg.library_sort };
    const next = [...views.filter((v) => v.name !== name), view];
    await cfgSet({ saved_views: next });
    closeModal(); toast(t("views.saved"));
  });
  $("#view-save").onclick = save;
  $$("[data-view-apply]").forEach((b) => (b.onclick = () => run(async () => {
    const v = views[+b.dataset.viewApply]; if (!v) return;
    await cfgSet({ home_status: v.status, home_tag: v.tag, home_genre: v.genre, group_by: v.group_by, view: v.view, library_sort: v.sort });
    app.f = { status: v.status === "none" ? "" : v.status, genre: v.genre, tag: v.tag, fav: false };
    app.query = v.query || ""; $("#q").value = app.query;
    closeModal(); app.view = "library"; app.open = true; renderAll();
  })));
  $$("[data-view-del]").forEach((b) => (b.onclick = () => run(async () => { const next = views.filter((_, i) => i !== +b.dataset.viewDel); await cfgSet({ saved_views: next }); closeModal(); toast(t("views.deleted")); })));
}

function batchRemoveModal() {
  const n = app.marked.size;
  if (!n) return;
  modal(`<h2>${esc(t("batch.remove"))}</h2><p class="desc">${esc(t("batch.removeBody", { n }))}</p>${foot(t("batch.remove"), "red")}`);
  $("#ok").onclick = () => run(async () => {
    const removed = await invoke("remove_games", { ids: [...app.marked] });
    for (const id of app.marked) app.data.games = app.data.games.filter((g) => g.id !== id);
    app.marked.clear(); closeModal(); app.sel = filtered()[0]?.id ?? null; renderAll(); toast(t("toast.removed") + (removed ? ` · ${removed}` : ""));
  });
}

function batchStatusModal() {
  const ids = [...app.marked];
  modal(`<h2>${esc(t("batch.status"))}</h2><div class="pills">${ST_ORDER.map((status) => `<button data-batch-status="${status}">${esc(t("status." + status))}</button>`).join("")}</div>${foot(t("x.close"))}`);
  $("#ok").onclick = closeModal;
  $$("[data-batch-status]").forEach((button) => (button.onclick = () => run(async () => {
    for (const id of ids) setGame(await invoke("set_status", { id, status: button.dataset.batchStatus }));
    app.marked.clear(); closeModal(); renderAll(); toast(t("toast.saved"));
  })));
}
function batchFavorite() {
  const ids = [...app.marked];
  const enable = ids.some((id) => !game(id)?.favorite);
  run(async () => {
    for (const id of ids) setGame(await invoke("set_favorite", { id, favorite: enable }));
    app.marked.clear(); renderAll(); toast(t("toast.saved"));
  });
}
function renderBatch() {
  const bar = $("#batchbar"); if (!bar) return;
  bar.hidden = !app.marked.size;
  if (!app.marked.size) { bar.innerHTML = ""; return; }
  bar.innerHTML = `<span>${esc(t("batch.selected", { n: app.marked.size }))}</span><button class="btn sec sm" data-act="batch-fav">${ic("star")}${esc(t("batch.favorite"))}</button><button class="btn sec sm" data-act="batch-status">${ic("tag")}${esc(t("batch.status"))}</button><button class="btn sec sm" data-act="batch-clear">${esc(t("batch.clear"))}</button><button class="btn red sm" data-act="batch-remove">${esc(t("batch.remove"))}</button>`;
}
