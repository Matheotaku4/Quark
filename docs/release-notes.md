## Quark 1.7.4 — Détection du dossier de sauvegarde GOG

### Nouveauté

- **Dossiers de sauvegarde GOG proposés automatiquement** : dans la fiche d'un jeu GOG, le bouton **Détecter le dossier…** parcourt le préfixe Wine (`Saved Games`, `Documents`, `My Games`, `AppData/Local`, `AppData/Roaming`) et liste les dossiers dont le nom correspond au titre du jeu. L'utilisateur clique sur le bon candidat ; Quark n'en choisit jamais un à sa place. Le bouton existant de sélection manuelle reste disponible.

### Vérifications

- `cargo fmt --all --check` : OK.
- `cargo test --locked` : 320 tests réussis.
- `cargo clippy --all-targets --locked -- -D warnings` : OK.
- Test dédié : un préfixe factice propose `The Witcher 3` et `Cyberpunk 2077` et ignore `Microsoft`.

### Limites connues

- La détection repose sur le nom des dossiers, pas sur le manifeste Ludusavi ; un jeu dont le dossier de sauvegarde ne reprend pas son titre doit encore être choisi à la main.
- Aucun jeu GOG réel n'a été installé dans cet environnement ; le parcours est couvert par un test de système de fichiers temporaire, pas par une synchronisation cloud réelle.

## Quark 1.7.3 — Installations protégées et interface réellement multilingue

### Correctifs

- **Synchro GOG et Epic** : `merge_gog` et `merge_epic_with_verified_installs` conservent désormais les états `Installing` et `Queued` pendant une synchronisation. Une bibliothèque rafraîchie ne fait plus disparaître un téléchargement en cours.
- **Données brutes** : les six appels `crate::i18n::rich(...)` appliqués à des données (toast, heure de sauvegarde, chemin de fichier, nom de jeu, description de ligne, détail de recherche) utilisent `raw(...)`. Un titre ou un chemin n'est plus traduit mot à mot.
- **Catalogue français** : ajout de `assets/i18n/fr.json`, aligné sur les 742 clés de `en.json`. Un test refuse un catalogue incomplet ou une valeur qui diffère de la source.
- **Formats localisés** : tailles disque et de téléchargement, décimales, dates, mois, durées de jeu, temps relatifs, runtimes Proton/anti-triche, textures en cache, séries et totaux mesurés suivent la langue de l'interface. L'espagnol, l'allemand et le portugais ne retombent plus sur les libellés français ou anglais.

### Vérifications

- `cargo fmt --all --check` : OK.
- `cargo test --locked` : 319 tests réussis.
- `cargo clippy --all-targets --locked -- -D warnings` : OK.
- Tests dédiés : synchro GOG, synchro Epic, catalogue français, tailles dans les cinq langues, libellés de date dans les cinq langues.

### Limites connues

- Les comptes GOG, Epic et Steam réels, la zone de notification du bureau et l'overlay GPU n'ont pas été validés dans cet environnement ; les chemins concernés restent couverts par leurs tests existants.

## Quark 1.7.2 — Les installations itch.io en cours restent visibles

### Correctifs

- Une synchronisation de bibliothèque itch.io ne remplace plus une entrée `Installing` ou `Queued` par `NotInstalled` : l'installation en cours reste visible avec son état et son éventuel dossier.

### Vérifications

- `cargo fmt --all --check` : OK.
- `cargo test --locked` : 316 tests réussis.
- `cargo clippy --all-targets --locked -- -D warnings` : OK.
- Test dédié : un rafraîchissement ne perd ni une installation terminée ni une installation en cours.

## Quark 1.7.1 — Correctifs itch.io et identifiants

### Correctifs

- **Extraction itch.io** : le format est lu dans le nom d'upload publié (`jeu.zip`, `jeu.tar.gz`) et non dans le fichier temporaire `.part`. Les installations d'archives ne sont plus refusées juste après le téléchargement.
- **Titres UTF-8** : le dossier d'installation est tronqué par caractères et non par octets ; un titre long avec des accents ne fait plus paniquer le fil d'installation.
- **Identifiants** : les mises à jour de `credentials.json` passent par un verrou et relisent le fichier avant écriture. Un rafraîchissement de jeton Epic et l'enregistrement d'une clé Steam ou itch.io ne s'écrasent plus mutuellement.
- **Dossier d'installation** : une installation itch.io réussie conserve aussi sa racine, utilisée par les rafraîchissements de bibliothèque.

### Vérifications

- `cargo fmt --all --check` : OK.
- `cargo test --locked` : 315 tests réussis.
- `cargo clippy --all-targets --locked -- -D warnings` : OK.
- Tests dédiés : extraction ZIP sous nom temporaire, extraction par nom d'upload, titre UTF-8 long, fusion d'une installation locale, mise à jour des identifiants sans perte de champs.

### Limites connues

- L'API itch.io n'a pas été appelée avec une vraie clé dans cet environnement ; les analyseurs, la sélection d'upload et l'extraction sont couverts par des tests.
- Les installateurs `.exe`, `.dmg` et `.apk` restent manuels.

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
