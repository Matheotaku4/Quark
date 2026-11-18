## Quark 1.8.5 — Média façon Steam et journaux de connexion

### Nouveautés

- **L'onglet « Actualités » devient « Média ».** La fiche jeu charge la fiche publique Steam du jeu et affiche sa bande-annonce avant ses captures. Les jeux sans fiche Steam reliée affichent un message explicite au lieu d'un flux vide.
- **Disposition proche de Steam.** La bande-annonce occupe la largeur disponible, avec un bouton de lecture sur sa vignette. Les captures suivent en grille 16:9 (trois colonnes sur une fenêtre large), avec un liseré d'accent au survol. Un clic ouvre la vidéo ou l'image complète dans le navigateur.
- **Les connexions GOG et Epic entrent dans le journal du jeu.** Chaque session commence par l'état de la session gogdl, des sessions Epic directe et Legendary, la présence des outils et la boutique du jeu lancé. Les jetons et codes d'autorisation ne sont jamais journalisés.
- **Cache et chargement.** Les médias sont chargés une fois par jeu, à l'ouverture de l'onglet Média, et le bouton « Actualiser » permet de réessayer après une erreur réseau.

### Vérifications

- `cargo fmt --all -- --check` : OK.
- `cargo test --locked` : 342 tests réussis, 3 tests réseau ignorés par défaut.
- `cargo test --locked -- --ignored` : 3 tests réseau réussis (médias réels de The Witcher 3, résolution du pack GOG et builds Windows).
- `cargo clippy --all-targets --locked -- -D warnings` : OK.
- Rendu hors écran de la nouvelle fiche Média en 1360×1800 : bande-annonce, bouton de lecture, titre et grille de six captures visibles.

### Limites connues

- Steam sert de plus en plus ses bandes-annonces en HLS/DASH. Quark affiche la vignette et ouvre alors la fiche Steam, qui joue la vidéo, au lieu de fabriquer un lien mort. Les anciens trailers MP4/WebM s'ouvrent directement.
- L'onglet Média s'appuie sur la fiche Steam : un jeu GOG, Epic ou local non relié à Steam n'a pas de galerie pour l'instant.
- Les diagnostics de connexion indiquent la présence des sessions et des outils, pas leur validité côté serveur : un jeton expiré reste marqué « connecté » jusqu'à la prochaine requête de la boutique.
