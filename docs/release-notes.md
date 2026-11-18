## Quark 1.8.1 — Version de maintenance

### Ce qui change

- **Le carrousel est couvert par un test permanent.** Le parcours qui mémorise la vignette réellement affichée, retrouve le jeu après un réordonnancement des récents et revient à l'accueil est désormais vérifié par la suite de tests, au lieu d'un contrôle manuel. Le repli sur la première vignette et l'ouverture depuis la bibliothèque sont couverts aussi.
- **Aucun changement visible.** L'interface, les réglages et le format de `config.json` sont identiques à 1.8.0 ; aucune migration n'est nécessaire.

### Corrections

- Aucune correction supplémentaire n'a été nécessaire après la campagne de QA : les cinq findings CodeRabbit et l'affinage du marqueur du carrousel sont déjà livrés dans 1.8.0.

### Vérifications

- `cargo fmt --check` : OK.
- `cargo test --locked` : 327 tests réussis.
- `cargo clippy --all-targets --locked -- -D warnings` : OK.
- Campagne 1.8.0 : 90 rendus (6 vues × 5 langues × 3 largeurs), 674 rendus au total, sans échec.

### Limites connues

- La QA visuelle est réalisée par rendus hors écran ; les parcours réels (connexion boutique, lancement de jeu) ne sont pas couverts par ces captures.
- La détection GOG et les synchronisations de boutiques restent celles de la version précédente ; aucun jeu GOG réel n'est installé dans cet environnement.
