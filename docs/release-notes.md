## Quark 1.8.2 — Les sauvegardes personnalisées survivent aux synchros

### Correction

- **Epic et Steam : les dossiers de sauvegarde choisis ne disparaissent plus.** `merge_epic` et `merge_sync` repartaient de la fiche rafraîchie, dont `custom_save_paths` est vide, puis remplaçaient l'ancienne fiche. Les chemins personnalisés sont désormais fusionnés avec ceux déjà stockés, sans doublon, avant l'écriture.
- **La fusion résiste aussi aux synchros partielles.** Steam conserve les chemins même quand la liste possédée n'est pas disponible et que seuls les jeux locaux sont fusionnés. Les installations Epic vérifiées gardent la même union de chemins.
- **Deux tests de régression** couvrent `merge_epic` et `merge_sync` avec un dossier personnalisé existant et une fiche rafraîchie sans chemin.

### Vérifications

- `cargo fmt --check` : OK.
- `cargo test --locked` : 333 tests réussis.
- `cargo clippy --all-targets --locked -- -D warnings` : OK.

### Limites connues

- La correction est couverte par des tests unitaires de fusion ; aucune synchronisation Epic ou Steam réelle n'a été effectuée dans cet environnement, faute de comptes configurés.
