# Recherche par extensions — design

## Intention

La recherche de jeux sur des sites de téléchargement ne doit plus être une
fonctionnalité intégrée et toujours visible. Elle devient une extension
facultative, décrite par un manifeste JSON hébergé hors du dépôt Quark. Quark
ne fournit plus aucune source par défaut : sans extension installée, les
boutons et les commandes de recherche sont absents ou désactivés.

L'utilisateur ajoute une URL `raw.githubusercontent.com` (ou toute autre URL
HTTPS) dans **Réglages > Extensions**. Quark télécharge le manifeste, le
valide, le met en cache, puis utilise les sites qu'il déclare. Le dépôt
externe peut ainsi être mis à jour sans nouvelle version de Quark.

## Objectifs

- Désactiver la recherche par défaut, sans supprimer le code des adaptateurs
  existants.
- Décrire les sept sites actuels dans un manifeste externe.
- Permettre d'ajouter d'autres sites sans recompiler Quark, via des drivers
  génériques JSON et HTML.
- Garder le comportement actuel des sept adaptateurs (en-têtes AnkerGames,
  action Next.js d'AstralGames, encodage Windows-1251 d'Online-Fix, etc.).
- Afficher un état clair dans **Réglages > Extensions** : URL, nom, version,
  nombre de sites, date de mise à jour, erreurs.
- Ne faire aucun appel réseau au démarrage et aucun sondage périodique.

## Non-objectifs

- Exécuter du code fourni par une extension (pas de JavaScript, pas de WASM).
- Télécharger des fichiers depuis les sites trouvés.
- Gérer des comptes, des cookies ou des jetons sur les sites de recherche.
- Remplacer les intégrations Steam/Epic/GOG existantes.
- Synchroniser automatiquement le dépôt externe à chaque démarrage.

## Anti-objectifs

- Une extension ne doit jamais pouvoir lire un fichier local, lancer un
  processus ou contourner la validation des commandes Tauri.
- Une extension ne doit pas pouvoir cibler `localhost`, une adresse privée ou
  un schéma autre que HTTPS.
- Une panne de l'extension ne doit pas bloquer la bibliothèque, la boutique ou
  le lancement des jeux.

## Approches étudiées

### A. Manifeste déclaratif + drivers intégrés — retenue

Les sept parseurs restent dans `src/game_search.rs` et deviennent des drivers
nommés. Le manifeste choisit un driver par site. Deux drivers génériques, JSON
et HTML, couvrent les nouveaux sites.

Avantages : comportement actuel conservé, faible risque de régression, vraie
extensibilité. Inconvénient : les drivers génériques sont moins puissants que
du code sur mesure.

### B. Tout décrire dans le manifeste

Réécrire les sept sites avec les drivers génériques, sans code spécifique.

Avantage : système entièrement piloté par les données. Inconvénients : risque
élevé de casser les cas particuliers, beaucoup plus de travail, pas de gain
utilisateur immédiat.

### C. Plugins de code

Extensions JavaScript ou WASM.

Avantage : extensibilité illimitée. Inconvénients : surface d'attaque,
consommation, distribution et compatibilité. Rejeté.

## Format du manifeste

Le manifeste est un objet JSON, taille maximale 256 Kio, version de schéma 1.

```json
{
  "schemaVersion": 1,
  "id": "quark-search-sites",
  "name": "Sites de recherche Quark",
  "version": "1.0.0",
  "description": "Sources de recherche de jeux.",
  "homepage": "https://github.com/example/quark-search-sites",
  "sites": [
    {
      "id": "gamebounty",
      "label": "GameBounty",
      "host": "gamebounty.st",
      "searchUrl": "https://gamebounty.st/search?q={query}",
      "driver": "gamebounty"
    },
    {
      "id": "ankergames",
      "label": "AnkerGames",
      "host": "ankergames.net",
      "searchUrl": "https://ankergames.net/search?q={query}",
      "driver": "ankergames"
    },
    {
      "id": "astralgames",
      "label": "AstralGames",
      "host": "astralgames.net",
      "searchUrl": "https://astralgames.net/search?q={query}",
      "driver": "astralgames"
    },
    {
      "id": "kryoto",
      "label": "Kryoto",
      "host": "kryo.to",
      "searchUrl": "https://kryo.to/catalog?q={query}",
      "driver": "kryoto"
    },
    {
      "id": "onlinefix",
      "label": "Online-Fix",
      "host": "online-fix.me",
      "searchUrl": "https://online-fix.me/index.php?do=search&subaction=search&story={query}",
      "driver": "onlinefix"
    },
    {
      "id": "zeigames",
      "label": "ZeiGames",
      "host": "zeigames.com",
      "searchUrl": "https://zeigames.com/search/?q={query}",
      "driver": "zeigames"
    },
    {
      "id": "scenecat",
      "label": "scene.cat",
      "host": "scene.cat",
      "searchUrl": "https://scene.cat/?q={query}",
      "driver": "scenecat"
    }
  ]
}
```

### Drivers intégrés

`gamebounty`, `ankergames`, `astralgames`, `kryoto`, `onlinefix`, `zeigames`,
`scenecat`. Le driver appelle le parseur existant. Les URL de requête restent
celles du code actuel ; `searchUrl` ne sert qu'au bouton « Chercher sur le
site ».

### Driver JSON générique

```json
{
  "id": "example",
  "label": "Example",
  "host": "example.com",
  "searchUrl": "https://example.com/search?q={query}",
  "driver": "json",
  "request": {
    "method": "GET",
    "url": "https://example.com/api/search?q={query}",
    "headers": { "Accept": "application/json" }
  },
  "list": "data.results",
  "title": "title",
  "url": "https://example.com/game/{slug}",
  "detail": ["{size}", "{version}"],
  "cover": ["cover", "poster"],
  "limit": 8
}
```

- `request.method` : `GET` ou `POST`.
- `request.body` : modèle facultatif pour POST, avec `{query}`.
- `request.encoding` : `utf-8` (défaut) ou `windows-1251`.
- `list` : chemin à points vers le tableau de résultats.
- `title`, `cover` : chemins à points.
- `url`, `detail` : modèles avec `{chemin}` résolu dans l'élément.
- `limit` : 1 à 20, 8 par défaut.

### Driver HTML générique

```json
{
  "id": "example-html",
  "label": "Example HTML",
  "host": "example.com",
  "searchUrl": "https://example.com/search?q={query}",
  "driver": "html",
  "request": {
    "url": "https://example.com/search?q={query}",
    "headers": { "Accept": "text/html" }
  },
  "items": "div.result",
  "title": { "selector": "h2" },
  "url": { "selector": "a", "attr": "href" },
  "detail": [{ "selector": ".size" }, { "selector": ".version" }],
  "cover": { "selector": "img", "attr": "data-src" },
  "limit": 8
}
```

Chaque champ accepte `selector`, `attr` (sinon texte), `template`
(`https://site/game/{value}`), `stripPrefix` et `stripSuffix`. Les sélecteurs
CSS sont interprétés par la crate `scraper`. Les URL relatives sont résolues
contre l'URL de la requête. Le texte est normalisé et les entités HTML sont
décodées par le parseur.

## Validation

- `schemaVersion` doit valoir `1`.
- `id`, `name`, `version` non vides ; `id` limité à `[a-z0-9._-]`, 64 caractères.
- 1 à 64 sites, identifiants uniques dans le manifeste.
- Deux extensions installées ne peuvent pas partager le même `id` : la
  deuxième est refusée avec un message explicite.
- `searchUrl` et `request.url` : HTTPS obligatoire, hôte public, pas
  d'identifiants dans l'URL.
- `searchUrl` doit contenir `{query}`.
- En-têtes autorisés uniquement : `Accept`, `Accept-Language`, `Content-Type`,
  `User-Agent`, `Referer`, `Origin`, `Next-Action`, `X-Requested-With`.
  Valeur de 512 caractères maximum, sans retour à la ligne.
- Driver inconnu, sélecteur CSS invalide, chemin JSON vide ou modèle d'URL
  invalide : rejet du manifeste avec un message clair.
- Le manifeste entier est rejeté si un seul site est invalide : pas de
  chargement partiel silencieux.

## Sécurité réseau

- HTTPS uniquement pour l'URL du manifeste et les requêtes des drivers.
- Hôtes privés refusés : `localhost`, `*.localhost`, `127.0.0.0/8`, `10.0.0.0/8`,
  `172.16.0.0/12`, `192.168.0.0/16`, `169.254.0.0/16`, `::1`, `fc00::/7`,
  `fe80::/10`.
- Redirections limitées à 5 ; l'URL finale est revérifiée.
- Manifeste : 256 Kio, délai 15 s.
- Réponses de recherche : limites existantes de `src/net.rs` (8 Mio HTML,
  4 Mio JSON), délai 22 s.
- Aucun cookie, aucun en-tête `Authorization`, aucun fichier local.
- Les erreurs ne contiennent jamais le corps de la réponse.

## Cache et réglages

`Settings` reçoit un seul champ :

```rust
pub extension_sources: Vec<String>,
```

Vide par défaut. Le cache est un fichier séparé dans
`config::data_dir()/extensions.json` :

```json
{
  "version": 1,
  "entries": [
    {
      "url": "https://raw.githubusercontent.com/.../quark-sites.json",
      "fetchedAt": 1795819890,
      "manifest": { "...": "..." }
    }
  ]
}
```

- Le cache ne contient que des manifestes déjà validés.
- Au démarrage, Quark lit le cache et ne fait aucun appel réseau.
- `extension_add` télécharge et valide avant d'ajouter l'URL aux réglages.
- `extension_refresh` remplace l'entrée du cache ; en cas d'échec, l'ancienne
  entrée reste utilisable et l'erreur est renvoyée à l'interface.
- `extension_remove` retire l'URL des réglages et l'entrée du cache.
- Une entrée de cache dont l'URL n'est plus dans les réglages est ignorée.

## API Tauri

Nouvelles commandes :

- `extensions_add(url)` : télécharge, valide, met en cache, persiste le réglage.
- `extensions_remove(url)` : retire la source et son cache.
- `extensions_refresh(url: Option<String>)` : actualise une source ou toutes.
- `site_search(query)` : utilise les sites actifs du cache.

`bootstrap_data` expose en plus :

- `extensions: Vec<ExtensionStatusDto>` ;
- `searchSites: Vec<SearchSiteDto>`.

`ExtensionStatusDto` : `url`, `id`, `name`, `version`, `siteCount`,
`fetchedAt`, `stale`, `error`.
`SearchSiteDto` : `uid`, `extensionId`, `extensionName`, `id`, `label`,
`host`, `searchUrl`.

Les commandes sont ajoutées à `quark-tauri/build.rs` et à
`quark-tauri/capabilities/default.json`.

## Interface

### Réglages > Extensions

Nouvel onglet entre « Boutiques & outils » et « Statistiques ».

- Champ URL + bouton **Ajouter**.
- Liste des extensions : nom, version, nombre de sites, URL, date.
- Boutons **Actualiser** et **Retirer** par extension.
- État vide : « Aucune extension installée. La recherche de sites est
  désactivée. »
- Lien vers le modèle d'extension du dépôt.
- Message de sécurité : « Quark ne charge que des manifestes JSON. Aucun code
  n'est exécuté. »

### Points d'entrée de recherche

Tant que `searchSites` est vide :

- pas de bouton dans l'ajout d'un jeu ;
- pas d'icône « sites » dans la fiche d'un jeu ;
- pas de clic de recherche sur les cartes de la boutique ;
- pas d'entrée dans la palette de commandes.

`webSearchModal` reste protégé : si elle est appelée sans site, elle affiche
un message renvoyant vers **Réglages > Extensions**.

Avec au moins un site, la fenêtre de recherche utilise `searchSites` pour les
groupes, les filtres et les liens « Chercher sur le site ». Les résultats et
erreurs utilisent `uid` comme identifiant de groupe.

## Fichiers touchés

- `src/extensions.rs` (nouveau) : manifeste, validation, cache, drivers
  génériques, statuts.
- `src/game_search.rs` : `SearchSite` devient un driver interne ; les
  fonctions de parseur restent ; `search` prend une définition de site.
- `src/lib.rs` : export du module.
- `Cargo.toml` : dépendance `scraper` pour les sélecteurs CSS du driver HTML.
- `src/config.rs` : `extension_sources`.
- `quark-tauri/src/features.rs` : commandes extensions et `site_search`.
- `quark-tauri/src/main.rs` : commandes, `Bootstrap`.
- `quark-tauri/build.rs`, `capabilities/default.json` : permissions.
- `quark-tauri/ui/settings.js`, `dialogs.js`, `library.js`, `store.js`,
  `app.js`, `i18n-extra.js`, `i18n-ru.js`, `styles-modern.css`.
- `docs/extensions/README.md`, `docs/extensions/quark-sites.json`.
- `README.md`, `docs/release-notes.md`.

## Tests

- Validation du manifeste : valide, schéma inconnu, URL non HTTPS, hôte privé,
  doublon de site, driver inconnu, sélecteur invalide, en-tête interdit.
- Driver JSON : chemin de liste, titre, URL, détails, couverture, limite,
  filtrage de pertinence.
- Driver HTML : sélecteurs, attributs, URL relative, texte, limite.
- Cache : chargement, sauvegarde, entrée orpheline ignorée, cache corrompu
  ignoré.
- Recherche Tauri : ordre des sites, `uid`, erreurs par site, absence de site.
- Interface : échappement des chaînes d'extension, absence des boutons sans
  site.
- `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings`,
  `cargo test --locked`, tests Tauri, `cargo deny check`.

## Migration

- Aucune source n'est installée automatiquement.
- Les utilisateurs qui veulent les sept sites actuels ajoutent l'URL raw du
  manifeste externe.
- Le fichier `docs/extensions/quark-sites.json` sert de modèle et peut être
  copié tel quel dans le dépôt externe.

## Version

Passage à **2.3.0** pour les deux crates, car la recherche change de nature et
de visibilité.

## Ce qui reste à décider par l'implémenteur

- Style exact des cartes d'extension dans la feuille moderne.
- Libellés définitifs des messages d'erreur réseau.
- Faut-il proposer un bouton « Ajouter le manifeste officiel » plus tard ; hors
  périmètre de cette version.
