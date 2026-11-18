## Quark 1.8.3 — GOG installe le bon jeu, l'accueil reste dans la bibliothèque

### Corrections

- **GOG : un pack ne masque plus le jeu installable.** Amazon Prime expose Call of Juarez: Gunslinger sous deux fiches de même titre : le jeu (`2015389384`), qui possède des builds Windows generation 2, et le pack (`1662043288`), qui n'en a aucun. La déduplication gardait le pack parce que son identifiant était plus petit, puis gogdl échouait avec « Game doesn't support content system api ». Elle conserve désormais la fiche `game` en priorité et ne se rabat sur le pack que s'il est la seule entrée.
- **L'accueil reste dans la bibliothèque.** L'onglet Accueil est retiré de la barre haute. Une entrée **Accueil** se trouve sous **Filtres**, dans la colonne de gauche, et cette colonne garde la liste des jeux visible pendant que le carrousel occupe le panneau central. `Échap` revient à la liste.
- **Un test de régression** reproduit les deux fiches réelles de Call of Juarez: Gunslinger et vérifie que l'identifiant du jeu installable est conservé.

### Vérifications

- `cargo fmt --check` : OK.
- `cargo test --locked` : 334 tests réussis.
- `cargo clippy --all-targets --locked -- -D warnings` : OK.
- Rendu hors écran de l'accueil en français et en anglais : la colonne Filtres/Accueil et la liste restent visibles.

### Limite connue

- Si un compte ne possède que la fiche « pack » et aucune fiche de jeu correspondante, gogdl ne peut toujours pas installer ce contenu sans builds. Quark conserve alors le pack pour ne pas perdre l'entrée de bibliothèque.
