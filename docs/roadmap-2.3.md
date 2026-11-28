# Quark 2.3 — pistes de travail

Document de travail, pas un engagement de version. Les éléments sont classés
par valeur pour l'utilisateur et par risque technique.

## Cadre

- **Interface 2.1.1 figée.** Liste à gauche, fiche à droite, barre supérieure
  et réglages actuels. Les évolutions ci-dessous ajoutent des fonctions dans
  cette structure ; elles ne la remplacent pas.
- **Tauri reste la base.** Priorité à la latence, à la mémoire contenue et au
  démarrage rapide.
- **Pas de télémétrie, pas de téléchargement de jeux, pas d'extension active
  par défaut.** Les manifestes restent des données, sans exécution de code.

## Priorité 1 — finir l'expérience extensions

1. **Santé des extensions dans Réglages → Extensions.** Afficher clairement
   l'état de chaque manifeste : à jour, en erreur, obsolète, dernier succès,
   message lisible. Bouton « tester » qui lance une requête factice.
2. **Activer ou désactiver un site sans retirer l'extension.** Cases à cocher
   par site, stockées dans `config.json`, pour garder un manifeste tout en
   masquant les sources lentes ou hors sujet.
3. **Importer un manifeste autrement que par URL.** Fichier local, collage de
   JSON ou glisser-déposer, avec validation avant ajout et message d'erreur
   précis.
4. **Éditeur guidé de driver générique.** Formulaire pour `json` et `html`
   (URL de recherche, sélecteurs, champs), aperçu du résultat et bouton
   « tester ». Objectif : ajouter un site inconnu sans écrire de code.
5. **Cache de recherche.** Résultats récents par site et par requête, durée de
   vie courte, bouton actualiser. Limiter la concurrence et annuler les
   requêtes quand le dialogue se ferme.
6. **Meilleurs messages d'erreur.** Distinguer DNS, TLS, HTTP, délai, réponse
   vide, sélecteur introuvable et manifeste invalide. Jamais de corps de
   réponse dans l'interface.

## Priorité 1 — performance et fiabilité

7. **Budget de démarrage et de mémoire.** Mesurer le temps jusqu'à la première
   image, la mémoire au repos et le temps d'ouverture d'une fiche. Fixer des
   seuils de non-régression dans la CI.
8. **Chargement différé des vignettes.** Images de boutique et couvertures
   chargées à la demande, cache disque avec purge, placeholder stable et
   réessai contrôlé. Éviter les sauts de mise en page.
9. **Liste virtualisée pour les grandes bibliothèques.** Ne rendre que les
   lignes visibles, sans changer l'apparence de la liste.
10. **Diagnostic exportable.** Un fichier texte ou ZIP contenant version,
    système, thèmes, état des extensions et dernières erreurs, sans secret ni
    chemin personnel complet.
11. **Sauvegarde et restauration des réglages.** Export/import de `config.json`
    avec validation, utile avant une migration ou un changement de machine.
12. **Récupération après mise à jour.** Conserver le binaire précédent,
    proposer un retour arrière et expliquer clairement un échec de signature.

## Priorité 2 — plateforme et distribution

13. **Wayland de première classe.** Raccourcis globaux via le portail du
    compositeur, comportement identique en X11, message explicite si le
    compositeur refuse.
14. **Paquets Linux.** Flatpak et AppImage en plus de l'archive, puis paquet
    `.deb`. Même binaire, mêmes signatures, même mise à jour.
15. **Build ARM64.** Vérifier les dépendances WebKitGTK et publier une archive
    `linux-aarch64` si les tests passent.
16. **Démarrage en tray plus discret.** Réduire la mémoire du processus tray et
    éviter de lancer WebKit quand seule l'icône est demandée.

## Priorité 2 — bibliothèque et compatibilité

17. **Import Heroic / Lutris.** Détecter les jeux et préfixes existants, avec
    confirmation avant toute écriture. Aucune dépendance obligatoire.
18. **Gestionnaire de versions Proton.** Télécharger, vérifier et sélectionner
    une version par jeu, avec repli sur le Proton système.
19. **Détection des doublons.** Proposer de fusionner deux entrées qui pointent
    vers le même exécutable ou le même AppID, sans supprimer automatiquement.
20. **Recherche et filtres sauvegardés.** Enregistrer des vues nommées dans la
    bibliothèque, en réutilisant le format `saved_views` existant.
21. **Steam Cloud explicite.** Si le client Steam est présent, afficher son
    état sans usurper sa synchronisation ; sinon, expliquer pourquoi Quark ne
    le gère pas.

## Priorité 3 — qualité et accessibilité

22. **Tests de bout en bout de l'interface.** Scénarios rejouables pour
    bibliothèque, fiche, boutique, médias et extensions, avec captures de
    référence pour détecter les régressions visuelles.
23. **Tests de robustesse des manifestes.** Fuzzing du parseur, limites de
    taille, JSON malformé, sélecteurs invalides, doublons d'identifiants.
24. **Accessibilité.** Navigation complète au clavier, focus visible, libellés
    ARIA, contraste vérifié sur les 13 thèmes, taille de police indépendante
    du zoom.
25. **Langues supplémentaires.** Ajouter l'italien et le polonais si les
    traductions sont maintenues ; vérifier les libellés avec le test i18n.
26. **Mode sans réseau.** Vérifier que la bibliothèque, les réglages et les
    fiches locales restent utilisables avec le réseau coupé, sans attente.

## À ne pas faire en 2.3

- Remplacer l'interface 2.1.1 ou introduire une seconde interface.
- Réactiver une extension ou un site par défaut.
- Télécharger, extraire ou installer des jeux depuis les sites de recherche.
- Ajouter une télémétrie ou un compte Quark.
- Exécuter du code fourni par une extension.

## Ordre suggéré

1. Extensions : santé, activation par site, test de manifeste, cache.
2. Performance : mesures, images différées, liste virtualisée.
3. Fiabilité : diagnostic, sauvegarde, retour arrière.
4. Distribution : Wayland, Flatpak/AppImage, ARM64.
5. Le reste selon les retours utilisateurs.

## Critères de sortie pour 2.3

- Aucune régression visuelle sur les captures de référence 2.1.1.
- Démarrage et mémoire mesurés, avec seuils tenus sur une machine de test.
- Un manifeste invalide ne peut pas casser l'interface ni bloquer le démarrage.
- La recherche reste vide sans extension et n'effectue aucune requête réseau
  tant qu'aucun manifeste n'est ajouté.
