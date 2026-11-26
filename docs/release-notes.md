# Quark 2.0.1 — interface affinée et mémoire maîtrisée

### Interface

- **Recherche de bibliothèque plus claire.** Le champ « Rechercher… » est plus court et le bouton de filtres est aligné sur la même ligne.
- **Animations discrètes entre les onglets** de la fiche de jeu, avec respect de `prefers-reduced-motion`.
- **Personnalisation en deux onglets.** « Métadonnées » pour le nom, le développeur, les genres, les tags et les notes ; « Images » pour choisir jaquette, bannière, logo et icône depuis SteamGridDB.
- **Mes jeux plus léger.** Les jaquettes ne sont chargées qu'à l'approche de la zone visible, avec un rendu différé des cartes hors écran : le défilement reste fluide même avec une grande bibliothèque.
- **Russe ajouté** à l'interface et aux messages du cœur Rust, en plus du français, de l'anglais, de l'espagnol, de l'allemand et du portugais.

### Lecture des vidéos

- **Plein écran corrigé.** Les bandes-annonces utilisent maintenant un mode cinéma maison : la vidéo occupe toute la fenêtre, le bouton plein écran natif de WebKitGTK (responsable de l'écran noir) est masqué, et `Échap` ou un double-clic rend la main.

### Fenêtre et mémoire

- **Clic gauche sur l'icône de notification :** la fenêtre Quark s'ouvre directement.
- **Clic droit :** menu avec « Fermer Quark ».
- **Fermeture vers l'icône :** la WebView est détruite au lieu d'être simplement masquée ; la mémoire du moteur est libérée et la fenêtre est reconstruite au clic sur l'icône.
- **Réglage d'onglet par défaut** de la fiche de jeu : Aperçu, Médias, Succès, Actualités, Historique, Notes, Lancement ou Journal.
- **Options dépendantes grisées** : les réglages gamescope sont désactivés quand gamescope est décoché, et l'identifiant umu quand umu est décoché.

### Nettoyage

- **Suppression de l'overlay Quark** (panneau en jeu, DLL proxy FPS et fenêtre dédiée). La capture d'écran globale reste disponible dans les réglages.
- Retrait des branches de développement obsolètes ; `main` reste la seule branche suivie.

### Vérifications

- `cargo fmt --all -- --check` : OK.
- `cargo clippy --all-targets --locked -- -D warnings` : OK sur les deux crates.
- `cargo test --locked` : tests du cœur réussis, tests réseau ignorés.
- Build de release Tauri et vérification visuelle sous Xvfb.
