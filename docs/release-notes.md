## Quark 1.8.6 — La bande-annonce se lit dans Quark

### Nouveautés

- **Lecture intégrée.** L'onglet Média lit désormais les bandes-annonces Steam directement dans la fiche : HLS, DASH, MP4 et WebM, avec le son. Quitter la fiche, changer d'onglet ou sélectionner une capture arrête la lecture et libère le lecteur.
- **Une vraie pellicule façon Steam.** La grande vue montre la vidéo ou la capture sélectionnée. En dessous, une liste horizontale présente la vidéo en premier, puis les captures. Un clic sur une capture remplace la vidéo par l'image ; un clic sur la vidéo la remet en grand et démarre la lecture.
- **Navigation horizontale.** La pellicule se parcourt à la molette, en la faisant glisser ou avec les flèches précédent/suivant. La sélection reste entourée d'un liseré d'accent.
- **Lecteur isolé et repli propre.** GStreamer tourne dans un processus séparé : une erreur du décodeur ne peut pas emporter Quark. Si GStreamer ou ses greffons manquent, la fiche affiche la vignette, explique l'erreur et propose d'ouvrir la page Steam.
- **Journaux de connexion conservés.** Chaque session GOG ou Epic continue d'écrire l'état de gogdl, des sessions Epic directe et Legendary, la présence des outils et la boutique du jeu lancé, sans jeton.

### Vérifications

- `cargo fmt --all -- --check` : OK.
- `cargo test --locked` : 344 tests réussis, 4 tests réseau ignorés par défaut.
- `cargo test --locked -- --ignored` : 4 tests réseau réussis (bande-annonce HLS réelle dans le lecteur, médias Steam, résolution du pack GOG et builds Windows).
- `cargo clippy --all-targets --locked -- -D warnings` : OK.
- Lecture réelle de la bande-annonce HLS de The Witcher 3 : première image reçue dans le lecteur, puis arrêt propre.
- Rendus hors écran 1360×1600 : grande vue, pellicule vidéo puis captures, sélection d'une capture et défilement horizontal.

### Prérequis de lecture intégrée

GStreamer n'est pas embarqué dans l'archive. La lecture fonctionne quand `gst-launch-1.0` et les greffons HLS/H.264 sont installés, par exemple sous Ubuntu/Debian :

```bash
sudo apt install gstreamer1.0-tools gstreamer1.0-plugins-base \
  gstreamer1.0-plugins-good gstreamer1.0-libav
```

Sans ces paquets, Quark reste utilisable et propose la fiche Steam à la place.

### Limites connues

- La lecture intégrée s'appuie sur GStreamer et sur la sortie audio du système ; le premier tampon HLS peut prendre quelques secondes.
- Le lecteur intégré propose lecture/pause et relecture, mais pas encore de barre de progression ni de sélection de piste audio.
- L'onglet Média s'appuie sur la fiche Steam : un jeu GOG, Epic ou local non relié à Steam n'a pas de galerie pour l'instant.
- Les diagnostics de connexion indiquent la présence des sessions et des outils, pas leur validité côté serveur : un jeton expiré reste marqué « connecté » jusqu'à la prochaine requête de la boutique.
