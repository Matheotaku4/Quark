# Recherche par extensions — plan d'implémentation

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Rendre la recherche de sites facultative, pilotée par un manifeste JSON externe, tout en conservant les sept adaptateurs actuels comme drivers intégrés.

**Architecture:** Un module `quark::extensions` valide les manifestes, tient un cache local et expose les sites actifs. `quark::game_search` devient un registre de drivers : sept drivers intégrés qui appellent les parseurs existants, plus deux drivers génériques JSON et HTML. Tauri expose l'ajout, l'actualisation, le retrait et la recherche ; l'interface masque tous les points d'entrée tant qu'aucun site n'est actif.

**Tech Stack:** Rust 2024, Tauri 2, `serde_json`, `reqwest`, `scraper` pour les sélecteurs CSS, JavaScript vanilla pour l'interface.

**Spec:** `docs/superpowers/specs/2026-11-27-search-extensions-design.md`

## Global Constraints

- Aucune source de recherche installée par défaut : `Settings::extension_sources` est vide.
- Manifeste : `schemaVersion` 1, 256 Kio maximum, 64 sites maximum.
- URL de manifeste et de requête : HTTPS uniquement, hôte public uniquement.
- En-têtes autorisés : `Accept`, `Accept-Language`, `Content-Type`, `User-Agent`, `Referer`, `Origin`, `Next-Action`, `X-Requested-With`.
- Aucun code d'extension exécuté ; aucun cookie, aucun `Authorization`.
- Aucun appel réseau au démarrage ; réseau uniquement sur ajout ou actualisation.
- Limite de résultats par site : 8 par défaut, 20 maximum.
- Les sept adaptateurs actuels gardent leur comportement et leurs tests.
- Version cible : 2.3.0 pour `quark` et `quark-tauri`.
- Tous les libellés d'interface doivent exister en fr, en, es, de, pt, ru.

## Review Focus

- Une extension malveillante pointe vers `localhost` ou une adresse privée : la validation doit refuser avant toute requête.
- Un manifeste contient un site invalide parmi des sites valides : tout le manifeste est refusé, aucun chargement partiel.
- Un site HTML valide ne trouve plus sa structure sur la page : la recherche renvoie un résultat vide ou une erreur par site, sans bloquer les autres.
- Le cache est corrompu ou une source est retirée : Quark démarre quand même et n'affiche aucun site fantôme.
- Aucune extension n'est installée : aucun bouton de recherche ne doit apparaître, y compris dans la palette et la boutique.

---

### Task 1: Modèle de manifeste et validation

**Files:**
- Modify: `Cargo.toml`
- Create: `src/extensions.rs`
- Modify: `src/game_search.rs`
- Modify: `src/lib.rs`
- Test: `src/extensions.rs` (module `tests`)

**Interfaces:**
- Consumes: `serde`, `serde_json`, `reqwest::Url`, `std::net::IpAddr`.
- Produces:
  - `game_search::SiteDefinition { id, label, host, search_url, driver }`
  - `game_search::SiteDriver` avec variantes intégrées et `Json`/`Html`
  - `extensions::ExtensionManifest { schema_version, id, name, version, description, homepage, sites }`
  - `extensions::parse_manifest(bytes: &[u8]) -> Result<ExtensionManifest, String>`
  - `extensions::validate_manifest(&ExtensionManifest) -> Result<(), String>`

- [ ] **Step 1: Ajouter la dépendance `scraper`**

Dans `Cargo.toml`, ajouter `scraper = "0.29"` après `encoding_rs`.

- [ ] **Step 2: Écrire les tests de validation**

Dans `src/extensions.rs`, ajouter des tests qui construisent un manifeste valide avec un site `gamebounty`, puis vérifient :

```rust
#[test]
fn un_manifeste_valide_est_accepte() { /* 1 site gamebounty, label GameBounty */ }

#[test]
fn un_manifeste_avec_un_doublon_de_site_est_refuse() { /* deux sites id "same" */ }

#[test]
fn une_source_non_https_est_refusee() { /* homepage ou searchUrl en http:// */ }

#[test]
fn un_hote_prive_est_refuse() { /* searchUrl https://127.0.0.1/... et https://localhost/... */ }

#[test]
fn un_identifiant_invalide_est_refuse() { /* id "Bad Id!" */ }

#[test]
fn un_manifeste_trop_gros_est_refuse() { /* parse_manifest avec 256 Kio + 1 octet */ }

#[test]
fn un_driver_inconnu_est_refuse() { /* JSON avec "driver": "nope" */ }
```

- [ ] **Step 3: Lancer les tests pour vérifier l'échec**

Run: `cargo test --locked extensions::tests -- --nocapture`
Expected: FAIL, `ExtensionManifest` et `parse_manifest` n'existent pas.

- [ ] **Step 4: Définir les types de site dans `game_search.rs`**

Ajouter les types sérialisables suivants, sans encore modifier les parseurs :

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SiteDefinition {
    pub id: String,
    pub label: String,
    pub host: String,
    pub search_url: String,
    #[serde(flatten)]
    pub driver: SiteDriver,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "driver", rename_all = "lowercase")]
pub enum SiteDriver {
    #[serde(rename = "gamebounty")] GameBounty,
    #[serde(rename = "ankergames")] AnkerGames,
    #[serde(rename = "astralgames")] AstralGames,
    #[serde(rename = "kryoto")] Kryoto,
    #[serde(rename = "onlinefix")] OnlineFix,
    #[serde(rename = "zeigames")] ZeiGames,
    #[serde(rename = "scenecat")] SceneCat,
    Json(JsonDriver),
    Html(HtmlDriver),
}
```

Déclarer `JsonDriver`, `HtmlDriver`, `RequestSpec`, `FieldSpec`, `HttpMethod` et `ResponseEncoding` avec `#[serde(rename_all = "camelCase")]`, `Default` pour `HttpMethod` (GET) et `ResponseEncoding` (UTF-8), et `default_limit()` retournant 8.

- [ ] **Step 5: Implémenter `ExtensionManifest` et la validation**

Dans `src/extensions.rs`, définir les constantes `SCHEMA_VERSION = 1`, `MAX_MANIFEST_BYTES = 256 * 1024`, `MAX_SITES = 64`. `parse_manifest` refuse un corps trop gros, parse en JSON, puis appelle `validate_manifest`.

`validate_manifest` vérifie : schéma, identifiants `[a-z0-9._-]{1,64}`, noms non vides, 1..=64 sites, doublons, `searchUrl` HTTPS avec `{query}`, hôte public, et `game_search::validate_site` pour chaque driver. `validate_site` vérifie les URL de requête et les sélecteurs CSS avec `scraper::Selector::parse`.

- [ ] **Step 6: Exporter le module**

Dans `src/lib.rs`, ajouter `pub mod extensions;`.

- [ ] **Step 7: Lancer les tests**

Run: `cargo test --locked extensions::tests -- --nocapture`
Expected: PASS.

- [ ] **Step 8: Commit**

```bash
git add Cargo.toml Cargo.lock src/extensions.rs src/game_search.rs src/lib.rs
git commit -m "ajoute le modele de manifeste des extensions"
```

---

### Task 2: Registre de drivers et sept adaptateurs

**Files:**
- Modify: `src/game_search.rs`
- Test: `src/game_search.rs` (module `tests`)
- Test: `quark-tauri/src/features.rs` (retirer l'ancien test)

**Interfaces:**
- Consumes: `SiteDefinition`, `SiteDriver`, `SiteHit`.
- Produces:
  - `game_search::search(site: &SiteDefinition, query: &str) -> Result<Vec<SiteHit>, String>`
  - `game_search::validate_site(site: &SiteDefinition) -> Result<(), String>`

- [ ] **Step 1: Écrire les tests de désérialisation et de dispatch**

Remplacer les tests qui utilisent `SearchSite::ALL` par :

```rust
#[test]
fn les_sept_drivers_integres_se_deserialisent() { /* 7 SiteDefinition, assert driver et label */ }

#[test]
fn un_site_integre_garde_son_url_de_repli() { /* search_url contient {query} */ }

#[test]
fn un_driver_json_se_deserialise() { /* driver "json", request, list, title, url */ }
```

- [ ] **Step 2: Lancer les tests pour vérifier l'échec**

Run: `cargo test --locked game_search::tests -- --nocapture`
Expected: FAIL, `SiteDriver` n'a pas encore de désérialisation complète.

- [ ] **Step 3: Remplacer `SearchSite` par `SiteDriver`**

Supprimer l'enum `SearchSite`, `ALL`, `label`, `host`, `search_url`. Conserver les fonctions `search_anker`, `search_astral`, `search_gamebounty`, `search_kryoto`, `search_onlinefix`, `search_zeigames`, `search_scene` et leurs parseurs.

Implémenter :

```rust
pub fn search(site: &SiteDefinition, query: &str) -> Result<Vec<SiteHit>, String> {
    let query = query.trim();
    if query.len() < 2 {
        return Ok(Vec::new());
    }
    let hits = match &site.driver {
        SiteDriver::GameBounty => search_gamebounty(query)?,
        SiteDriver::AnkerGames => search_anker(query)?,
        SiteDriver::AstralGames => search_astral(query)?,
        SiteDriver::Kryoto => search_kryoto(query)?,
        SiteDriver::OnlineFix => search_onlinefix(query)?,
        SiteDriver::ZeiGames => search_zeigames(query)?,
        SiteDriver::SceneCat => search_scene(query)?,
        SiteDriver::Json(_) => return Err("driver json indisponible".to_string()),
        SiteDriver::Html(_) => return Err("driver html indisponible".to_string()),
    };
    Ok(filter_relevant(hits, query)
        .into_iter()
        .take(site_limit(&site.driver))
        .collect())
}
```

`site_limit` retourne `driver.limit` pour JSON/HTML et 8 pour les drivers intégrés.

- [ ] **Step 4: Mettre à jour les tests existants**

Conserver les tests de parseurs. Remplacer les deux tests de repli par un test qui construit les sept `SiteDefinition` et vérifie `search_url("...")` côté interface, pas côté Rust.

Retirer de `quark-tauri/src/features.rs` le test `les_resultats_de_recherche_suivent_l_ordre_des_sources`, qui référence `SearchSite::ALL`.

- [ ] **Step 5: Lancer les tests**

Run: `cargo test --locked game_search::tests -- --nocapture`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add src/game_search.rs quark-tauri/src/features.rs
git commit -m "transforme les sources de recherche en drivers"
```

---

### Task 3: Driver JSON générique

**Files:**
- Modify: `src/game_search.rs`
- Test: `src/game_search.rs` (module `tests`)

**Interfaces:**
- Consumes: `JsonDriver`, `RequestSpec`, `SiteHit`.
- Produces:
  - `game_search::search_json(driver: &JsonDriver, query: &str) -> Result<Vec<SiteHit>, String>`
  - `game_search::parse_json(driver: &JsonDriver, body: &str) -> Result<Vec<SiteHit>, String>`

- [ ] **Step 1: Écrire les tests du driver JSON**

```rust
#[test]
fn le_driver_json_lit_une_liste_et_construit_les_liens() {
    let driver = json_driver_example();
    let body = r#"{"data":{"results":[
      {"title":"Portal 2","slug":"portal-2","size":"8 GB","version":"1.0",
       "cover":"https://cdn/portal.jpg"}
    ]}}"#;
    let hits = parse_json(&driver, body).unwrap();
    assert_eq!(hits[0].title, "Portal 2");
    assert_eq!(hits[0].url, "https://example.com/game/portal-2");
    assert_eq!(hits[0].detail, "8 GB · 1.0");
    assert_eq!(hits[0].cover.as_deref(), Some("https://cdn/portal.jpg"));
}

#[test]
fn le_driver_json_ecarte_les_resultats_hors_sujet() { /* Portal 2 vs POSTAL 2 */ }

#[test]
fn le_driver_json_refuse_une_liste_absente() { /* {} => Err */ }

#[test]
fn le_driver_json_respecte_la_limite() { /* 3 items, limit 2 */ }
```

- [ ] **Step 2: Lancer les tests pour vérifier l'échec**

Run: `cargo test --locked game_search::tests::le_driver_json -- --nocapture`
Expected: FAIL, `parse_json` n'existe pas.

- [ ] **Step 3: Implémenter `search_json` et `parse_json`**

`search_json` construit l'URL avec `{query}` remplacé, envoie la requête (méthode, en-têtes autorisés, corps, encodage), puis appelle `parse_json`. `parse_json` résout `driver.list` par chemin à points, itère, résout `title`, `url` (modèle), `detail` (modèles joints par ` · `), `cover` (premier chemin non vide), et ignore les entrées sans titre ou sans URL HTTP(S).

Ajouter `dot_path(value: &Value, path: &str) -> Option<&Value>` et `render_template(template: &str, item: &Value) -> String`.

- [ ] **Step 4: Lancer les tests**

Run: `cargo test --locked game_search::tests::le_driver_json -- --nocapture`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/game_search.rs
git commit -m "ajoute le driver json generique"
```

---

### Task 4: Driver HTML générique

**Files:**
- Modify: `src/game_search.rs`
- Test: `src/game_search.rs` (module `tests`)

**Interfaces:**
- Consumes: `HtmlDriver`, `FieldSpec`, `scraper::{Html, Selector}`.
- Produces:
  - `game_search::search_html(driver: &HtmlDriver, query: &str) -> Result<Vec<SiteHit>, String>`
  - `game_search::parse_html(driver: &HtmlDriver, body: &str, base_url: &str) -> Result<Vec<SiteHit>, String>`
  - `game_search::validate_site(site: &SiteDefinition) -> Result<(), String>` vérifie aussi les sélecteurs HTML.

- [ ] **Step 1: Écrire les tests du driver HTML**

```rust
#[test]
fn le_driver_html_lit_les_selecteurs_et_les_attributs() {
    let driver = html_driver_example();
    let body = r#"<div class="result">
      <h2>Portal 2</h2><a href="/game/portal-2">lien</a>
      <span class="size">8 GB</span><img data-src="/cover.jpg">
    </div>"#;
    let hits = parse_html(&driver, body, "https://example.com/search").unwrap();
    assert_eq!(hits[0].title, "Portal 2");
    assert_eq!(hits[0].url, "https://example.com/game/portal-2");
    assert_eq!(hits[0].detail, "8 GB");
    assert_eq!(hits[0].cover.as_deref(), Some("https://example.com/cover.jpg"));
}

#[test]
fn le_driver_html_lit_un_modele_d_url() { /* attr id + stripPrefix + template */ }

#[test]
fn le_driver_html_refuse_un_selecteur_invalide() { /* items "div[" => Err */ }

#[test]
fn le_driver_html_respecte_la_limite() { /* 3 items, limit 2 */ }
```

- [ ] **Step 2: Lancer les tests pour vérifier l'échec**

Run: `cargo test --locked game_search::tests::le_driver_html -- --nocapture`
Expected: FAIL, `parse_html` n'existe pas.

- [ ] **Step 3: Implémenter `search_html` et `parse_html`**

`parse_html` parse le document avec `scraper::Html::parse_document`, compile `driver.items`, itère les éléments, extrait chaque `FieldSpec` (`selector`, `attr`, texte sinon, `stripPrefix`, `stripSuffix`, `template`), résout les URL relatives avec `Url::join`, joint les détails, et limite. `search_html` fait la requête HTTP puis appelle `parse_html` avec l'URL finale.

- [ ] **Step 4: Lancer les tests**

Run: `cargo test --locked game_search::tests::le_driver_html -- --nocapture`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml Cargo.lock src/game_search.rs
git commit -m "ajoute le driver html generique"
```

---

### Task 5: Cache, statuts et sites actifs

**Files:**
- Modify: `src/extensions.rs`
- Test: `src/extensions.rs` (module `tests`)

**Interfaces:**
- Consumes: `ExtensionManifest`, `SiteDefinition`, `game_search::validate_site`.
- Produces:
  - `extensions::ExtensionCache { version, entries }`
  - `extensions::CachedExtension { url, fetched_at, manifest }`
  - `extensions::ExtensionStatus { url, id, name, version, site_count, fetched_at, stale, error }`
  - `extensions::ExtensionSite { uid, extension_id, extension_name, site }`
  - `extensions::load_cache_from(path: &Path) -> ExtensionCache`
  - `extensions::save_cache_to(path: &Path, cache: &ExtensionCache) -> Result<(), String>`
  - `extensions::add_manifest(cache: &mut ExtensionCache, sources: &mut Vec<String>, url: &str, manifest: ExtensionManifest) -> Result<ExtensionStatus, String>`
  - `extensions::remove_source(cache: &mut ExtensionCache, sources: &mut Vec<String>, url: &str) -> bool`
  - `extensions::statuses(cache, sources) -> Vec<ExtensionStatus>`
  - `extensions::active_sites(cache, sources) -> Vec<ExtensionSite>`

- [ ] **Step 1: Écrire les tests du cache**

```rust
#[test]
fn le_cache_se_charge_et_se_sauvegarde() { /* temp dir, add_manifest, save, load */ }

#[test]
fn un_cache_corrompu_est_ignore() { /* write "{" then load => default */ }

#[test]
fn les_sources_actives_sont_filtrees() { /* cache 2 entries, sources 1 => 1 site */ }

#[test]
fn deux_extensions_ne_peuvent_pas_partager_le_meme_id() { /* add_manifest twice => Err */ }

#[test]
fn le_statut_indique_une_extension_perimee() { /* fetched_at ancien => stale */ }

#[test]
fn remove_source_retire_l_entree_et_la_source() { /* => true, cache vide */ }
```

- [ ] **Step 2: Lancer les tests pour vérifier l'échec**

Run: `cargo test --locked extensions::tests -- --nocapture`
Expected: FAIL, cache et statuts n'existent pas.

- [ ] **Step 3: Implémenter le cache**

`load_cache_from` lit le fichier, parse `ExtensionCache`, ignore un fichier absent ou invalide. `save_cache_to` écrit dans un fichier temporaire puis renomme. `add_manifest` valide, refuse un `id` déjà présent dans une autre source, remplace l'entrée de même URL, et retourne le statut. `active_sites` parcourt les sources dans l'ordre, ignore les entrées absentes, et construit `uid = format!("{}/{}", manifest.id, site.id)`.

`stale` vaut `now - fetched_at > 7 * 24 * 3600`.

- [ ] **Step 4: Implémenter le téléchargement**

`fetch_manifest(url: &str) -> Result<ExtensionManifest, String>` vérifie l'URL HTTPS publique, envoie une requête GET avec `Accept: application/json`, limite à `MAX_MANIFEST_BYTES`, vérifie l'URL finale, puis `parse_manifest`.

`add_source(cache, sources, url)` appelle `fetch_manifest` puis `add_manifest`. `refresh_source` remplace l'entrée ; en cas d'échec, l'ancienne entrée reste.

- [ ] **Step 5: Lancer les tests**

Run: `cargo test --locked extensions::tests -- --nocapture`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add src/extensions.rs
git commit -m "ajoute le cache et les statuts des extensions"
```

---

### Task 6: Réglages, commandes Tauri et bootstrap

**Files:**
- Modify: `src/config.rs`
- Modify: `quark-tauri/src/features.rs`
- Modify: `quark-tauri/src/main.rs`
- Modify: `quark-tauri/build.rs`
- Modify: `quark-tauri/capabilities/default.json`
- Test: `src/config.rs`, `quark-tauri/src/features.rs`

**Interfaces:**
- Consumes: `extensions::{load_cache_from, save_cache_to, add_source, remove_source, refresh_source, statuses, active_sites}`.
- Produces:
  - `Settings::extension_sources: Vec<String>`
  - `Bootstrap { extensions, search_sites, .. }`
  - commandes `extensions_add`, `extensions_remove`, `extensions_refresh`, `site_search`.

- [ ] **Step 1: Écrire les tests de réglages**

```rust
#[test]
fn les_reglages_par_defaut_n_ont_aucune_extension() {
    assert!(Settings::default().extension_sources.is_empty());
}
```

- [ ] **Step 2: Ajouter le champ**

Dans `Settings`, ajouter `pub extension_sources: Vec<String>` avec un commentaire indiquant que la liste est vide par défaut. Dans `Default`, initialiser à `Vec::new()`.

- [ ] **Step 3: Ajouter les DTO et le bootstrap**

Dans `quark-tauri/src/main.rs`, ajouter `extensions: Vec<ExtensionStatusDto>` et `search_sites: Vec<SearchSiteDto>` à `Bootstrap`. Dans `bootstrap`, charger le cache avec `extensions::load_cache_from(&quark::config::data_dir().join("extensions.json"))`, puis `statuses` et `active_sites` avec `settings.extension_sources`.

Définir `ExtensionStatusDto` et `SearchSiteDto` avec `#[serde(rename_all = "camelCase")]`.

- [ ] **Step 4: Écrire les commandes**

Dans `features.rs`, ajouter :

```rust
#[tauri::command]
pub async fn extensions_add(state: State<'_, AppState>, url: String) -> Result<Vec<ExtensionStatusDto>, String>;

#[tauri::command]
pub fn extensions_remove(state: State<'_, AppState>, url: String) -> Result<Vec<ExtensionStatusDto>, String>;

#[tauri::command]
pub async fn extensions_refresh(state: State<'_, AppState>, url: Option<String>) -> Result<Vec<ExtensionStatusDto>, String>;
```

Chaque commande extrait `extension_sources` et le chemin du cache avant `blocking`, puis persiste les réglages après le travail réseau. `extensions_refresh` actualise une URL ou toutes, conserve les anciennes entrées en cas d'échec, et renvoie les statuts avec `error`.

Modifier `site_search` pour prendre `State<'_, AppState>`, charger le cache, appeler `active_sites`, lancer les recherches en parallèle avec `std::thread::scope`, et utiliser `site.uid` comme `site_id`.

- [ ] **Step 5: Déclarer les permissions**

Ajouter `extensions_add`, `extensions_remove`, `extensions_refresh` à la liste de `quark-tauri/build.rs` et les permissions `allow-extensions-add`, `allow-extensions-remove`, `allow-extensions-refresh` à `capabilities/default.json`.

- [ ] **Step 6: Lancer les tests Tauri**

Run: `cargo test --locked --manifest-path quark-tauri/Cargo.toml`
Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add src/config.rs quark-tauri/src/features.rs quark-tauri/src/main.rs quark-tauri/build.rs quark-tauri/capabilities/default.json
git commit -m "expose les extensions dans les reglages et l'api tauri"
```

---

### Task 7: Interface Réglages > Extensions et gating

**Files:**
- Create: `quark-tauri/ui/extensions.js`
- Create: `ui-tests/extensions.test.mjs`
- Modify: `quark-tauri/ui/index.html`
- Modify: `quark-tauri/ui/settings.js`
- Modify: `quark-tauri/ui/dialogs.js`
- Modify: `quark-tauri/ui/library.js`
- Modify: `quark-tauri/ui/store.js`
- Modify: `quark-tauri/ui/app.js`
- Modify: `quark-tauri/ui/i18n-extra.js`
- Modify: `quark-tauri/ui/i18n-ru.js`
- Modify: `quark-tauri/ui/styles-modern.css`
- Test: `ui-tests/extensions.test.mjs`, `ui-tests/escape.test.mjs`

**Interfaces:**
- Consumes: `app.data.extensions`, `app.data.searchSites`, commandes `extensions_add`, `extensions_remove`, `extensions_refresh`.
- Produces:
  - `extensionSites(data) -> Array`
  - `extensionSearchUrl(site, query) -> string`
  - onglet `extensions` dans `STABS`.

- [ ] **Step 1: Écrire les tests JavaScript**

Dans `ui-tests/extensions.test.mjs` :

```js
test("liste vide sans extension", () => {
  assert.deepEqual(extensionSites({}), []);
  assert.deepEqual(extensionSites({ searchSites: [] }), []);
});

test("construit l'URL de recherche avec encodage", () => {
  const site = { searchUrl: "https://example.com/search?q={query}" };
  assert.equal(extensionSearchUrl(site, "Portal 2 & co"),
    "https://example.com/search?q=Portal%202%20%26%20co");
});
```

Dans `ui-tests/escape.test.mjs`, ajouter :

```js
test("échappe le nom d'une extension", () => {
  assert.equal(html`<b>${"<script>alert(1)</script>"}</b>`,
    "<b>&lt;script&gt;alert(1)&lt;/script&gt;</b>");
});
```

- [ ] **Step 2: Lancer les tests pour vérifier l'échec**

Run: `node --test ui-tests/extensions.test.mjs ui-tests/escape.test.mjs`
Expected: FAIL, `extensions.js` n'existe pas.

- [ ] **Step 3: Créer `extensions.js`**

Définir `extensionSites(data)` et `extensionSearchUrl(site, query)` avec un export CommonJS conditionnel pour les tests. Charger le fichier dans `index.html` avant `dialogs.js`.

- [ ] **Step 4: Remplacer `SITE_META` dans `dialogs.js`**

Supprimer la constante `SITE_META`. Utiliser `extensionSites(app.data)` dans `renderWebResults`, `drawChips`, et pour les erreurs. `site.id` devient `site.uid` ; `site.search(q)` devient `extensionSearchUrl(site, q)`.

Dans `webSearchModal`, si la liste est vide, afficher un message `web.disabled` et un bouton vers les réglages, sans appeler `site_search`.

- [ ] **Step 5: Masquer les points d'entrée**

- `dialogs.js` : ne pas afficher le bouton `web-open` si `extensionSites(app.data).length === 0`.
- `library.js` : n'ajouter l'icône `sites` que si des sites existent.
- `store.js` : n'ajouter `data-sites` que si des sites existent.
- `app.js` : ne pas ajouter l'action de recherche à la palette si aucun site.

- [ ] **Step 6: Ajouter l'onglet Réglages > Extensions**

Dans `settings.js`, ajouter `["extensions", "Extensions", "Extensions"]` à `STABS` et `extensions: () => [extensionsSettings()]` à `tabsDef`. `extensionsSettings()` rend :

- un champ `#ext-url` et un bouton `#ext-add` ;
- la liste `app.data.extensions` avec nom, version, nombre de sites, URL, date ;
- les boutons `data-ext-refresh` et `data-ext-remove` ;
- l'état vide et le lien vers `docs/extensions/README.md`.

Dans `bindSettings`, lier `#ext-add` à `invoke("extensions_add", { url })`, puis recharger `bootstrap_data`, `app.data`, `renderSheetKeepScroll()`. Lier les boutons de chaque carte à `extensions_refresh` et `extensions_remove`.

- [ ] **Step 7: Ajouter les libellés**

Ajouter les clés `settings.extensions`, `ext.title`, `ext.hint`, `ext.urlPlaceholder`, `ext.add`, `ext.empty`, `ext.refresh`, `ext.remove`, `ext.sites`, `ext.updated`, `ext.error`, `ext.disabled`, `ext.template`, `web.disabled` dans `i18n-extra.js` pour fr/en/es/de/pt et dans `i18n-ru.js` pour ru.

- [ ] **Step 8: Ajouter le style**

Dans `styles-modern.css`, ajouter `.extcard`, `.exthead`, `.exturl`, `.extmeta`, `.extempty`, `.extactions`, `.extsource`, en réutilisant `--panel`, `--surface`, `--g`, `--accent`, `--danger`.

- [ ] **Step 9: Lancer les tests JavaScript**

Run: `node --test ui-tests/extensions.test.mjs ui-tests/escape.test.mjs`
Expected: PASS.

- [ ] **Step 10: Commit**

```bash
git add quark-tauri/ui ui-tests
git commit -m "ajoute l'onglet extensions et masque la recherche par defaut"
```

---

### Task 8: Modèle externe, documentation et version

**Files:**
- Create: `docs/extensions/README.md`
- Create: `docs/extensions/quark-sites.json`
- Modify: `README.md`
- Modify: `docs/release-notes.md`
- Modify: `Cargo.toml`
- Modify: `quark-tauri/Cargo.toml`
- Modify: `Cargo.lock`
- Test: `src/extensions.rs` (test du modèle)

**Interfaces:**
- Consumes: format de manifeste des tâches 1 à 5.
- Produces: manifeste modèle des sept sites, documentation d'installation.

- [ ] **Step 1: Écrire le test du manifeste modèle**

Dans `src/extensions.rs` :

```rust
#[test]
fn le_manifeste_modele_est_valide() {
    let bytes = include_str!("../docs/extensions/quark-sites.json").as_bytes();
    let manifest = parse_manifest(bytes).expect("manifeste modele");
    assert_eq!(manifest.sites.len(), 7);
}
```

- [ ] **Step 2: Lancer le test pour vérifier l'échec**

Run: `cargo test --locked extensions::tests::le_manifeste_modele_est_valide -- --nocapture`
Expected: FAIL, fichier absent.

- [ ] **Step 3: Créer `docs/extensions/quark-sites.json`**

Copier le manifeste du design avec les sept sites et leurs drivers intégrés. Vérifier que chaque `searchUrl` contient `{query}`.

- [ ] **Step 4: Écrire `docs/extensions/README.md`**

Documenter : hébergement sur GitHub raw, ajout dans Quark, schéma, drivers intégrés, exemples JSON et HTML, sécurité, limites, et dépannage. Indiquer que le fichier `quark-sites.json` peut être copié tel quel dans le dépôt externe.

- [ ] **Step 5: Mettre à jour le README et les notes de version**

Ajouter une section « Recherche par extensions » dans `README.md` et une entrée 2.3.0 dans `docs/release-notes.md`.

- [ ] **Step 6: Passer les versions à 2.3.0**

Modifier `Cargo.toml` et `quark-tauri/Cargo.toml`. Lancer `cargo check` puis `cargo check --manifest-path quark-tauri/Cargo.toml` pour mettre `Cargo.lock` à jour, puis revérifier avec `--locked`.

- [ ] **Step 7: Lancer le test du modèle**

Run: `cargo test --locked extensions::tests::le_manifeste_modele_est_valide -- --nocapture`
Expected: PASS.

- [ ] **Step 8: Commit**

```bash
git add docs/extensions README.md docs/release-notes.md Cargo.toml Cargo.lock quark-tauri/Cargo.toml src/extensions.rs
git commit -m "documente le manifeste externe et passe en 2.3.0"
```

---

### Task 9: Vérification complète

**Files:**
- Modify: tout fichier nécessaire aux corrections.

- [ ] **Step 1: Formater**

Run: `cargo fmt --all`
Expected: aucune erreur.

- [ ] **Step 2: Vérifier le format**

Run: `cargo fmt --all --check`
Expected: aucune différence.

- [ ] **Step 3: Clippy**

Run: `cargo clippy --all-targets --locked -- -D warnings`
Expected: aucune erreur.

- [ ] **Step 4: Tests du cœur**

Run: `cargo test --locked`
Expected: tous les tests passent.

- [ ] **Step 5: Tests Tauri**

Run: `cargo test --locked --manifest-path quark-tauri/Cargo.toml`
Expected: tous les tests passent.

- [ ] **Step 6: Tests JavaScript**

Run: `node --test ui-tests/*.test.mjs`
Expected: tous les tests passent.

- [ ] **Step 7: Licences et avis**

Run: `cargo deny check`
Expected: `advisories ok, bans ok, licenses ok, sources ok`.

- [ ] **Step 8: Vérification manuelle**

Lancer `cargo run --manifest-path quark-tauri/Cargo.toml -- --demo-empty` et vérifier :

- aucun bouton de recherche dans l'ajout d'un jeu, la fiche, la boutique ou la palette ;
- l'onglet Réglages > Extensions affiche l'état vide ;
- l'ajout d'une URL invalide affiche une erreur ;
- l'ajout d'une URL HTTPS valide affiche l'extension et active les boutons de recherche ;
- le retrait de l'extension masque à nouveau les boutons.

- [ ] **Step 9: Commit final**

```bash
git add -A
git commit -m "verifie la recherche par extensions"
```
