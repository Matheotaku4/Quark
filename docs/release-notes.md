# Quark 2.2.3 — images, médias et extensions de recherche

### Interface

- **Interface 2.1.1 conservée.** Liste des jeux à gauche, fiche du jeu à droite, barre supérieure et réglages de la 2.1.1. Aucune feuille de style ni structure HTML n'est remplacée.
- **Corrections ciblées.** Les images de la boutique, les vignettes de médias, les aperçus d'illustrations et les couvertures de la recherche multi-sites s'affichent de nouveau.

### Correctifs

- **CSP compatible avec les images distantes.** `img-src`, `media-src` et `connect-src` autorisent les visuels et bandes-annonces Steam et SteamGridDB ; les protocoles locaux `quarkimg:` et `asset:` restent limités au nécessaire.
- **Recherche multi-sites pilotée par l'extension.** Les sites, leurs noms, leurs hôtes et leurs couleurs viennent uniquement des manifestes ajoutés par l'utilisateur. Aucun site n'est proposé par défaut.
- **Pas d'attente réseau au démarrage.** Aucun manifeste n'est chargé tant que l'utilisateur n'en ajoute pas. L'interface est prévenue quand la liste des sites change.

### Recherche par extensions

- **Aucune extension préinstallée.** La recherche de sites est vide au premier lancement. L'utilisateur ajoute lui-même un ou plusieurs manifestes HTTPS.
- **Sites libres.** Une extension peut déclarer n'importe quel site public, même inconnu du launcher, avec un driver JSON ou HTML.
- **Onglet Extensions.** Réglages → Extensions liste les manifestes actifs, leur version, leur date et leurs erreurs. On peut y ajouter une URL HTTPS, actualiser ou retirer une extension.
- **Extensions personnalisées.** Un manifeste décrit une API JSON ou une page HTML avec des sélecteurs CSS, sans exécuter de code.

### Sécurité

- HTTPS et hôte public obligatoires pour les manifestes et les requêtes.
- En-têtes limités à une liste blanche ; cookies et `Authorization` refusés.
- Manifeste de 256 Kio maximum, 64 sites maximum, un site invalide rejette le manifeste.
- La CSP reste stricte : aucun script inline, aucun `unsafe-eval`.

### Documentation

- `docs/extensions/README.md` décrit le manifeste, les drivers génériques et le dépannage.
- `README.md` et le wiki décrivent l'ajout manuel des extensions et les correctifs d'affichage.

### Vérifications

- `cargo fmt --all --check` : OK.
- `cargo clippy --all-targets --locked -- -D warnings` : OK.
- `cargo test --locked` : 348 tests cœur, 14 tests Tauri.
- Tests UI : 3 tests d'échappement.
- Vérification visuelle : recherche vide sans extension, ajout manuel d'un manifeste, sites dynamiques et filtre par site.
