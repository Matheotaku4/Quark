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

function addModal() {
  modal(`<h2>${esc(t("add.title"))}</h2>${fieldsFor()}${foot(t("add.submit"))}`); wireBrowse();
  $("#ok").onclick = () => run(async () => {
    const exe = $("#f-exe").value.trim(); if (!exe) return toast(t("add.exeRequired"), true);
    let g = await invoke("add_game", { name: $("#f-name").value.trim(), exe });
    if ($("#f-args").value.trim()) g = await invoke("update_game", { id: g.id, args: $("#f-args").value });
    setGame(g); closeModal(); app.f = { status: "", genre: "", tag: "", fav: false }; app.query = ""; $("#q").value = ""; app.view = "library"; renderAll(); select(g.id); toast(t("toast.added"));
  });
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
  const g = game(app.sel), art = (k, l, url) => `<div class="art"><div class="thumb" id="th-${k}" style="${url ? `background-image:url('${esc(url)}')` : ""}"></div><button class="btn sec sm" data-art="${k}">${esc(l)}</button></div>`;
  modal(`<h2>${esc(t("x.customize"))}</h2><div class="arts">${art("cover", t("detail.changeCover"), g.coverUrl)}${art("hero", t("detail.changeHero"), g.heroUrl)}${art("logo", "Logo", g.logoUrl)}${art("icon", "Icône", g.iconUrl)}</div>
    <div class="row2"><input type="text" id="art-q" value="${esc(g.name)}" placeholder="${esc(t("add.name"))}"><button class="btn sec sm" id="art-auto">${ic("search")}${esc(t("art.auto"))}</button></div>
    ${lab(t("add.name"), inp("c-name", g.name))}<div class="row2">${lab(t("overview.developer"), inp("c-dev", g.developer))}${lab(t("overview.released"), inp("c-rel", g.released))}</div>
    ${lab(t("overview.genres"), inp("c-gen", g.genres.join(", ")))}${lab(t("overview.tags"), inp("c-tags", g.tags.join(", ")))}
    ${lab(t("overview.about"), `<textarea id="c-desc">${esc(g.description)}</textarea>`)}${lab(t("tab.notes"), `<textarea id="c-notes" placeholder="${esc(t("notes.placeholder"))}">${esc(g.notes)}</textarea>`)}${foot(t("x.save"))}`, true);
  $("#art-auto").onclick = () => run(async () => { $("#art-auto").disabled = true; try { const n = await invoke("artwork_fetch", { id: g.id, query: $("#art-q").value.trim() || null }); setGame(n); closeModal(); renderAll(); toast(t("toast.saved")); } finally { $("#art-auto").disabled = false; } });
  $$("[data-art]").forEach((b) => (b.onclick = () => run(async () => { const n = await invoke("pick_artwork", { id: g.id, kind: b.dataset.art }); setGame(n); const url = { cover: n.coverUrl, hero: n.heroUrl, logo: n.logoUrl, icon: n.iconUrl }[b.dataset.art]; if (url) $("#th-" + b.dataset.art).style.backgroundImage = `url('${url}')`; renderAll(); })));
  const list = (s) => s.split(",").map((x) => x.trim()).filter(Boolean);
  $("#ok").onclick = () => run(async () => {
    await gameSet(g.id, { name: $("#c-name").value.trim() || g.name, developer: $("#c-dev").value, released: $("#c-rel").value, genres: list($("#c-gen").value), tags: list($("#c-tags").value), description: $("#c-desc").value, notes: $("#c-notes").value });
    closeModal(); renderAll(); toast(t("toast.saved"));
  });
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
async function gameSettingsModal() {
  const id = app.sel, raw = await run(() => invoke("get_game_raw", { id })); if (!raw) return;
  app.runtimes ||= await invoke("list_runtimes").catch(() => []);
  const w = raw, st = { tab: "settings", sec: "general" };
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
  const dllView = () => { const l = parseDll(w.env?.find((e) => e.name === "WINEDLLOVERRIDES")?.value);
    return `<div class="envs">${l.map((d, i) => `<div class="row2 dll"><b>${esc(d.name)}</b><label class="ck"><input type="checkbox" data-dn="${i}" ${d.native ? "checked" : ""}>${t("x.native")}</label><label class="ck"><input type="checkbox" data-db="${i}" ${d.builtin ? "checked" : ""}>${t("x.builtin")}</label><button class="ib sm" data-ddel="${i}">${ic("x")}</button></div>`).join("") || `<div class="dim">${esc(t("x.noDll"))}</div>`}</div>
      <div class="row2"><input type="text" id="dll-new" placeholder="winmm, d3d11…"><button class="btn sec sm" id="dll-add">${ic("plus")}${esc(t("x.addDll"))}</button></div>`; };
  const setDll = (l) => { const e = envList(), i = e.findIndex((x) => x.name === "WINEDLLOVERRIDES"), v = dllValue(l); if (!l.length) { if (i >= 0) e.splice(i, 1); } else i >= 0 ? (e[i].value = v) : e.push({ name: "WINEDLLOVERRIDES", value: v }); };
  const draw = () => {
    const secs = [["general", "x.general"], ["proton", "x.proton"], ["env", "x.env"], ["data", "x.data"]];
    modal(`<h2>${esc(w.name)}</h2><div class="tabs sm"><button data-t="settings" class="${st.tab === "settings" ? "on" : ""}">${esc(t("x.settingsTab"))}</button><button data-t="dll" class="${st.tab === "dll" ? "on" : ""}">${esc(t("x.dll"))}</button></div>
      ${st.tab === "settings" ? `<div class="pills">${secs.map(([k, l]) => `<button data-s="${k}" class="${st.sec === k ? "on" : ""}">${esc(t(l))}</button>`).join("")}</div><div class="gsbody">${sections[st.sec]()}</div>` : `<div class="gsbody">${dllView()}</div>`}${foot(t("x.save"))}`, true);
    const root = $("#modal"); wireBrowse(); bindModel(root, w);
    $$("[data-t]", root).forEach((b) => (b.onclick = () => { st.tab = b.dataset.t; draw(); })); $$("[data-s]", root).forEach((b) => (b.onclick = () => { st.sec = b.dataset.s; draw(); }));
    const pm = $("#pm"); pm && (pm.onchange = () => { w.proton = pm.value === "default" || pm.value === "native" ? { mode: pm.value } : { mode: "runtime", runtime: pm.value }; });
    $$("[data-ev]", root).forEach((el) => (el.oninput = () => (envList()[+el.dataset.i][el.dataset.ev] = el.value))); $$("[data-evdel]", root).forEach((b) => (b.onclick = () => { envList().splice(+b.dataset.evdel, 1); draw(); }));
    const ea = $("#env-add"); ea && (ea.onclick = () => { envList().push({ name: "", value: "" }); draw(); });
    const l = parseDll(w.env?.find((e) => e.name === "WINEDLLOVERRIDES")?.value);
    $$("[data-dn]", root).forEach((c) => (c.onchange = () => { l[+c.dataset.dn].native = c.checked; setDll(l); })); $$("[data-db]", root).forEach((c) => (c.onchange = () => { l[+c.dataset.db].builtin = c.checked; setDll(l); }));
    $$("[data-ddel]", root).forEach((b) => (b.onclick = () => { l.splice(+b.dataset.ddel, 1); setDll(l); draw(); }));
    const da = $("#dll-add"); da && (da.onclick = () => { const n = $("#dll-new").value.split(",").map((x) => x.trim()).filter(Boolean); n.forEach((x) => l.push({ name: x, native: true, builtin: true })); setDll(l); draw(); });
    const op = $("#open-px"); op && (op.onclick = () => run(() => invoke("open_url", { target: game(id).prefix })));
    $("#ok").onclick = () => run(async () => {
      envList(); w.env = w.env.filter((e) => e.name.trim());
      await gameSet(id, { name: w.name, exe: w.exe, args: w.args, proton: w.proton, prefix: w.prefix, umu: w.umu, umu_game_id: w.umu_game_id, proton_log: w.proton_log, anticheat: w.anticheat, launch: w.launch, env: w.env, steam_app_id: w.steam_app_id, sgdb_id: w.sgdb_id });
      closeModal(); renderAll(); toast(t("toast.saved"));
    });
  };
  draw();
}

async function updatesModal() {
  modal(`<h2>${esc(t("x.updates"))}</h2><p class="desc" id="upd">${esc(t("log.loading"))}</p>${foot(t("settings.close"))}`); $("#ok").onclick = closeModal;
  const u = await run(() => invoke("check_update")); if (!u) return; app.upd = u; renderTop();
  const el = $("#upd"); el && (el.innerHTML = u.newer ? `${esc(u.name || u.tag)} <b>(${esc(u.tag)})</b><br><button class="btn sm" data-open="${esc(u.url)}">${esc(t("x.updates"))} →</button>` : `Quark ${esc(u.current)} ✓`);
}
