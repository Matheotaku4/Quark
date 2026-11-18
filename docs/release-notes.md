## Quark 1.8.4 — Les packs GOG et les installations Epic vont au bout

### Corrections

- **Un pack Amazon Prime seul est résolu vers le jeu de base.** Le compte GOG peut ne renvoyer que le pack (`1662043288`) sans la fiche du jeu (`2015389384`). Quark interroge maintenant le catalogue GOG, exige une correspondance exacte de titre et une fiche `game` Windows, puis utilise l'identifiant du jeu installable. Si aucun jeu ne correspond, le pack reste visible mais gogdl n'est plus lancé.
- **GOG ne termine plus une installation sans avoir rien téléchargé.** gogdl conserve un manifeste par jeu ; quand il pointe vers un dossier supprimé, gogdl répond « Nothing to do », sort en 0 et ne crée aucun dossier. Comme Heroic, Quark supprime ce manifeste avant chaque installation neuve. Un contrôle public des builds Windows est aussi effectué avant l'appel à gogdl, avec un message explicite si le titre n'est pas téléchargeable.
- **L'erreur « aucun dossier de jeu » est maintenant actionnable.** Elle rappelle de vérifier la connexion GOG et, en dernier recours, d'utiliser l'installateur hors ligne de GOG Galaxy.
- **Une installation Epic terminée n'est plus bloquée sur « Installation… ».** La fiche était encore `Installing` au moment de la fusion ; `merge_epic` conservait cet état transitoire au lieu de prendre l'état `Installed` vérifié par Legendary. La liste vérifiée fait désormais autorité, ce qui marque le jeu installé et l'ajoute à la bibliothèque.

### Vérifications

- `cargo fmt --check` : OK.
- `cargo test --locked` : 338 tests réussis, 2 tests réseau GOG ignorés par défaut.
- `cargo test --locked -- --ignored` : 2 tests réseau GOG réussis (résolution réelle du pack vers `2015389384`, builds Windows du jeu présents et absents du pack).
- `cargo clippy --all-targets --locked -- -D warnings` : OK.
- gogdl 1.4.0 réel : `info 1662043288` sort en erreur « Game doesn't support content system api » ; `info 2015389384` sort en 0 avec `folder_name: "Call of Juarez Gunslinger"` et cinq builds Windows.

### Limites connues

- Aucun compte GOG ou Epic n'est configuré dans cet environnement : le téléchargement complet et l'installation réelle n'ont pas pu être exécutés. Les corrections sont couvertes par les tests unitaires, les tests réseau GOG et les commandes réelles de gogdl.
- La résolution d'un pack dépend d'un titre exact dans le catalogue. Un pack de bundle ou un titre localisé sans correspondance exacte est conservé et bloqué par le contrôle des builds, avec le message explicite prévu.
