"use strict";
// Sauvegardes locales Ludusavi et synchronisation cloud Epic/GOG.

function saveFilesTable(preview) {
  return `<div class="savelist">${preview.files.map((f) => `<div class="saverow ${f.failed ? "bad" : ""}"><span class="path" title="${esc(f.path)}">${esc(f.path)}</span><em>${esc(f.change || "")}</em><small>${f.bytes != null ? fmtBytes(f.bytes) : ""}</small></div>`).join("") || `<div class="dim">${esc(t("saves.none"))}</div>`}</div>
    ${preview.errors.length ? `<div class="sline err">${preview.errors.map(esc).join("<br>")}</div>` : ""}`;
}

function fmtBytes(bytes) {
  if (bytes < 1024) return bytes + " o";
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + " Ko";
  return (bytes / 1024 / 1024).toFixed(1) + " Mo";
}

function renderSavesPanel(g) {
  return `<div class="sec0" id="saves"><h3>${esc(t("saves.title"))}</h3>
    <p class="hint">${esc(t("saves.hint"))}</p>
    <div class="foot l"><button class="btn sec sm" id="sv-preview">${esc(t("saves.preview"))}</button><button class="btn sm" id="sv-backup">${esc(t("saves.backup"))}</button><button class="btn sec sm" id="sv-list">${esc(t("saves.backups"))}</button><button class="btn sec sm" id="sv-cloud">${ic("refresh")}${esc(t("saves.cloud"))}</button></div>
    <div class="row2"><input type="text" id="sv-folder" placeholder="${esc(t("saves.folder"))}"><button class="btn sec sm" id="sv-add">${ic("plus")}${esc(t("saves.addFolder"))}</button></div>
    <div id="sv-out"></div></div>`;
}

function bindSavesPanel(g) {
  const out = $("#sv-out");
  $("#sv-preview").onclick = () => run(async () => { out.innerHTML = `<div class="dim">${esc(t("log.loading"))}</div>`; const p = await invoke("save_preview", { id: g.id }); out.innerHTML = `<div class="kv"><span>${esc(t("saves.changed"))}</span><b>${p.changedGames ?? "—"} · ${p.processedBytes != null ? fmtBytes(p.processedBytes) : "—"}</b></div>${saveFilesTable(p)}`; });
  $("#sv-backup").onclick = () => run(async () => { $("#sv-backup").disabled = true; try { await invoke("save_backup", { id: g.id, phase: "before" }); toast(t("saves.backupDone")); } finally { $("#sv-backup").disabled = false; } });
  $("#sv-list").onclick = () => run(async () => { const list = await invoke("save_backups", { id: g.id }); modal(`<h2>${esc(t("saves.backups"))}</h2>${list.length ? `<div class="viewlist">${list.map((b) => `<div class="viewrow"><div><b>${esc(b.when)}</b><small>${b.locked ? "🔒" : ""}</small></div><button class="btn sec sm" data-restore="${esc(b.id)}">${esc(t("saves.restore"))}</button></div>`).join("")}</div>` : `<p class="dim">${esc(t("saves.none"))}</p>`}<div class="foot"><button class="btn sec" data-act="close">${esc(t("x.close"))}</button></div>`, true);
    $$("[data-restore]").forEach((b) => (b.onclick = () => restoreBackup(g, b.dataset.restore))); });
  $("#sv-cloud").onclick = () => run(async () => { const r = await invoke("cloud_sync", { id: g.id }); toast(r.message || t("saves.cloudDone")); });
  $("#sv-add").onclick = () => run(async () => { const path = await invoke("pick_folder"); if (!path) return; await invoke("save_add_folder", { id: g.id, path }); toast(t("toast.saved")); });
}

async function restoreBackup(g, backupId) {
  const preview = await invoke("save_restore_preview", { id: g.id, backupId });
  modal(`<h2>${esc(t("saves.restore"))}</h2><p class="desc">${esc(t("saves.confirmRestore"))}</p>${saveFilesTable(preview)}${foot(t("saves.restore"), "red")}`, true);
  $("#ok").onclick = () => run(async () => { await invoke("save_restore", { id: g.id, backupId, expectedPaths: preview.files.map((f) => f.path) }); closeModal(); toast(t("saves.restoreDone")); });
}
