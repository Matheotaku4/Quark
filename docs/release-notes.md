## Quark 1.4.4 — Interface compacte et temps HowLongToBeat

### Interface

- L'interface compacte devient le réglage par défaut : barre du haut de 54 px, colonne latérale de 252 px, marges et espacements resserrés. Les tailles de police ne changent pas.
- **Apparence → Taille de l'interface** réunit six réglages : interface compacte, largeur de la barre latérale, hauteur des lignes de la bibliothèque, largeur des jaquettes, hauteur de la bannière et largeur du panneau d'aperçu. Un bouton rétablit les valeurs par défaut.
- L'onglet **Accueil** quitte la barre du haut. La ligne **Accueil** reste juste au-dessus de **Filtres**, et la colonne de gauche reste visible en vue grille pour que l'Accueil soit toujours accessible.
- La fiche d'un jeu n'a plus qu'un seul fond : la bannière du jeu floutée et assombrie. Le bandeau séparé en haut de page est supprimé.
- Les boutons **Paramètres**, **Personnaliser**, **Journal** et **Supprimer** passent en haut à droite de la fiche, avec une couleur distincte pour chaque action.
- L'aperçu de la fiche reste côte à côte avec les actualités dès 1024 px de largeur.

### HowLongToBeat

- L'aperçu affiche désormais les temps **HowLongToBeat** du jeu : histoire principale, histoire + extras et complétionniste, avec un lien vers sa page HowLongToBeat.
- Les temps sont récupérés une fois par jeu et mis en cache en mémoire. Aucune clé API n'est demandée.
- Une indisponibilité du service ou l'absence de temps soumis affiche un message discret, sans bloquer la fiche ni les actualités.

### Technique

- Le rendu hors écran conserve des temps HowLongToBeat de démonstration et ne fait aucun appel réseau.
- La densité compacte est réappliquée après un changement de thème, sans modifier les polices.
- La mise en page et les nouveaux réglages ont été vérifiés de 1024 à 1440 px, sans débordement horizontal.
