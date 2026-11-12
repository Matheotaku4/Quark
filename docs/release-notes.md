## Quark 1.4.3 — Paramètres en cartes

### Interface

- Les paramètres d'un jeu passent dans une fenêtre large (960 px au maximum) avec l'en-tête du jeu, les onglets **Général / PlayStation 2 / DLL overrides**, une navigation de section de 220 px, des cartes pour chaque groupe et un pied fixe.
- Chaque groupe de réglages devient une carte avec son titre et son explication : lancement, ma fiche, fiche Steam, Proton, options de lancement, anti-triche, variables d'environnement, données. Les champs, les chemins, les lignes d'option et les lignes de statut sont partagés entre toutes les pages.
- Les paramètres globaux occupent toute la fenêtre : en-tête, navigation de 220 px, contenu en cartes. **Apparence** affiche les thèmes dans une grille de cinq colonnes ; les autres onglets utilisent les mêmes cartes, contrôles segmentés et lignes d'option.
- Les options de lancement sont regroupées en deux colonnes (GameMode, MangoHud, PRIME, umu-launcher), gamescope ouvre un panneau dédié pour la résolution, la limite d'images et le FSR, et la commande wrapper garde sa réinitialisation.
- Les variables d'environnement utilisent une colonne de nom fixe, un `=` aligné, la valeur sur le reste de la ligne et une poubelle en bout de ligne.
- Les contrôles segmentés passent automatiquement en mode compact quand leurs libellés déborderaient ; les cartes **Ma fiche** et **Fiche Steam** s'empilent sous 670 px de contenu.
- La mise en page reste utilisable de 1024 à 1440 px, sans débordement horizontal du modal ni des paramètres.

### Correctifs

- Le pied du modal ne force plus la fenêtre à s'élargir : sa largeur tient compte de ses marges.
- Les contrôles segmentés de deux groupes différents ne partagent plus le même identifiant egui, ce qui évitait un avertissement de widget dupliqué sur la page Apparence.
