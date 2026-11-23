# Quark 1.8.8 — interface Tauri 2

### Interface

- **Nouvelle interface Tauri 2.** Le rendu egui est remplacé par une interface HTML/CSS/JS sans framework, affichée par WebKitGTK. Le cœur métier reste en Rust (`src/lib.rs`) et ne dépend plus d'egui.
- **Bibliothèque proche de Steam.** Liste des jeux à gauche (statut, temps de jeu, recherche, tri, groupes), fiche détaillée à droite : bannière, logo, boutons Jouer/Terminé, onglets Aperçu, Médias, Succès, Actualités, Historique, Notes, Lancement et Journal.
- **Accueil, Boutique, Mes jeux.** Carrousel plein écran, catalogue Steam avec réductions et recommandations, gestion des installations par boutique.
- **Réglages complets.** Huit sections, recherche instantanée, thèmes, langue, overlay, sauvegardes, statistiques, disque et à propos. Enregistrement automatique.
- **Overlay et zone de notification.** `Maj+Retour arrière` ouvre le panneau en jeu ; le raccourci est configurable. L'icône de notification reste optionnelle.
- **Cinq langues.** Français, anglais, espagnol, allemand et portugais, y compris les écrans ajoutés par le portage.

### Vérifications

- `cargo fmt --all -- --check` : OK sur les deux crates.
- `cargo clippy --all-targets --locked -- -D warnings` : OK sur les deux crates.
- `cargo test --locked` : 313 tests réussis, 3 tests réseau ignorés.
