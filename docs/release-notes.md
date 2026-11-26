# Quark 2.0.2 — volume mémorisé, icône utile, mémoire maîtrisée

### Lecture des vidéos

- **Volume mémorisé.** Le curseur de la bande-annonce conserve sa position entre deux lancements et entre deux vidéos.
- **Courbe logarithmique.** La position du curseur suit une échelle de 20 dB, comme les lecteurs vidéo modernes : les petits réglages restent précis et le passage au maximum est progressif.
- **Synchronisation avec les contrôles natifs.** Un changement de volume effectué dans la barre du lecteur met aussi à jour le curseur Quark.

### Icône de notification

- **Clic gauche :** ouvre directement la fenêtre Quark.
- **Clic droit :** menu avec « Ouvrir Quark » et « Fermer Quark ».
- **Une seule instance.** Un second lancement affiche la fenêtre déjà ouverte puis quitte, au lieu de créer une deuxième WebView.
- Le menu suit la langue de l'interface et reste à jour après un changement de langue.

### Mes jeux

- **Jaquettes déchargées hors écran.** Les couvertures quittent la mémoire dès qu'elles sortent de la zone visible et sont rechargées au retour.
- **Chargement explicite.** Le bouton « Charger plus » remplace le défilement infini qui pouvait enchaîner des centaines de cartes et faire grimper la mémoire.
- **Observateurs nettoyés** à chaque changement de page, pour éviter les anciens callbacks et la charge CPU résiduelle.
- **Liste et grille conservées** : la bibliothèque garde la liste à gauche et la fiche à droite, comme avant.

### Vérifications

- `cargo fmt --all -- --check` : OK.
- `cargo clippy --all-targets --locked -- -D warnings` : OK.
- `cargo test --locked` : tests du cœur réussis.
- Build Tauri et vérification visuelle sous Xvfb.
