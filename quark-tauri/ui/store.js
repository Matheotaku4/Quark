"use strict";
const STORE_TITLES = { "Offres spéciales": "st.specials", "Meilleures ventes": "st.topSellers", "Nouveautés": "st.newReleases", "Prochainement": "st.comingSoon", "Populaires sur Steam": "st.popular" };
const stTitle = (title) => { if (app.lang === "fr") return title; const key = STORE_TITLES[title]; return key ? t(key) : title; };
// Boutique Steam : catalogue en cache, actualisation, recherche, « à la une », réductions, recommandations.

app.store = { page: null, loading: false, error: null, q: "", results: null, searching: false, sErr: null, started: false };

const discountOf = (i) => i.price?.discountPercent || 0;
function money(p) {
  if (!p) return "";
  if (p.finalPrice === 0 && p.initial === 0) return L("Gratuit", "Free");
  try { return new Intl.NumberFormat(locale(), { style: "currency", currency: p.currency }).format(p.finalPrice / 100); } catch { return `${p.currency} ${(p.finalPrice / 100).toFixed(2)}`; }
}
function moneyOld(p) { try { return new Intl.NumberFormat(locale(), { style: "currency", currency: p.currency }).format(p.initial / 100); } catch { return `${p.currency} ${(p.initial / 100).toFixed(2)}`; } }
const thousands = (n) => new Intl.NumberFormat(locale()).format(n);
function priceLine(i) {
  if (!i.price) return "";
  const d = discountOf(i);
  return `${d ? `<em class="disc">-${d} %</em>` : ""}<b>${esc(money(i.price))}</b>${d ? `<s>${esc(moneyOld(i.price))}</s>` : ""}`;
}
function allItems() { return (app.store.page?.sections || []).flatMap((s) => s.items); }
function uniq(list) { const seen = new Set(); return list.filter((i) => !seen.has(i.appId) && seen.add(i.appId)); }
const heroOf = () => allItems().filter((i) => i.artworkUrl).sort((a, b) => discountOf(b) - discountOf(a))[0];
const dealsOf = (n) => uniq(allItems().filter((i) => discountOf(i) > 0).sort((a, b) => discountOf(b) - discountOf(a))).slice(0, n);
function leadersOf(n) {
  const l = uniq(allItems().slice().sort((a, b) => (b.playersPeak || 0) - (a.playersPeak || 0))).slice(0, n);
  return l.length ? l : (app.store.page?.sections[0]?.items || []).slice(0, n);
}

function storeCard(i, rank) {
  const d = discountOf(i);
  return `<button class="scard" data-sites="${esc(i.name)}" title="${esc(i.name)}"><div class="simg" style="${i.artworkUrl ? `background-image:url('${esc(i.artworkUrl)}')` : ""}">
    ${rank ? `<span class="rank">${rank}</span>` : ""}${d ? `<span class="dbadge">-${d}%</span>` : ""}${i.linuxAvailable ? `<span class="lx">Linux</span>` : ""}</div>
    <div class="sinfo"><b>${esc(i.name)}</b><div class="price">${priceLine(i)}</div>${i.playersPeak ? `<small>${esc(thousands(i.playersPeak))} ${esc(L("joueurs en pic", "peak players"))}</small>` : ""}</div></button>`;
}
const sectionHead = (id, title, caption) => `<div class="shead" id="${id}"><h3>${esc(title)}</h3>${caption ? `<small>${esc(caption)}</small>` : ""}</div>`;
const grid = (items) => `<div class="sgrid">${items.map((i) => storeCard(i)).join("")}</div>`;
function sectionCaption(s) {
  const peaks = s.items.filter((i) => i.playersPeak).length, parts = [`${s.items.length} ${L("jeux", "games")}`];
  if (peaks) parts.push(`${peaks} ${L("pics de joueurs", "player peaks")}`);
  if (/populaire|popular/i.test(s.title)) parts.push(L("classement officiel Steam", "official Steam ranking"));
  return parts.join(" · ");
}

function renderStore() {
  const s = app.store, p = s.page, hero = p && heroOf(), deals = p ? dealsOf(8) : [], reco = p?.recommendations || [];
  const chips = p ? [deals.length && ["st-deals", L("Top réductions", "Top deals")], reco.length && ["st-reco", L("Recommandé pour vous", "Recommended for you")], ...p.sections.map((x, n) => [`st-s${n}`, stTitle(x.title)])].filter(Boolean) : [];
  return `<div class="store">
    <div class="sbar"><input type="text" id="store-q" placeholder="${esc(L("Rechercher un jeu sur Steam…", "Search Steam…"))}" value="${esc(s.q)}">
      <button class="btn sec" id="store-search" ${s.q.trim().length < 2 || s.searching ? "disabled" : ""}>${esc(L("Rechercher", "Search"))}</button>
      <button class="btn" id="store-refresh" ${s.loading ? "disabled" : ""}>${ic("refresh")}${esc(L("Actualiser", "Refresh"))}</button></div>
    ${s.searching ? `<div class="sline">${esc(L("Recherche dans Steam…", "Searching Steam…"))}</div>` : ""}${s.sErr ? `<div class="sline err">${esc(s.sErr)}</div>` : ""}
    ${s.results ? `${sectionHead("st-res", L("RÉSULTATS", "RESULTS"), `${s.results.length} ${L("correspondance(s)", "match(es)")}`)}${s.results.length ? grid(s.results) : `<div class="sline">${esc(L("Aucun jeu ne correspond à cette recherche.", "No game matches this search."))}</div>`}` : ""}
    ${s.loading ? `<div class="sline">${esc(L("Chargement du catalogue Steam…", "Loading the Steam catalog…"))}</div>` : ""}
    ${s.error ? `<div class="sline">${esc(s.error)}${p ? " · " + esc(L("les dernières offres en cache restent affichées.", "the last cached offers are still shown.")) : ""}</div>` : ""}
    ${p?.notice ? `<div class="sline">${esc(p.notice)}</div>` : ""}${p?.stale && !s.loading ? `<div class="sline">${esc(L("Cache local · données potentiellement périmées", "Local cache · data may be out of date"))}</div>` : ""}
    ${p ? `<div class="schips">${chips.map(([id, l]) => `<button data-jump="${id}">${esc(l)}</button>`).join("")}</div>
      ${hero ? `<div class="stop"><div class="shero" style="background-image:url('${esc(hero.artworkUrl)}')"><div class="gh-fade"></div><div class="sh-in">
          <span class="pill">${esc(L("À LA UNE", "FEATURED"))}</span><h2>${esc(hero.name)}</h2><div class="price big">${priceLine(hero)}</div>
          <button class="btn" data-open="${esc(hero.storeUrl)}">${esc(L("Voir sur Steam", "View on Steam"))}</button></div></div>
        <aside class="ov"><h3>${esc(L("Les plus joués", "Most played"))}</h3>${leadersOf(5).map((i, n) => `<button class="lead" data-open="${esc(i.storeUrl)}"><span>${String(n + 1).padStart(2, "0")}</span><b>${esc(i.name)}</b>${i.playersPeak ? `<em>${esc(thousands(i.playersPeak))}</em>` : ""}</button>`).join("")}</aside></div>` : ""}
      ${deals.length ? `${sectionHead("st-deals", L("TOP RÉDUCTIONS", "TOP DEALS"), L("Les plus fortes remises du catalogue, mises à jour avec lui", "The biggest discounts in the catalog"))}<div class="rail2">${deals.map((i, n) => storeCard(i, n + 1)).join("")}</div>` : ""}
      ${reco.length ? `${sectionHead("st-reco", L("RECOMMANDÉ POUR VOUS", "RECOMMENDED FOR YOU"), L(`D'après ${p.recommendationSource} · aucune donnée de navigation n'est envoyée à Steam`, `Based on ${p.recommendationSource} · no browsing data is sent to Steam`))}${grid(reco)}` : ""}
      ${p.sections.map((x, n) => `${sectionHead(`st-s${n}`, stTitle(x.title), sectionCaption(x))}${grid(x.items)}`).join("")}`
    : !s.loading ? `<div class="sline">${esc(s.error ? L("Le catalogue Steam est indisponible et aucun cache n'est encore enregistré.", "The Steam catalog is unavailable and nothing is cached yet.") : L("Le catalogue Steam apparaîtra ici.", "The Steam catalog will appear here."))}</div>` : ""}
  </div>`;
}

function repaintStore() {
  if (app.view !== "store") return;
  const sheet = $("#sheet"), top = sheet.scrollTop, had = document.activeElement?.id === "store-q", pos = had ? document.activeElement.selectionStart : 0;
  sheet.innerHTML = renderStore(); bindStore(); sheet.scrollTop = top;
  if (had) { const q = $("#store-q"); q.focus(); q.setSelectionRange(pos, pos); }
}
function bindStore() {
  const q = $("#store-q"); if (!q) return;
  q.oninput = () => { app.store.q = q.value; const b = $("#store-search"); b && (b.disabled = q.value.trim().length < 2 || app.store.searching); };
  q.onkeydown = (e) => { if (e.key === "Enter") { e.stopPropagation(); storeSearch(); } };
  $("#store-search").onclick = storeSearch; $("#store-refresh").onclick = storeRefresh;
  $$("[data-jump]").forEach((b) => (b.onclick = () => document.getElementById(b.dataset.jump)?.scrollIntoView({ block: "start" })));
  $$("[data-sites]").forEach((card) => (card.onclick = () => webSearchModal(card.dataset.sites)));
}
async function storeSearch() {
  const s = app.store; if (s.q.trim().length < 2 || s.searching) return;
  s.searching = true; s.sErr = null; repaintStore();
  try { s.results = await invoke("store_search", { query: s.q }); } catch (e) { s.sErr = String(e); }
  s.searching = false; repaintStore();
}
async function storeRefresh() {
  const s = app.store; if (s.loading) return;
  s.loading = true; s.error = null; repaintStore();
  try { s.page = await invoke("store_refresh"); } catch (e) { s.error = String(e); }
  s.loading = false; repaintStore();
}
/** Appelé à chaque arrivée sur l'onglet : affiche le cache tout de suite, puis actualise s'il est périmé. */
function storeEnter() {
  const s = app.store; if (s.started) return; s.started = true;
  invoke("store_load").then((p) => { if (p) { s.page = p; repaintStore(); } if (!p || p.stale) storeRefresh(); }).catch(() => storeRefresh());
}
