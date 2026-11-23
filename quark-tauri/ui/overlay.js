"use strict";
const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

const esc = (s) => String(s ?? "").replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[c]);
const hm = (s) => { s = Math.max(0, Math.round(s)); const h = Math.floor(s / 3600), m = Math.floor((s % 3600) / 60); return h ? `${h} h ${String(m).padStart(2, "0")}` : `${m} min`; };

const TXT = {
  fr: ["Aucun jeu en cours.", "Lance un jeu depuis Quark pour voir la session ici.", "EN COURS", "Chargement…", "Chargement des succès…", "Succès indisponibles pour ce jeu.", "succès", "pour masquer", "autre(s) session(s)", "Fermer"],
  en: ["No game running.", "Launch a game from Quark to see the session here.", "RUNNING", "Loading…", "Loading achievements…", "Achievements unavailable for this game.", "achievements", "to hide", "other session(s)", "Close"],
  es: ["Ningún juego en ejecución.", "Inicia un juego desde Quark para ver la sesión aquí.", "EN CURSO", "Cargando…", "Cargando logros…", "Logros no disponibles para este juego.", "logros", "para ocultar", "otra(s) sesión(es)", "Cerrar"],
  de: ["Kein Spiel läuft.", "Starte ein Spiel aus Quark, um die Sitzung hier zu sehen.", "LÄUFT", "Lädt…", "Erfolge werden geladen…", "Erfolge für dieses Spiel nicht verfügbar.", "Erfolge", "zum Ausblenden", "weitere Sitzung(en)", "Schließen"],
  pt: ["Nenhum jogo em execução.", "Inicie um jogo no Quark para ver a sessão aqui.", "EM CURSO", "Carregando…", "Carregando conquistas…", "Conquistas indisponíveis para este jogo.", "conquistas", "para ocultar", "outra(s) sessão(ões)", "Fechar"],
};
let lang = "fr", hotkey = "Shift+Backspace";
let opts = { locked: true, compact: false, progress: true };
const txt = () => TXT[lang] || TXT.fr;
const LANGS = { french: "fr", english: "en", spanish: "es", german: "de", portuguese: "pt" };
invoke("get_settings_raw").then((s) => {
  lang = LANGS[s.ui_language] || s.ui_language || "fr";
  hotkey = s.overlay_hotkey || hotkey;
  opts = { locked: s.overlay_show_locked !== false, compact: !!s.overlay_compact, progress: s.overlay_show_progress !== false };
  document.getElementById("panel").classList.toggle("compact", opts.compact);
  document.getElementById("close").title = txt()[9];
  refresh();
}).catch(() => {});

async function refresh() {
  const body = document.getElementById("body");
  const foot = document.getElementById("foot");
  try {
    const running = await invoke("running_games");
    if (!running.length) {
      body.innerHTML = `<div class="empty">${esc(txt()[0])}</div>`;
      foot.textContent = txt()[1];
      return;
    }
    const run = running[0];
    body.innerHTML = `<div class="game"><div><b>${esc(run.name)}</b><small>${esc(run.label)} · ${hm(run.elapsed)}</small></div><span class="live">${esc(txt()[2])}</span></div><div id="ach" class="ach"><div class="muted">${esc(txt()[4])}</div></div>`;
    foot.textContent = running.length > 1 ? `+ ${running.length - 1} ${txt()[8]}` : `${hotkey} ${txt()[7]}`;
    const a = await invoke("game_achievements", { id: run.id }).catch(() => null);
    const box = document.getElementById("ach");
    if (!box) return;
    if (!a) { box.innerHTML = `<div class="muted">${esc(txt()[5])}</div>`; return; }
    const rows = a.rows.filter((r) => opts.locked || r.unlocked).slice(0, opts.compact ? 9 : 6);
    const progress = opts.progress ? `<div class="bar"><i style="width:${a.total ? Math.round((a.unlocked / a.total) * 100) : 0}%"></i></div><div class="count">${a.unlocked} / ${a.total} ${esc(txt()[6])}</div>` : "";
    box.innerHTML = `${progress}${rows.map((r) => `<div class="row ${r.unlocked ? "on" : ""}"><i>${r.unlocked ? "✓" : "•"}</i><span>${esc(r.name)}</span><em>${r.rarity != null ? r.rarity.toFixed(0) + " %" : ""}</em></div>`).join("")}`;
  } catch (e) {
    body.innerHTML = `<div class="empty">${esc(e)}</div>`;
  }
}

document.getElementById("close").onclick = () => invoke("overlay_hide");
document.addEventListener("keydown", (e) => { if (e.key === "Escape") invoke("overlay_hide"); });
listen("quark://overlay-open", refresh);
listen("quark://session-ended", refresh);
listen("quark://session-stopped", refresh);
refresh();
setInterval(() => { if (document.visibilityState === "visible") refresh(); }, 5000);
