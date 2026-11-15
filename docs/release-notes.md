## Quark 1.6.2 — Site officiel, statut manuel et overlay plus lisible

### Nouveautés

- **Site officiel** : un site vitrine statique dans `site/` est publié sur GitHub Pages à l'adresse [matheotaku4.github.io/Quark](https://matheotaku4.github.io/Quark/). Le workflow `pages.yml` ne publie que ce dossier, jamais la documentation interne de `docs/`.
- **« À jouer » reste « À jouer »** : lancer ou jouer un jeu ne modifie plus son statut. Un jeu `Backlog` reste `Backlog` après une session, un point de contrôle ou la fermeture de Quark ; le changement de statut est désormais uniquement manuel.
- **Typographie plus nette** : le texte courant utilise Archivo Medium, avec des tailles entières (corps 14 px, petit 12 px) et des légendes remontées d'un cran. La densité de l'interface ne change pas.
- **Diagnostic de l'overlay FPS** : quand **Afficher les FPS dans le jeu** est actif, Quark écrit `overlay.log` dans le dossier de logs du jeu. Le journal indique si les proxies ont été chargés, quelle voie de création de swap chain a été vue, si `Present`/`Present1` a été accroché et pourquoi le compteur est masqué.
- **Couverture CoreWindow** : le proxy Windows accroche aussi `CreateSwapChainForCoreWindow`, utilisé par les jeux UWP/Game Pass, en plus des chemins Direct3D 11 et 12 existants.

### Correctifs

- Le statut d'un jeu n'est plus écrasé par `Game::record_session` ; les points de contrôle continuent de mettre à jour le temps de jeu et l'historique.
- Les tailles de police fractionnaires qui rendaient les petits libellés mous ont été remplacées par une échelle entière.
- Le journal de l'overlay permet de distinguer une DLL non chargée d'une API non couverte (Vulkan/OpenGL/D3D9), d'un lanceur qui démarre un autre exécutable ou d'un overlay masqué par le premier plan.
- Le chemin du journal est passé à la DLL via `Z:` sous Wine/Proton, sans exposer de chemin POSIX à un binaire Windows.

### Vérifications

- `cargo fmt --all --check` : OK.
- `cargo clippy --all-targets --locked -- -D warnings` : OK.
- `cargo test --locked` : 300 tests réussis.
- `cargo build --locked` : réussi ; rendus de l'interface vérifiés à 1x.
- `cargo check --locked --manifest-path tools/overlay-dll/Cargo.toml --target x86_64-pc-windows-gnu` : OK (la liaison MinGW complète reste faite par le workflow de release).
- Site : vérification locale des pages et des assets, puis publication GitHub Pages.

### Limites connues

- L'overlay n'a pas été exécuté avec un GPU, Windows, Wine ou Proton réels dans l'environnement de développement : le build croisé, le journal et les tests statiques sont vérifiés, pas le rendu en jeu.
- Les jeux Vulkan, OpenGL, Direct3D 9 et les jeux natifs Linux restent hors du proxy DXGI. Pour ces jeux, MangoHud reste l'option prévue.
- Un jeu qui remplace lui-même `d3d11.dll` ou `dxgi.dll` conserve sa DLL : Quark ne l'écrase jamais.
- Le compteur peut rester invisible en plein écran exclusif selon le compositeur ; le journal indique alors que la fenêtre du jeu n'est pas détectée au premier plan.
