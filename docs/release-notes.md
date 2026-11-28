# Quark 2.2.2 — retour de l'interface 2.1.1

### Interface

- **Interface 2.1.1 restaurée.** La bibliothèque plate d'origine revient : liste des jeux à gauche, fiche du jeu à droite, barre supérieure et réglages de la 2.1.1. La feuille de style « moderne » introduite en 2.2.0 est retirée.
- **Aucune fonction perdue.** Les mises à jour signées, la CSP stricte, la validation des commandes Tauri et l'échappement des templates restent en place.
- **Recherche par extensions conservée.** La logique d'extension reste disponible en arrière-plan, sans nouvel onglet ni bouton dans l'interface 2.1.1.

### Recherche par extensions

- **Désactivée par défaut.** Aucun site de téléchargement n'est contacté tant que `settings.extension_sources` est vide.
- **Configuration par fichier.** Ajoutez l'URL HTTPS du manifeste dans `settings.extension_sources` de `~/.config/quark/config.json`, puis redémarrez Quark. Le cache local est écrit dans `extensions.json`.
- **Sept adaptateurs fournis.** GameBounty, AnkerGames, AstralGames, Kryoto, Online-Fix, ZeiGames et scene.cat sont dans `docs/extensions/quark-sites.json`, prêts à être hébergés dans un dépôt public.
- **Sites personnalisés.** Un manifeste peut décrire une API JSON ou une page HTML avec des sélecteurs CSS, sans exécuter de code.

### Sécurité

- HTTPS et hôte public obligatoires pour les manifestes et les requêtes.
- En-têtes limités à une liste blanche ; cookies et `Authorization` refusés.
- Manifeste de 256 Kio maximum, 64 sites maximum, un site invalide rejette le manifeste.
- Le cache local évite les appels réseau une fois le manifeste téléchargé ; un manifeste absent est récupéré au démarrage.

### Documentation

- `docs/extensions/README.md` décrit le format et le dépannage.
- `docs/extensions/AGENT_BUILD_EXTENSION.md` donne à un agent IA les instructions exactes pour publier l'extension des sept sites.

### Vérifications

- `cargo fmt --all --check` : OK.
- `cargo clippy --all-targets --locked -- -D warnings` : OK.
- `cargo test --locked` : 348 tests réussis.
- `cargo test --manifest-path quark-tauri/Cargo.toml --locked` : 14 tests réussis.
- `node --test ui-tests/*.test.mjs` : 3 tests réussis.
