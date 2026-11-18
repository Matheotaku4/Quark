## Quark 1.8.1 — Correctifs boutiques et navigation

### Corrections

- **Vue grille : la fiche s'ouvre en mode liste.** Un clic sur une jaquette affiche désormais la fiche avec la colonne de gauche et le panneau de droite, au lieu de remplacer toute la page. Le réglage de vue n'est pas modifié : **Retour** ou `Échap` ramène à la grille.
- **Mes jeux : plus d'espace noir.** La colonne réservée aux transferts n'existe que lorsqu'un téléchargement est actif ; sans transfert, la bibliothèque utilise toute la largeur.
- **Epic : plus d'installation bloquée.** Un jeu déjà présent sur le disque ne repart plus en « Installation… ». Les états `installing`/`queued` laissés par une session interrompue sont réclassés au démarrage d'après le dossier réel.
- **GOG : `réponse gogdl illisible` corrigée.** Une sortie `gogdl info` entourée de lignes de journal est maintenant lue correctement. Si `info` échoue malgré tout, le téléchargement n'est plus annulé avant de commencer : le dossier est retrouvé après coup par son manifeste `goggame-<id>.info`.
- **GOG : sauvegardes et temps de jeu.** Après une partie, le dossier de sauvegarde est détecté automatiquement dans le préfixe Wine, la synchronisation `gogdl save-sync` se lance à la fin de session, et la session est publiée sur `gameplay.gog.com` avec le même contrat que Heroic (`session_date` et durée en minutes). Le bouton de synchronisation manuelle utilise aussi cette détection.
- **Carrousel.** Le test de régression permanent livré avec la préparation de cette version couvre la vignette réellement affichée, le réordonnancement des récents et le repli sur la première vignette.

### Vérifications

- `cargo fmt --check` : OK.
- `cargo test --locked` : 331 tests réussis.
- `cargo clippy --all-targets --locked -- -D warnings` : OK.
- Campagne 1.8.0 : 90 rendus (6 vues × 5 langues × 3 largeurs), 674 rendus au total, sans échec.

### Limites connues

- Les comptes Epic et GOG réels ne sont pas disponibles dans cet environnement : la publication d'une session GOG et la synchronisation de sauvegarde sont couvertes par leurs tests de contrat, pas par un aller-retour réseau réel.
- La détection automatique du dossier de sauvegarde GOG reste fondée sur le préfixe Wine et le nom du jeu ; les emplacements Galaxy distants ne sont pas encore lus. L'utilisateur peut toujours choisir un autre dossier dans la fiche.
- La QA visuelle est réalisée par rendus hors écran ; les parcours réels (connexion boutique, lancement de jeu) ne sont pas couverts par ces captures.
