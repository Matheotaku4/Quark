# « Mes jeux » rapide et fiable — plan d’implémentation

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Rendre « Mes jeux » fluide, corriger les jaquettes manquantes et moderniser l’interface sans ajouter de framework front.

**Architecture:** On conserve Tauri 2 + HTML/CSS/JS. Le backend construit le DTO des boutiques dans `spawn_blocking`, avec un contexte précalculé (outils + cache de couvertures). Une commande dédiée télécharge les couvertures visibles par lots de 4. Le front garde une barre d’outils stable et ne repeint que la grille, avec pagination par 24 et sans `content-visibility`.

**Tech Stack:** Rust (Tauri 2), HTML/CSS/JS sans build, `quark` core crate.

**Spec:** `docs/superpowers/specs/2026-11-21-interface-2.1-design.md`

## Global Constraints

- Tauri 2 ; `quark` edition 2024 ; `quark-tauri` edition 2021.
- Aucun framework front (pas de React/Svelte/Vue).
- Six langues : fr, en, es, de, pt, ru.
- `my_games_list == true` signifie liste compacte ; `false` signifie grille.
- Aucun téléchargement de fichier depuis les sites de recherche.
- Pas de `content-visibility` sur les cartes ; pas de `backdrop-filter` sur les grandes surfaces.
- `main` ne doit pas être modifiée ; travail sur `feat/interface-2.1.0`.
- `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`, `cargo test --locked` pour le crate racine ; les mêmes commandes avec `--manifest-path quark-tauri/Cargo.toml` doivent aussi passer.
- Le mode `--demo` et `--demo-empty` doit rester fonctionnel.

## Review Focus

- Bibliothèque de 10 000 jeux : l’ouverture de « Mes jeux » ne fait pas de `stat()` par jeu et reste réactive.
- URL de jaquette absente, 403 ou lente : l’interface affiche le monogramme puis remplace par l’image locale, sans zone vide.
- Jeu Epic/GOG sans `cover_url` : la couverture est résolue par `covers::ensure` ou le monogramme reste.
- Filtres combinés (boutique + état + recherche) : aucun résultat inattendu, compteur cohérent.
- WebKitGTK/Wayland : aucune carte ne disparaît au survol ou au défilement.

---

## Préparation

- [ ] Créer la branche et mémoriser `main` :

```bash
git switch -c feat/interface-2.1.0
git rev-parse main > /tmp/quark-main.sha
```

- [ ] Vérifier que le dépôt est propre :

```bash
git status --short
```

Expected: aucun fichier suivi modifié ; les documents de spec/plan sont les seuls fichiers non suivis.

---

### Task 1: DTO des boutiques asynchrone et précalculé

**Files:**
- Modify: `quark-tauri/src/providers.rs`
- Test: `quark-tauri/src/providers.rs` (module `tests` en bas)

**Interfaces:**
- Consumes: `quark::providers::model::{ProviderGame, ProviderGameKey}`, `quark::providers::covers`.
- Produces:
  - `struct ProviderContext { credentials: credentials::Credentials, legendary_ready: bool, gogdl_ready: bool, cached_covers: HashSet<PathBuf> }`
  - `fn cached_cover_paths(dir: &Path) -> HashSet<PathBuf>`
  - `fn provider_context() -> ProviderContext`
  - `fn provider_game_dto(record: &ProviderGame, ctx: &ProviderContext) -> ProviderGameDto`
  - `fn providers_dto_from(games: Vec<ProviderGame>, error: Option<String>) -> ProvidersDto`
  - `#[tauri::command] pub async fn providers_data(state: State<'_, AppState>) -> Result<ProvidersDto, String>`
  - `ProviderGameDto` gagne `cover_cached: bool`.

- [ ] **Step 1: Écrire le test du précalcul du cache**

Ajouter dans `quark-tauri/src/providers.rs` :

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cached_cover_paths_ne_garde_que_les_png_du_dossier() {
        let dir = std::env::temp_dir().join(format!(
            "quark-provider-covers-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("steam-1.png"), b"png").unwrap();
        std::fs::write(dir.join("steam-2.jpg"), b"jpg").unwrap();
        std::fs::write(dir.join("notes.txt"), b"txt").unwrap();

        let paths = cached_cover_paths(&dir);
        assert_eq!(paths.len(), 1);
        assert!(paths.contains(&dir.join("steam-1.png")));

        let _ = std::fs::remove_dir_all(&dir);
    }
}
```

- [ ] **Step 2: Lancer le test et vérifier l’échec**

Run: `cargo test --manifest-path quark-tauri/Cargo.toml cached_cover_paths_ne_garde_que_les_png_du_dossier -v`
Expected: FAIL car `cached_cover_paths` n’existe pas.

- [ ] **Step 3: Implémenter le contexte et le DTO asynchrone**

Dans `quark-tauri/src/providers.rs` :

```rust
struct ProviderContext {
    credentials: credentials::Credentials,
    legendary_ready: bool,
    gogdl_ready: bool,
    cached_covers: HashSet<PathBuf>,
}

fn cached_cover_paths(dir: &Path) -> HashSet<PathBuf> {
    std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "png"))
        .collect()
}

fn provider_context() -> ProviderContext {
    ProviderContext {
        credentials: credentials::load(&config::credentials_path()).unwrap_or_default(),
        legendary_ready: tools::ManagedToolId::Legendary.installed_path().is_file(),
        gogdl_ready: tools::ManagedToolId::Gogdl.installed_path().is_file(),
        cached_covers: cached_cover_paths(&providers::covers::cache_dir()),
    }
}
```

- `ProviderContext` fournit `fn cover(&self, key: &ProviderGameKey) -> Option<PathBuf>` qui teste `self.cached_covers.contains(&providers::covers::cache_path(key))`.
- `provider_game_dto` prend `ctx` et calcule `let cached = ctx.cover(&record.key);` une seule fois.
- `cover_url` vaut `cached.and_then(|path| super::asset_url(Some(&path))).or_else(|| record.cover_url.clone())`.
- `cover_cached` vaut `cached.is_some()`.
- `can_install` utilise `ctx.legendary_ready` / `ctx.gogdl_ready` au lieu d’appels `is_file()` par jeu.
- `accounts` et `tools_dto` utilisent les booléens du contexte.
- `providers_dto_from(games, error)` construit un `ProviderContext` une fois, mappe les jeux, trie par titre minuscule, retourne le DTO.
- `providers_data` devient `async`, clone `library.provider_library.games` sous verrou, puis `tauri::async_runtime::spawn_blocking(move || providers_dto_from(games, None)).await`.
- `provider_sync` et `provider_disconnect` réutilisent `providers_dto_from` après persist ; si la commande est synchrone, elle peut appeler `providers_dto_from` directement, sinon passer par `spawn_blocking`.
- Supprimer l’ancien `providers_dto(&AppState, ...)` ou le remplacer par un wrapper qui clone les jeux puis appelle `providers_dto_from`.

- [ ] **Step 4: Lancer le test et la compilation**

Run: `cargo test --manifest-path quark-tauri/Cargo.toml cached_cover_paths_ne_garde_que_les_png_du_dossier -v`
Expected: PASS.
Run: `cargo check --manifest-path quark-tauri/Cargo.toml`
Expected: OK.

- [ ] **Step 5: Commit**

```bash
git add quark-tauri/src/providers.rs
git commit -m "perf(providers): build store DTO off the UI thread"
```

---

### Task 2: Commande de téléchargement des couvertures visibles

**Files:**
- Modify: `quark-tauri/src/providers.rs`
- Modify: `quark-tauri/src/main.rs` (enregistrement de la commande)
- Test: `quark-tauri/src/providers.rs`

**Interfaces:**
- Consumes: `covers::ensure(&ProviderGame) -> Result<(PathBuf, Option<String>), String>`, `ProviderContext` de la tâche 1.
- Produces:
  - `#[derive(Deserialize)] #[serde(rename_all = "camelCase")] pub struct ProviderKeyDto { pub provider: String, pub product_id: String }`
  - `#[derive(Serialize)] #[serde(rename_all = "camelCase")] pub struct ProviderCoverDto { pub provider: String, pub product_id: String, pub cover_url: Option<String>, pub error: Option<String> }`
  - `fn normalize_cover_keys(keys: Vec<ProviderKeyDto>) -> Result<Vec<ProviderGameKey>, String>`
  - `#[tauri::command] pub async fn provider_ensure_covers(state: State<'_, AppState>, keys: Vec<ProviderKeyDto>) -> Result<Vec<ProviderCoverDto>, String>`

- [ ] **Step 1: Écrire le test de normalisation**

```rust
#[test]
fn les_cles_de_couverture_sont_dedupliquees_et_bornees() {
    let keys = (0..40)
        .map(|index| ProviderKeyDto {
            provider: "steam".to_string(),
            product_id: index.to_string(),
        })
        .collect();
    let normalized = normalize_cover_keys(keys).unwrap();
    assert_eq!(normalized.len(), 32);
    assert_eq!(normalized[0].provider, Provider::Steam);

    let duplicate = vec![
        ProviderKeyDto { provider: "steam".into(), product_id: "1".into() },
        ProviderKeyDto { provider: "steam".into(), product_id: "1".into() },
    ];
    assert_eq!(normalize_cover_keys(duplicate).unwrap().len(), 1);
}

#[test]
fn une_boutique_inconnue_est_refusee() {
    let keys = vec![ProviderKeyDto {
        provider: "ubisoft".into(),
        product_id: "1".into(),
    }];
    assert!(normalize_cover_keys(keys).is_err());
}
```

- [ ] **Step 2: Lancer les tests et vérifier l’échec**

Run: `cargo test --manifest-path quark-tauri/Cargo.toml les_cles_de_couverture -v`
Expected: FAIL car les types/fonctions n’existent pas.

- [ ] **Step 3: Implémenter la commande**

- `normalize_cover_keys` convertit `provider` via `serde_json::from_value::<Provider>`, refuse une chaîne vide, déduplique avec `HashSet<ProviderGameKey>` et tronque à 32.
- `provider_ensure_covers` clone les enregistrements correspondants depuis `state.library`, puis exécute `covers::ensure` par paquets de 4 dans `spawn_blocking`.
- Chaque résultat produit un `ProviderCoverDto` : `cover_url` = `super::asset_url(Some(&path))` en cas de succès, sinon `None` ; `error` = message court en cas d’échec.
- Après les téléchargements, reprendre le verrou, mettre à jour `record.cover_url` quand `ensure` a retourné une URL découverte, et `persist` une seule fois si quelque chose a changé.
- Un enregistrement introuvable produit `error: Some("jeu de boutique introuvable")`, sans faire échouer les autres.
- Enregistrer la commande dans `quark-tauri/src/main.rs` à côté de `providers_data`.

- [ ] **Step 4: Lancer les tests et vérifier la compilation**

Run: `cargo test --manifest-path quark-tauri/Cargo.toml les_cles_de_couverture -v`
Expected: PASS.
Run: `cargo check --manifest-path quark-tauri/Cargo.toml`
Expected: OK.

- [ ] **Step 5: Commit**

```bash
git add quark-tauri/src/providers.rs quark-tauri/src/main.rs
git commit -m "feat(providers): cache visible store covers in batches"
```

---

### Task 3: État, filtres, tri et barre d’outils de « Mes jeux »

**Files:**
- Modify: `quark-tauri/ui/providers.js`
- Modify: `quark-tauri/ui/i18n-extra.js` (libellés ajoutés ici, traductions complétées tâche 6)
- Test: vérification visuelle sous Xvfb

**Interfaces:**
- Consumes: `app.pv.data` (`ProvidersDto` avec `games`, `accounts`, `tools`), `app.cfg.my_games_list`, `cfgSet`.
- Produces:
  - `const PV_PAGE = 24;`
  - `const PV_FILTERS_KEY = "quark.mygames.filters.v1";`
  - `function pvLayout() -> "list" | "grid"`
  - `function pvLoadPrefs() -> void`
  - `function pvSavePrefs() -> void`
  - `function pvGames() -> ProviderGameDto[]`
  - `function pvMatches(game, normalizedQuery) -> boolean`
  - `function pvSort(a, b) -> number`
  - `function renderMyGames() -> string`
  - `function renderMyGamesResults() -> string`
  - `function bindMyGames() -> void`
  - `function repaintMyGamesResults() -> void`

- [ ] **Step 1: Remplacer l’état et les filtres**

Dans `providers.js` :

```js
const PV_PAGE = 24;
const PV_FILTERS_KEY = "quark.mygames.filters.v1";
Object.assign(app, {
  pv: {
    data: null,
    loading: false,
    error: null,
    q: "",
    providers: new Set(),
    state: "all",
    sort: "title",
    shown: PV_PAGE,
    coverTried: new Set(),
    coverFailures: new Map(),
    coverQueue: new Set(),
    imgObs: null,
  },
});
```

- `pvLoadPrefs` lit `{providers:[], state, sort}` depuis `localStorage`, valide les valeurs (`steam|epic|gog|itch`, `all|installed|missing`, `title|installed|synced`) et remplit `app.pv`.
- `pvSavePrefs` écrit `{providers:[...app.pv.providers], state, sort}`.
- `pvLayout` retourne `app.cfg.my_games_list ? "list" : "grid"`.
- `pvGames` filtre sur `providers` (vide = toutes), `state` (`installed` = `installState === "installed"`, `missing` = inverse), et `q` normalisé sur `[title, PV_LABEL[provider], installPath]`.
- `pvSort` : `installed` met les installés d’abord puis trie par titre ; `synced` trie par `lastSynced` décroissant puis titre ; `title` trie par titre avec `locale()`.
- `renderMyGamesResults` rend pour l’instant les `games.slice(0, shown)` avec le `providerCard` existant ; `repaintMyGamesResults` remplace uniquement `#pv-results` et `#pv-count`.
- `providersLoad` appelle `pvLoadPrefs()` au premier chargement réussi, avant de repeindre les résultats.

- [ ] **Step 2: Construire la barre d’outils**

`renderMyGames` produit :

```html
<div class="mygames">
  <div class="pvhero">… Comptes / Tout synchroniser / Importer de Steam …</div>
  <div class="pvtoolbar">
    <input id="pv-q" type="search" …>
    <div class="pchips" id="pv-providers">… Toutes / Steam / Epic / GOG / itch.io …</div>
    <select id="pv-state">…</select>
    <select id="pv-sort">…</select>
    <button id="pv-layout" class="tb">…</button>
    <button class="tb" data-pv="reload">…</button>
  </div>
  <div class="pvcount" id="pv-count"></div>
  <div id="pv-results"></div>
</div>
```

- Les puces utilisent `data-pvprov="steam|epic|gog|itch|"` ; la puce vide « Toutes » vide `app.pv.providers`.
- `app.pv.providers` est un `Set` ; une puce est active si `providers.size === 0` pour « Toutes », sinon si le provider est dans le Set.
- `bindMyGames` ne lie que la barre d’outils ; le champ de recherche est débouncé à 120 ms et appelle `repaintMyGamesResults()`.
- Le changement de layout appelle `cfgSet({ my_games_list: next === "list" })`, puis `repaintMyGamesResults()`.
- `providersLoad` met à jour `app.pv.data` et appelle `repaintMyGamesResults()` si la vue est `mygames`, sans `renderSheet()` complet.

- [ ] **Step 3: Vérifier visuellement**

Run:
```bash
xvfb-run -a -s "-screen 0 1360x860x24" bash -c '
  WEBKIT_DISABLE_DMABUF_RENDERER=1 cargo run --manifest-path quark-tauri/Cargo.toml -- --demo &
  APP=$!; sleep 12
  xdotool mousemove 330 24 click 1; sleep 2
  import -window root /tmp/quark-mygames-toolbar.png
  kill $APP
'
```
Expected: la page « Mes jeux » montre la barre d’outils, les puces multi-boutiques, le tri et le bouton liste/grille ; aucune erreur console visible.

- [ ] **Step 4: Commit**

```bash
git add quark-tauri/ui/providers.js quark-tauri/ui/i18n-extra.js
git commit -m "feat(mygames): add persistent filters, sorting and layout state"
```

---

### Task 4: Cartes, lignes, pagination et file de couvertures

**Files:**
- Modify: `quark-tauri/ui/providers.js`
- Modify: `quark-tauri/ui/styles.css`
- Test: vérification visuelle + test Rust de la tâche 2

**Interfaces:**
- Consumes: `invoke("provider_ensure_covers", { keys: [{ provider, productId }] })`, `ProviderCoverDto[]`, `app.pv`.
- Produces:
  - `function providerCard(game) -> string`
  - `function providerRow(game) -> string`
  - `function renderMyGamesResults() -> string`
  - `function repaintMyGamesResults() -> void`
  - `function pvMore() -> void`
  - `function pvQueueCovers() -> void`
  - `function pvApplyCoverUpdates(updates) -> void`
  - `function pvLazyImages() -> void`

- [ ] **Step 1: Remplacer `providerCard` et ajouter `providerRow`**

- `providerCard` garde les actions existantes (`open`, `adopt`, `install`, `uninstall`) et ajoute :
  - `data-pv-cover` / `data-provider` / `data-product` sur le conteneur de jaquette, même quand `coverUrl` est absent ;
  - `data-cover-cached="1"` sur le conteneur et l’image si `game.coverCached` ;
  - un bouton `···` avec `data-pv-menu` pour ouvrir un petit menu (ouvrir le dossier, copier le chemin, détails) ;
  - un fallback monogramme si `coverUrl` absent, sans supprimer le conteneur `data-pv-cover`.
- `providerRow` affiche une ligne compacte : jaquette 40 × 54, titre, boutique, état, chemin, actions.
- `renderMyGamesResults` choisit `pvLayout()` et rend `games.slice(0, shown)` avec `providerCard` ou `providerRow`, plus le bouton `#pv-more` si nécessaire.
- `repaintMyGamesResults` remplace uniquement `#pv-results` et `#pv-count`, puis appelle `pvLazyImages()` et `pvQueueCovers()`.

- [ ] **Step 2: Implémenter la file de couvertures**

```js
function pvQueueCovers() {
  const nodes = $$("#pv-results [data-pv-cover]");
  const keys = [];
  for (const node of nodes) {
    const provider = node.dataset.provider;
    const productId = node.dataset.product;
    const key = `${provider}:${productId}`;
    if (node.dataset.coverCached === "1") continue;
    if (app.pv.coverTried.has(key)) continue;
    if ((app.pv.coverFailures.get(key) || 0) >= 3) continue;
    if (app.pv.coverQueue.has(key)) continue;
    app.pv.coverQueue.add(key);
    keys.push({ provider, productId });
    if (keys.length >= 8) break;
  }
  if (!keys.length) return;
  invoke("provider_ensure_covers", { keys })
    .then((updates) => { pvApplyCoverUpdates(updates); pvQueueCovers(); })
    .catch(() => keys.forEach((key) => app.pv.coverQueue.delete(`${key.provider}:${key.productId}`)));
}

function pvApplyCoverUpdates(updates) {
  for (const update of updates) {
    const key = `${update.provider}:${update.productId}`;
    app.pv.coverQueue.delete(key);
    if (!update.coverUrl) {
      app.pv.coverFailures.set(key, (app.pv.coverFailures.get(key) || 0) + 1);
      continue;
    }
    app.pv.coverTried.add(key);
    app.pv.coverFailures.delete(key);
    const node = $(`#pv-results [data-pv-cover][data-provider="${update.provider}"][data-product="${update.productId}"]`);
    if (!node) continue;
    node.dataset.coverCached = "1";
    let img = node.querySelector("img");
    if (!img) {
      img = document.createElement("img");
      img.alt = "";
      img.loading = "lazy";
      img.decoding = "async";
      node.prepend(img);
    }
    img.src = update.coverUrl;
    img.hidden = false;
    const monogram = node.querySelector(".pvmono");
    if (monogram) monogram.remove();
  }
}
```

- `pvLazyImages` observe les images distantes avec `root: $("#sheet")`, `rootMargin: "300px"` ; quand une image devient visible, elle reçoit `src = dataset.src` si elle n’en a pas. Ne jamais retirer `src` des images déjà chargées (évite le clignotement).
- `pvMore` ajoute les 24 suivants avec `insertAdjacentHTML("beforebegin", ...)`, met à jour `shown`, puis `pvLazyImages()` + `pvQueueCovers()`.

- [ ] **Step 3: Supprimer `content-visibility` et ajuster les styles**

Dans `styles.css` :
- supprimer la ligne `.pcard{content-visibility:auto;...}` ;
- ajouter `.pvgrid` (grille responsive), `.pvlist`, `.pvrow`, `.pvtoolbar`, `.pvcount`, `.pccover`, `.pcactions` ;
- conserver les fonds opaques et les bordures fines ; aucune surface transparente sur les cartes.

- [ ] **Step 4: Vérifier visuellement**

Run:
```bash
grep -n "content-visibility" quark-tauri/ui/styles.css || true
xvfb-run -a -s "-screen 0 1360x860x24" bash -c '
  WEBKIT_DISABLE_DMABUF_RENDERER=1 cargo run --manifest-path quark-tauri/Cargo.toml -- --demo &
  APP=$!; sleep 12
  xdotool mousemove 330 24 click 1; sleep 2
  import -window root /tmp/quark-mygames-grid.png
  kill $APP
'
```
Expected: aucune sortie pour `content-visibility` ; la grille affiche 24 cartes, les actions et le bouton « Charger plus » ; le survol ne fait pas disparaître de carte.

- [ ] **Step 5: Commit**

```bash
git add quark-tauri/ui/providers.js quark-tauri/ui/styles.css
git commit -m "feat(mygames): render stable grid/list cards with cover queue"
```

---

### Task 5: Système visuel et modernisation des écrans

**Files:**
- Modify: `quark-tauri/ui/styles.css`
- Modify: `quark-tauri/ui/index.html` (si nécessaire)
- Modify: `quark-tauri/ui/library.js` (markup de la liste/fiche)
- Modify: `quark-tauri/ui/store.js` (markup des cartes)
- Modify: `quark-tauri/ui/settings.js` (markup des cartes)
- Test: captures Xvfb des écrans

**Interfaces:**
- Consumes: classes existantes (`#top`, `.side`, `.sheet`, `.row`, `.tile`, `.scard`, `.card`, `.modal`).
- Produces: tokens CSS et composants visuels partagés ; aucun changement de logique.

- [ ] **Step 1: Ajouter les tokens et les composants**

Dans `:root` :
```css
--space-1:4px; --space-2:8px; --space-3:12px; --space-4:16px; --space-6:24px;
--radius-sm:8px; --radius-md:12px; --radius-lg:16px;
--surface-2:#111; --surface-3:#181818;
```

- Uniformiser les cartes, puces, boutons, champs, tabs et sections.
- Supprimer `backdrop-filter` des surfaces principales et des cartes ; garder éventuellement un seul usage sur la barre supérieure si le rendu reste stable.
- Remplacer les fonds `color-mix(... transparent)` des cartes par `var(--surface)` ou `var(--surface-2)`.
- Garder les transitions à 120–160 ms et `prefers-reduced-motion`.

- [ ] **Step 2: Moderniser la bibliothèque sans changer la structure**

- Conserver `aside.side` + `main.sheet`, la liste groupée et la fiche.
- Ajuster les espacements, la taille des jaquettes, les tabs, le panneau d’aperçu et les boutons d’action.
- Vérifier que la fiche garde le bouton Play, le statut, les actions et les onglets.

- [ ] **Step 3: Moderniser Accueil, Boutique, Réglages et modales**

- Accueil : hero plus sobre, rails alignés.
- Boutique : cartes plus lisibles, sections et badges harmonisés.
- Réglages : cartes et formulaires alignés.
- Modales : en-tête, pied de page et boutons cohérents.

- [ ] **Step 4: Captures de contrôle**

Run:
```bash
xvfb-run -a -s "-screen 0 1360x860x24" bash -c '
  WEBKIT_DISABLE_DMABUF_RENDERER=1 cargo run --manifest-path quark-tauri/Cargo.toml -- --demo &
  APP=$!; sleep 12
  import -window root /tmp/quark-library.png
  xdotool mousemove 275 24 click 1; sleep 2; import -window root /tmp/quark-store.png
  xdotool mousemove 330 24 click 1; sleep 2; import -window root /tmp/quark-mygames.png
  kill $APP
'
```
Expected: les trois captures sont lisibles, sans chevauchement, sans carte vide, et la bibliothèque garde liste à gauche / fiche à droite.

- [ ] **Step 5: Commit**

```bash
git add quark-tauri/ui/styles.css quark-tauri/ui/index.html quark-tauri/ui/library.js quark-tauri/ui/store.js quark-tauri/ui/settings.js
git commit -m "style(ui): unify the visual system across screens"
```

---

### Task 6: Vérification finale de la phase « Mes jeux »

**Files:**
- Modify: `docs/superpowers/plans/2026-11-21-mygames-performance-design.md` (cocher les étapes)
- Test: tout le dépôt

- [ ] **Step 1: Lancer le formatage**

Run:
```bash
cargo fmt --check
cargo fmt --manifest-path quark-tauri/Cargo.toml --check
```
Expected: aucune sortie.

- [ ] **Step 2: Lancer Clippy**

Run:
```bash
cargo clippy --all-targets --locked -- -D warnings
cargo clippy --manifest-path quark-tauri/Cargo.toml --all-targets --locked -- -D warnings
```
Expected: aucune erreur.

- [ ] **Step 3: Lancer les tests**

Run:
```bash
cargo test --locked
cargo test --manifest-path quark-tauri/Cargo.toml --locked
```
Expected: tous les tests passent ; noter le nombre.

- [ ] **Step 4: Vérifier que `main` n’a pas bougé**

Run: `git rev-parse main`
Expected: le SHA de `main` avant le début du travail, inchangé.

- [ ] **Step 5: Commit final de la phase**

```bash
git add docs/superpowers/plans/2026-11-21-mygames-performance-design.md
git commit -m "docs(plan): track mygames implementation progress"
```
