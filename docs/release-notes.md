## Quark 1.5.0 — Favoris, notifications et terminal

### Nouveautés

- **Favoris** : une étoile dans la fiche et le menu contextuel, un bandeau **Favoris** sur l'accueil, et le tri **Favoris d'abord** dans l'accueil et la bibliothèque. Le choix est enregistré dans `config.json`.
- **Notifications bureau** : fin d'installation, fin de sauvegarde Ludusavi après session et fin de session, avec trois réglages indépendants dans **Paramètres → En jeu**. Rien n'est envoyé quand la fenêtre Quark est au premier plan ; l'envoi passe par `notify-send`.
- **Lancement en ligne de commande** : `quark launch <jeu>` lance un jeu sans ouvrir l'interface et `quark list` affiche la bibliothèque. La correspondance exacte passe avant la recherche insensible à la casse ; une ambiguïté ou un titre inconnu quitte avec le code 1.
- **Support PlayStation 2 retiré** : PCSX2, les jaquettes PS2 et RetroAchievements ne sont plus inclus. Les anciennes configurations restent lisibles, les champs retirés sont ignorés et les autres jeux sont conservés.

### Correctifs et qualité

- La vérification des mises à jour utilise de nouveau le dépôt `Matheotaku4/Quark` au lieu de l'ancien `Quark-Launcher`.
- Le code mort a été supprimé : `cargo check --locked` ne produit plus aucun avertissement.
- Le README et le wiki décrivent la version 1.5.0 ; le workflow de test utilise `--locked`.
- Les tests obsolètes des APIs mortes ont été retirés et les tests CLI/favoris ajoutés.

### Vérifications

- `cargo check --locked` : 0 avertissement.
- `cargo test --locked` : 286 tests réussis.
- `cargo build --release --locked` : réussi.
- `quark list`, `quark launch` (succès, ambiguïté, titre inconnu) et `quark --update` vérifiés.
- Rendu hors écran du bandeau Favoris et de la carte **Notifications bureau**.

### Limites connues

- `notify-send` et un serveur de notifications ne sont pas installés dans l'environnement de compilation : le chemin de code compile, mais l'affichage réel d'une notification n'a pas pu être vérifié ici.
- Un lancement Proton réel n'est pas vérifiable sans Steam/Proton/Wine dans cet environnement.
