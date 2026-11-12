## Quark 1.4.5 — Fiche de jeu affinée et DLL récursives

### Fiche de jeu

- La bannière nette du jeu revient en haut de la fiche. La jaquette verticale est retirée.
- Le titre affiche le logo SteamGridDB quand il existe. Le nom en texte sert uniquement de repli lorsqu'aucun logo n'est disponible.
- Le temps de jeu et la dernière session ne sont plus affichés à droite du bouton **Jouer**. Ils restent uniquement dans le panneau **Aperçu**.
- **Apparence → Actions de la fiche** permet de placer **Paramètres**, **Personnaliser**, **Journal** et **Supprimer** en haut à droite ou directement à droite de **Jouer**. Le réglage est enregistré.
- Les cartes **Ma fiche** et **Fiche Steam** sont toujours empilées sur toute la largeur : leurs champs ne se chevauchent plus à 1024 px ni à 1360 px.
- Le menu du clic droit reprend les couleurs des actions de la fiche : engrenage bleu, crayon doré, journal vert et poubelle rouge.

### DLL overrides

- La recherche des DLL parcourt désormais tous les sous-dossiers du dossier source, avec une limite de profondeur et sans suivre les liens symboliques.
- Les DLL sont affichées avec leur chemin relatif, triées par nom. Le nom d'override reste le nom du fichier sans extension.
- Le résultat est mis en cache quelques secondes pour ne pas relire l'arborescence du jeu à chaque image.

### Vérifications

- Compilation `cargo build --release --locked`, tests unitaires et rendus hors écran de la fiche, de la grille, des paramètres et des dialogues aux largeurs 900, 1024 et 1360 px.
