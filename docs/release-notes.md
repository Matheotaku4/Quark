# Quark 2.1.1 — recherche AnkerGames rétablie et artworks plus sûrs

### Recherche multi-sites

- **AnkerGames répond de nouveau.** Son API refusait les clients non navigateur par un HTTP 403. Quark envoie maintenant l'origine, le referer et l'agent utilisateur attendus, et les résultats AnkerGames réapparaissent avec les six autres sources.
- **Sept sources vérifiées.** GameBounty, AnkerGames, AstralGames, Kryoto, Online-Fix, ZeiGames et scene.cat ont été interrogés en direct ; une source indisponible continue d'afficher sa propre erreur.

### Artworks

- **WebP mesuré sans décodage.** Les dimensions sont lues dans l'en-tête VP8X, VP8 ou VP8L. Une grande image distante n'est plus décodée en mémoire juste pour connaître sa taille.
- **Décodage borné.** Le repli restant refuse au-delà de 8192 pixels de côté et 64 Mio d'allocation, ce qui évite les pics mémoire sur un artwork volumineux ou corrompu.

### Vérifications

- `cargo fmt --all --check` : OK.
- `cargo clippy --all-targets --locked -- -D warnings` : OK sur le cœur et l'interface.
- `cargo test --locked` : tests du cœur réussis.
- `cargo test --manifest-path quark-tauri/Cargo.toml --locked` : tests de l'interface réussis.
- Recherche AnkerGames testée en direct (`cyberpunk`) : résultats reçus.

# Quark 2.1.0 — interface plus rapide, jaquettes et recherche multi-sites

### Mes jeux

- **Chargement plus léger.** Les données de boutiques sont construites hors du fil d'interface, les jaquettes visibles sont téléchargées par petits lots et le défilement infini est remplacé par un bouton « Charger plus » explicite.
- **Préférences conservées.** Les filtres, le tri et le choix liste/grille sont mémorisés entre deux lancements.
- **Cartes stables.** Les jaquettes apparaissent sans artefact de rendu ; le menu « ··· » ouvre le dossier, copie le chemin et affiche les détails du jeu.

### Recherche multi-sites

- **Sept sources dans un ordre fixe.** GameBounty arrive en premier, puis AnkerGames, AstralGames, Kryoto, Online-Fix, ZeiGames et scene.cat.
- **Erreurs par source.** Une source indisponible affiche son propre message sans masquer les résultats des autres.
- **Aucun téléchargement.** Quark ouvre les pages dans le navigateur et ne récupère jamais de fichier depuis ces sites.
- **Accès direct.** La fiche de jeu, chaque carte de la boutique et le bouton « Sites de téléchargement » du modal d'ajout ouvrent la recherche avec le nom du jeu.

### Icône de notification

- **Clic qui bascule.** Un clic gauche ouvre Quark si la fenêtre est fermée ou réduite, et referme la fenêtre vers l'icône si elle est déjà visible. La croix garde le même comportement.

### Proton

- **Runtimes redétectés à chaque lancement.** Un Proton installé après le démarrage de Quark est pris en compte sans redémarrer.
- **Runtime supprimé ou renommé.** Le jeu se rabat sur le Proton disponible par défaut avec un avertissement visible, au lieu de refuser de se lancer.

### Interface

- **Surfaces nettes.** Cartes opaques, espacements et rayons partagés, états de survol stables et suppression des flous coûteux.
- **Bibliothèque inchangée.** La liste à gauche et la fiche à droite conservent la disposition proche de Steam.

### Vérifications

- `cargo fmt --check` et `cargo fmt --manifest-path quark-tauri/Cargo.toml --check` : OK.
- `cargo clippy --all-targets --locked -- -D warnings` : OK sur le cœur et l'interface.
- `cargo test --locked` : tests du cœur réussis.
- `cargo test --manifest-path quark-tauri/Cargo.toml --locked` : tests de l'interface réussis.
- Captures réelles sous Xvfb en français et en anglais.
