## Quark 1.4.6 — Correctif de la fenêtre Nouveautés

### Correctif

- La fenêtre **Nouveautés** affichait encore le résumé des versions 1.4.0 et 1.4.4 après la mise à jour vers 1.4.5. Elle décrit maintenant les changements réels de cette version.
- Le résumé français et anglais couvre la bannière et le logo SteamGridDB, le temps de jeu déplacé dans l'aperçu, l'emplacement des actions de la fiche, la recherche DLL récursive, les cartes Ma fiche / Fiche Steam empilées et les couleurs du menu contextuel.

### Vérifications

- Fenêtre réelle sous X11 virtuel (Xvfb) à 1360 × 900 puis 1024 × 768, avec souris et clavier : bannière, logo et repli texte, les deux placements d'actions, persistance du réglage, cartes empilées, DLL en sous-dossier et menu contextuel.
- `cargo test --locked` : 291 tests réussis, un test préexistant volontairement filtré.
- `cargo build --release --locked` réussi ; archive et somme SHA-256 vérifiées après publication.
