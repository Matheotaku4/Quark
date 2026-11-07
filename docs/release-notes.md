## Quark 1.4.1 — Barre du haut libellée et pastilles de la Boutique

### Interface

- Sur les fenêtres larges, « Paramètres », « Transferts · n » et « Rechercher ou commande · Ctrl K » ont un libellé dans la barre du haut ; en dessous, les icônes restent.
- Boutique : une rangée de pastilles (Top réductions, Recommandé, puis chaque section) fait défiler la page jusqu'à la section choisie.

### Correctifs

- Les imports `RichText` inutilisés et deux littéraux flottants signalés par le compilateur sont corrigés (ces derniers deviendront une erreur dans une future version de Rust).
- Les tests qui supposent le français prennent désormais le même verrou de langue que ceux d'`i18n` : `game_search` ne peut plus lire « MB » pendant qu'un autre test passe en anglais.
