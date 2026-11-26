> **Mise à jour (21 novembre 2026)** — L'interface egui a été remplacée par une
> interface **Tauri 2** (`quark-tauri/`) : liste de jeux à gauche, fiche à droite,
> front en HTML/CSS/JS sans framework (`quark-tauri/ui/`). La racine du dépôt est
> désormais une bibliothèque (`src/lib.rs`) sans dépendance graphique, consommée
> par `quark-tauri/src/main.rs`. Les sections ci-dessous décrivent l'ancienne
> interface egui et ne sont plus à jour. Compilation : `cd quark-tauri && cargo run`
> (Rust ≥ 1.88, `libwebkit2gtk-4.1-dev`, `libayatana-appindicator3-dev`).
>
> État du portage : bibliothèque liste/fiche, accueil, boutique, mes jeux,
> réglages (8 sections), succès, médias, actualités, historique, notes, lancement,
> journal, sauvegardes Ludusavi, overlay (`Maj+Retour arrière`, raccourci
> configurable), icône de notification et cinq langues. Vérifié sous Xvfb avec
> `--demo` : `cargo fmt`, `cargo clippy -D warnings` et `cargo test` (313 tests)
> passent sur les deux crates.

# Passation — état du dépôt et suite du travail

Dernière mise à jour : 21 novembre 2026. Ce document s'adresse à l'agent (ou au
développeur) qui reprend Quark. Il décrit ce qui a été fait, ce qui est vérifié,
comment travailler dans cet environnement, et ce qui reste à faire.

## En bref

- L'interface de Quark est **egui / eframe 0.32** et le reste. Une tentative de
  remplacement par Slint a été menée sur une branche isolée, puis **abandonnée à
  la demande du mainteneur** ; la branche a été supprimée en local et sur GitHub.
- `main` n'a **jamais** été modifiée par cette tentative : elle pointe toujours
  sur `a0de6b9` (merge de la PR #45, version 1.8.8).
- Aucun travail n'est en cours, aucune branche locale à terminer. L'arbre de
  travail est propre, à l'exception de `.apt/` (non suivi, voir plus bas).

## Journal de la session

1. Portage complet de l'interface vers Slint (`slint_app.rs`, `ui/main.slint`) :
   overlay X11, tray `ksni`, galerie médias GStreamer, statistiques, succès et
   options par jeu. Deux commits : `c08b38c` puis `fbaaff9`.
2. Contrôle du rendu sous Xvfb, corrections de layout, tests unitaires verts.
3. **Décision du mainteneur : abandon de Slint, retour à egui.** La branche
   `slint-ui` a été supprimée en local et sur GitHub (`fbaaff9` n'existe plus
   nulle part), ainsi que les dossiers de travail `.x11/`, `.ui/` et `.ref-ui/`.
4. Retour sur `main`, compilation, format et tests revérifiés (voir ci-dessous).

Conséquence : **il n'existe aucun code Slint dans le dépôt**. Si l'idée revient
un jour, il faut repartir de zéro ; ce document ne garde que la trace de la
décision.

## État vérifié

| Contrôle | Commande | Résultat |
| --- | --- | --- |
| Compilation | `cargo build` | OK (40 s à froid) |
| Format | `cargo fmt --all -- --check` | OK |
| Tests | `cargo test` | 362 réussis, 0 échec, 4 ignorés |
| Clippy | non relancé cette session | dernier état connu : OK en 1.8.8 |

Les 4 tests ignorés dépendent du réseau ou de GStreamer :
`providers::gog` et `providers::gog_library` (réseau GOG), `media` (réseau
Steam) et `video::tests::lit_une_vraie_bande_annonce_hls` (GStreamer + réseau
Steam).

## Environnement de travail (VM Linux sans écran)

- Pas de serveur X : `Xvfb` et `xvfb-run` sont absents. Pas de `gst-launch-1.0`
  sur le `PATH`, pas de `notify-send`, pas de navigateur.
- `dbus-daemon` et `pkg-config` sont présents.
- **`.apt/` (816 Mo) est un sysroot local non versionné** : fichiers `.pc`,
  bibliothèques dans `.apt/root/usr/lib/x86_64-linux-gnu`, et 254 `.deb` en
  cache dans `.apt/cache/archives` (GStreamer 1.24 avec ses greffons et ses
  outils, X11, GTK…). Il permet de compiler sans accès réseau ; ne pas le
  supprimer, ne pas le committer (il est maintenant dans `.gitignore`).
- Le dossier `.x11/` (binaire Xvfb reconstruit, scripts de capture et
  configuration de démonstration) a été supprimé avec le nettoyage. Pour une
  vraie session graphique, il faut réinstaller Xvfb (paquet `xvfb`, ou binaire
  reconstruit comme lors de la session précédente) avant de lancer l'application.

Compilation et tests, avec le sysroot local :

```bash
export PATH="$HOME/.cargo/bin:$PATH"
export PKG_CONFIG_PATH="$PWD/.apt"
export LIBRARY_PATH="$PWD/.apt/root/usr/lib/x86_64-linux-gnu"
cargo build
cargo test
```

Sans ces variables, la compilation échoue faute de fichiers `*.pc` (X11,
GStreamer).

## Vérifier l'interface ici

L'application rend l'interface **hors écran, sans GPU** : c'est le moyen de
contrôle à utiliser dans cette VM.

```bash
cargo run -- --ui artifacts/home.png --demo --size 1360x860
```

Options utiles (liste complète : `cargo run -- --help`) :

- `--demo` : bibliothèque de démonstration, aucun accès à la configuration
  réelle (tout vit dans un dossier temporaire) ; `--demo-count 0` pour une
  bibliothèque vide.
- `--view list|grid`, `--group status|store|genre|year|none`,
  `--tab library|store|my-games`, `--theme …`, `--lang fr|en|de|es|pt`.
- `--select <jeu>`, `--info`, `--running`, `--marked`, `--home`.
- `--dialog none|add|artwork|exe|scan|settings|settings-defaults|settings-ingame|settings-env|settings-artworks|settings-stats|settings-disk|settings-about|game|game-dll|logs|updates|customize|remove|steam|overlay|install|site-search|command|command-search|command-global|history|saved-views|onboarding|settings-search|detail-home|onboarding-settings|whats-new`.
- `--hover X,Y`, `--click X,Y`, `--button left|right`,
  `--key up|down|left|right|enter|escape`, `--scale`, `--frames`,
  `--providers-demo`, `--providers-error`.

Exemples de rendus de référence :

```bash
cargo run -- --ui artifacts/library-grid.png --demo --view grid
cargo run -- --ui artifacts/game.png --demo --select "Nom du jeu" --dialog game
cargo run -- --ui artifacts/settings-stats.png --demo --dialog settings-stats
cargo run -- --ui artifacts/overlay.png --demo --dialog overlay
```

Les PNG vont dans `artifacts/`, ignoré par Git. Les captures publiées de la
1.8.8 sont dans `docs/images/` (`home`, `library`, `settings`, `store`,
`onboarding`, `whats-new`, `filters`, en `fr` et `en`) : elles servent de
référence visuelle.

Un rendu hors écran ne remplace pas une session réelle : il ne teste ni le tray,
ni la lecture vidéo, ni l'overlay X11.

## Ce que je comptais faire ensuite

1. **Baseline visuelle** : rendre `--ui --demo` sur les écrans clés
   (bibliothèque liste et grille, fiche, réglages, overlay) et les comparer à
   `docs/images/`. `main` n'ayant pas bougé, ce sont des rendus de référence à
   poser avant toute modification, pas une correction.
2. **Clippy** : `cargo clippy --all-targets --locked -- -D warnings` pour
   retrouver la baseline de la 1.8.8.
3. **Vidéo** : le sysroot `.apt` contient GStreamer 1.24. Tenter une lecture
   réelle avec
   `GST_PLUGIN_PATH="$PWD/.apt/root/usr/lib/x86_64-linux-gnu/gstreamer-1.0"` et
   `LD_LIBRARY_PATH` pointé sur `.apt/root/usr/lib/x86_64-linux-gnu`, puis
   exécuter le test ignoré `video::tests::lit_une_vraie_bande_annonce_hls`.
4. **Tray** : il faut D-Bus et un hôte StatusNotifier. Au minimum, lancer
   l'application dans `dbus-run-session` et vérifier qu'elle démarre et
   s'enregistre ; sans hôte, seule la relecture du code est possible.
5. **Ensuite** : aucune fonctionnalité n'est planifiée. Demander au mainteneur
   la prochaine priorité avant d'écrire du code.

## Règles à respecter

- Ne pas committer `.apt/` (816 Mo) ni `target/` (12 Go) ; `.apt/` est désormais
  dans `.gitignore`.
- Ne pas ressusciter la branche Slint : la décision egui est assumée.
- Le dépôt distant ne publie que `main`. Les références locales
  `origin/ui/*` sont périmées (ces branches n'existent plus côté GitHub) : ne
  pas s'en servir comme point de départ.
- `remote.origin.fetch` est **vide** dans ce clone : `git fetch` n'actualise pas
  `origin/main` (il ne remplit que `FETCH_HEAD`, et `origin/main` reste bloqué
  sur `41c664d`). Pour l'état réel : `git ls-remote --heads origin` ; pour lire
  une version distante : `git fetch origin main` puis
  `git show FETCH_HEAD:<fichier>`.
- Les rendus de contrôle vont dans `artifacts/` (ignoré par Git).
- Style du dépôt : messages de commit en français, `cargo fmt --check` avant de
  pousser, `docs/release-notes.md` ne décrit que la version en cours.
