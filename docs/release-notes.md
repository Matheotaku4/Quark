# Quark 2.2.3 — images, médias et extension officielle

### Interface

- **Interface 2.1.1 conservée.** Liste des jeux à gauche, fiche du jeu à droite, barre supérieure et réglages de la 2.1.1. Aucune feuille de style ni structure HTML n'est remplacée.
- **Corrections ciblées.** Les images de la boutique, les vignettes de médias, les aperçus d'illustrations et les couvertures de la recherche multi-sites s'affichent de nouveau.

### Correctifs

- **CSP compatible avec les images distantes.** `img-src`, `media-src` et `connect-src` autorisent les visuels et bandes-annonces Steam et SteamGridDB ; les protocoles locaux `quarkimg:` et `asset:` restent limités au nécessaire.
- **Recherche multi-sites pilotée par l'extension.** Les sites, leurs noms et leurs couleurs viennent du manifeste chargé au lieu d'une liste figée dans l'interface.
- **Pas d'attente réseau au démarrage.** Le manifeste officiel est chargé depuis une copie embarquée, puis actualisé en arrière-plan. L'interface est prévenue quand la liste des sites change.

### Recherche par extensions

- **Extension officielle activée par défaut.** Le manifeste [`Quark-sites-extension`](https://github.com/Matheotaku4/Quark-sites-extension) fournit GameBounty, AnkerGames, AstralGames, Kryoto, Online-Fix, ZeiGames et scene.cat.
- **Copie de secours.** Une copie du manifeste est fournie avec Quark : la recherche fonctionne au premier lancement, même si GitHub est injoignable.
- **Onglet Extensions.** Réglages → Extensions liste les manifestes actifs, leur version, leur date et leurs erreurs. On peut y ajouter une URL HTTPS, actualiser ou retirer une extension.
- **Extensions personnalisées.** Un manifeste peut décrire une API JSON ou une page HTML avec des sélecteurs CSS, sans exécuter de code.
- **Site mis à jour.** La page GitHub Pages de Quark renvoie vers l'extension et son dépôt.

### Sécurité

- HTTPS et hôte public obligatoires pour les manifestes et les requêtes.
- En-têtes limités à une liste blanche ; cookies et `Authorization` refusés.
- Manifeste de 256 Kio maximum, 64 sites maximum, un site invalide rejette le manifeste.
- La CSP reste stricte : aucun script inline, aucun `unsafe-eval`.

### Documentation

- `docs/extensions/README.md` décrit le manifeste officiel, les extensions personnalisées et le dépannage.
- `README.md` et le wiki décrivent l'activation par défaut et les correctifs d'affichage.

### Vérifications

- `cargo fmt --all --check` : OK.
- `cargo clippy --all-targets --locked -- -D warnings` : OK.
- `cargo test --locked` : OK.
- Vérification visuelle : boutique, médias, aperçus d'illustrations et recherche multi-sites.
