# Quark 1.8.7 — corrections GOG, overlay Epic et lecteur vidéo

### Correctifs

- **Une installation GOG déjà présente n'est plus retéléchargée.** Avant de lancer gogdl, Quark cherche `goggame-<id>.info` dans le dossier choisi et adopte le jeu s'il contient aussi un exécutable. Le manifeste de reprise gogdl n'est supprimé qu'après cette vérification. Après un échec de gogdl, Quark n'adopte jamais le dossier : il pourrait être partiel.
- **Les erreurs gogdl redeviennent lisibles.** Une erreur « Not enough disk space » est expliquée en français au lieu d'afficher le tuple de taille de la dernière ligne de progression.
- **L'overlay Epic est proposé au lancement.** Si l'overlay EOS manque, Quark propose de l'installer avant de lancer le jeu, attend la fin du téléchargement puis active l'overlay dans le préfixe s'il existe déjà. Un bouton permet aussi de lancer sans overlay.
- **L'overlay installé dans un préfixe est reconnu.** Quark vérifie la DLL EOS dans les dossiers du launcher Epic, les artefacts EOS et `OverlayPath` de `user.reg`, au lieu de regarder uniquement son propre dossier global. Il active alors l'overlay sans proposer un téléchargement inutile.
- **L'overlay Epic est visible dans les diagnostics.** Le journal de session indique désormais `Epic · overlay EOS installé (<chemin>)` — y compris quand l'overlay vient du préfixe du jeu — ou `absent`, et un lancement sans overlay ajoute un avertissement explicite au lieu de rester silencieux.
- **Le statut de l'overlay tient compte du préfixe.** La carte **Paramètres → Boutiques & outils** ne dit plus « non installé » quand la DLL EOS est présente dans le préfixe d'un jeu Epic lié ; elle affiche le chemin réellement détecté.
- **Le statut Epic distingue les deux sessions.** Une connexion Legendary seule n'est plus présentée comme une session Epic directe ; l'interface indique « Connecté via Legendary · connexion directe absente », propose d'installer l'overlay EOS et laisse un bouton pour connecter la session directe.
- **L'adoption GOG exige la preuve du manifeste.** Un dossier n'est repris que si son `goggame-<id>.info` est un JSON complet avec le bon identifiant, si l'exécutable déclaré par ses `playTasks` existe (casse comprise) et si aucun marqueur de reprise gogdl/Galaxy n'est présent. Un `setup.exe` ou un fichier de `__redist` ne suffit plus. La synchronisation de bibliothèque, la reprise après fermeture et la sonde périodique utilisent la même vérification : un dossier incomplet repasse immédiatement en « non installé ». Un échec de gogdl n'adopte jamais son dossier partiel.

### Lecteur vidéo

- **La lecture continue sans bouger la souris.** Le lecteur planifie lui-même ses repeints (environ 60 Hz) au lieu d'attendre un événement d'entrée : la vidéo ne s'arrête plus quand le pointeur est immobile.
- **Volume réglable.** Un curseur sous la vidéo règle le flux PulseAudio/PipeWire du processus GStreamer via `pactl`, sans redémarrer la lecture. Le réglage est conservé d'une bande-annonce à l'autre. Sans `pactl` ou sans serveur PulseAudio/PipeWire, le curseur reste masqué et la lecture fonctionne comme avant.

### Vérifications

- `cargo fmt --all -- --check` : OK.
- `cargo test --locked --offline` : 359 tests réussis, 4 tests réseau ignorés.
- `cargo clippy --all-targets --locked -- -D warnings` : OK.

# Quark 1.8.6 — La bande-annonce se lit dans Quark

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
