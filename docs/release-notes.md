# Quark 2.2.1 — recherche par extensions

### Recherche

- **Désactivée par défaut.** Aucun site de téléchargement n'est contacté tant que `settings.extension_sources` est vide.
- **Configuration par fichier.** Ajoutez l'URL HTTPS du manifeste dans `settings.extension_sources` de `~/.config/quark/config.json`, puis redémarrez Quark. Le cache local est écrit dans `extensions.json`. Deux extensions ne peuvent pas partager le même identifiant.
- **Sept adaptateurs conservés.** GameBounty, AnkerGames, AstralGames, Kryoto, Online-Fix, ZeiGames et scene.cat sont fournis dans `docs/extensions/quark-sites.json`, prêts à être hébergés dans un dépôt public. L'interface 2.2.0 reste inchangée ; la recherche utilise le manifeste configuré.
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
