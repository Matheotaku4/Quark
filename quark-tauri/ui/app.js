"use strict";

const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

const LANGS = { french: "fr", english: "en", spanish: "es", german: "de", portuguese: "pt" };
const LOCALES = { fr: "fr-FR", en: "en-US", es: "es-ES", de: "de-DE", pt: "pt-BR" };

const I18N = {
  fr: {
    "nav.library": "Bibliothèque",
    "nav.settings": "Paramètres",
    "search.global": "Rechercher un jeu…",
    "search.library": "Filtrer la bibliothèque",
    "library.title": "Bibliothèque",
    "library.countOne": "{n} jeu",
    "library.countMany": "{n} jeux",
    "library.add": "Ajouter",
    "filter.all": "Tous",
    "filter.favorites": "Favoris",
    "filter.playing": "En cours",
    "filter.backlog": "À jouer",
    "filter.finished": "Terminés",
    "sort.label": "Trier",
    "sort.name": "Nom",
    "sort.recent": "Joués récemment",
    "sort.playtime": "Temps de jeu",
    "sort.added": "Ajoutés récemment",
    "status.none": "Sans statut",
    "status.backlog": "À jouer",
    "status.playing": "En cours",
    "status.finished": "Terminé",
    "status.dropped": "Abandonné",
    "status.label": "Statut",
    "detail.play": "Jouer",
    "detail.stop": "Arrêter",
    "detail.stopping": "Arrêt…",
    "detail.runningFor": "En cours depuis {t}",
    "detail.favorite": "Favori",
    "detail.unfavorite": "Retirer des favoris",
    "detail.openFolder": "Dossier",
    "detail.edit": "Modifier",
    "detail.remove": "Retirer",
    "detail.more": "Plus d'actions",
    "detail.changeCover": "Changer la jaquette",
    "detail.changeHero": "Changer la bannière",
    "detail.never": "Jamais lancé",
    "detail.noPlaytime": "Aucune session",
    "detail.playingNow": "En cours",
    "detail.exeMissing": "Exécutable introuvable",
    "tab.overview": "Aperçu",
    "tab.activity": "Activité",
    "tab.notes": "Notes",
    "tab.launch": "Lancement",
    "tab.log": "Journal",
    "overview.about": "À propos",
    "overview.details": "Détails",
    "overview.developer": "Développeur",
    "overview.genres": "Genres",
    "overview.released": "Sortie",
    "overview.added": "Ajouté le",
    "overview.playtime": "Temps de jeu",
    "overview.lastPlayed": "Dernière session",
    "overview.exe": "Exécutable",
    "overview.source": "Source",
    "overview.tags": "Tags",
    "overview.noTags": "Aucun tag",
    "overview.addTag": "Ajouter un tag…",
    "overview.installations": "Installations",
    "overview.noDescription": "Aucune description pour ce jeu.",
    "activity.total": "Temps total",
    "activity.sessions": "Sessions",
    "activity.average": "Moyenne",
    "activity.last": "Dernière session",
    "activity.last14": "14 derniers jours",
    "activity.recent": "Sessions récentes",
    "activity.empty": "Aucune session enregistrée.",
    "notes.placeholder": "Notes personnelles, stratégies, objectifs…",
    "notes.saved": "Enregistré",
    "notes.saving": "Enregistrement…",
    "launch.mode": "Mode",
    "launch.anticheat": "Anti-triche",
    "launch.args": "Arguments",
    "launch.env": "Variables d'environnement",
    "launch.options": "Options graphiques",
    "launch.none": "Aucune",
    "launch.prefix": "Préfixe Proton",
    "launch.readonly": "Ces réglages proviennent de la configuration existante de Quark.",
    "launch.native": "Natif Linux",
    "launch.automatic": "Automatique",
    "launch.umu": "umu-launcher",
    "launch.executable": "Exécutable",
    "launch.wrapper": "Wrapper",
    "log.empty": "Aucun journal pour ce jeu.",
    "log.loading": "Chargement…",
    "log.refresh": "Rafraîchir",
    "settings.title": "Paramètres",
    "settings.appearance": "Apparence",
    "settings.theme": "Thème",
    "settings.language": "Langue",
    "settings.compact": "Interface compacte",
    "settings.compactHint": "Réduit les marges et la hauteur des lignes.",
    "settings.hero": "Bannière de fond",
    "settings.heroHint": "Affiche l'image de fond de la fiche quand elle existe.",
    "settings.data": "Données",
    "settings.configPath": "Configuration",
    "settings.dataDir": "Données",
    "settings.openFolder": "Ouvrir le dossier",
    "settings.reload": "Recharger la bibliothèque",
    "settings.about": "À propos",
    "settings.version": "Version",
    "settings.memory": "Mémoire résidente",
    "settings.demo": "Mode démonstration",
    "settings.close": "Fermer",
    "settings.loadError": "Erreur de chargement",
    "add.title": "Ajouter un jeu",
    "add.name": "Nom du jeu",
    "add.exe": "Exécutable",
    "add.exePlaceholder": "Chemin de l'exécutable",
    "add.browse": "Parcourir…",
    "add.args": "Arguments (facultatif)",
    "add.submit": "Ajouter à la bibliothèque",
    "add.cancel": "Annuler",
    "add.exeRequired": "Choisissez un exécutable.",
    "edit.title": "Modifier le jeu",
    "edit.save": "Enregistrer",
    "remove.title": "Retirer le jeu ?",
    "remove.body": "{name} sera retiré de la bibliothèque. Les fichiers du jeu ne sont pas supprimés.",
    "remove.confirm": "Retirer",
    "empty.title": "Sélectionnez un jeu",
    "empty.subtitle": "Choisissez un titre dans la liste pour voir sa fiche, lancer le jeu et retrouver vos notes.",
    "empty.add": "Ajouter un jeu",
    "empty.settings": "Ouvrir les paramètres",
    "list.empty": "Aucun jeu ne correspond à cette recherche.",
    "list.emptyLibrary": "Votre bibliothèque est vide. Ajoutez un premier jeu pour commencer.",
    "toast.launched": "{name} est lancé.",
    "toast.stopped": "{name} est arrêté.",
    "toast.added": "Jeu ajouté à la bibliothèque.",
    "toast.removed": "Jeu retiré de la bibliothèque.",
    "toast.saved": "Modifications enregistrées.",
    "toast.demo": "Mode démonstration : aucune modification n'est écrite.",
    "toast.error": "Erreur",
    "running.one": "{n} jeu en cours",
    "running.many": "{n} jeux en cours",
    "window.minimize": "Réduire",
    "window.maximize": "Agrandir",
    "window.close": "Fermer",
    "source.local": "Locale",
  },
  en: {
    "nav.library": "Library",
    "nav.settings": "Settings",
    "search.global": "Search games…",
    "search.library": "Filter library",
    "library.title": "Library",
    "library.countOne": "{n} game",
    "library.countMany": "{n} games",
    "library.add": "Add",
    "filter.all": "All",
    "filter.favorites": "Favorites",
    "filter.playing": "Playing",
    "filter.backlog": "Backlog",
    "filter.finished": "Finished",
    "sort.label": "Sort",
    "sort.name": "Name",
    "sort.recent": "Recently played",
    "sort.playtime": "Playtime",
    "sort.added": "Recently added",
    "status.none": "No status",
    "status.backlog": "Backlog",
    "status.playing": "Playing",
    "status.finished": "Finished",
    "status.dropped": "Dropped",
    "status.label": "Status",
    "detail.play": "Play",
    "detail.stop": "Stop",
    "detail.stopping": "Stopping…",
    "detail.runningFor": "Running for {t}",
    "detail.favorite": "Favorite",
    "detail.unfavorite": "Remove from favorites",
    "detail.openFolder": "Folder",
    "detail.edit": "Edit",
    "detail.remove": "Remove",
    "detail.more": "More actions",
    "detail.changeCover": "Change cover",
    "detail.changeHero": "Change banner",
    "detail.never": "Never played",
    "detail.noPlaytime": "No sessions",
    "detail.playingNow": "Playing",
    "detail.exeMissing": "Executable not found",
    "tab.overview": "Overview",
    "tab.activity": "Activity",
    "tab.notes": "Notes",
    "tab.launch": "Launch",
    "tab.log": "Log",
    "overview.about": "About",
    "overview.details": "Details",
    "overview.developer": "Developer",
    "overview.genres": "Genres",
    "overview.released": "Released",
    "overview.added": "Added on",
    "overview.playtime": "Playtime",
    "overview.lastPlayed": "Last played",
    "overview.exe": "Executable",
    "overview.source": "Source",
    "overview.tags": "Tags",
    "overview.noTags": "No tags",
    "overview.addTag": "Add a tag…",
    "overview.installations": "Installations",
    "overview.noDescription": "No description for this game.",
    "activity.total": "Total time",
    "activity.sessions": "Sessions",
    "activity.average": "Average",
    "activity.last": "Last session",
    "activity.last14": "Last 14 days",
    "activity.recent": "Recent sessions",
    "activity.empty": "No sessions recorded.",
    "notes.placeholder": "Personal notes, strategies, goals…",
    "notes.saved": "Saved",
    "notes.saving": "Saving…",
    "launch.mode": "Mode",
    "launch.anticheat": "Anti-cheat",
    "launch.args": "Arguments",
    "launch.env": "Environment variables",
    "launch.options": "Graphics options",
    "launch.none": "None",
    "launch.prefix": "Proton prefix",
    "launch.readonly": "These settings come from the existing Quark configuration.",
    "launch.native": "Native Linux",
    "launch.automatic": "Automatic",
    "launch.umu": "umu-launcher",
    "launch.executable": "Executable",
    "launch.wrapper": "Wrapper",
    "log.empty": "No log for this game.",
    "log.loading": "Loading…",
    "log.refresh": "Refresh",
    "settings.title": "Settings",
    "settings.appearance": "Appearance",
    "settings.theme": "Theme",
    "settings.language": "Language",
    "settings.compact": "Compact interface",
    "settings.compactHint": "Reduces margins and row height.",
    "settings.hero": "Background banner",
    "settings.heroHint": "Shows the game banner when one is available.",
    "settings.data": "Data",
    "settings.configPath": "Configuration",
    "settings.dataDir": "Data",
    "settings.openFolder": "Open folder",
    "settings.reload": "Reload library",
    "settings.about": "About",
    "settings.version": "Version",
    "settings.memory": "Resident memory",
    "settings.demo": "Demo mode",
    "settings.close": "Close",
    "settings.loadError": "Loading error",
    "add.title": "Add a game",
    "add.name": "Game name",
    "add.exe": "Executable",
    "add.exePlaceholder": "Executable path",
    "add.browse": "Browse…",
    "add.args": "Arguments (optional)",
    "add.submit": "Add to library",
    "add.cancel": "Cancel",
    "add.exeRequired": "Choose an executable.",
    "edit.title": "Edit game",
    "edit.save": "Save",
    "remove.title": "Remove game?",
    "remove.body": "{name} will be removed from the library. Game files are not deleted.",
    "remove.confirm": "Remove",
    "empty.title": "Select a game",
    "empty.subtitle": "Pick a title from the list to see its page, launch it and find your notes.",
    "empty.add": "Add a game",
    "empty.settings": "Open settings",
    "list.empty": "No game matches this search.",
    "list.emptyLibrary": "Your library is empty. Add a first game to get started.",
    "toast.launched": "{name} is launching.",
    "toast.stopped": "{name} has stopped.",
    "toast.added": "Game added to the library.",
    "toast.removed": "Game removed from the library.",
    "toast.saved": "Changes saved.",
    "toast.demo": "Demo mode: nothing is written.",
    "toast.error": "Error",
    "running.one": "{n} game running",
    "running.many": "{n} games running",
    "window.minimize": "Minimize",
    "window.maximize": "Maximize",
    "window.close": "Close",
    "source.local": "Local",
  },
  es: {
    "nav.library": "Biblioteca",
    "nav.settings": "Ajustes",
    "search.global": "Buscar un juego…",
    "search.library": "Filtrar biblioteca",
    "library.title": "Biblioteca",
    "library.countOne": "{n} juego",
    "library.countMany": "{n} juegos",
    "library.add": "Añadir",
    "filter.all": "Todos",
    "filter.favorites": "Favoritos",
    "filter.playing": "Jugando",
    "filter.backlog": "Pendientes",
    "filter.finished": "Terminados",
    "sort.label": "Ordenar",
    "sort.name": "Nombre",
    "sort.recent": "Jugados recientemente",
    "sort.playtime": "Tiempo de juego",
    "sort.added": "Añadidos recientemente",
    "status.none": "Sin estado",
    "status.backlog": "Pendiente",
    "status.playing": "Jugando",
    "status.finished": "Terminado",
    "status.dropped": "Abandonado",
    "status.label": "Estado",
    "detail.play": "Jugar",
    "detail.stop": "Detener",
    "detail.stopping": "Deteniendo…",
    "detail.runningFor": "Jugando desde {t}",
    "detail.favorite": "Favorito",
    "detail.unfavorite": "Quitar de favoritos",
    "detail.openFolder": "Carpeta",
    "detail.edit": "Editar",
    "detail.remove": "Quitar",
    "detail.more": "Más acciones",
    "detail.changeCover": "Cambiar portada",
    "detail.changeHero": "Cambiar banner",
    "detail.never": "Nunca jugado",
    "detail.noPlaytime": "Sin sesiones",
    "detail.playingNow": "Jugando",
    "detail.exeMissing": "Ejecutable no encontrado",
    "tab.overview": "Resumen",
    "tab.activity": "Actividad",
    "tab.notes": "Notas",
    "tab.launch": "Lanzamiento",
    "tab.log": "Registro",
    "overview.about": "Acerca de",
    "overview.details": "Detalles",
    "overview.developer": "Desarrollador",
    "overview.genres": "Géneros",
    "overview.released": "Lanzamiento",
    "overview.added": "Añadido el",
    "overview.playtime": "Tiempo de juego",
    "overview.lastPlayed": "Última sesión",
    "overview.exe": "Ejecutable",
    "overview.source": "Fuente",
    "overview.tags": "Etiquetas",
    "overview.noTags": "Sin etiquetas",
    "overview.addTag": "Añadir etiqueta…",
    "overview.installations": "Instalaciones",
    "overview.noDescription": "Sin descripción para este juego.",
    "activity.total": "Tiempo total",
    "activity.sessions": "Sesiones",
    "activity.average": "Promedio",
    "activity.last": "Última sesión",
    "activity.last14": "Últimos 14 días",
    "activity.recent": "Sesiones recientes",
    "activity.empty": "Sin sesiones registradas.",
    "notes.placeholder": "Notas personales, estrategias, objetivos…",
    "notes.saved": "Guardado",
    "notes.saving": "Guardando…",
    "launch.mode": "Modo",
    "launch.anticheat": "Antitrampas",
    "launch.args": "Argumentos",
    "launch.env": "Variables de entorno",
    "launch.options": "Opciones gráficas",
    "launch.none": "Ninguna",
    "launch.prefix": "Prefijo Proton",
    "launch.readonly": "Estos ajustes provienen de la configuración existente de Quark.",
    "launch.native": "Nativo Linux",
    "launch.automatic": "Automático",
    "launch.umu": "umu-launcher",
    "launch.executable": "Ejecutable",
    "launch.wrapper": "Wrapper",
    "log.empty": "Sin registro para este juego.",
    "log.loading": "Cargando…",
    "log.refresh": "Actualizar",
    "settings.title": "Ajustes",
    "settings.appearance": "Apariencia",
    "settings.theme": "Tema",
    "settings.language": "Idioma",
    "settings.compact": "Interfaz compacta",
    "settings.compactHint": "Reduce los márgenes y la altura de las filas.",
    "settings.hero": "Banner de fondo",
    "settings.heroHint": "Muestra el banner del juego cuando existe.",
    "settings.data": "Datos",
    "settings.configPath": "Configuración",
    "settings.dataDir": "Datos",
    "settings.openFolder": "Abrir carpeta",
    "settings.reload": "Recargar biblioteca",
    "settings.about": "Acerca de",
    "settings.version": "Versión",
    "settings.memory": "Memoria residente",
    "settings.demo": "Modo demostración",
    "settings.close": "Cerrar",
    "settings.loadError": "Error de carga",
    "add.title": "Añadir un juego",
    "add.name": "Nombre del juego",
    "add.exe": "Ejecutable",
    "add.exePlaceholder": "Ruta del ejecutable",
    "add.browse": "Examinar…",
    "add.args": "Argumentos (opcional)",
    "add.submit": "Añadir a la biblioteca",
    "add.cancel": "Cancelar",
    "add.exeRequired": "Elige un ejecutable.",
    "edit.title": "Editar juego",
    "edit.save": "Guardar",
    "remove.title": "¿Quitar el juego?",
    "remove.body": "{name} se quitará de la biblioteca. Los archivos del juego no se eliminan.",
    "remove.confirm": "Quitar",
    "empty.title": "Selecciona un juego",
    "empty.subtitle": "Elige un título de la lista para ver su ficha, iniciarlo y encontrar tus notas.",
    "empty.add": "Añadir un juego",
    "empty.settings": "Abrir ajustes",
    "list.empty": "Ningún juego coincide con esta búsqueda.",
    "list.emptyLibrary": "Tu biblioteca está vacía. Añade un primer juego para empezar.",
    "toast.launched": "{name} se está lanzando.",
    "toast.stopped": "{name} se ha detenido.",
    "toast.added": "Juego añadido a la biblioteca.",
    "toast.removed": "Juego quitado de la biblioteca.",
    "toast.saved": "Cambios guardados.",
    "toast.demo": "Modo demostración: no se escribe nada.",
    "toast.error": "Error",
    "running.one": "{n} juego en curso",
    "running.many": "{n} juegos en curso",
    "window.minimize": "Minimizar",
    "window.maximize": "Maximizar",
    "window.close": "Cerrar",
    "source.local": "Local",
  },
  de: {
    "nav.library": "Bibliothek",
    "nav.settings": "Einstellungen",
    "search.global": "Spiele suchen…",
    "search.library": "Bibliothek filtern",
    "library.title": "Bibliothek",
    "library.countOne": "{n} Spiel",
    "library.countMany": "{n} Spiele",
    "library.add": "Hinzufügen",
    "filter.all": "Alle",
    "filter.favorites": "Favoriten",
    "filter.playing": "Spielend",
    "filter.backlog": "Wartend",
    "filter.finished": "Abgeschlossen",
    "sort.label": "Sortieren",
    "sort.name": "Name",
    "sort.recent": "Zuletzt gespielt",
    "sort.playtime": "Spielzeit",
    "sort.added": "Zuletzt hinzugefügt",
    "status.none": "Kein Status",
    "status.backlog": "Wartend",
    "status.playing": "Spielend",
    "status.finished": "Abgeschlossen",
    "status.dropped": "Abgebrochen",
    "status.label": "Status",
    "detail.play": "Spielen",
    "detail.stop": "Beenden",
    "detail.stopping": "Wird beendet…",
    "detail.runningFor": "Läuft seit {t}",
    "detail.favorite": "Favorit",
    "detail.unfavorite": "Aus Favoriten entfernen",
    "detail.openFolder": "Ordner",
    "detail.edit": "Bearbeiten",
    "detail.remove": "Entfernen",
    "detail.more": "Mehr Aktionen",
    "detail.changeCover": "Cover ändern",
    "detail.changeHero": "Banner ändern",
    "detail.never": "Nie gespielt",
    "detail.noPlaytime": "Keine Sitzungen",
    "detail.playingNow": "Läuft",
    "detail.exeMissing": "Ausführbare Datei fehlt",
    "tab.overview": "Übersicht",
    "tab.activity": "Aktivität",
    "tab.notes": "Notizen",
    "tab.launch": "Start",
    "tab.log": "Protokoll",
    "overview.about": "Über",
    "overview.details": "Details",
    "overview.developer": "Entwickler",
    "overview.genres": "Genres",
    "overview.released": "Erschienen",
    "overview.added": "Hinzugefügt am",
    "overview.playtime": "Spielzeit",
    "overview.lastPlayed": "Zuletzt gespielt",
    "overview.exe": "Ausführbare Datei",
    "overview.source": "Quelle",
    "overview.tags": "Tags",
    "overview.noTags": "Keine Tags",
    "overview.addTag": "Tag hinzufügen…",
    "overview.installations": "Installationen",
    "overview.noDescription": "Keine Beschreibung für dieses Spiel.",
    "activity.total": "Gesamtzeit",
    "activity.sessions": "Sitzungen",
    "activity.average": "Durchschnitt",
    "activity.last": "Letzte Sitzung",
    "activity.last14": "Letzte 14 Tage",
    "activity.recent": "Letzte Sitzungen",
    "activity.empty": "Keine Sitzungen aufgezeichnet.",
    "notes.placeholder": "Persönliche Notizen, Strategien, Ziele…",
    "notes.saved": "Gespeichert",
    "notes.saving": "Wird gespeichert…",
    "launch.mode": "Modus",
    "launch.anticheat": "Anti-Cheat",
    "launch.args": "Argumente",
    "launch.env": "Umgebungsvariablen",
    "launch.options": "Grafikoptionen",
    "launch.none": "Keine",
    "launch.prefix": "Proton-Präfix",
    "launch.readonly": "Diese Einstellungen stammen aus der bestehenden Quark-Konfiguration.",
    "launch.native": "Nativ Linux",
    "launch.automatic": "Automatisch",
    "launch.umu": "umu-launcher",
    "launch.executable": "Ausführbare Datei",
    "launch.wrapper": "Wrapper",
    "log.empty": "Kein Protokoll für dieses Spiel.",
    "log.loading": "Wird geladen…",
    "log.refresh": "Aktualisieren",
    "settings.title": "Einstellungen",
    "settings.appearance": "Erscheinungsbild",
    "settings.theme": "Design",
    "settings.language": "Sprache",
    "settings.compact": "Kompakte Oberfläche",
    "settings.compactHint": "Verringert Abstände und Zeilenhöhe.",
    "settings.hero": "Hintergrundbanner",
    "settings.heroHint": "Zeigt das Spiele-Banner, wenn vorhanden.",
    "settings.data": "Daten",
    "settings.configPath": "Konfiguration",
    "settings.dataDir": "Daten",
    "settings.openFolder": "Ordner öffnen",
    "settings.reload": "Bibliothek neu laden",
    "settings.about": "Über",
    "settings.version": "Version",
    "settings.memory": "Arbeitsspeicher",
    "settings.demo": "Demo-Modus",
    "settings.close": "Schließen",
    "settings.loadError": "Ladefehler",
    "add.title": "Spiel hinzufügen",
    "add.name": "Spielname",
    "add.exe": "Ausführbare Datei",
    "add.exePlaceholder": "Pfad zur ausführbaren Datei",
    "add.browse": "Durchsuchen…",
    "add.args": "Argumente (optional)",
    "add.submit": "Zur Bibliothek hinzufügen",
    "add.cancel": "Abbrechen",
    "add.exeRequired": "Wähle eine ausführbare Datei.",
    "edit.title": "Spiel bearbeiten",
    "edit.save": "Speichern",
    "remove.title": "Spiel entfernen?",
    "remove.body": "{name} wird aus der Bibliothek entfernt. Die Spieldateien werden nicht gelöscht.",
    "remove.confirm": "Entfernen",
    "empty.title": "Wähle ein Spiel",
    "empty.subtitle": "Wähle einen Titel aus der Liste, um die Seite zu sehen, das Spiel zu starten und Notizen zu finden.",
    "empty.add": "Spiel hinzufügen",
    "empty.settings": "Einstellungen öffnen",
    "list.empty": "Kein Spiel passt zu dieser Suche.",
    "list.emptyLibrary": "Deine Bibliothek ist leer. Füge ein erstes Spiel hinzu.",
    "toast.launched": "{name} wird gestartet.",
    "toast.stopped": "{name} wurde beendet.",
    "toast.added": "Spiel zur Bibliothek hinzugefügt.",
    "toast.removed": "Spiel aus der Bibliothek entfernt.",
    "toast.saved": "Änderungen gespeichert.",
    "toast.demo": "Demo-Modus: Es wird nichts geschrieben.",
    "toast.error": "Fehler",
    "running.one": "{n} Spiel läuft",
    "running.many": "{n} Spiele laufen",
    "window.minimize": "Minimieren",
    "window.maximize": "Maximieren",
    "window.close": "Schließen",
    "source.local": "Lokal",
  },
  pt: {
    "nav.library": "Biblioteca",
    "nav.settings": "Configurações",
    "search.global": "Pesquisar jogos…",
    "search.library": "Filtrar biblioteca",
    "library.title": "Biblioteca",
    "library.countOne": "{n} jogo",
    "library.countMany": "{n} jogos",
    "library.add": "Adicionar",
    "filter.all": "Todos",
    "filter.favorites": "Favoritos",
    "filter.playing": "Jogando",
    "filter.backlog": "Pendentes",
    "filter.finished": "Terminados",
    "sort.label": "Ordenar",
    "sort.name": "Nome",
    "sort.recent": "Jogados recentemente",
    "sort.playtime": "Tempo de jogo",
    "sort.added": "Adicionados recentemente",
    "status.none": "Sem estado",
    "status.backlog": "Pendente",
    "status.playing": "Jogando",
    "status.finished": "Terminado",
    "status.dropped": "Abandonado",
    "status.label": "Estado",
    "detail.play": "Jogar",
    "detail.stop": "Parar",
    "detail.stopping": "Parando…",
    "detail.runningFor": "Jogando há {t}",
    "detail.favorite": "Favorito",
    "detail.unfavorite": "Remover dos favoritos",
    "detail.openFolder": "Pasta",
    "detail.edit": "Editar",
    "detail.remove": "Remover",
    "detail.more": "Mais ações",
    "detail.changeCover": "Alterar capa",
    "detail.changeHero": "Alterar banner",
    "detail.never": "Nunca jogado",
    "detail.noPlaytime": "Sem sessões",
    "detail.playingNow": "Jogando",
    "detail.exeMissing": "Executável não encontrado",
    "tab.overview": "Visão geral",
    "tab.activity": "Atividade",
    "tab.notes": "Notas",
    "tab.launch": "Inicialização",
    "tab.log": "Registro",
    "overview.about": "Sobre",
    "overview.details": "Detalhes",
    "overview.developer": "Desenvolvedor",
    "overview.genres": "Gêneros",
    "overview.released": "Lançamento",
    "overview.added": "Adicionado em",
    "overview.playtime": "Tempo de jogo",
    "overview.lastPlayed": "Última sessão",
    "overview.exe": "Executável",
    "overview.source": "Fonte",
    "overview.tags": "Tags",
    "overview.noTags": "Sem tags",
    "overview.addTag": "Adicionar tag…",
    "overview.installations": "Instalações",
    "overview.noDescription": "Sem descrição para este jogo.",
    "activity.total": "Tempo total",
    "activity.sessions": "Sessões",
    "activity.average": "Média",
    "activity.last": "Última sessão",
    "activity.last14": "Últimos 14 dias",
    "activity.recent": "Sessões recentes",
    "activity.empty": "Nenhuma sessão registrada.",
    "notes.placeholder": "Notas pessoais, estratégias, objetivos…",
    "notes.saved": "Salvo",
    "notes.saving": "Salvando…",
    "launch.mode": "Modo",
    "launch.anticheat": "Anti-cheat",
    "launch.args": "Argumentos",
    "launch.env": "Variáveis de ambiente",
    "launch.options": "Opções gráficas",
    "launch.none": "Nenhuma",
    "launch.prefix": "Prefixo Proton",
    "launch.readonly": "Estas configurações vêm da configuração existente do Quark.",
    "launch.native": "Nativo Linux",
    "launch.automatic": "Automático",
    "launch.umu": "umu-launcher",
    "launch.executable": "Executável",
    "launch.wrapper": "Wrapper",
    "log.empty": "Nenhum registro para este jogo.",
    "log.loading": "Carregando…",
    "log.refresh": "Atualizar",
    "settings.title": "Configurações",
    "settings.appearance": "Aparência",
    "settings.theme": "Tema",
    "settings.language": "Idioma",
    "settings.compact": "Interface compacta",
    "settings.compactHint": "Reduz margens e altura das linhas.",
    "settings.hero": "Banner de fundo",
    "settings.heroHint": "Mostra o banner do jogo quando existe.",
    "settings.data": "Dados",
    "settings.configPath": "Configuração",
    "settings.dataDir": "Dados",
    "settings.openFolder": "Abrir pasta",
    "settings.reload": "Recarregar biblioteca",
    "settings.about": "Sobre",
    "settings.version": "Versão",
    "settings.memory": "Memória residente",
    "settings.demo": "Modo demonstração",
    "settings.close": "Fechar",
    "settings.loadError": "Erro de carregamento",
    "add.title": "Adicionar jogo",
    "add.name": "Nome do jogo",
    "add.exe": "Executável",
    "add.exePlaceholder": "Caminho do executável",
    "add.browse": "Procurar…",
    "add.args": "Argumentos (opcional)",
    "add.submit": "Adicionar à biblioteca",
    "add.cancel": "Cancelar",
    "add.exeRequired": "Escolha um executável.",
    "edit.title": "Editar jogo",
    "edit.save": "Salvar",
    "remove.title": "Remover jogo?",
    "remove.body": "{name} será removido da biblioteca. Os arquivos do jogo não são apagados.",
    "remove.confirm": "Remover",
    "empty.title": "Selecione um jogo",
    "empty.subtitle": "Escolha um título na lista para ver a ficha, iniciar o jogo e encontrar suas notas.",
    "empty.add": "Adicionar jogo",
    "empty.settings": "Abrir configurações",
    "list.empty": "Nenhum jogo corresponde a esta pesquisa.",
    "list.emptyLibrary": "Sua biblioteca está vazia. Adicione um primeiro jogo para começar.",
    "toast.launched": "{name} está iniciando.",
    "toast.stopped": "{name} foi parado.",
    "toast.added": "Jogo adicionado à biblioteca.",
    "toast.removed": "Jogo removido da biblioteca.",
    "toast.saved": "Alterações salvas.",
    "toast.demo": "Modo demonstração: nada é gravado.",
    "toast.error": "Erro",
    "running.one": "{n} jogo em execução",
    "running.many": "{n} jogos em execução",
    "window.minimize": "Minimizar",
    "window.maximize": "Maximizar",
    "window.close": "Fechar",
    "source.local": "Local",
  },
};

const STATUS_KEYS = {
  none: "status.none",
  backlog: "status.backlog",
  playing: "status.playing",
  finished: "status.finished",
  dropped: "status.dropped",
};

const STATUS_ORDER = ["none", "backlog", "playing", "finished", "dropped"];
const STATUS_COLORS = {
  none: "#6a7076",
  backlog: "#5a8fd6",
  playing: "#4cb06a",
  finished: "#b4c7dc",
  dropped: "#c96a5a",
};

const ICONS = (() => {
  const svg = (body, filled = false) =>
    `<svg viewBox="0 0 24 24" ${filled ? 'fill="currentColor" stroke="none"' : 'fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"'}>${body}</svg>`;
  return {
    library: svg('<path d="m16 6 4 14"/><path d="M12 6v14"/><path d="M8 8v12"/><path d="M4 4v16"/>'),
    settings: svg('<path d="M4 6h9M17 6h3M4 12h4M12 12h8M4 18h11M19 18h1"/><circle cx="15" cy="6" r="2"/><circle cx="10" cy="12" r="2"/><circle cx="17" cy="18" r="2"/>'),
    plus: svg('<path d="M12 5v14M5 12h14"/>'),
    minus: svg('<path d="M5 12h14"/>'),
    maximize: svg('<rect x="5" y="5" width="14" height="14" rx="1.5"/>'),
    close: svg('<path d="M6 6l12 12M18 6 6 18"/>'),
    play: svg('<path d="M8 5v14l11-7z"/>', true),
    stop: svg('<rect x="6" y="6" width="12" height="12" rx="1.5"/>', true),
    star: svg('<path d="M12 3.5l2.7 5.6 6.1.9-4.4 4.3 1 6.1-5.4-2.9-5.4 2.9 1-6.1-4.4-4.3 6.1-.9z"/>'),
    starFilled: svg('<path d="M12 3.5l2.7 5.6 6.1.9-4.4 4.3 1 6.1-5.4-2.9-5.4 2.9 1-6.1-4.4-4.3 6.1-.9z"/>', true),
    dots: svg('<circle cx="5" cy="12" r="1.5"/><circle cx="12" cy="12" r="1.5"/><circle cx="19" cy="12" r="1.5"/>', true),
    folder: svg('<path d="M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v9a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/>'),
    edit: svg('<path d="M12 20h9"/><path d="M16.5 3.5a2.1 2.1 0 0 1 3 3L7 19l-4 1 1-4z"/>'),
    trash: svg('<path d="M3 6h18M8 6V4h8v2M6 6l1 14h10l1-14M10 11v6M14 11v6"/>'),
    image: svg('<rect x="3" y="4" width="18" height="16" rx="2"/><circle cx="8.5" cy="9.5" r="1.5"/><path d="m21 15-5-5L5 21"/>'),
    refresh: svg('<path d="M21 12a9 9 0 1 1-2.6-6.4M21 3v6h-6"/>'),
    check: svg('<circle cx="12" cy="12" r="9"/><path d="m8 12 3 3 5-6"/>'),
    alert: svg('<path d="M12 3 2 21h20L12 3z"/><path d="M12 9v5M12 17h.01"/>'),
    x: svg('<path d="M6 6l12 12M18 6 6 18"/>'),
    monitor: svg('<rect x="2" y="3" width="20" height="14" rx="2"/><path d="M8 21h8M12 17v4"/>'),
  };
})();

const app = {
  data: null,
  lang: "fr",
  selectedId: null,
  filter: "all",
  query: "",
  sort: "name",
  tab: "overview",
  running: new Map(),
  notesTimer: null,
  logRequest: 0,
  menuOpen: false,
  modalOpen: false,
  editId: null,
  pollBusy: false,
};

const $ = (selector, root = document) => root.querySelector(selector);
const $$ = (selector, root = document) => Array.from(root.querySelectorAll(selector));

function esc(value) {
  return String(value == null ? "" : value).replace(/[&<>"']/g, (character) => ({
    "&": "&amp;",
    "<": "&lt;",
    ">": "&gt;",
    '"': "&quot;",
    "'": "&#39;",
  })[character]);
}

function t(key, vars) {
  const dict = I18N[app.lang] || I18N.fr;
  let text = dict[key];
  if (text == null) text = I18N.fr[key];
  if (text == null) text = key;
  if (vars) {
    for (const [name, value] of Object.entries(vars)) {
      text = text.split(`{${name}}`).join(String(value));
    }
  }
  return text;
}

function locale() {
  return LOCALES[app.lang] || LOCALES.fr;
}

function num(value, digits = 1) {
  return new Intl.NumberFormat(locale(), { maximumFractionDigits: digits }).format(value);
}

function fmtPlaytime(secs) {
  if (!secs || secs < 60) return t("detail.noPlaytime");
  if (secs < 3600) return `${Math.floor(secs / 60)} min`;
  return `${num(secs / 3600, 1)} h`;
}

function fmtDuration(secs) {
  const total = Math.max(0, Math.floor(secs));
  const hours = Math.floor(total / 3600);
  const minutes = Math.floor((total % 3600) / 60);
  const seconds = total % 60;
  const pad = (value) => String(value).padStart(2, "0");
  return hours > 0 ? `${hours}:${pad(minutes)}:${pad(seconds)}` : `${minutes}:${pad(seconds)}`;
}

function fmtRelative(timestamp) {
  if (!timestamp) return t("detail.never");
  const diff = timestamp - Math.floor(Date.now() / 1000);
  const abs = Math.abs(diff);
  const formatter = new Intl.RelativeTimeFormat(locale(), { numeric: "auto" });
  if (abs < 60) return formatter.format(Math.round(diff), "second");
  if (abs < 3600) return formatter.format(Math.round(diff / 60), "minute");
  if (abs < 86400) return formatter.format(Math.round(diff / 3600), "hour");
  if (abs < 2592000) return formatter.format(Math.round(diff / 86400), "day");
  if (abs < 31536000) return formatter.format(Math.round(diff / 2592000), "month");
  return formatter.format(Math.round(diff / 31536000), "year");
}

function fmtDate(timestamp) {
  if (!timestamp) return "—";
  return new Intl.DateTimeFormat(locale(), { day: "numeric", month: "long", year: "numeric" }).format(new Date(timestamp * 1000));
}

function fmtDay(timestamp) {
  return new Intl.DateTimeFormat(locale(), { weekday: "short" }).format(new Date(timestamp * 1000));
}

function normalize(text) {
  return String(text || "")
    .toLowerCase()
    .normalize("NFD")
    .replace(/[\u0300-\u036f]/g, "");
}

function currentGame() {
  if (!app.data || !app.selectedId) return null;
  return app.data.games.find((game) => game.id === app.selectedId) || null;
}

function setGame(game) {
  if (!app.data || !game) return;
  const index = app.data.games.findIndex((candidate) => candidate.id === game.id);
  if (index >= 0) app.data.games[index] = game;
  else app.data.games.push(game);
}

function statusLabel(id) {
  return t(STATUS_KEYS[id] || "status.none");
}

function sourceLabel(game) {
  return game.provider === "local" ? t("source.local") : game.sourceLabel;
}

function hexToRgba(hex, alpha) {
  const value = String(hex || "#2e9bff").replace("#", "");
  const full = value.length === 3 ? value.split("").map((part) => part + part).join("") : value;
  const number = parseInt(full, 16);
  return `rgba(${(number >> 16) & 255}, ${(number >> 8) & 255}, ${number & 255}, ${alpha})`;
}

function onAccent(hex) {
  const value = String(hex || "#2e9bff").replace("#", "");
  const full = value.length === 3 ? value.split("").map((part) => part + part).join("") : value;
  const number = parseInt(full, 16);
  const r = (number >> 16) & 255;
  const g = (number >> 8) & 255;
  const b = number & 255;
  const luminance = 0.299 * r + 0.587 * g + 0.114 * b;
  return luminance > 150 ? "#0a0a0a" : "#ffffff";
}

function applyTheme(themeId) {
  const themes = app.data.themes || [];
  const theme = themes.find((candidate) => candidate.id === themeId) || themes[0];
  if (!theme) return;
  for (const [name, value] of Object.entries(theme.vars)) {
    document.documentElement.style.setProperty(`--${name}`, value);
  }
  document.documentElement.style.setProperty("--accent-soft", hexToRgba(theme.vars.accent, 0.15));
  document.documentElement.style.setProperty("--on-accent", onAccent(theme.vars.accent));
  document.documentElement.dataset.theme = theme.id;
}

function visibleGames() {
  const query = normalize(app.query.trim());
  const games = app.data.games.filter((game) => {
    if (app.filter === "favorites" && !game.favorite) return false;
    if (["playing", "backlog", "finished"].includes(app.filter) && game.status !== app.filter) return false;
    if (!query) return true;
    const haystack = normalize([game.name, game.developer, (game.genres || []).join(" "), (game.tags || []).join(" ")].join(" "));
    return haystack.includes(query);
  });
  games.sort((left, right) => {
    let result = 0;
    if (app.sort === "recent") result = (right.lastPlayed || 0) - (left.lastPlayed || 0);
    else if (app.sort === "playtime") result = (right.playtimeSecs || 0) - (left.playtimeSecs || 0);
    else if (app.sort === "added") result = (right.addedAt || 0) - (left.addedAt || 0);
    if (result === 0) result = left.name.localeCompare(right.name, locale());
    return result;
  });
  return games;
}

function renderFilters() {
  const filters = [
    ["all", "filter.all"],
    ["favorites", "filter.favorites"],
    ["playing", "filter.playing"],
    ["backlog", "filter.backlog"],
    ["finished", "filter.finished"],
  ];
  const chips = $("#status-filters");
  if (chips) {
    chips.innerHTML = filters
      .map(
        ([id, key]) =>
          `<button class="chip ${app.filter === id ? "active" : ""}" data-filter="${id}" role="tab" aria-selected="${app.filter === id}">${t(key)}</button>`,
      )
      .join("");
  }
  const sorts = [
    ["name", "sort.name"],
    ["recent", "sort.recent"],
    ["playtime", "sort.playtime"],
    ["added", "sort.added"],
  ];
  const select = $("#sort-select");
  if (select) {
    select.innerHTML = sorts
      .map(([id, key]) => `<option value="${id}" ${app.sort === id ? "selected" : ""}>${t(key)}</option>`)
      .join("");
  }
}

function rowTemplate(game) {
  const running = app.running.has(game.id);
  const meta = [running ? t("detail.playingNow") : statusLabel(game.status), game.playtimeSecs ? fmtPlaytime(game.playtimeSecs) : ""]
    .filter(Boolean)
    .join('<span class="sep">·</span>');
  const cover = game.coverUrl
    ? `<img src="${esc(game.coverUrl)}" alt="" loading="lazy">`
    : `<span class="row-monogram" style="--hue:${game.hue}">${esc(game.monogram)}</span>`;
  return `<button class="game-row ${app.selectedId === game.id ? "selected" : ""} ${running ? "running" : ""}" data-id="${esc(game.id)}" role="option" aria-selected="${app.selectedId === game.id}" style="--status:${game.statusColor}">
    <span class="row-cover">${cover}</span>
    <span class="row-main"><span class="row-name">${esc(game.name)}</span><span class="row-meta">${meta}</span></span>
    <span class="row-dot"></span>
  </button>`;
}

function renderList() {
  if (!app.data) return;
  const games = visibleGames();
  const total = app.data.games.length;
  const count = $("#library-count");
  if (count) {
    if (games.length === total) {
      count.textContent = t(games.length === 1 ? "library.countOne" : "library.countMany", { n: games.length });
    } else {
      count.textContent = `${games.length} / ${total}`;
    }
  }
  const list = $("#game-list");
  if (list) list.innerHTML = games.map(rowTemplate).join("");
  const empty = $("#list-empty");
  if (empty) {
    const message = total === 0 ? t("list.emptyLibrary") : t("list.empty");
    empty.textContent = message;
    empty.classList.toggle("hidden", games.length > 0);
  }
  if (games.length > 0 && !games.some((game) => game.id === app.selectedId)) {
    selectGame(games[0].id);
  }
}

function selectGame(id, scroll = false) {
  if (!id) return;
  const changed = app.selectedId !== id;
  app.selectedId = id;
  if (changed) app.tab = "overview";
  $$(".game-row").forEach((row) => {
    const selected = row.dataset.id === id;
    row.classList.toggle("selected", selected);
    row.setAttribute("aria-selected", String(selected));
  });
  renderDetail();
  if (scroll) {
    const row = $(`.game-row[data-id="${CSS.escape(id)}"]`);
    if (row) row.scrollIntoView({ block: "nearest" });
  }
}

function selectInitial() {
  const games = visibleGames();
  const running = games.find((game) => app.running.has(game.id));
  if (running) return selectGame(running.id);
  if (games.length > 0) return selectGame(games[0].id);
  app.selectedId = null;
  renderDetail();
}

function heroTemplate(game, running) {
  const eyebrow = [sourceLabel(game), ...(game.genres || []).slice(0, 2)].filter(Boolean);
  const badges = `<span class="badge">${esc(sourceLabel(game))}</span>${running ? `<span class="badge live">${t("detail.playingNow")}</span>` : ""}`;
  const meta = [game.developer, game.released, game.playtimeSecs ? fmtPlaytime(game.playtimeSecs) : t("detail.noPlaytime"), fmtRelative(game.lastPlayed)]
    .filter(Boolean)
    .map((part) => esc(part))
    .join('<span class="dot"></span>');
  const title = game.logoUrl
    ? `<img class="hero-logo" src="${esc(game.logoUrl)}" alt="${esc(game.name)}">`
    : `<h2 class="hero-title">${esc(game.name)}</h2>`;
  const showBanner = app.data.settings.heroBackground;
  const background = showBanner && game.heroUrl
    ? `<div class="hero-bg" style="background-image:url('${esc(game.heroUrl)}')"></div>`
    : `<div class="hero-bg hero-fallback"></div>`;
  return `<section class="hero" style="--hue:${game.hue}">
    ${background}
    <div class="hero-shade"></div>
    <div class="hero-top">
      <div class="hero-badges">${badges}</div>
      <div class="hero-actions">
        <button class="icon-btn ${game.favorite ? "is-fav" : ""}" data-action="favorite" title="${game.favorite ? t("detail.unfavorite") : t("detail.favorite")}" aria-label="${game.favorite ? t("detail.unfavorite") : t("detail.favorite")}">${game.favorite ? ICONS.starFilled : ICONS.star}</button>
        <button class="icon-btn" data-action="menu" title="${t("detail.more")}" aria-label="${t("detail.more")}">${ICONS.dots}</button>
        <div class="game-menu hidden" id="game-menu">
          <button class="menu-item" data-action="change-cover">${ICONS.image}<span>${t("detail.changeCover")}</span></button>
          <button class="menu-item" data-action="change-hero">${ICONS.image}<span>${t("detail.changeHero")}</span></button>
          <button class="menu-item" data-action="folder">${ICONS.folder}<span>${t("detail.openFolder")}</span></button>
          <div class="menu-sep"></div>
          <button class="menu-item danger" data-action="remove">${ICONS.trash}<span>${t("detail.remove")}</span></button>
        </div>
      </div>
    </div>
    <div class="hero-body">
      <div class="hero-eyebrow">${eyebrow.map((part) => `<span>${esc(part)}</span>`).join('<span class="dot"></span>')}</div>
      ${title}
      <div class="hero-meta">${meta}</div>
    </div>
  </section>`;
}

function actionBarTemplate(game, running) {
  const stopping = running && running.stopping;
  const playButton = running
    ? `<button class="btn btn-stop" data-action="stop">${ICONS.stop}<span>${stopping ? t("detail.stopping") : t("detail.stop")}</span></button>`
    : `<button class="btn btn-play" data-action="play" ${game.exeMissing ? "disabled" : ""} title="${game.exeMissing ? t("detail.exeMissing") : t("detail.play")}">${ICONS.play}<span>${t("detail.play")}</span></button>`;
  const statusOptions = STATUS_ORDER.map(
    (id) => `<button class="status-option ${game.status === id ? "active" : ""}" data-action="set-status" data-status="${id}" role="option" aria-selected="${game.status === id}">
      <span class="dot" style="--status:${STATUS_COLORS[id]}"></span><span>${statusLabel(id)}</span>${game.status === id ? '<span class="status-check">✓</span>' : ""}
    </button>`,
  ).join("");
  const timer = running
    ? `<span class="run-timer" data-run-timer>${t("detail.runningFor", { t: fmtDuration(running.elapsed || 0) })}</span>`
    : "";
  return `<div class="actionbar">
    ${playButton}
    <div class="status-control">
      <button class="status-pill" type="button" data-action="status-menu" style="--status:${game.statusColor}" aria-haspopup="listbox" aria-expanded="false" aria-label="${t("status.label")}: ${statusLabel(game.status)}">
        <span class="dot"></span><span class="status-value">${statusLabel(game.status)}</span>
      </button>
      <div class="status-menu hidden" id="status-menu" role="listbox" aria-label="${t("status.label")}">${statusOptions}</div>
    </div>
    <button class="btn btn-ghost" data-action="folder" title="${t("detail.openFolder")}">${ICONS.folder}<span class="btn-label">${t("detail.openFolder")}</span></button>
    <button class="btn btn-ghost" data-action="edit" title="${t("detail.edit")}">${ICONS.edit}<span class="btn-label">${t("detail.edit")}</span></button>
    <button class="btn btn-ghost danger" data-action="remove" title="${t("detail.remove")}">${ICONS.trash}</button>
    ${timer}
  </div>`;
}

function tabsTemplate() {
  const tabs = [
    ["overview", "tab.overview"],
    ["activity", "tab.activity"],
    ["notes", "tab.notes"],
    ["launch", "tab.launch"],
    ["log", "tab.log"],
  ];
  return `<div class="tabs">${tabs
    .map(([id, key]) => `<button class="tab ${app.tab === id ? "active" : ""}" data-tab="${id}">${t(key)}</button>`)
    .join("")}</div>`;
}

function renderDetail() {
  const pane = $("#detail-pane");
  if (!pane) return;
  app.menuOpen = false;
  const game = currentGame();
  if (!game) {
    pane.innerHTML = emptyDetailTemplate();
    updateRunningPill();
    return;
  }
  const running = app.running.get(game.id) || null;
  pane.innerHTML = `${heroTemplate(game, running)}${actionBarTemplate(game, running)}${tabsTemplate()}<section class="tab-panel">${tabPanel(game)}</section>`;
  updateRunningPill();
  if (app.tab === "log") loadLog();
}

function emptyDetailTemplate() {
  return `<div class="detail-empty">
    <div class="empty-glow"></div>
    <img class="empty-mark" src="quark.png" alt="">
    <h2>${t("empty.title")}</h2>
    <p>${t("empty.subtitle")}</p>
    <div class="empty-actions">
      <button class="btn btn-primary" data-action="empty-add">${ICONS.plus}<span>${t("empty.add")}</span></button>
      <button class="btn btn-ghost" data-action="empty-settings">${ICONS.settings}<span>${t("empty.settings")}</span></button>
    </div>
  </div>`;
}

function tabPanel(game) {
  switch (app.tab) {
    case "activity":
      return activityPanel(game);
    case "notes":
      return notesPanel(game);
    case "launch":
      return launchPanel(game);
    case "log":
      return logPanel(game);
    default:
      return overviewPanel(game);
  }
}

function detailRow(label, value) {
  if (!value) return "";
  return `<div class="detail-row"><dt>${esc(label)}</dt><dd title="${esc(value)}">${esc(value)}</dd></div>`;
}

function overviewPanel(game) {
  const tags = (game.tags || [])
    .map((tag) => `<span class="tag">${esc(tag)}<button data-action="tag-remove" data-tag="${esc(tag)}" aria-label="×">×</button></span>`)
    .join("");
  const description = game.description
    ? `<p class="description">${esc(game.description)}</p>`
    : `<p class="description empty">${t("overview.noDescription")}</p>`;
  const installations = (game.installations || [])
    .map(
      (installation) => `<div class="installation">
        <div class="installation-head"><span class="installation-name">${esc(installation.label)}</span><span class="source-badge">${esc(installation.source)}</span></div>
        <div class="installation-sub" title="${esc(installation.exe)}">${esc(installation.exe || installation.path || "—")}</div>
      </div>`,
    )
    .join("");
  return `<div class="overview-grid">
    <div>
      <h3 class="section-label">${t("overview.about")}</h3>
      ${description}
      <h3 class="section-label">${t("overview.tags")}</h3>
      <div class="tags">${tags || `<span class="tag" style="opacity:.6">${t("overview.noTags")}</span>`}<input class="tag-add" id="tag-input" maxlength="32" placeholder="${t("overview.addTag")}"></div>
    </div>
    <aside>
      <div class="card">
        <h3>${t("overview.details")}</h3>
        ${detailRow(t("overview.developer"), game.developer)}
        ${detailRow(t("overview.genres"), (game.genres || []).join(", "))}
        ${detailRow(t("overview.released"), game.released)}
        ${detailRow(t("overview.added"), fmtDate(game.addedAt))}
        ${detailRow(t("overview.playtime"), fmtPlaytime(game.playtimeSecs))}
        ${detailRow(t("overview.lastPlayed"), fmtRelative(game.lastPlayed))}
        ${detailRow(t("overview.source"), sourceLabel(game))}
        ${detailRow(t("overview.exe"), game.exe)}
      </div>
      <div class="card">
        <h3>${t("overview.installations")}</h3>
        ${installations || `<div class="installation-sub">—</div>`}
      </div>
    </aside>
  </div>`;
}

function fmtSession(secs) {
  const value = Math.max(0, Math.floor(secs || 0));
  if (value < 60) return `${value} s`;
  if (value < 3600) return `${Math.floor(value / 60)} min`;
  return `${num(value / 3600, 1)} h`;
}

function activityPanel(game) {
  const sessions = game.sessions || [];
  const total = game.playtimeSecs || 0;
  const count = sessions.length;
  const average = count ? total / count : 0;
  const last = count ? sessions[0] : null;
  const days = 14;
  const today = new Date();
  today.setHours(0, 0, 0, 0);
  const startDay = new Date(today);
  startDay.setDate(startDay.getDate() - (days - 1));
  const buckets = Array.from({ length: days }, (_, index) => {
    const date = new Date(startDay);
    date.setDate(startDay.getDate() + index);
    return { date, secs: 0 };
  });
  for (const session of sessions) {
    const date = new Date(session.start * 1000);
    date.setHours(0, 0, 0, 0);
    const index = Math.round((date - startDay) / 86400000);
    if (index >= 0 && index < days) buckets[index].secs += session.secs;
  }
  const max = Math.max(1, ...buckets.map((bucket) => bucket.secs));
  const chart = buckets
    .map((bucket) => {
      const height = Math.max(3, Math.round((bucket.secs / max) * 100));
      return `<div class="chart-col" title="${fmtDate(Math.floor(bucket.date.getTime() / 1000))} · ${fmtSession(bucket.secs)}">
        <div class="chart-bar" style="height:${height}%"></div>
        <span class="chart-day">${fmtDay(Math.floor(bucket.date.getTime() / 1000))}</span>
      </div>`;
    })
    .join("");
  const recent = sessions
    .slice(0, 8)
    .map(
      (session) =>
        `<div class="session-row"><span class="session-date">${fmtDate(session.start)}</span><span class="session-secs">${fmtSession(session.secs)}</span></div>`,
    )
    .join("");
  return `<div class="stat-grid">
    <div class="stat-card"><div class="stat-value">${total ? fmtPlaytime(total) : "0 h"}</div><div class="stat-label">${t("activity.total")}</div></div>
    <div class="stat-card"><div class="stat-value">${num(count, 0)}</div><div class="stat-label">${t("activity.sessions")}</div></div>
    <div class="stat-card"><div class="stat-value">${fmtSession(average)}</div><div class="stat-label">${t("activity.average")}</div></div>
    <div class="stat-card"><div class="stat-value">${last ? fmtRelative(last.start) : "—"}</div><div class="stat-label">${t("activity.last")}</div></div>
  </div>
  <div class="chart-card">
    <h3 class="section-label">${t("activity.last14")}</h3>
    <div class="chart">${chart}</div>
  </div>
  <h3 class="section-label">${t("activity.recent")}</h3>
  <div class="session-list">${recent || `<div class="log-empty">${t("activity.empty")}</div>`}</div>`;
}

function notesPanel(game) {
  return `<div class="notes-wrap">
    <span class="notes-status" id="notes-status"></span>
    <textarea class="notes-area" id="notes-area" spellcheck="true" placeholder="${t("notes.placeholder")}">${esc(game.notes)}</textarea>
  </div>`;
}

function anticheatLabel(id) {
  switch (id) {
    case "battleye":
      return "BattlEye";
    case "easyanticheat":
      return "EasyAntiCheat";
    case "both":
      return "BattlEye + EAC";
    default:
      return t("launch.none");
  }
}

function optionChip(label, enabled) {
  return `<span class="option-chip ${enabled ? "" : "off"}">${esc(label)}</span>`;
}

function launchPanel(game) {
  const mode = game.umu ? t("launch.umu") : game.isNative ? t("launch.native") : game.proton === "Auto" ? t("launch.automatic") : game.proton;
  const options = [];
  if (game.launch.gamemode) options.push(optionChip("GameMode", true));
  if (game.launch.mangohud) options.push(optionChip("MangoHud", true));
  if (game.launch.gamescope) {
    let label = `gamescope ${game.launch.gamescopeWidth}×${game.launch.gamescopeHeight}`;
    if (game.launch.gamescopeFsr) label += " · FSR";
    if (game.launch.gamescopeFps) label += ` · ${game.launch.gamescopeFps} fps`;
    options.push(optionChip(label, true));
  }
  if (game.launch.prime) options.push(optionChip("PRIME", true));
  if (game.launch.wrapper) options.push(optionChip(game.launch.wrapper, true));
  const env = (game.env || [])
    .map((variable) => `<tr><td>${esc(variable.name)}</td><td>${esc(variable.value)}</td></tr>`)
    .join("");
  const installations = (game.installations || [])
    .map(
      (installation) => `<div class="installation">
        <div class="installation-head"><span class="installation-name">${esc(installation.label)}</span><span class="source-badge">${esc(installation.source)}</span></div>
        <div class="installation-sub" title="${esc(installation.exe)}">${esc(installation.exe || installation.path || "—")}</div>
      </div>`,
    )
    .join("");
  return `<div class="launch-grid">
    <div class="card">
      <h3>${t("launch.mode")}</h3>
      <div class="detail-row"><dt>${t("launch.mode")}</dt><dd>${esc(mode)}</dd></div>
      <div class="detail-row"><dt>${t("launch.anticheat")}</dt><dd>${esc(anticheatLabel(game.anticheat))}</dd></div>
      <div class="detail-row"><dt>${t("launch.prefix")}</dt><dd title="${esc(game.prefix)}">${esc(game.prefix)}</dd></div>
    </div>
    <div class="card">
      <h3>${t("launch.executable")}</h3>
      <div class="detail-row"><dt>${t("overview.exe")}</dt><dd title="${esc(game.exe)}">${esc(game.exe || "—")}</dd></div>
      <div class="detail-row"><dt>${t("launch.args")}</dt><dd title="${esc(game.args)}">${esc(game.args || "—")}</dd></div>
      <div class="detail-row"><dt>${t("launch.wrapper")}</dt><dd title="${esc(game.launch.wrapper)}">${esc(game.launch.wrapper || "—")}</dd></div>
    </div>
    <div class="card">
      <h3>${t("launch.options")}</h3>
      <div class="option-chips">${options.join("") || optionChip(t("launch.none"), false)}</div>
    </div>
    <div class="card">
      <h3>${t("overview.installations")}</h3>
      ${installations || `<div class="installation-sub">—</div>`}
    </div>
  </div>
  <div class="card">
    <h3>${t("launch.env")}</h3>
    ${
      env
        ? `<table class="env-table"><thead><tr><th>Variable</th><th>Valeur</th></tr></thead><tbody>${env}</tbody></table>`
        : `<div class="installation-sub">—</div>`
    }
  </div>
  <p class="launch-note">${t("launch.readonly")}</p>`;
}

function logPanel(game) {
  return `<div class="log-toolbar">
    <h3 class="section-label" style="margin:0">${t("tab.log")}</h3>
    <button class="btn btn-ghost btn-sm" data-action="log-refresh">${ICONS.refresh}<span>${t("log.refresh")}</span></button>
  </div>
  <pre class="log-output" id="log-output">${t("log.loading")}</pre>`;
}

async function loadLog() {
  const game = currentGame();
  const output = $("#log-output");
  if (!game || !output) return;
  const request = ++app.logRequest;
  try {
    const text = await invoke("read_log", { id: game.id });
    if (request === app.logRequest && $("#log-output")) {
      $("#log-output").textContent = text && text.trim() ? text : t("log.empty");
    }
  } catch (error) {
    if (request === app.logRequest && $("#log-output")) {
      $("#log-output").textContent = t("log.empty");
    }
  }
}

async function launchGame(id) {
  const game = app.data.games.find((candidate) => candidate.id === id);
  if (!game) return;
  try {
    const updated = await invoke("launch_game", { id });
    setGame(updated);
    app.running.set(id, { id, name: updated.name, label: updated.runningLabel || "", elapsed: 0, stopping: false });
    renderList();
    renderDetail();
    toast(t("toast.launched", { name: updated.name }), "success");
  } catch (error) {
    toast(String(error), "error");
  }
}

async function stopGame(id) {
  try {
    const running = await invoke("stop_game", { id });
    app.running.set(id, { ...running, id });
    renderDetail();
    updateRunningPill();
  } catch (error) {
    toast(String(error), "error");
  }
}

async function toggleFavorite() {
  const game = currentGame();
  if (!game) return;
  try {
    const updated = await invoke("set_favorite", { id: game.id, favorite: !game.favorite });
    setGame(updated);
    renderList();
    renderDetail();
  } catch (error) {
    toast(String(error), "error");
  }
}

async function changeStatus(value) {
  const game = currentGame();
  if (!game) return;
  try {
    const updated = await invoke("set_status", { id: game.id, status: value });
    setGame(updated);
    renderList();
    renderDetail();
  } catch (error) {
    toast(String(error), "error");
  }
}

function scheduleNotesSave() {
  const area = $("#notes-area");
  if (!area) return;
  const id = app.selectedId;
  const text = area.value;
  const status = $("#notes-status");
  if (status) {
    status.textContent = t("notes.saving");
    status.classList.add("visible");
  }
  clearTimeout(app.notesTimer);
  app.notesTimer = setTimeout(async () => {
    try {
      await invoke("set_notes", { id, notes: text });
      const game = app.data.games.find((candidate) => candidate.id === id);
      if (game) game.notes = text;
      const indicator = $("#notes-status");
      if (indicator) {
        indicator.textContent = t("notes.saved");
        setTimeout(() => indicator.classList.remove("visible"), 1200);
      }
    } catch (error) {
      toast(String(error), "error");
    }
  }, 500);
}

async function addTagFromInput() {
  const input = $("#tag-input");
  if (!input) return;
  const value = input.value.trim();
  if (!value) return;
  const game = currentGame();
  if (!game) return;
  try {
    const updated = await invoke("add_tag", { id: game.id, tag: value });
    setGame(updated);
    renderDetail();
    const again = $("#tag-input");
    if (again) again.focus();
  } catch (error) {
    toast(String(error), "error");
  }
}

async function removeTag(tag) {
  const game = currentGame();
  if (!game) return;
  try {
    const updated = await invoke("remove_tag", { id: game.id, tag });
    setGame(updated);
    renderDetail();
  } catch (error) {
    toast(String(error), "error");
  }
}

async function changeArtwork(kind) {
  const game = currentGame();
  if (!game) return;
  try {
    const updated = await invoke("pick_artwork", { id: game.id, kind });
    setGame(updated);
    renderList();
    renderDetail();
    toast(t("toast.saved"), "success");
  } catch (error) {
    if (String(error).toLowerCase().includes("aucune image")) return;
    toast(String(error), "error");
  }
}

function setQuery(value, source) {
  app.query = value;
  const global = $("#global-search");
  const local = $("#library-search");
  if (global && global !== source) global.value = value;
  if (local && local !== source) local.value = value;
  renderList();
}

function renderStaticLabels() {
  const global = $("#global-search");
  if (global) {
    global.placeholder = t("search.global");
    global.setAttribute("aria-label", t("search.global"));
  }
  const local = $("#library-search");
  if (local) {
    local.placeholder = t("search.library");
    local.setAttribute("aria-label", t("search.library"));
  }
  const title = $("#library-title");
  if (title) title.textContent = t("library.title");
  const add = $("#add-game-btn");
  if (add) add.innerHTML = `${ICONS.plus}<span>${t("library.add")}</span>`;
  const railLibrary = $("#rail-library");
  if (railLibrary) {
    railLibrary.innerHTML = ICONS.library;
    railLibrary.title = t("nav.library");
    railLibrary.setAttribute("aria-label", t("nav.library"));
  }
  const railSettings = $("#rail-settings");
  if (railSettings) {
    railSettings.innerHTML = ICONS.settings;
    railSettings.title = t("nav.settings");
    railSettings.setAttribute("aria-label", t("nav.settings"));
  }
  const winMin = $("#win-min");
  if (winMin) {
    winMin.innerHTML = ICONS.minus;
    winMin.setAttribute("aria-label", t("window.minimize"));
    winMin.title = t("window.minimize");
  }
  const winMax = $("#win-max");
  if (winMax) {
    winMax.innerHTML = ICONS.maximize;
    winMax.setAttribute("aria-label", t("window.maximize"));
    winMax.title = t("window.maximize");
  }
  const winClose = $("#win-close");
  if (winClose) {
    winClose.innerHTML = ICONS.close;
    winClose.setAttribute("aria-label", t("window.close"));
    winClose.title = t("window.close");
  }
  const sort = $("#sort-select");
  if (sort) sort.setAttribute("aria-label", t("sort.label"));
  const pane = $("#library-pane");
  if (pane) pane.setAttribute("aria-label", t("nav.library"));
}

function applyFromSettings() {
  const settings = app.data.settings;
  app.lang = LANGS[settings.uiLanguage] || "fr";
  document.documentElement.lang = app.lang;
  applyTheme(settings.theme);
  document.body.classList.toggle("compact", Boolean(settings.compact));
  const demo = $("#demo-pill");
  if (demo) {
    demo.classList.toggle("hidden", !app.data.demo);
    demo.textContent = app.data.demo ? t("settings.demo") : "";
  }
}

function renderAll() {
  renderStaticLabels();
  renderFilters();
  renderList();
  renderDetail();
  updateRunningPill();
}

function updateRunningPill() {
  const pill = $("#running-pill");
  const count = app.running.size;
  if (pill) {
    pill.classList.toggle("hidden", count === 0);
    if (count > 0) pill.textContent = t(count === 1 ? "running.one" : "running.many", { n: count });
  }
  const game = currentGame();
  const running = game ? app.running.get(game.id) : null;
  const timer = $("[data-run-timer]");
  if (timer && running) {
    timer.textContent = t("detail.runningFor", { t: fmtDuration(running.elapsed || 0) });
  }
}

async function pollRunning() {
  if (!app.data || app.pollBusy) return;
  app.pollBusy = true;
  try {
    const list = await invoke("running_games");
    const next = new Map(list.map((entry) => [entry.id, entry]));
    const ended = [...app.running.keys()].filter((id) => !next.has(id));
    const started = [...next.keys()].filter((id) => !app.running.has(id));
    app.running = next;
    if (ended.length || started.length) {
      for (const id of new Set([...ended, ...started])) {
        try {
          const game = await invoke("get_game", { id });
          setGame(game);
        } catch (error) {
          /* jeu retiré entre-temps */
        }
      }
      renderList();
      if (app.selectedId) renderDetail();
      for (const id of ended) {
        const game = app.data.games.find((candidate) => candidate.id === id);
        if (game) toast(t("toast.stopped", { name: game.name }), "success");
      }
    }
    updateRunningPill();
  } catch (error) {
    /* le backend démarre encore */
  } finally {
    app.pollBusy = false;
  }
}

async function setupEvents() {
  await listen("quark://session-started", (event) => {
    const payload = event.payload || {};
    if (!payload.id) return;
    app.running.set(payload.id, {
      id: payload.id,
      label: payload.label || "",
      elapsed: 0,
      stopping: false,
    });
    renderList();
    if (app.selectedId === payload.id) renderDetail();
    updateRunningPill();
  });
  await listen("quark://session-ended", (event) => {
    const payload = event.payload || {};
    if (!payload.id) return;
    app.running.delete(payload.id);
    if (payload.game) setGame(payload.game);
    renderList();
    if (app.selectedId === payload.id) renderDetail();
    updateRunningPill();
  });
  await listen("quark://session-stopping", (event) => {
    const payload = event.payload || {};
    const running = payload.id ? app.running.get(payload.id) : null;
    if (running) running.stopping = true;
    if (payload.id && app.selectedId === payload.id) renderDetail();
  });
}

function openModal(html) {
  app.modalOpen = true;
  const root = $("#modal-root");
  root.innerHTML = `<div class="modal-backdrop">${html}</div>`;
  const focus = root.querySelector("input, textarea, button.btn-primary");
  if (focus) setTimeout(() => focus.focus(), 40);
}

function closeModal() {
  app.modalOpen = false;
  app.editId = null;
  const root = $("#modal-root");
  if (root) root.innerHTML = "";
}

function toast(message, type = "info") {
  const root = $("#toasts");
  if (!root) return;
  const element = document.createElement("div");
  element.className = `toast ${type === "error" ? "error" : ""}`;
  element.style.transition = "opacity .2s ease, transform .2s ease";
  element.innerHTML = `<span class="toast-icon">${type === "error" ? ICONS.alert : ICONS.check}</span><span>${esc(message)}</span>`;
  root.appendChild(element);
  setTimeout(() => {
    element.style.opacity = "0";
    element.style.transform = "translateX(14px)";
    setTimeout(() => element.remove(), 220);
  }, 4200);
}

function themeCard(theme, active) {
  const swatches = [theme.vars.bg, theme.vars.surface, theme.vars.accent, theme.vars.text]
    .map((color) => `<span class="theme-swatch" style="background:${color}"></span>`)
    .join("");
  return `<button class="theme-card ${active ? "active" : ""}" data-theme="${theme.id}">
    <span class="theme-swatches">${swatches}</span>
    <span class="theme-name">${esc(theme.label)}</span>
  </button>`;
}

function toggleRow(label, hint, action, active) {
  return `<div class="toggle-row">
    <div><div class="toggle-label">${label}</div><div class="toggle-hint">${hint}</div></div>
    <button class="switch ${active ? "on" : ""}" data-toggle="${action}" role="switch" aria-checked="${active}"></button>
  </div>`;
}

function openSettings() {
  if (!app.data) return;
  const settings = app.data.settings;
  const themes = app.data.themes.map((theme) => themeCard(theme, theme.id === settings.theme)).join("");
  const languages = app.data.languages
    .map(
      (language) =>
        `<button class="segment ${language.id === settings.uiLanguage ? "active" : ""}" data-lang="${language.id}">${esc(language.label)}</button>`,
    )
    .join("");
  const memory = app.data.memoryMb == null ? "—" : `${app.data.memoryMb} MB`;
  const loadError = app.data.loadError
    ? `<section class="settings-section"><h3>${t("settings.loadError")}</h3><div class="data-value" style="white-space:normal;font-family:var(--font);color:var(--danger)">${esc(app.data.loadError)}</div></section>`
    : "";
  openModal(`<div class="modal" id="settings-modal">
    <div class="modal-head"><h2>${t("settings.title")}</h2><button class="modal-close" data-close>${ICONS.x}</button></div>
    <div class="modal-body">
      <section class="settings-section">
        <h3>${t("settings.appearance")}</h3>
        <div class="theme-grid">${themes}</div>
        <div style="height:12px"></div>
        ${toggleRow(t("settings.compact"), t("settings.compactHint"), "compact", settings.compact)}
        ${toggleRow(t("settings.hero"), t("settings.heroHint"), "hero", settings.heroBackground)}
      </section>
      <section class="settings-section">
        <h3>${t("settings.language")}</h3>
        <div class="segmented">${languages}</div>
      </section>
      <section class="settings-section">
        <h3>${t("settings.data")}</h3>
        <div class="data-row"><span class="data-key">${t("settings.configPath")}</span><span class="data-value" title="${esc(app.data.configPath)}">${esc(app.data.configPath)}</span></div>
        <div class="data-row"><span class="data-key">${t("settings.dataDir")}</span><span class="data-value" title="${esc(app.data.dataDir)}">${esc(app.data.dataDir)}</span></div>
        <div style="display:flex;gap:8px;margin-top:12px;flex-wrap:wrap">
          <button class="btn btn-ghost btn-sm" data-action="open-config">${ICONS.folder}<span>${t("settings.openFolder")}</span></button>
          <button class="btn btn-ghost btn-sm" data-action="reload">${ICONS.refresh}<span>${t("settings.reload")}</span></button>
        </div>
      </section>
      <section class="settings-section">
        <h3>${t("settings.about")}</h3>
        <div class="about-grid">
          <span>${t("settings.version")} <strong>${esc(app.data.version)}</strong></span>
          <span>${t("settings.memory")} <strong>${memory}</strong></span>
          ${app.data.demo ? `<span><strong>${t("settings.demo")}</strong></span>` : ""}
        </div>
      </section>
      ${loadError}
    </div>
    <div class="modal-foot"><button class="btn btn-primary" data-close>${t("settings.close")}</button></div>
  </div>`);
}

async function saveSetting(patch) {
  try {
    app.data.settings = await invoke("save_settings", { patch });
    applyFromSettings();
    renderStaticLabels();
    renderFilters();
    renderList();
    renderDetail();
    updateRunningPill();
    if ($("#settings-modal")) openSettings();
  } catch (error) {
    toast(String(error), "error");
  }
}

function fileStem(path) {
  const parts = String(path || "").split(/[\\/]/);
  const name = parts[parts.length - 1] || "";
  const dot = name.lastIndexOf(".");
  return dot > 0 ? name.slice(0, dot) : name;
}

function openAddModal() {
  openModal(`<div class="modal small" id="add-modal">
    <div class="modal-head"><h2>${t("add.title")}</h2><button class="modal-close" data-close>${ICONS.x}</button></div>
    <div class="modal-body">
      <div class="field"><label for="add-name">${t("add.name")}</label><input class="input" id="add-name" autocomplete="off" placeholder="${t("add.name")}"></div>
      <div class="field"><label for="add-exe">${t("add.exe")}</label><div class="input-row"><input class="input" id="add-exe" placeholder="${t("add.exePlaceholder")}"><button class="btn btn-ghost" data-action="browse-exe">${t("add.browse")}</button></div></div>
      <div class="field"><label for="add-args">${t("add.args")}</label><input class="input" id="add-args" autocomplete="off"></div>
    </div>
    <div class="modal-foot"><button class="btn btn-ghost" data-close>${t("add.cancel")}</button><button class="btn btn-primary" data-action="submit-add">${t("add.submit")}</button></div>
  </div>`);
}

function openEditModal(game) {
  app.editId = game.id;
  openModal(`<div class="modal small" id="edit-modal">
    <div class="modal-head"><h2>${t("edit.title")}</h2><button class="modal-close" data-close>${ICONS.x}</button></div>
    <div class="modal-body">
      <div class="field"><label for="edit-name">${t("add.name")}</label><input class="input" id="edit-name" autocomplete="off" value="${esc(game.name)}"></div>
      <div class="field"><label for="edit-exe">${t("add.exe")}</label><div class="input-row"><input class="input" id="edit-exe" value="${esc(game.exe)}"><button class="btn btn-ghost" data-action="browse-exe">${t("add.browse")}</button></div></div>
      <div class="field"><label for="edit-args">${t("add.args")}</label><input class="input" id="edit-args" autocomplete="off" value="${esc(game.args)}"></div>
    </div>
    <div class="modal-foot"><button class="btn btn-ghost" data-close>${t("add.cancel")}</button><button class="btn btn-primary" data-action="submit-edit">${t("edit.save")}</button></div>
  </div>`);
}

function confirmRemove(game) {
  openModal(`<div class="modal small" id="remove-modal">
    <div class="modal-head"><h2>${t("remove.title")}</h2><button class="modal-close" data-close>${ICONS.x}</button></div>
    <div class="modal-body"><p style="margin:0;color:var(--text-dim);line-height:1.65">${t("remove.body", { name: esc(game.name) })}</p></div>
    <div class="modal-foot"><button class="btn btn-ghost" data-close>${t("add.cancel")}</button><button class="btn btn-stop" data-action="confirm-remove" data-id="${esc(game.id)}">${t("remove.confirm")}</button></div>
  </div>`);
}

async function browseExecutable() {
  try {
    const path = await invoke("pick_executable");
    if (!path) return;
    const exe = $("#add-exe") || $("#edit-exe");
    if (exe) exe.value = path;
    const name = $("#add-name");
    if (name && !name.value.trim()) name.value = fileStem(path);
  } catch (error) {
    toast(String(error), "error");
  }
}

async function submitAdd() {
  const exeInput = $("#add-exe");
  const nameInput = $("#add-name");
  const argsInput = $("#add-args");
  const exe = exeInput ? exeInput.value.trim() : "";
  if (!exe) {
    toast(t("add.exeRequired"), "error");
    return;
  }
  try {
    let game = await invoke("add_game", { name: nameInput ? nameInput.value.trim() : "", exe });
    if (argsInput && argsInput.value.trim()) {
      game = await invoke("update_game", { id: game.id, args: argsInput.value });
    }
    setGame(game);
    closeModal();
    app.filter = "all";
    app.query = "";
    setQuery("");
    app.selectedId = game.id;
    app.tab = "overview";
    renderFilters();
    renderList();
    renderDetail();
    toast(t("toast.added"), "success");
  } catch (error) {
    toast(String(error), "error");
  }
}

async function submitEdit() {
  if (!app.editId) return;
  const name = $("#edit-name");
  const exe = $("#edit-exe");
  const args = $("#edit-args");
  try {
    const game = await invoke("update_game", {
      id: app.editId,
      name: name ? name.value : undefined,
      exe: exe ? exe.value : undefined,
      args: args ? args.value : undefined,
    });
    setGame(game);
    closeModal();
    renderList();
    renderDetail();
    toast(t("toast.saved"), "success");
  } catch (error) {
    toast(String(error), "error");
  }
}

async function confirmRemoveGame(id) {
  try {
    await invoke("remove_game", { id });
    app.data.games = app.data.games.filter((game) => game.id !== id);
    closeModal();
    if (app.selectedId === id) {
      app.selectedId = null;
      const games = visibleGames();
      if (games.length > 0) app.selectedId = games[0].id;
    }
    renderAll();
    toast(t("toast.removed"), "success");
  } catch (error) {
    toast(String(error), "error");
  }
}

async function reloadLibrary() {
  try {
    app.data = await invoke("reload_library");
    applyFromSettings();
    renderAll();
    selectInitial();
    openSettings();
    toast(t("toast.saved"), "success");
  } catch (error) {
    toast(String(error), "error");
  }
}

function closeStatusMenu() {
  const menu = $("#status-menu");
  if (menu) menu.classList.add("hidden");
  const pill = $(".status-pill[data-action=\"status-menu\"]");
  if (pill) pill.setAttribute("aria-expanded", "false");
}

function toggleStatusMenu() {
  const menu = $("#status-menu");
  if (!menu) return;
  const open = menu.classList.toggle("hidden") === false;
  const pill = $(".status-pill[data-action=\"status-menu\"]");
  if (pill) pill.setAttribute("aria-expanded", String(open));
}

function toggleGameMenu(force) {
  const menu = $("#game-menu");
  if (!menu) return;
  app.menuOpen = typeof force === "boolean" ? force : !app.menuOpen;
  menu.classList.toggle("hidden", !app.menuOpen);
}

async function onDetailClick(event) {
  const tab = event.target.closest("[data-tab]");
  if (tab) {
    app.tab = tab.dataset.tab;
    app.menuOpen = false;
    renderDetail();
    return;
  }
  const actionElement = event.target.closest("[data-action]");
  if (!actionElement) {
    toggleGameMenu(false);
    return;
  }
  const action = actionElement.dataset.action;
  if (action === "menu") {
    closeStatusMenu();
    toggleGameMenu();
    return;
  }
  if (action === "status-menu") {
    toggleGameMenu(false);
    return toggleStatusMenu();
  }
  if (action !== "set-status") closeStatusMenu();
  toggleGameMenu(false);
  if (action === "empty-add") return openAddModal();
  if (action === "empty-settings") return openSettings();
  const game = currentGame();
  if (!game) return;
  switch (action) {
    case "play":
      return launchGame(game.id);
    case "stop":
      return stopGame(game.id);
    case "set-status":
      return changeStatus(actionElement.dataset.status);
    case "favorite":
      return toggleFavorite();
    case "folder":
      try {
        await invoke("open_folder", { id: game.id });
      } catch (error) {
        toast(String(error), "error");
      }
      return;
    case "edit":
      return openEditModal(game);
    case "remove":
      return confirmRemove(game);
    case "change-cover":
      return changeArtwork("cover");
    case "change-hero":
      return changeArtwork("hero");
    case "tag-remove":
      return removeTag(actionElement.dataset.tag);
    case "log-refresh":
      return loadLog();
    default:
      return;
  }
}

async function onModalClick(event) {
  if (event.target.classList.contains("modal-backdrop")) return closeModal();
  if (event.target.closest("[data-close]")) return closeModal();
  const theme = event.target.closest(".theme-card[data-theme]");
  if (theme) return saveSetting({ theme: theme.dataset.theme });
  const language = event.target.closest("[data-lang]");
  if (language) return saveSetting({ uiLanguage: language.dataset.lang });
  const toggle = event.target.closest("[data-toggle]");
  if (toggle) {
    const settings = app.data.settings;
    if (toggle.dataset.toggle === "compact") return saveSetting({ compact: !settings.compact });
    if (toggle.dataset.toggle === "hero") return saveSetting({ heroBackground: !settings.heroBackground });
    return;
  }
  const actionElement = event.target.closest("[data-action]");
  if (!actionElement) return;
  switch (actionElement.dataset.action) {
    case "browse-exe":
      return browseExecutable();
    case "submit-add":
      return submitAdd();
    case "submit-edit":
      return submitEdit();
    case "confirm-remove":
      return confirmRemoveGame(actionElement.dataset.id);
    case "reload":
      return reloadLibrary();
    case "open-config":
      try {
        await invoke("open_config_folder", {});
      } catch (error) {
        toast(String(error), "error");
      }
      return;
    default:
      return;
  }
}

function isTyping(target) {
  if (!target) return false;
  const tag = target.tagName;
  return tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT" || target.isContentEditable;
}

function moveSelection(delta) {
  const games = visibleGames();
  if (!games.length) return;
  let index = games.findIndex((game) => game.id === app.selectedId);
  if (index < 0) index = 0;
  else index = Math.min(games.length - 1, Math.max(0, index + delta));
  selectGame(games[index].id, true);
}

function onKeyDown(event) {
  if (event.key === "Escape") {
    if (app.modalOpen) return closeModal();
    if (app.menuOpen) return toggleGameMenu(false);
    const statusMenu = $("#status-menu");
    if (statusMenu && !statusMenu.classList.contains("hidden")) return closeStatusMenu();
    if (app.query) return setQuery("");
    return;
  }
  if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "k") {
    event.preventDefault();
    const input = $("#global-search");
    if (input) {
      input.focus();
      input.select();
    }
    return;
  }
  if (app.modalOpen) return;
  if (event.key === "/" && !isTyping(event.target)) {
    event.preventDefault();
    const input = $("#library-search");
    if (input) input.focus();
    return;
  }
  if ((event.key === "ArrowDown" || event.key === "ArrowUp") && !isTyping(event.target)) {
    event.preventDefault();
    moveSelection(event.key === "ArrowDown" ? 1 : -1);
  }
}

function bindStatic() {
  $("#win-min").addEventListener("click", () => invoke("win_minimize", {}));
  $("#win-max").addEventListener("click", () => invoke("win_toggle_maximize", {}));
  $("#win-close").addEventListener("click", () => invoke("win_close", {}));

  const titlebar = $("#titlebar");
  titlebar.addEventListener("mousedown", (event) => {
    if (event.button !== 0) return;
    if (event.target.closest("button,input,kbd")) return;
    invoke("win_start_drag", {});
  });
  titlebar.addEventListener("dblclick", (event) => {
    if (event.target.closest("button,input")) return;
    invoke("win_toggle_maximize", {});
  });

  $("#rail-library").addEventListener("click", () => {
    const input = $("#library-search");
    if (input) input.focus();
  });
  $("#rail-settings").addEventListener("click", () => openSettings());
  $("#add-game-btn").addEventListener("click", () => openAddModal());

  $("#global-search").addEventListener("input", (event) => setQuery(event.target.value, event.target));
  $("#library-search").addEventListener("input", (event) => setQuery(event.target.value, event.target));
  $("#sort-select").addEventListener("change", (event) => {
    app.sort = event.target.value;
    renderList();
  });
  $("#status-filters").addEventListener("click", (event) => {
    const chip = event.target.closest("[data-filter]");
    if (!chip) return;
    app.filter = chip.dataset.filter;
    renderFilters();
    renderList();
  });
  $("#game-list").addEventListener("click", (event) => {
    const row = event.target.closest(".game-row");
    if (row) selectGame(row.dataset.id);
  });

  const detail = $("#detail-pane");
  detail.addEventListener("click", onDetailClick);
  detail.addEventListener("change", (event) => {
    if (event.target.matches('[data-action="status"]')) changeStatus(event.target.value);
  });
  detail.addEventListener("input", (event) => {
    if (event.target.id === "notes-area") scheduleNotesSave();
  });
  detail.addEventListener("keydown", (event) => {
    if (event.target.id === "tag-input" && event.key === "Enter") {
      event.preventDefault();
      addTagFromInput();
    }
  });

  $("#modal-root").addEventListener("click", onModalClick);

  document.addEventListener("click", (event) => {
    if (!event.target.closest(".hero-actions")) toggleGameMenu(false);
    if (!event.target.closest(".status-control")) closeStatusMenu();
  });
  document.addEventListener("keydown", onKeyDown);

  $$(".resize-handle").forEach((handle) => {
    handle.addEventListener("mousedown", (event) => {
      event.preventDefault();
      invoke("win_start_resize", { direction: handle.dataset.resize });
    });
  });
}

function showFatal(error) {
  const pane = $("#detail-pane");
  if (pane) {
    pane.innerHTML = `<div class="detail-empty"><div class="empty-glow"></div><img class="empty-mark" src="quark.png" alt=""><h2>Quark</h2><p>${esc(String(error))}</p></div>`;
  }
  console.error(error);
}

async function boot() {
  bindStatic();
  renderStaticLabels();
  try {
    app.data = await invoke("bootstrap_data");
    applyFromSettings();
    renderAll();
    selectInitial();
    await setupEvents().catch((error) => console.warn("events", error));
    pollRunning();
    setInterval(pollRunning, 2000);
  } catch (error) {
    showFatal(error);
  } finally {
    const splash = $("#splash");
    if (splash) {
      splash.classList.add("done");
      setTimeout(() => splash.remove(), 450);
    }
  }
}

boot();
