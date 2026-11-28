# Quark 2.2.1 — recherche par extensions

### Recherche

- **Désactivée par défaut.** Aucun site de téléchargement n'est contacté tant qu'aucune extension n'est installée. Les boutons de recherche disparaissent de la fiche de jeu, de la boutique, de l'ajout d'un jeu et de la palette.
- **Réglages → Extensions.** Ajout d'un manifeste HTTPS, actualisation à la demande, retrait et cache local. Deux extensions ne peuvent pas partager le même identifiant.
- **Sept adaptateurs conservés.** GameBounty, AnkerGames, AstralGames, Kryoto, Online-Fix, ZeiGames et scene.cat sont fournis dans `docs/extensions/quark-sites.json`, prêts à être hébergés dans un dépôt public.
- **Sites personnalisés.** Un manifeste peut décrire une API JSON ou une page HTML avec des sélecteurs CSS, sans exécuter de code.

### Sécurité

- HTTPS et hôte public obligatoires pour les manifestes et les requêtes.
- En-têtes limités à une liste blanche ; cookies et `Authorization` refusés.
- Manifeste de 256 Kio maximum, 64 sites maximum, un site invalide rejette le manifeste.
- Aucun appel réseau au démarrage : le cache local suffit.

### Documentation

- `docs/extensions/README.md` décrit le format et le dépannage.
- `docs/extensions/AGENT_BUILD_EXTENSION.md` donne à un agent IA les instructions exactes pour publier l'extension des sept sites.

### Vérifications

- `cargo fmt --all --check` : OK.
- `cargo clippy --all-targets --locked -- -D warnings` : OK.
- `cargo test --locked` : 348 tests réussis.
- `cargo test --manifest-path quark-tauri/Cargo.toml --locked` : 14 tests réussis.
- `node --test ui-tests/*.test.mjs` : 8 tests réussis.
- `cargo deny check` : avis, licences, bans et sources OK.
