# Quark 2.2.4 — images restaurées et extensions opt-in

### Interface

- **Interface 2.1.1 définitive.** Liste des jeux à gauche, fiche du jeu à
  droite, barre supérieure et réglages de la 2.1.1. Aucune refonte visuelle
  n'est prévue : cette interface est la seule prise en charge.
- **Corrections ciblées.** Les images de la boutique, les vignettes de médias,
  les aperçus d'illustrations et les couvertures de la recherche multi-sites
  s'affichent de nouveau.

### Correctifs

- **CSP compatible avec les images distantes.** `img-src`, `media-src` et
  `connect-src` autorisent les visuels et bandes-annonces Steam et SteamGridDB ;
  les protocoles locaux `quarkimg:` et `asset:` restent limités au nécessaire.
- **Recherche multi-sites fiable.** Les réponses asynchrones ne dessinent plus
  dans un dialogue fermé et les sites sont identifiés par extension, sans
  collision entre deux manifestes.
- **Pas d'attente réseau au démarrage.** Aucun manifeste n'est chargé tant que
  l'utilisateur n'en ajoute pas.

### Recherche par extensions

- **Aucune extension préinstallée.** La recherche de sites est vide au premier
  lancement. L'utilisateur ajoute lui-même un ou plusieurs manifestes HTTPS.
- **Sites libres.** Une extension peut déclarer n'importe quel site public, même
  inconnu du launcher, avec un driver JSON ou HTML.
- **Onglet Extensions.** Réglages → Extensions liste les manifestes actifs, leur
  version, leur date et leurs erreurs. On peut y ajouter une URL HTTPS,
  actualiser ou retirer une extension.
- **Extensions personnalisées.** Un manifeste décrit une API JSON ou une page
  HTML avec des sélecteurs CSS, sans exécuter de code.

### Sécurité

- HTTPS et hôte public obligatoires pour les manifestes et les requêtes.
- En-têtes limités à une liste blanche ; cookies et `Authorization` refusés.
- Manifeste de 256 Kio maximum, 64 sites maximum, un site invalide rejette le
  manifeste.
- La CSP reste stricte : aucun script inline, aucun `unsafe-eval`.

### Compatibilité

- `config.json` conserve son format ; les nouvelles clés ont une valeur par
  défaut.
- L'ancienne extension par défaut est retirée une seule fois lors de la
  migration. Elle peut être rajoutée manuellement ensuite.
- 2.2.4 regroupe les correctifs préparés pour 2.2.3, qui n'a pas été publiée
  séparément.

### Documentation

- `docs/extensions/README.md` décrit le manifeste, les drivers génériques et le
  dépannage.
- `README.md` et le wiki décrivent l'ajout manuel des extensions et l'interface
  2.1.1.

### Vérifications

- `cargo fmt --all --check` : OK.
- `cargo clippy --all-targets --locked -- -D warnings` : OK.
- `cargo test --locked` : 349 tests cœur, 14 tests Tauri.
- Tests UI : 3 tests d'échappement.
- Vérification visuelle : recherche vide sans extension, ajout manuel d'un
  manifeste, sites dynamiques et filtre par site.
