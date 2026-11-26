# Quark 2.0.0 — mode Deck et interface Tauri 2

### Interface

- **Bibliothèque proche de Steam.** Liste des jeux à gauche, fiche détaillée à droite, avec bannière, logo, boutons Jouer/Terminé et onglets Aperçu, Médias, Succès, Actualités, Historique, Notes, Lancement et Journal.
- **Interface Tauri 2 légère.** HTML/CSS/JS sans framework dans une WebView système ; le cœur métier reste en Rust et ne dépend plus d'egui.
- **Mode Deck.** Détection automatique de la Steam Deck, de gamescope ou d'une manette, navigation complète au clavier et à la manette, plein écran, mise en page agrandie et barre de raccourcis. Réglage Auto / Toujours / Désactivé dans les paramètres.
- **Téléchargements de boutiques.** Progression en direct, débit, ETA, trois téléchargements simultanés maximum et annulation depuis le panneau Transferts.
- **Ajout plus rapide.** Scan d'un dossier avec détection des exécutables et ajout en lot, recherche multi-sites, choix d'illustrations parmi les résultats SteamGridDB.
- **Recherche globale étendue.** Palette Ctrl+K sur toute la bibliothèque, les jeux de boutique et une recherche web directe.
- **Actions par lot.** Statut et favori sur plusieurs jeux, en plus du retrait groupé.
- **Succès comparés.** Progression côte à côte des jeux de boutique associés.

### Vérifications

- `cargo fmt --all -- --check` : OK.
- `cargo clippy --all-targets --locked -- -D warnings` : OK sur les deux crates.
- `cargo test --locked` : 317 tests réussis, 3 tests réseau ignorés.
