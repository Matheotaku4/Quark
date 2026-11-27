"use strict";
// Navigation, bibliothèque (liste groupée / grille), fiche de jeu et accueil.

const NAV = [["home", "x.home"], ["store", "x.store"], ["mygames", "x.mygames"], ["library", "nav.library"]];
const ST_ORDER = ["playing", "backlog", "finished", "dropped", "none"];
const SORTS = { name: (a, b) => a.name.localeCompare(b.name, locale()), recent: (a, b) => (b.lastPlayed || 0) - (a.lastPlayed || 0), added: (a, b) => b.addedAt - a.addedAt,
  playtime: (a, b) => b.playtimeSecs - a.playtimeSecs, favorites: (a, b) => (b.favorite - a.favorite) || (b.lastPlayed || 0) - (a.lastPlayed || 0) };

function srcLabel(g) {
  const raw = g.sourceLabel || "";
  if (raw === "Locale" || raw === "Local") return t("source.local");
  if (raw === "Plusieurs" || raw === "Multiple") return t("source.multiple");
  return raw;
}

function renderTop() {
  $("#top-tabs").innerHTML = NAV.map(([k, l]) => `<button data-view="${k}" class="${app.view === k ? "on" : ""}">${esc(t(l))}</button>`).join("");
  const grid = app.cfg.view === "grid", upd = app.upd && app.upd.newer;
  $("#top-right").innerHTML = `<button class="tb" data-act="toggle-view" title="${esc(t("x.view"))}">${ic(grid ? "list" : "grid")}</button>
    <button class="tb" data-act="add" title="${esc(t("x.add"))}">${ic("plus")}</button><button class="tb" data-act="palette" title="${esc(t("x.search"))}">${ic("search")}</button>
    <button class="tb" data-act="updates" title="${esc(t("x.updates"))}">${ic("down")}${upd ? '<i class="pip"></i>' : ""}</button>
    ${app.transfers.size ? `<button class="tb on" data-act="transfers" title="${esc(t("pv.transfers"))}">${ic("down")}<i class="pip"></i></button>` : ""}
    ${QuarkDeck.available() ? `<button class="tb${QuarkDeck.active() ? " on" : ""}" data-act="deck" title="${esc(t("deck.mode"))}">${ic("gamepad")}</button>` : ""}
    <button class="tb ${app.view === "settings" ? "on" : ""}" data-act="settings" title="${esc(t("settings.title"))}">${ic("gear")}</button><span class="vsep"></span>
    <button class="wb" data-win="win_minimize">${ic("min")}</button><button class="wb" data-win="win_toggle_maximize">${ic("max")}</button><button class="wb x" data-win="win_close">${ic("x")}</button>`;
  document.body.dataset.view = app.view; document.body.classList.toggle("grid", grid && app.view === "library");
}

// ---------- liste ----------
function filtered() {
  const q = norm(app.query.trim()), f = app.f;
  return app.data.games.filter((g) => (!f.status || g.status === f.status) && (!f.genre || g.genres.includes(f.genre)) && (!f.tag || g.tags.includes(f.tag)) && (!f.fav || g.favorite)
    && (!q || norm([g.name, g.developer, ...g.tags, ...g.genres].join(" ")).includes(q))).sort(SORTS[app.cfg.library_sort] || SORTS.name);
}
function groupOf(g) {
  switch (app.cfg.group_by) {
    case "status": return [g.status, t("status." + g.status)];
    case "store": return [srcLabel(g), srcLabel(g)];
    case "genre": return [g.genres[0] || "—", g.genres[0] || "—"];
    case "year": return [(g.released || "").slice(0, 4) || "—", (g.released || "").slice(0, 4) || "—"];
    default: return ["", ""];
  }
}
function row(g) {
  const live = app.running.has(g.id), art = g.iconUrl || g.coverUrl;
  const sub = live ? `<em data-live="${g.id}">${clock(elapsed(g.id))}</em>` : `${esc(rel(g.lastPlayed))} · ${hm(g.playtimeSecs)}`;
  return `<button class="row${g.id === app.sel ? " sel" : ""}${live ? " live" : ""}${app.marked.has(g.id) ? " marked" : ""}" data-id="${g.id}" style="${accent(g)}">
    <i class="ico">${art ? `<img src="${esc(art)}" alt="" loading="lazy">` : esc(g.monogram)}</i>
    <span class="meta"><b>${esc(g.name)}</b><small>${sub}</small></span>
    ${app.marked.has(g.id) ? `<i class="check">✓</i>` : `<i class="dot" style="background:${g.status === "none" ? "transparent" : g.statusColor}"></i>`}</button>`;
}
function renderList() {
  const list = filtered(), by = app.cfg.group_by;
  if (by === "none") $("#list").innerHTML = list.map(row).join("");
  else {
    const m = new Map();
    for (const g of list) { const [k, l] = groupOf(g); (m.get(k) || m.set(k, { l, items: [], c: g.statusColor }).get(k)).items.push(g); }
    let keys = [...m.keys()];
    keys = by === "status" ? keys.sort((a, b) => ST_ORDER.indexOf(a) - ST_ORDER.indexOf(b)) : by === "year" ? keys.sort().reverse() : keys.sort((a, b) => a.localeCompare(b, locale()));
    $("#list").innerHTML = keys.map((k) => { const x = m.get(k), cl = app.collapsed.has(k);
      return `<h4 data-group="${esc(k)}" class="${cl ? "cl" : ""}">${ic("caret")}${by === "status" && k !== "none" ? `<i class="dot" style="background:${x.c}"></i>` : ""}<span>${esc(x.l)}</span><em>${x.items.length}</em></h4>${cl ? "" : x.items.map(row).join("")}`; }).join("");
  }
  if (!list.length) $("#list").innerHTML = `<div class="none">${esc(app.data.games.length ? t("list.empty") : t("list.emptyLibrary"))}</div>`;
  $("#count").textContent = t(list.length === 1 ? "library.countOne" : "library.countMany", { n: list.length });
  $("#lib-title").textContent = t("nav.library");
  const act = $(".row.sel", $("#list")); act && act.scrollIntoView({ block: "nearest" });
  $("#fl-btn").classList.toggle("on", !!(app.f.status || app.f.genre || app.f.tag || app.f.fav));
  renderBatch();
}
function renderSide() {
  $("#q").placeholder = t("search.library");
  const genres = [...new Set(app.data.games.flatMap((g) => g.genres))].sort(), tags = [...new Set(app.data.games.flatMap((g) => g.tags))].sort();
  $("#fl-pop").innerHTML = `<label>${esc(t("status.label"))}</label>${sel("f-status", [["", t("x.allStatuses")], ...ST_ORDER.map((s) => [s, t("status." + s)])], app.f.status)}
    <label>${esc(t("overview.genres"))}</label>${sel("f-genre", [["", t("x.allGenres")], ...genres.map((x) => [x, x])], app.f.genre)}
    <label>${esc(t("overview.tags"))}</label>${sel("f-tag", [["", t("x.allTags")], ...tags.map((x) => [x, x])], app.f.tag)}
    ${sw("f-fav", t("x.favOnly"), "", app.f.fav)}<button class="btn sec sm" data-act="reset-filters">${esc(t("x.reset"))}</button>`;
  $("#fl-label").textContent = t("x.filters");
  const foot = $(".side-foot");
  if (foot && !$("#views-btn")) foot.insertAdjacentHTML("beforeend", `<button class="tb" id="views-btn" data-act="views" title="${esc(t("views.title"))}">${ic("list")}</button>`);
  $("#sort").innerHTML = ["recent", "name", "playtime", "added", "favorites"].map((k) => `<option value="${k}" ${app.cfg.library_sort === k ? "selected" : ""}>${esc(k === "favorites" ? t("detail.favorite") : t("sort." + k))}</option>`).join("");
  $("#group").innerHTML = ["status", "store", "genre", "year", "none"].map((k) => `<option value="${k}" ${app.cfg.group_by === k ? "selected" : ""}>${esc(t("x.groupBy"))} · ${esc(({ status: t("status.label"), store: t("overview.source"), genre: t("overview.genres"), year: t("overview.released"), none: "—" })[k])}</option>`).join("");
}

// ---------- grille ----------
function renderGrid() {
  const list = filtered();
  return `<div class="gridwrap"><div class="grid" style="--tile:${Math.round(app.cfg.grid_tile_width || 170)}px">${list.map((g) => `<button class="tile${app.running.has(g.id) ? " live" : ""}${app.marked.has(g.id) ? " marked" : ""}" data-id="${g.id}" style="${accent(g)}">
    <div class="cover" style="${g.coverUrl ? `background-image:url('${esc(g.coverUrl)}')` : ""}">${g.coverUrl ? "" : `<b>${esc(g.monogram)}</b>`}${g.favorite ? `<span class="fav">${ic("star", "on")}</span>` : ""}${app.marked.has(g.id) ? `<span class="check">✓</span>` : ""}</div>
    <span class="meta"><b>${esc(g.name)}</b><small>${hm(g.playtimeSecs)}</small></span></button>`).join("")}</div>${list.length ? "" : `<div class="none">${esc(t("list.empty"))}</div>`}</div>`;
}

// ---------- fiche de jeu ----------
const iconBtn = (act, icn, color, title, on) => `<button class="ib${on ? " on" : ""}" data-act="${act}" style="--c:${color}" title="${esc(title)}">${ic(icn, on ? "on" : "")}</button>`;
function renderGame(g) {
  const live = app.running.has(g.id), r = app.running.get(g.id), art = g.heroUrl || g.coverUrl;
  const sub = [g.developer, g.released, g.genres.slice(0, 3).join(", "), srcLabel(g), g.isNative ? "Linux" : ""].filter(Boolean);
  const tabs = [["overview", "x.overview"], ["media", "x.media"], ["ach", "x.ach"], ["news", "x.news"], ["hist", "x.hist"], ["notes", "tab.notes"], ["launch", "tab.launch"], ["log", "tab.log"]];
  const placement = app.cfg.game_actions === "beside_play";
  const actions = `<div class="acts">${iconBtn("fav", "star", "#ffc94d", t(g.favorite ? "detail.unfavorite" : "detail.favorite"), g.favorite)}${iconBtn("sites", "ext", "var(--accent)", t("web.sites"))}${iconBtn("gsettings", "gear", "var(--accent)", t("x.settingsGame"))}${iconBtn("customize", "edit", "#ffc94d", t("x.customize"))}${iconBtn("log", "log", "#4cd07d", t("x.logBtn"))}${iconBtn("remove", "trash", "var(--danger)", t("x.remove"))}</div>`;
  return `<div class="gp" style="${accent(g)}">
    <div class="gh">
      <div class="gh-art" style="${art ? `background-image:url('${esc(art)}')` : ""}"></div>${art ? "" : `<span class="gh-mono">${esc(g.monogram)}</span>`}<div class="gh-fade"></div>
      <div class="gh-top">${document.body.classList.contains("grid") ? `<button class="btn sec sm back" data-act="back">${ic("back")}${esc(t("x.back"))}</button>` : ""}<span id="pdb"></span><span class="sp"></span>${placement ? "" : actions}</div>
      <div class="gh-main">
        ${g.logoUrl ? `<img class="logo" src="${esc(g.logoUrl)}" alt="${esc(g.name)}">` : `<h1>${esc(g.name)}</h1>`}
        <div class="sub">${sub.map((s) => `<span>${esc(s)}</span>`).join("")}</div>
        <div class="act">
          <button class="play${live ? " run" : ""}" data-act="play">${ic(live ? "stop" : "play")}${esc(t(live ? (r.stopping ? "detail.stopping" : "detail.stop") : "detail.play"))}</button>
          <details class="menu st"><summary class="chip" style="--sc:${g.status === "none" ? "var(--text-faint)" : g.statusColor}"><i></i>${esc(t("status." + g.status))}</summary><div class="pop">${ST_ORDER.map((s) => `<button data-status="${s}">${esc(t("status." + s))}</button>`).join("")}</div></details>
          ${live ? `<span class="clock" data-live="${g.id}">${clock(elapsed(g.id))}</span>` : ""}${placement ? actions : ""}
        </div>
        ${g.exeMissing ? `<span class="warn">⚠ ${esc(t("detail.exeMissing"))}</span>` : ""}
      </div>
    </div>
    <div class="gb"><div class="gcol"><div class="tabs">${tabs.map(([k, l]) => `<button data-tab="${k}" class="${app.tab === k ? "on" : ""}">${esc(t(l))}</button>`).join("")}</div><div id="tabc"></div></div>
      <aside class="ov" id="ov"></aside></div></div>`;
}
function weeks(g) {
  const w = new Array(12).fill(0), now = Date.now() / 1000;
  for (const s of g.sessions) { const i = 11 - Math.floor((now - s.start) / 604800); if (i >= 0 && i < 12) w[i] += s.secs; }
  return w;
}
function renderOverview(g) {
  const live = app.running.has(g.id), row2 = (l, v) => (v ? `<div class="kv"><span>${esc(l)}</span><b>${v}</b></div>` : "");
  $("#ov").innerHTML = `<h3>${esc(t("x.overview"))}</h3>${row2(t("overview.playtime"), hm(g.playtimeSecs + (live ? elapsed(g.id) : 0)))}${row2(t("overview.lastPlayed"), esc(live ? t("detail.playingNow") : rel(g.lastPlayed)))}
    ${row2(t("x.sessions"), g.sessions.length)}${row2(t("overview.genres"), esc(g.genres.join(" · ")))}${row2(t("overview.developer"), esc(g.developer))}${row2(t("overview.released"), esc(g.released))}${row2(t("overview.source"), esc(srcLabel(g)))}
    ${g.installations.length > 1 ? `<div class="hr"></div><h3>${esc(t("installs.title"))}</h3>${g.installations.map((i) => `<button class="instrow${i.active ? " on" : ""}" data-installation="${esc(i.id)}"><b>${esc(i.label)}</b><small>${esc(i.path || i.exe)}</small></button>`).join("")}` : ""}
    <div class="hr"></div><div id="hltb"><h3 class="blue">HowLongToBeat</h3><div class="dim">…</div></div>`;
}
function stopMedia() {
  exitCinema();
  if (app.hls) { app.hls.destroy(); app.hls = null; }
}

function exitCinema() {
  const state = app.cinema;
  if (!state) return;
  app.cinema = null;
  document.body.classList.remove("cinema");
  state.video.classList.remove("cinema-video");
  if (state.holder.isConnected) state.holder.replaceWith(state.video);
  else state.video.remove();
  if (!state.deck) invoke("win_set_fullscreen", { active: false }).catch(() => {});
}

function toggleCinema(video) {
  if (app.cinema) { exitCinema(); return; }
  const holder = document.createElement("div");
  video.before(holder);
  document.body.append(video);
  video.classList.add("cinema-video");
  document.body.classList.add("cinema");
  app.cinema = { video, holder, deck: document.body.classList.contains("deck") };
  if (!app.cinema.deck) invoke("win_set_fullscreen", { active: true }).catch(() => {});
  video.focus?.();
}

document.addEventListener("keydown", (event) => {
  if (event.key === "Escape" && app.cinema) { event.preventDefault(); exitCinema(); }
});

function animateTab() {
  const box = $("#tabc"); if (!box) return;
  box.classList.remove("tab-anim"); void box.offsetWidth; box.classList.add("tab-anim");
}

function watchTab() {
  const box = $("#tabc");
  if (!box || box.dataset.watched) return;
  box.dataset.watched = "1";
  new MutationObserver(animateTab).observe(box, { childList: true });
}

function renderTab(g) {
  stopMedia();
  const box = $("#tabc"); if (!box) return;
  watchTab();
  if (app.tab === "news") return loadNews(g);
  if (app.tab === "media") return loadMedia(g);
  if (app.tab === "ach") return loadAchievements(g);
  if (app.tab === "notes") return renderNotesTab(g);
  if (app.tab === "launch") return renderLaunchTab(g);
  if (app.tab === "log") return loadLogTab(g);
  if (app.tab === "overview") return renderOverviewTab(g);
  const w = weeks(g), max = Math.max(1, ...w), list = g.sessions.slice().reverse().slice(0, 40);
  box.innerHTML = `<div class="sec0"><h3>${esc(t("x.weeks"))}<span>${esc(hm(w[11]))}</span></h3><div class="bars">${w.map((v, i) => `<i class="${i === 11 ? "now " : ""}${v ? "v" : ""}" style="height:${Math.max(6, (v / max) * 100)}%" title="${hm(v)}"></i>`).join("")}</div></div>
    <div class="sec0"><h3>${esc(t("activity.recent"))}</h3>${list.length ? `<div class="sess">${list.map((s) => `<div><span>${date(s.start)}</span><span>${hm(s.secs)}</span></div>`).join("")}</div>` : `<div class="dim">${esc(t("activity.empty"))}</div>`}</div>`;
}

function renderOverviewTab(g) {
  const tags = g.tags.length ? `<div class="taglist">${g.tags.map((tag) => `<span>${esc(tag)}</span>`).join("")}</div>` : "";
  const about = g.description ? `<p class="abouttext">${esc(g.description)}</p>` : `<p class="dim">${esc(t("x.noDescription"))}</p>`;
  $("#tabc").innerHTML = `<div class="sec0"><h3>${esc(t("overview.about"))}</h3>${about}${tags}</div>
    <div class="sec0"><h3>${esc(t("overview.details"))}</h3><div class="kv"><span>${esc(t("overview.developer"))}</span><b>${esc(g.developer || "—")}</b></div>
    <div class="kv"><span>${esc(t("overview.genres"))}</span><b>${esc(g.genres.join(" · ") || "—")}</b></div><div class="kv"><span>${esc(t("overview.released"))}</span><b>${esc(g.released || "—")}</b></div>
    <div class="kv"><span>${esc(t("overview.added"))}</span><b>${esc(date(g.addedAt))}</b></div><div class="kv"><span>${esc(t("overview.source"))}</span><b>${esc(srcLabel(g))}</b></div></div>`;
}

function renderNotesTab(g) {
  $("#tabc").innerHTML = `<div class="sec0"><h3>${esc(t("tab.notes"))}</h3><textarea id="game-notes" class="notesbox" placeholder="${esc(t("notes.placeholder"))}">${esc(g.notes)}</textarea>
    <div class="foot l"><button class="btn sm" id="notes-save">${esc(t("x.save"))}</button></div></div>`;
  $("#notes-save").onclick = () => run(async () => { await invoke("set_notes", { id: g.id, notes: $("#game-notes").value }); g.notes = $("#game-notes").value; toast(t("toast.saved")); });
}

function renderLaunchTab(g) {
  const opts = [g.launch.gamemode && "GameMode", g.launch.mangohud && "MangoHud", g.launch.gamescope && `gamescope ${g.launch.gamescope_width}×${g.launch.gamescope_height}`, g.launch.prime && "PRIME", g.launch.gamescope_fsr && "FSR", g.umu && "umu"].filter(Boolean);
  $("#tabc").innerHTML = `<div class="sec0"><h3>${esc(t("tab.launch"))}</h3><div class="kv"><span>Runtime</span><b>${esc(g.proton)}</b></div><div class="kv"><span>${esc(t("launch.prefix"))}</span><b class="path">${esc(g.prefix)}</b></div>
    <div class="kv"><span>${esc(t("add.args"))}</span><b>${esc(g.args || "—")}</b></div><div class="kv"><span>${esc(t("launch.options"))}</span><b>${esc(opts.join(" · ") || "—")}</b></div>
    <div class="kv"><span>${esc(t("launch.anticheat"))}</span><b>${esc(g.anticheat || "—")}</b></div>
    <div class="kv"><span>${esc(t("launch.env"))}</span><b>${esc(g.env.length ? g.env.map((e) => e.name).join(", ") : "—")}</b></div>
    <div class="foot l"><button class="btn sec sm" data-act="gsettings">${ic("gear")}${esc(t("x.settingsGame"))}</button><button class="btn sec sm" data-act="open-folder">${ic("folder")}${esc(t("detail.openFolder"))}</button></div></div>
    ${renderSavesPanel(g)}`;
  bindSavesPanel(g);
}

async function loadLogTab(g) {
  const box = $("#tabc"); if (!box) return;
  box.innerHTML = `<div class="sec0"><div class="newshead"><h3>${esc(t("tab.log"))}</h3><button class="btn sec sm" id="log-refresh">${ic("refresh")}${esc(t("x.refresh"))}</button></div><pre class="logbox" id="log-out">${esc(t("log.loading"))}</pre></div>`;
  const load = async () => {
    const o = $("#log-out"); if (!o) return;
    try { const txt = await invoke("read_log", { id: g.id }); o.textContent = txt.trim() ? txt.split("\n").slice(-300).join("\n") : t("log.empty"); }
    catch { o.textContent = t("log.empty"); }
    o.scrollTop = o.scrollHeight;
  };
  $("#log-refresh").onclick = load; load();
}

async function loadMedia(g) {
  const box = $("#tabc"); if (!box) return;
  const cache = (app.cache.media ||= {});
  if (!cache[g.id]) {
    box.innerHTML = `<div class="none">${esc(t("log.loading"))}</div>`;
    cache[g.id] = await invoke("game_media", { id: g.id }).catch((e) => ({ err: String(e) }));
  }
  if (app.sel !== g.id || app.tab !== "media" || !$("#tabc")) return;
  const m = cache[g.id];
  if (m.err) { box.innerHTML = `<div class="none">${esc(bt(m.err))}</div>`; return; }
  const trailer = m.trailers[0];
  box.innerHTML = `${trailer ? `<div class="trailerwrap"><button class="trailerplay" data-trailer="${esc(trailer.video)}" data-poster="${esc(trailer.thumbnail)}"><img src="${esc(trailer.thumbnail)}" alt=""><span class="tplay">${ic("play")}</span></button><div class="trailerbar"><b>${esc(trailer.title || t("x.trailer"))}</b><div class="trailerbar-actions"><label class="trailervol" title="${esc(t("Volume"))}">${ic("volume")}<input type="range" min="0" max="1" step="0.01" value="${mediaSlider()}" aria-label="${esc(t("Volume"))}"></label><button class="btn sec sm" data-open="${esc(trailer.page)}">${ic("ext")}Steam</button></div></div><p class="trailerfail" hidden>${esc(t("x.trailerUnavailable"))}</p></div>` : `<div class="none">${esc(t("x.noMedia"))}</div>`}
    ${m.screenshots.length ? `<div class="shots">${m.screenshots.map((p) => `<button data-open="${esc(p.full)}"><img loading="lazy" decoding="async" src="${esc(p.thumbnail)}" alt=""></button>`).join("")}</div>` : ""}`;
}

const MEDIA_VOLUME_DEFAULT = 0.8;
function mediaSlider() {
  const value = Number(app.cfg.media_volume);
  return Number.isFinite(value) ? Math.min(1, Math.max(0, value)) : MEDIA_VOLUME_DEFAULT;
}
// Courbe logarithmique (20 dB) : la position du curseur est une sensation
// d'écoute, pas une amplitude linéaire.
const mediaVolume = (position) => (position <= 0 ? 0 : Math.pow(10, (position - 1) * 2));
const mediaSliderFromVolume = (volume) => (volume <= 0 ? 0 : Math.min(1, Math.max(0, 1 + Math.log10(volume) / 2)));
let mediaVolumeTimer;
function persistMediaVolume(position) {
  app.cfg.media_volume = position;
  clearTimeout(mediaVolumeTimer);
  mediaVolumeTimer = setTimeout(() => cfgSet({ media_volume: position }).catch(() => {}), 250);
}

function loadHls() {
  if (window.Hls) return Promise.resolve(true);
  if (!app.hlsLoad) app.hlsLoad = new Promise((resolve) => {
    const s = document.createElement("script");
    s.src = "hls.min.js"; s.onload = () => { if (!window.Hls) app.hlsLoad = null; resolve(!!window.Hls); }; s.onerror = () => { app.hlsLoad = null; resolve(false); };
    document.head.append(s);
  });
  return app.hlsLoad;
}

async function playTrailer(btn) {
  const box = btn.closest(".trailerwrap"); if (!box) return;
  const src = btn.dataset.trailer;
  if (!src) return;
  const hlsSrc = /\.m3u8(?:[?#]|$)/i.test(src);
  if (hlsSrc && !(await loadHls())) { box.querySelector(".trailerfail")?.removeAttribute("hidden"); return; }
  if (!btn.isConnected) return;
  const video = document.createElement("video");
  let hls = null, failed = false;
  const fail = () => {
    if (failed) return;
    failed = true;
    hls?.destroy(); app.hls = null;
    if (app.cinema?.video === video) exitCinema();
    video.replaceWith(btn);
    box.querySelector(".trailerfs")?.remove();
    const warn = $(".trailerfail", box);
    if (warn) warn.hidden = false;
  };
  video.controls = video.autoplay = video.playsInline = true;
  video.poster = btn.dataset.poster || "";
  const slider = $(".trailervol input", box);
  let syncingVolume = false;
  const applyVolume = (position) => {
    syncingVolume = true;
    video.volume = mediaVolume(position);
    if (slider) slider.value = String(position);
    syncingVolume = false;
  };
  applyVolume(mediaSlider());
  if (slider) slider.oninput = () => {
    applyVolume(+slider.value);
    persistMediaVolume(+slider.value);
  };
  video.addEventListener("volumechange", () => {
    if (syncingVolume) return;
    const position = mediaSliderFromVolume(video.volume);
    if (slider) slider.value = String(position);
    persistMediaVolume(position);
  });
  video.addEventListener("error", fail, { once: true });
  if (hlsSrc && window.Hls && Hls.isSupported()) {
    hls = app.hls = new Hls({ enableWorker: false, lowLatencyMode: false, backBufferLength: 30 });
    hls.on(Hls.Events.ERROR, (_evt, data) => { if (data.fatal) fail(); });
    hls.on(Hls.Events.MANIFEST_PARSED, () => video.play().catch(() => {}));
    hls.loadSource(src);
    hls.attachMedia(video);
  } else {
    video.src = src;
  }
  btn.replaceWith(video);
  video.addEventListener("dblclick", () => toggleCinema(video));
  const bar = box.querySelector(".trailerbar");
  if (bar) {
    const full = document.createElement("button");
    full.className = "btn sec sm trailerfs";
    full.innerHTML = `${ic("max")}${esc(t("x.fullscreen"))}`;
    full.onclick = () => toggleCinema(video);
    bar.append(full);
  }
  video.play().catch(() => {});
}

async function loadAchievements(g) {
  const box = $("#tabc"); if (!box) return;
  const cache = (app.cache.ach ||= {});
  if (!cache[g.id]) {
    box.innerHTML = `<div class="none">${esc(t("log.loading"))}</div>`;
    cache[g.id] = await invoke("game_achievements", { id: g.id }).catch((e) => ({ err: String(e) }));
  }
  if (app.sel !== g.id || app.tab !== "ach" || !$("#tabc")) return;
  const a = cache[g.id];
  if (a.err) { box.innerHTML = `<div class="none">${esc(bt(a.err))}</div>`; return; }
  const pct = a.total ? Math.round((a.unlocked / a.total) * 100) : 0;
  box.innerHTML = `<div class="achhead"><h3>${esc(t("x.ach"))}<span>${a.unlocked} / ${a.total}</span></h3><div class="pbar"><i style="width:${pct}%"></i></div><small>${pct} % · ${esc(a.provider)}</small><button class="btn sec sm" data-act="compare-ach">${ic("grid")}${esc(t("ach.compare"))}</button></div>
    <div class="achlist">${a.rows.map((r) => `<div class="ach ${r.unlocked ? "on" : ""}"><div class="achico">${r.iconUrl ? `<img src="${esc(r.iconUrl)}" alt="">` : ic(r.unlocked ? "star" : "x")}</div><div><b>${esc(r.name)}</b><small>${esc(r.description)}</small></div><div class="achmeta">${r.rarity != null ? `<em>${r.rarity.toFixed(1)} %</em>` : ""}<span>${r.unlocked ? "✓" : Math.round((r.progress || 0) * 100) + " %"}</span></div></div>`).join("")}</div>`;
}
async function achievementsCompareModal() {
  await providersLoad().catch(() => {});
  const ids = [...new Set((app.pv.data?.games || []).filter((g) => g.quarkGameId).map((g) => g.quarkGameId))].slice(0, 8);
  if (!ids.length) return toast(t("ach.compareEmpty"), true);
  modal(`<h2>${esc(t("ach.compareTitle"))}</h2><div class="dim">${esc(t("log.loading"))}</div>`, true);
  const rows = await invoke("achievements_compare", { ids }).catch((e) => { toast(String(e), true); return []; });
  const body = rows.length ? rows.map((row) => `<div class="achcmp ${row.error ? "err" : ""}"><div class="t-top"><b>${esc(row.name)}</b><small>${esc(row.provider || "—")}</small></div>${row.error ? `<div class="dim">${esc(row.error)}</div>` : `<div class="t-bar"><i style="width:${row.percent}%"></i></div><div class="t-meta"><span>${row.unlocked} / ${row.total}</span><b>${row.percent} %</b></div>`}</div>`).join("") : `<div class="dim">${esc(t("ach.none"))}</div>`;
  modal(`<h2>${esc(t("ach.compareTitle"))}</h2><div class="transfers">${body}</div><div class="foot"><button class="btn sec" data-act="close">${esc(t("x.close"))}</button></div>`, true);
}

async function loadNews(g, force) {
  const box = $("#tabc"), c = (app.cache.news ||= {});
  if (!g.steamAppId) { box.innerHTML = `<div class="none">${esc(t("x.noAppId"))}</div>`; return; }
  if (!c[g.id] || force) { box.innerHTML = `<div class="none">${esc(t("log.loading"))}</div>`; c[g.id] = await invoke("fetch_news", { appId: g.steamAppId, count: 8 }).catch((e) => ({ err: String(e) })); }
  if (app.sel !== g.id || app.tab !== "news" || !$("#tabc")) return;
  const d = c[g.id];
  $("#tabc").innerHTML = `<div class="newshead"><h3>${esc(t("x.news"))}</h3><button class="btn sec sm" data-act="news-refresh">${ic("refresh")}${esc(t("x.refresh"))}</button></div>` +
    (d.err ? `<div class="none">${esc(bt(d.err))}</div>` : d.length ? d.map((n, i) => `<details class="news"><summary><small>${esc(n.feed)}${n.author ? " · " + esc(n.author) : ""} · ${esc(rel(n.date))}</small><b>${esc(n.title)}</b><p>${esc(n.excerpt)}</p></summary>
      <div class="nbody">${esc(n.body)}</div><button class="btn sec sm" data-open="${esc(n.url)}">${ic("ext")}${esc(n.url ? "Steam" : "")}</button></details>`).join("") : `<div class="none">${esc(t("x.noNews"))}</div>`);
}
async function loadExtras(g) {
  const c = (app.cache.hltb ||= {}), p = (app.cache.pdb ||= {});
  if (!(g.id in c)) c[g.id] = await invoke("fetch_hltb", { title: g.name }).catch(() => null);
  if (g.steamAppId && !(g.id in p)) p[g.id] = await invoke("fetch_protondb", { appId: g.steamAppId }).catch(() => null);
  if (app.sel !== g.id) return;
  const h = c[g.id], box = $("#hltb"), d = p[g.id], b = $("#pdb");
  if (box) box.innerHTML = `<h3 class="blue">HowLongToBeat</h3>${h ? `<div class="kv"><span>${esc(t("x.main"))}</span><b>${hrs(h.main)}</b></div><div class="kv"><span>${esc(t("x.extra"))}</span><b>${hrs(h.mainExtra)}</b></div><div class="kv"><span>${esc(t("x.compl"))}</span><b>${hrs(h.completionist)}</b></div><button class="lnk" data-open="${esc(h.url)}">${esc(t("x.hltbLink"))} →</button>` : `<div class="dim">—</div>`}`;
  if (b && d) b.innerHTML = `<span class="badge" style="--bc:${cssColor(d.color)}" title="${esc(d.label)}">${esc(d.tier)} · ProtonDB</span>`;
}

// ---------- accueil ----------
function renderHome() {
  const games = app.data.games.slice().sort(SORTS[app.cfg.home_sort] || SORTS.recent), list = games.filter((g) => g.heroUrl || g.coverUrl).slice(0, 6), n = list.length;
  app.hi = n ? (app.hi || 0) % n : 0; const g = list[app.hi] || games[0];
  if (!g) return `<div class="empty"><h2>${esc(t("empty.title"))}</h2><p>${esc(t("empty.subtitle"))}</p><button class="btn" data-act="add">${esc(t("empty.add"))}</button></div>`;
  const rail = (title, items) => (items.length ? `<section class="rail"><h3>${esc(title)}</h3><div class="cards">${items.slice(0, 12).map((x) => `<button class="tile sm" data-id="${x.id}" data-go="library" style="${accent(x)}"><div class="cover" style="${x.coverUrl ? `background-image:url('${esc(x.coverUrl)}')` : ""}">${x.coverUrl ? "" : `<b>${esc(x.monogram)}</b>`}</div><span class="meta"><b>${esc(x.name)}</b><small>${hm(x.playtimeSecs)}</small></span></button>`).join("")}</div></section>` : "");
  const by = (f) => games.filter(f);
  return `<div class="home"><div class="gh big" style="${accent(g)}"><div class="gh-art" style="${g.heroUrl || g.coverUrl ? `background-image:url('${esc(g.heroUrl || g.coverUrl)}')` : ""}"></div>${g.heroUrl || g.coverUrl ? "" : `<span class="gh-mono">${esc(g.monogram)}</span>`}<div class="gh-fade"></div>
    <div class="gh-main">${g.logoUrl ? `<img class="logo" src="${esc(g.logoUrl)}" alt="">` : `<h1>${esc(g.name)}</h1>`}<div class="sub"><span>${esc(rel(g.lastPlayed))}</span><span>${hm(g.playtimeSecs)}</span></div>
      <div class="act"><button class="play" data-id="${g.id}" data-go="library" data-launch="1">${ic("play")}${esc(t("detail.play"))}</button><button class="btn sec" data-id="${g.id}" data-go="library">${esc(t("x.overview"))}</button></div></div>
    ${n > 1 ? `<div class="dots">${list.map((_, i) => `<button data-hi="${i}" class="${i === app.hi ? "on" : ""}"></button>`).join("")}</div>` : ""}</div>
    ${rail(t("x.continue"), by((x) => x.lastPlayed).sort(SORTS.recent))}${rail(t("x.favorites"), by((x) => x.favorite))}${rail(t("x.backlog"), by((x) => x.status === "backlog"))}</div>`;
}
const renderSoon = (name) => `<div class="empty"><h2>${esc(name)}</h2><p>${esc(t("x.soonBody"))}</p></div>`;

// ---------- orchestration ----------
function renderSheet() {
  stopMedia();
  stopMyGames();
  const el = $("#sheet"), v = app.view;
  if (v === "settings") { el.innerHTML = renderSettings(); bindSettings(); return; }
  if (v === "home") { el.innerHTML = renderHome(); return; }
  if (v === "store") { el.innerHTML = renderStore(); bindStore(); storeEnter(); return; }
  if (v === "mygames") { el.innerHTML = renderMyGames(); bindMyGames(); providersLoad(); return; }
  if (document.body.classList.contains("grid") && !app.open) { el.innerHTML = renderGrid(); return; }
  const g = game(app.sel);
  if (!g) { el.innerHTML = `<div class="empty"><h2>${esc(t("empty.title"))}</h2><p>${esc(t("empty.subtitle"))}</p><button class="btn" data-act="add">${esc(t("empty.add"))}</button></div>`; return; }
  el.innerHTML = renderGame(g); renderOverview(g); renderTab(g); loadExtras(g);
}
function renderAll() { renderTop(); renderSide(); renderList(); renderSheet(); QuarkDeck.refocus(); }
function updateSelection() {
  $$("#list .row").forEach((row) => row.classList.toggle("sel", row.dataset.id === app.sel));
  const act = $(".row.sel", $("#list")); act && act.scrollIntoView({ block: "nearest" });
}
function select(id) { app.sel = id; app.open = true; app.tab = app.cfg.detail_tab || "overview"; try { localStorage.setItem("quark.sel", id); } catch {} updateSelection(); renderSheet(); $("#sheet").scrollTop = 0; QuarkDeck.refocus(); }
function setView(v) { app.view = v; renderAll(); $("#sheet").scrollTop = 0; if (v === "mygames") providersLoad(); }
