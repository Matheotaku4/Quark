# Recherche multi-sites — plan d’implémentation

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Restaurer une recherche multi-sites complète à sept sources, avec GameBounty en premier, depuis la boutique et la fiche d’un jeu.

**Architecture:** L’agrégation reste côté Rust : `src/game_search.rs` interroge chaque source dans son propre fil, avec un adaptateur dédié. `site_search` renvoie les résultats et les erreurs par source. Le front affiche un modal groupé par site, GameBounty en tête, et ouvre les liens dans le navigateur.

**Tech Stack:** Rust (`reqwest`, `serde_json`, `encoding_rs`), Tauri 2, HTML/CSS/JS.

**Spec:** `docs/superpowers/specs/2026-11-21-interface-2.1-design.md`

## Global Constraints

- Sept sources, dans cet ordre : GameBounty, AnkerGames, AstralGames, Kryoto, Online-Fix, ZeiGames, scene.cat.
- GameBounty est toujours le premier groupe et le premier résultat.
- Aucun fichier n’est téléchargé ; seuls des liens de page sont ouverts.
- Chaque source échoue indépendamment ; une erreur ne bloque pas les autres.
- Six langues : fr, en, es, de, pt, ru.
- `main` ne doit pas être modifiée ; travail sur `feat/interface-2.1.0`.
- `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`, `cargo test --locked` pour le crate racine ; les mêmes commandes avec `--manifest-path quark-tauri/Cargo.toml` doivent aussi passer.
- Version finale 2.1.0 dans les manifests et `tauri.conf.json`.

## Review Focus

- GameBounty indisponible : les six autres sources s’affichent quand même, avec l’erreur GameBounty visible.
- AstralGames change d’action Next.js : erreur isolée + lien de recherche, pas de page vide.
- Titre non latin ou page Online-Fix en Windows-1251 : le titre s’affiche correctement, pas de mojibake.
- Résultat sans jaquette : monogramme ou ligne sans image, jamais de `<img>` cassé.
- Doublons ZeiGames/Online-Fix : une seule ligne par URL, GameBounty toujours en premier.

---

## Préparation

- [ ] Se placer sur la branche de travail (la créer si elle n’existe pas) :

```bash
git switch feat/interface-2.1.0 2>/dev/null || git switch -c feat/interface-2.1.0
```

- [ ] Vérifier que `main` n’a pas été modifiée :

```bash
git rev-parse main
```

---

### Task 1: Étendre `SearchSite`, priorité et GameBounty `.st`

**Files:**
- Modify: `src/game_search.rs`
- Test: `src/game_search.rs` (module `tests`)

**Interfaces:**
- Consumes: `SiteHit`, `search_key`, `filter_relevant`, `fetch_text`.
- Produces:
  - `enum SearchSite { GameBounty, AnkerGames, AstralGames, Kryoto, OnlineFix, ZeiGames, SceneCat }`
  - `const ALL: [SearchSite; 7]`
  - `fn label(self) -> &'static str`
  - `fn host(self) -> &'static str`
  - `fn search_url(self, query: &str) -> String`
  - `fn search_gamebounty(query: &str) -> Result<Vec<SiteHit>, String>`

- [ ] **Step 1: Écrire le test de priorité**

```rust
#[test]
fn gamebounty_est_la_premiere_source() {
    assert_eq!(SearchSite::ALL.len(), 7);
    assert_eq!(SearchSite::ALL[0], SearchSite::GameBounty);
    assert_eq!(SearchSite::ALL[0].host(), "gamebounty.st");
    assert_eq!(SearchSite::OnlineFix.host(), "online-fix.me");
    assert_eq!(SearchSite::ZeiGames.host(), "zeigames.com");
}

#[test]
fn les_sept_sources_ont_une_recherche_web_de_repli() {
    for site in SearchSite::ALL {
        let url = site.search_url("The Witcher 3");
        assert!(url.starts_with("https://"));
        assert!(url.contains("The+Witcher+3"));
    }
}
```

- [ ] **Step 2: Lancer le test et vérifier l’échec**

Run: `cargo test gamebounty_est_la_premiere_source -v`
Expected: FAIL car la longueur/priorité ne correspond pas.

- [ ] **Step 3: Implémenter l’enum et les URL**

- Remplacer les variantes et `ALL` par les sept sources dans l’ordre de priorité, et mettre à jour le commentaire « Les cinq sources » en « Les sept sources ».
- `label` : `"GameBounty"`, `"AnkerGames"`, `"AstralGames"`, `"Kryoto"`, `"Online-Fix"`, `"ZeiGames"`, `"scene.cat"`.
- `host` : `"gamebounty.st"`, `"ankergames.net"`, `"astralgames.net"`, `"kryo.to"`, `"online-fix.me"`, `"zeigames.com"`, `"scene.cat"`.
- `search_url` :
  - GameBounty `https://gamebounty.st/search?q=…`
  - AnkerGames `https://ankergames.net/search?q=…`
  - AstralGames `https://astralgames.net/search?q=…`
  - Kryoto `https://kryo.to/catalog?q=…`
  - OnlineFix `https://online-fix.me/index.php?do=search&subaction=search&story=…`
  - ZeiGames `https://zeigames.com/search/?q=…`
  - SceneCat `https://scene.cat/?q=…`
- `search` mappe les nouvelles variantes vers les adaptateurs.

- [ ] **Step 4: Mettre à jour GameBounty**

- `search_gamebounty` utilise `https://gamebounty.st/api/v1/search?q=…`.
- `parse_gamebounty` construit `https://gamebounty.st/{slug}` (sans suffixe).
- Mettre à jour le test existant `gamebounty_construit_le_lien_canonique` pour attendre `.st` et l’URL sans suffixe.

- [ ] **Step 5: Lancer les tests et compiler**

Run: `cargo test gamebounty_est_la_premiere_source les_sept_sources_ont_une_recherche_web_de_repli gamebounty_construit_le_lien_canonique -v`
Expected: PASS.
Run: `cargo check`
Expected: OK.

- [ ] **Step 6: Commit**

```bash
git add src/game_search.rs
git commit -m "feat(search): prioritize gamebounty and update its domain"
```

---

### Task 2: AstralGames via l’action Next.js

**Files:**
- Modify: `src/game_search.rs`
- Test: `src/game_search.rs`

**Interfaces:**
- Consumes: `fetch_text`, `json_root`, `clean_detail`, `http_url`.
- Produces:
  - `const ASTRAL_SEARCH_ACTION: &str = "602df744916efa21adc18a657f93057bc489bb7f6e";`
  - `fn search_astral(query: &str) -> Result<Vec<SiteHit>, String>`
  - `fn parse_astral_rsc(text: &str) -> Vec<SiteHit>`

- [ ] **Step 1: Écrire le test de parsing RSC**

```rust
#[test]
fn astral_lit_la_ligne_rsc_des_jeux() {
    let body = "0:{\"a\":\"$@1\"}\n1:{\"games\":[{\"name\":\"Cyberpunk 2077\",\"slug\":\"cyberpunk-2077\",\"image\":\"https://cdn/cover.jpg\",\"file_size\":\"141.82 GB\",\"build_id\":20383525}],\"hasMore\":false,\"total\":1}\n";
    let hits = parse_astral_rsc(body);
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].title, "Cyberpunk 2077");
    assert_eq!(hits[0].url, "https://astralgames.net/game/cyberpunk-2077");
    assert_eq!(hits[0].detail, "141.82 GB · Build 20383525");
    assert_eq!(hits[0].cover.as_deref(), Some("https://cdn/cover.jpg"));
}
```

- [ ] **Step 2: Lancer le test et vérifier l’échec**

Run: `cargo test astral_lit_la_ligne_rsc_des_jeux -v`
Expected: FAIL car `parse_astral_rsc` n’existe pas.

- [ ] **Step 3: Implémenter la requête et le parsing**

- `search_astral` construit `https://astralgames.net/search?q={query}`.
- POST avec :
  - `Next-Action: ASTRAL_SEARCH_ACTION`
  - `Content-Type: text/plain;charset=UTF-8`
  - `Accept: text/x-component`
  - corps `serde_json::json!([query, { "page": 1, "limit": 12, "sortBy": "relevance", "timeframe": "all", "tier": null }]).to_string()`
- Lire la réponse avec `crate::net::text` et un délai de 22 s.
- `parse_astral_rsc` parcourt les lignes ; pour chaque ligne commençant par `1:`, parse le JSON après le préfixe et lit `games[]`.
- Chaque jeu : `name`, `slug`, `image`, `file_size`, `build_id` ; détail = `clean_detail(&[file_size, "Build {id}"])` ; URL `/game/{slug}`.
- Si l’action échoue ou si aucune ligne `1:` n’est lisible, retourner `Err("AstralGames a refusé la recherche (action expirée)")`.

- [ ] **Step 4: Lancer le test et compiler**

Run: `cargo test astral_lit_la_ligne_rsc_des_jeux -v`
Expected: PASS.
Run: `cargo check`
Expected: OK.

- [ ] **Step 5: Commit**

```bash
git add src/game_search.rs
git commit -m "feat(search): query astralgames through its next action"
```

---

### Task 3: Online-Fix et décodage Windows-1251

**Files:**
- Modify: `Cargo.toml`
- Modify: `Cargo.lock`
- Modify: `src/game_search.rs`
- Test: `src/game_search.rs`

**Interfaces:**
- Consumes: `encoding_rs::WINDOWS_1251`.
- Produces:
  - `fn fetch_bytes(url: &str, accept: &str) -> Result<Vec<u8>, String>`
  - `fn decode_windows_1251(bytes: &[u8]) -> String`
  - `fn search_onlinefix(query: &str) -> Result<Vec<SiteHit>, String>`
  - `fn parse_onlinefix(text: &str) -> Vec<SiteHit>`

- [ ] **Step 1: Ajouter la dépendance**

Dans `Cargo.toml` :

```toml
encoding_rs = "0.8"
```

Puis `cargo check` et `cargo check --manifest-path quark-tauri/Cargo.toml` pour mettre les deux `Cargo.lock` à jour.

- [ ] **Step 2: Écrire les tests**

```rust
#[test]
fn onlinefix_decode_le_windows_1251() {
    let bytes = b"\xCF\xF0\xE8\xE2\xE5\xF2";
    assert_eq!(decode_windows_1251(bytes), "Привет");
}

#[test]
fn onlinefix_lit_les_articles_et_deduit_le_titre() {
    let body = r#"
      <div class="article clr">
        <a class="big-link" href="https://online-fix.me/games/rpg/17004-elden-ring-po-seti.html"></a>
        <a class="img" href="https://online-fix.me/games/rpg/17004-elden-ring-po-seti.html">
          <img data-src="https://online-fix.me/poster.jpg" alt="">
        </a>
        <h2 class="title">Elden Ring по сети</h2>
      </div>
    "#;
    let hits = parse_onlinefix(body);
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].title, "Elden Ring по сети");
    assert_eq!(hits[0].url, "https://online-fix.me/games/rpg/17004-elden-ring-po-seti.html");
    assert_eq!(hits[0].cover.as_deref(), Some("https://online-fix.me/poster.jpg"));
}
```

- [ ] **Step 3: Lancer les tests et vérifier l’échec**

Run: `cargo test onlinefix_decode_le_windows_1251 onlinefix_lit_les_articles_et_deduit_le_titre -v`
Expected: FAIL car les fonctions n’existent pas.

- [ ] **Step 4: Implémenter**

- `fetch_bytes` reprend `fetch_text` mais retourne `crate::net::bytes`.
- `decode_windows_1251` utilise `encoding_rs::WINDOWS_1251.decode(bytes).0.into_owned()`.
- `search_onlinefix` appelle
  `https://online-fix.me/index.php?do=search&subaction=search&story={query}`,
  puis `parse_onlinefix(&decode_windows_1251(&bytes))`.
- `parse_onlinefix` découpe sur `<div class="article clr">`, extrait :
  - `href` après `class="big-link"` ;
  - titre entre `<h2 class="title">` et `</h2>` ;
  - image entre `data-src="` et `"` ;
  - nettoie les balises et les entités.
- Limiter à 8 résultats et dédupliquer par URL.

- [ ] **Step 5: Lancer les tests et compiler**

Run: `cargo test onlinefix_decode_le_windows_1251 onlinefix_lit_les_articles_et_deduit_le_titre -v`
Expected: PASS.
Run: `cargo check`
Expected: OK.

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml Cargo.lock quark-tauri/Cargo.lock src/game_search.rs
git commit -m "feat(search): add online-fix with windows-1251 decoding"
```

---

### Task 4: ZeiGames

**Files:**
- Modify: `src/game_search.rs`
- Test: `src/game_search.rs`

**Interfaces:**
- Produces:
  - `fn search_zeigames(query: &str) -> Result<Vec<SiteHit>, String>`
  - `fn parse_zeigames(text: &str) -> Vec<SiteHit>`

- [ ] **Step 1: Écrire le test**

```rust
#[test]
fn zeigames_lit_les_sujets_et_deduit_les_doublons() {
    let body = r#"
      <a href="https://zeigames.com/topic/4295-elden-ring-free-download-v1171-all-dlc-online/" data-linktype="link" data-searchable>ELDEN RING Free Download (v1.17.1 + ALL DLC + Online)</a>
      <a href="https://zeigames.com/topic/4295-elden-ring-free-download-v1171-all-dlc-online/" data-linktype="link" data-searchable>ELDEN RING Free Download (v1.17.1 + ALL DLC + Online)</a>
      <a href="https://zeigames.com/topic/9913-elden-ring-nightreign-free-download-v1035-all-dlc-online/" data-linktype="link" data-searchable>ELDEN RING NIGHTREIGN Free Download (v1.03.5 + ALL DLC + Online)</a>
    "#;
    let hits = parse_zeigames(body);
    assert_eq!(hits.len(), 2);
    assert_eq!(hits[0].title, "ELDEN RING Free Download (v1.17.1 + ALL DLC + Online)");
    assert_eq!(hits[0].url, "https://zeigames.com/topic/4295-elden-ring-free-download-v1171-all-dlc-online/");
    assert!(hits[1].title.contains("NIGHTREIGN"));
}
```

- [ ] **Step 2: Lancer le test et vérifier l’échec**

Run: `cargo test zeigames_lit_les_sujets_et_deduit_les_doublons -v`
Expected: FAIL car `parse_zeigames` n’existe pas.

- [ ] **Step 3: Implémenter**

- `search_zeigames` appelle `https://zeigames.com/search/?q={query}`.
- `parse_zeigames` cherche chaque occurrence de `data-linktype="link"`, remonte au `<a href="` précédent, extrait l’URL `https://zeigames.com/topic/...`, puis le texte entre le `>` du lien et `</a>`.
- Décode les entités, déduplique par URL, limite à 8.
- Détail : `"ZeiGames"` ou vide ; pas de couverture.

- [ ] **Step 4: Lancer le test et compiler**

Run: `cargo test zeigames_lit_les_sujets_et_deduit_les_doublons -v`
Expected: PASS.
Run: `cargo check`
Expected: OK.

- [ ] **Step 5: Commit**

```bash
git add src/game_search.rs
git commit -m "feat(search): add zeigames html adapter"
```

---

### Task 5: `site_search` avec erreurs par source et ordre garanti

**Files:**
- Modify: `quark-tauri/src/features.rs`
- Test: `quark-tauri/src/features.rs` (test pur)

**Interfaces:**
- Consumes: `quark::game_search::{SearchSite, search}`.
- Produces:
  - `struct SiteHitDto { site_id, site, title, url, detail, cover }`
  - `struct SiteErrorDto { site_id, site, message }`
  - `struct SiteSearchDto { hits: Vec<SiteHitDto>, errors: Vec<SiteErrorDto> }`
  - `#[tauri::command] pub async fn site_search(query: String) -> Result<SiteSearchDto, String>`

- [ ] **Step 1: Écrire le test d’ordre et de sérialisation**

Ajouter un test pur dans `features.rs` :

```rust
#[test]
fn les_resultats_de_recherche_suivent_l_ordre_des_sources() {
    let sites = quark::game_search::SearchSite::ALL;
    assert_eq!(sites[0], quark::game_search::SearchSite::GameBounty);
    let labels: Vec<_> = sites.iter().map(|site| site.label()).collect();
    assert_eq!(labels.first().copied(), Some("GameBounty"));
}
```

- [ ] **Step 2: Lancer le test**

Run: `cargo test --manifest-path quark-tauri/Cargo.toml les_resultats_de_recherche_suivent_l_ordre_des_sources -v`
Expected: PASS (ou FAIL si l’enum n’est pas encore à jour ; il doit l’être après la tâche 1).

- [ ] **Step 3: Modifier le DTO et la commande**

- Ajouter `site_id` à `SiteHitDto` (`format!("{:?}", site).to_lowercase()` ou une méthode `id()`).
- Ajouter `SiteErrorDto` et `SiteSearchDto`.
- `site_search` crée `hits` et `errors`, lance les sept sources, et pousse les erreurs par source.
- Toujours retourner `Ok(SiteSearchDto { hits, errors })` pour une requête valide, même si toutes les sources échouent ; le front affiche les erreurs.
- Pour une requête de moins de 2 caractères, retourner `Ok` avec deux vecteurs vides.
- Ne pas inclure le corps des réponses dans les erreurs.

- [ ] **Step 4: Lancer la compilation**

Run: `cargo check --manifest-path quark-tauri/Cargo.toml`
Expected: OK.

- [ ] **Step 5: Commit**

```bash
git add quark-tauri/src/features.rs
git commit -m "feat(search): return per-source results and errors"
```

---

### Task 6: Modal multi-sites et points d’entrée

**Files:**
- Modify: `quark-tauri/ui/dialogs.js`
- Modify: `quark-tauri/ui/store.js`
- Modify: `quark-tauri/ui/app.js`
- Modify: `quark-tauri/ui/library.js`
- Modify: `quark-tauri/ui/styles.css`
- Modify: `quark-tauri/ui/i18n-extra.js`
- Modify: `quark-tauri/ui/i18n-ru.js`
- Test: capture Xvfb + appel réel

**Interfaces:**
- Consumes: `invoke("site_search", { query }) -> { hits, errors }`.
- Produces:
  - `function webSearchModal(query = "")`
  - `const SITE_META` (libellé, hôte, couleur)
  - `function renderWebResults(state) -> string`

- [ ] **Step 1: Remplacer `webSearchModal`**

- Modal large avec champ de recherche, bouton Rechercher, puces de sources, conteneur `#web-res`.
- État local : `{ q, loading, hits, errors, filter }`.
- `SITE_META` est un tableau ordonné :
  `gamebounty`, `ankergames`, `astralgames`, `kryoto`, `onlinefix`,
  `zeigames`, `scenecat`, avec pour chacun `{ id, label, host }`.
- À l’ouverture avec une requête de 2 caractères ou plus, lancer la recherche.
- `invoke("site_search", { query })` puis `renderWebResults`.
- `renderWebResults` groupe `hits` par `siteId`, dans l’ordre de `SITE_META` (GameBounty d’abord).
- Pour chaque source : titre, hôte, nombre de résultats, erreur éventuelle, lien « Chercher sur le site ».
- Chaque résultat : jaquette ou monogramme, titre, détail, bouton « Ouvrir » (`data-open`) et « Utiliser ce titre » (`data-web-use`).
- Ajouter la mention « Quark ne télécharge aucun fichier depuis ces sites. ».

- [ ] **Step 2: Ajouter les points d’entrée**

- `store.js` : `storeCard` utilise `data-sites="${esc(i.name)}"` au lieu de `data-open` ; `bindStore` lie `[data-sites]` à `webSearchModal(name)`. Le bouton « Voir sur Steam » du hero garde `data-open`.
- `app.js` : ajouter `acts.sites = () => webSearchModal(game(app.sel)?.name || "")`.
- `library.js` : ajouter un bouton d’action `sites` dans la fiche de jeu, à côté des actions existantes.
- Le bouton « Sites de téléchargement » du modal d’ajout appelle déjà `webSearchModal`.
- La palette garde l’action `web.title` avec `app.pal.q`.

- [ ] **Step 3: Vérifier le rendu**

Run:
```bash
xvfb-run -a -s "-screen 0 1360x860x24" bash -c '
  WEBKIT_DISABLE_DMABUF_RENDERER=1 cargo run --manifest-path quark-tauri/Cargo.toml -- --demo &
  APP=$!; sleep 12
  xdotool mousemove 275 24 click 1; sleep 2
  xdotool mousemove 300 300 click 1; sleep 3
  import -window root /tmp/quark-sites.png
  kill $APP
'
```
Expected: le modal s’ouvre, GameBounty apparaît en premier, les autres sources suivent, les erreurs éventuelles sont visibles et les liens sont cliquables.

- [ ] **Step 4: Commit**

```bash
git add quark-tauri/ui/dialogs.js quark-tauri/ui/store.js quark-tauri/ui/app.js quark-tauri/ui/library.js quark-tauri/ui/styles.css quark-tauri/ui/i18n-extra.js quark-tauri/ui/i18n-ru.js
git commit -m "feat(search): restore the multi-site search modal"
```

---

### Task 7: Traductions, version 2.1.0, documentation et vérification finale

**Files:**
- Modify: `quark-tauri/ui/i18n-extra.js`
- Modify: `quark-tauri/ui/i18n-ru.js`
- Modify: `assets/i18n/fr.json`, `en.json`, `es.json`, `de.json`, `pt.json`, `ru.json` (si des libellés sont utilisés côté Rust)
- Modify: `Cargo.toml`, `quark-tauri/Cargo.toml`, `quark-tauri/tauri.conf.json`
- Modify: `README.md`, `wiki/`, `docs/images/`, `site/assets/`
- Test: tout le dépôt

**Interfaces:**
- Consumes: toutes les tâches précédentes.
- Produces: version 2.1.0 et documentation à jour.

- [ ] **Step 1: Ajouter les libellés dans les six langues**

Libellés minimum :

- `pv.all`, `pv.filter.all`, `pv.filter.installed`, `pv.filter.missing`
- `pv.sort.title`, `pv.sort.installed`, `pv.sort.synced`
- `pv.layout.grid`, `pv.layout.list`, `pv.count`, `pv.more`
- `pv.menu.openFolder`, `pv.menu.copyPath`, `pv.menu.details`
- `web.title`, `web.placeholder`, `web.search`, `web.loading`, `web.none`
- `web.use`, `web.open`, `web.sources`, `web.disclaimer`
- `web.error`, `web.retry`, `web.steam`
- `web.site.gamebounty`, `web.site.ankergames`, `web.site.astralgames`,
  `web.site.kryoto`, `web.site.onlinefix`, `web.site.zeigames`,
  `web.site.scenecat`

Ajouter les traductions dans `i18n-extra.js` pour fr/en/es/de/pt et dans
`i18n-ru.js` pour le russe. Ne pas laisser une clé française dans une autre
langue.

- [ ] **Step 2: Bumper la version**

- `Cargo.toml` : `version = "2.1.0"`
- `quark-tauri/Cargo.toml` : `version = "2.1.0"`
- `quark-tauri/tauri.conf.json` : `"version": "2.1.0"`
- Vérifier `Cargo.lock` après `cargo check`.

- [ ] **Step 3: Mettre à jour README, wiki et notes de version**

- README : « Mes jeux » (filtres, tri, liste/grille, couvertures locales) et
  recherche multi-sites (7 sources, GameBounty en premier, aucun téléchargement).
- Wiki : pages concernées.
- Notes de version 2.1.0 : corrections de performance, jaquettes, artefacts de
  rendu, recherche multi-sites.

- [ ] **Step 4: Régénérer les captures**

Captures minimum : `my-games-{fr,en}.png`, `site-search-{fr,en}.png`,
`library-{fr,en}.png`. Copier dans `docs/images/` **et** `site/assets/`.

- [ ] **Step 5: Vérification complète**

Run:
```bash
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
cargo fmt --manifest-path quark-tauri/Cargo.toml --check
cargo clippy --manifest-path quark-tauri/Cargo.toml --all-targets --locked -- -D warnings
cargo test --manifest-path quark-tauri/Cargo.toml --locked
```
Expected: toutes les commandes passent.

- [ ] **Step 6: Commit final**

```bash
git add Cargo.toml Cargo.lock quark-tauri/Cargo.lock quark-tauri/Cargo.toml quark-tauri/tauri.conf.json README.md wiki docs/images site/assets quark-tauri/ui/i18n-extra.js quark-tauri/ui/i18n-ru.js
git commit -m "release: Quark 2.1.0 interface and multi-site search"
```

- [ ] **Step 7: Vérifier `main` et pousser la branche**

Run:
```bash
git rev-parse main
git status --short
git push origin feat/interface-2.1.0
```
Expected: `main` au SHA initial ; seuls les fichiers prévus sont modifiés ; la branche est poussée sans fusion.
