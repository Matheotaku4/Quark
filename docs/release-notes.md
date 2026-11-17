## Quark 1.8.0 — Accueil, fiche jeu et recherche globale

### Nouveautés

- **Un accueil autonome.** L'accueil possède désormais son propre onglet, un hero plein largeur et un carrousel de jeux récents, sans colonne latérale.
- **La fiche jeu en pleine page.** Depuis la grille, la fiche remplace la bibliothèque et un bouton **Retour** la referme. Le retour est contextuel : il ramène à l'accueil ou à la grille selon l'origine, avec la vignette du carrousel restaurée.
- **Une recherche globale unique.** `Ctrl+F`, `Ctrl+K`, `/` ou la loupe ouvrent la même palette. Elle cherche les jeux locaux, les jeux possédés dans les boutiques et les réglages, sans accents et avec une sous-chaîne réelle.
- **Les réglages par tâche.** Un champ au-dessus de la navigation trouve une action (« sauvegardes cloud », « overlay », « espace disque ») et ouvre directement la bonne page. Les résultats sont cliquables et traduits dans les cinq langues.
- **Des états asynchrones cohérents.** Actualités, chargements, erreurs et artworks manquants partagent les mêmes squelettes et cartes d'état ; le logo dispose d'un repli même quand la texture est vide.
- **Cinq langues aux trois largeurs.** Les écrans principaux sont vérifiés en français, anglais, allemand, espagnol et portugais à 900, 1024 et 1360 px, avec des points de rupture partagés.

### Vérifications

- `cargo fmt --check` : OK.
- `cargo test --locked` : 322 tests réussis.
- `cargo clippy --all-targets --locked -- -D warnings` : OK.
- QA visuelle : 90 rendus hors écran (6 vues × 5 langues × 3 largeurs), plus vérification du clic sur un résultat de réglage.
- Captures du site et de la documentation FR/EN régénérées avec la recherche globale.

### Limites connues

- La QA visuelle est réalisée par rendus hors écran ; les parcours réels (connexion boutique, lancement de jeu) ne sont pas couverts par ces captures.
- La détection GOG et les synchronisations de boutiques restent celles de la version précédente ; aucun jeu GOG réel n'est installé dans cet environnement.
