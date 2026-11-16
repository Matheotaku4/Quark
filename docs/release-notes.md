## Quark 1.7.0 — Icône qui bascule, itch.io et bibliothèque Epic directe

### Nouveautés

- **Icône de la zone de notification** : un clic ramène la fenêtre au premier plan quand elle est réduite ou en arrière-plan ; un second clic la ferme vers l'icône sans arrêter le helper. Le menu « Ouvrir Quark » ne fait que ramener la fenêtre. Le réveil passe par un signal Unix et un descripteur de réveil, donc egui ne dessine pas en continu quand la fenêtre est réduite.
- **itch.io** : ajout d'une clé API personnelle dans Paramètres → Boutiques & outils, lecture des clés de téléchargement possédées, sélection de l'archive Windows publiée, téléchargement avec progression et extraction sûre dans un dossier dédié. Les uploads sans archive sont signalés, jamais exécutés.
- **Epic Games direct** : la session OAuth directe suffit désormais à lire la bibliothèque possédée (titres et jaquettes) via l'API du launcher, avec renouvellement automatique du jeton. Legendary reste utilisé pour l'installation et le lancement.
- **Succès Steam** : avec la clé Steam Web API et le SteamID64 déjà utilisés pour les recommandations, Quark lit les succès du joueur, le schéma du jeu et la rareté globale. L'onglet Succès fonctionne désormais pour Steam ; une clé absente, un profil privé ou un jeu sans statistiques est signalé explicitement.
- Les nouveaux fournisseurs sont branchés sur les couvertures, les sauvegardes, les succès de session, le temps de jeu et la désinstallation.

### Vérifications

- `cargo fmt --all --check` : OK.
- `cargo clippy --all-targets --locked -- -D warnings` : OK.
- `cargo test --locked` : 310 tests réussis.
- `cargo check --locked` : OK.
- Rendu de Paramètres → Boutiques & outils vérifié hors écran, avec la ligne itch.io.

### Limites connues

- La bibliothèque Epic directe n'a pas pu être validée avec un vrai compte dans l'environnement de développement ; les analyseurs, le renouvellement de session et les erreurs sont testés.
- L'installation Epic passe toujours par Legendary ; l'API directe ne remplace pas encore le téléchargement des manifests et des chunks.
- itch.io ne propose que des archives pour l'installation automatique. Les installateurs `.exe`, `.dmg` ou `.apk` restent manuels. Aucun succès, sauvegarde cloud ou temps de jeu n'est publié par l'API itch.io.
- Le clic sur l'icône n'a pas été validé sur un bureau réel avec zone de notification dans cet environnement ; le code compile et le chemin de signal est isolé.
- Les succès Steam n'ont pas pu être validés avec une vraie clé Web API ; l'analyse des réponses, le schéma et la rareté sont couverts par des tests.

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
- Site : vérification locale des pages et des assets ; publication GitHub Pages à vérifier après le merge.

### Limites connues

- L'overlay n'a pas été exécuté avec un GPU, Windows, Wine ou Proton réels dans l'environnement de développement : le build croisé, le journal et les tests statiques sont vérifiés, pas le rendu en jeu.
- Les jeux Vulkan, OpenGL, Direct3D 9 et les jeux natifs Linux restent hors du proxy DXGI. Pour ces jeux, MangoHud reste l'option prévue.
- Un jeu qui remplace lui-même `d3d11.dll` ou `dxgi.dll` conserve sa DLL : Quark ne l'écrase jamais.
- Le compteur peut rester invisible en plein écran exclusif selon le compositeur ; le journal indique alors que la fenêtre du jeu n'est pas détectée au premier plan.
