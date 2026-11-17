## Quark 1.7.4 — Détection du dossier de sauvegarde GOG

### Nouveauté

- **Dossiers de sauvegarde GOG proposés automatiquement** : dans la fiche d'un jeu GOG, le bouton **Détecter le dossier…** parcourt le préfixe Wine (`Saved Games`, `Documents`, `My Games`, `AppData/Local`, `AppData/Roaming`) et liste les dossiers dont le nom correspond au titre du jeu. L'utilisateur clique sur le bon candidat ; Quark n'en choisit jamais un à sa place. Le bouton existant de sélection manuelle reste disponible.

### Vérifications

- `cargo fmt --all --check` : OK.
- `cargo test --locked` : 320 tests réussis.
- `cargo clippy --all-targets --locked -- -D warnings` : OK.
- Test dédié : un préfixe factice propose `The Witcher 3` et `Cyberpunk 2077` et ignore `Microsoft`.

### Limites connues

- La détection repose sur le nom des dossiers, pas sur le manifeste Ludusavi ; un jeu dont le dossier de sauvegarde ne reprend pas son titre doit encore être choisi à la main.
- Aucun jeu GOG réel n'a été installé dans cet environnement ; le parcours est couvert par un test de système de fichiers temporaire, pas par une synchronisation cloud réelle.
