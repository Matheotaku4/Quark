"use strict";
// Libellés ajoutés par le portage complet (boutiques, succès, sauvegardes,
// onboarding, vues enregistrées). Séparés pour garder i18n.js lisible.

const EXTRA_I18N = {
  fr: {
    "x.dllFolder": "Dossier analysé", "x.dllChange": "Changer de dossier", "x.dllScan": "Analyser", "x.dllNone": "Aucune DLL trouvée dans ce dossier.", "x.dllActive": "Overrides actifs", "pv.more": "Charger plus",
    "x.media": "Médias", "x.noDescription": "Aucune description.", "x.noMedia": "Aucun média disponible.",
    "x.trailer": "Bande-annonce", "x.trailerUnavailable": "Lecteur intégré indisponible : la bande-annonce peut être ouverte sur Steam.", "art.auto": "Rechercher automatiquement", "x.save": "Enregistrer", "x.cancel": "Annuler", "x.close": "Fermer",
    "notes.placeholder": "Vos notes sur ce jeu…", "launch.prefix": "Préfixe", "launch.options": "Options",
    "launch.env": "Variables d'environnement", "launch.anticheat": "Anti-triche",
    "pv.mygames": "Mes jeux", "pv.title": "Bibliothèque des boutiques", "pv.subtitle": "Steam, Epic Games, GOG et itch.io réunis au même endroit.",
    "pv.loading": "Chargement des boutiques…", "pv.emptyHint": "Connecte une boutique dans les réglages pour retrouver tes jeux.",
    "pv.empty": "Aucun jeu de boutique synchronisé.", "pv.noMatch": "Aucun jeu ne correspond aux filtres.",
    "pv.all": "Toutes", "pv.search": "Rechercher un jeu de boutique…", "pv.filter.all": "Tous", "pv.filter.installed": "Installés", "pv.filter.missing": "Non installés",
    "pv.syncAll": "Tout synchroniser", "pv.sync": "Synchroniser", "pv.synced": "Bibliothèques synchronisées.", "pv.syncHint": "Chaque boutique se synchronise indépendamment.",
    "pv.importSteam": "Importer de Steam", "pv.importSteamHint": "Jeux installés détectés dans les bibliothèques Steam locales.",
    "pv.import": "Importer", "pv.imported": "{n} jeu(x) importé(s).", "pv.noSteam": "Aucun jeu Steam installé détecté.", "pv.noneSelected": "Aucun jeu sélectionné.", "pv.already": "déjà dans Quark",
    "pv.open": "Ouvrir", "pv.adopt": "Ajouter à Quark", "pv.adopted": "{name} ajouté à la bibliothèque.", "pv.install": "Installer", "pv.installSteam": "Installer via Steam",
    "pv.steamInstallHint": "Steam s'ouvre sur la fiche du jeu et gère le téléchargement.", "pv.openSteam": "Ouvrir Steam",
    "pv.uninstall": "Désinstaller", "pv.uninstallBody": "Supprimer les fichiers de {name} du disque ?", "pv.uninstalling": "Désinstallation…", "pv.uninstalled": "Jeu désinstallé.",
    "pv.destination": "Dossier d'installation", "pv.installStarted": "Installation lancée.", "pv.installing": "Installation…",
    "pv.installedAt": "Installé · {path}", "pv.owned": "Possédé · non installé", "pv.state.installed": "Installé", "pv.state.not_installed": "Non installé",
    "pv.state.installing": "Installation…", "pv.state.failed": "Échec", "pv.state.unknown": "Inconnu", "pv.state.queued": "En attente",
    "pv.state.updating": "Mise à jour", "pv.state.repairing": "Réparation",
    "pv.accounts": "Comptes", "pv.accountsHint": "Les mots de passe ne passent jamais par Quark : seuls les codes d'autorisation sont échangés.",
    "pv.connect": "Connecter", "pv.disconnect": "Déconnecter", "pv.disconnectBody": "La session locale sera oubliée. Tes jeux et sauvegardes restent sur le disque.",
    "pv.disconnected": "Boutique déconnectée.", "pv.connecting": "Connexion…", "pv.authCode": "Code d'autorisation",
    "pv.codeRequired": "Colle le code d'autorisation.", "pv.openLogin": "Ouvrir la page de connexion",
    "pv.epicLoginHint": "Connecte-toi sur la page Legendary, copie la valeur authorizationCode puis colle-la ici.",
    "pv.gogLoginHint": "Connecte-toi sur GOG, copie le code affiché à la fin de l'adresse de retour puis colle-le ici.",
    "pv.steamCreds": "Identifiants Steam", "pv.steamHint": "Facultatif : la clé Web API et le SteamID64 ajoutent la bibliothèque possédée et les succès.",
    "pv.itchKey": "Clé API itch.io", "pv.itchHint": "Crée une clé API personnelle sur itch.io. Elle reste dans le fichier local protégé.",
    "pv.tools": "Outils gérés", "pv.toolsHint": "Legendary, gogdl et Ludusavi sont téléchargés depuis leurs releases officielles et vérifiés par SHA-256.",
    "pv.toolReady": "Installé et prêt", "pv.toolMissing": "Non installé", "pv.installTool": "Installer", "pv.toolInstalling": "Téléchargement de l'outil…", "pv.toolDone": "Outil installé.",
    "pv.reload": "Actualiser", "pv.connected": "Connecté",
    "onboarding.title": "Bienvenue dans Quark", "onboarding.subtitle": "Quelques choix pour commencer, tout reste modifiable ensuite.",
    "onboarding.language": "Langue", "onboarding.theme": "Thème", "onboarding.start": "Page d'ouverture", "onboarding.install": "Dossier des jeux de boutique",
    "onboarding.finish": "Commencer", "onboarding.skip": "Plus tard",
    "whatsnew.title": "Nouveautés", "whatsnew.body": "Cette version modernise toute l'interface : liste à gauche, fiche à droite, boutique, mes jeux, succès, médias et sauvegardes.",
    "views.title": "Vues enregistrées", "views.save": "Enregistrer la vue actuelle", "views.name": "Nom de la vue", "views.empty": "Aucune vue enregistrée.",
    "views.apply": "Appliquer", "views.delete": "Supprimer", "views.saved": "Vue enregistrée.", "views.deleted": "Vue supprimée.",
    "batch.selected": "{n} sélectionné(s)", "batch.remove": "Retirer la sélection", "batch.removeBody": "Retirer {n} jeu(x) de la bibliothèque ? Les fichiers ne sont pas supprimés.",
    "batch.clear": "Tout désélectionner",
    "settings.providers": "Boutiques & outils", "settings.savedViews": "Vues enregistrées",
    "overlay.title": "Overlay en jeu", "overlay.show": "Afficher l'overlay", "overlay.hide": "Masquer l'overlay", "overlay.hint": "Shift+Backspace ouvre le panneau au-dessus du jeu.",
    "saves.title": "Sauvegardes locales", "saves.hint": "Ludusavi crée des instantanés locaux avant et après le jeu. Rien n'est envoyé sans ton accord.", "saves.preview": "Aperçu", "saves.backup": "Créer une sauvegarde", "saves.backups": "Sauvegardes", "saves.restore": "Restaurer",
    "saves.none": "Aucune sauvegarde.", "saves.files": "Fichiers", "saves.changed": "Modifications", "saves.addFolder": "Ajouter un dossier",
    "saves.folder": "Dossier de sauvegarde", "saves.cloud": "Synchroniser le cloud", "saves.cloudDone": "Sauvegardes cloud synchronisées.",
    "saves.backupDone": "Sauvegarde créée.", "saves.restoreDone": "Sauvegarde restaurée.", "saves.confirmRestore": "Restaurer cette sauvegarde ? Les fichiers actuels seront remplacés.",
    "ach.none": "Aucun succès disponible.", "ach.locked": "Verrouillé", "ach.unlocked": "Débloqué",
    "installs.title": "Installations", "installs.active": "Active", "installs.activate": "Utiliser cette installation",
  },
  en: {
    "x.dllFolder": "Scanned folder", "x.dllChange": "Change folder", "x.dllScan": "Scan", "x.dllNone": "No DLL found in this folder.", "x.dllActive": "Active overrides", "pv.more": "Load more",
    "x.media": "Media", "x.noDescription": "No description.", "x.noMedia": "No media available.",
    "x.trailer": "Trailer", "x.trailerUnavailable": "Built-in player unavailable: the trailer can be opened on Steam.", "art.auto": "Search automatically", "x.save": "Save", "x.cancel": "Cancel", "x.close": "Close",
    "notes.placeholder": "Your notes about this game…", "launch.prefix": "Prefix", "launch.options": "Options",
    "launch.env": "Environment variables", "launch.anticheat": "Anti-cheat",
    "pv.mygames": "My games", "pv.title": "Store library", "pv.subtitle": "Steam, Epic Games, GOG and itch.io in one place.",
    "pv.loading": "Loading stores…", "pv.emptyHint": "Connect a store in Settings to find your games.",
    "pv.empty": "No store game synced yet.", "pv.noMatch": "No game matches these filters.",
    "pv.all": "All", "pv.search": "Search store games…", "pv.filter.all": "All", "pv.filter.installed": "Installed", "pv.filter.missing": "Not installed",
    "pv.syncAll": "Sync all", "pv.sync": "Sync", "pv.synced": "Libraries synced.", "pv.syncHint": "Each store syncs independently.",
    "pv.importSteam": "Import from Steam", "pv.importSteamHint": "Installed games found in local Steam libraries.",
    "pv.import": "Import", "pv.imported": "{n} game(s) imported.", "pv.noSteam": "No installed Steam game found.", "pv.noneSelected": "No game selected.", "pv.already": "already in Quark",
    "pv.open": "Open", "pv.adopt": "Add to Quark", "pv.adopted": "{name} added to the library.", "pv.install": "Install", "pv.installSteam": "Install via Steam",
    "pv.steamInstallHint": "Steam opens on the game page and handles the download.", "pv.openSteam": "Open Steam",
    "pv.uninstall": "Uninstall", "pv.uninstallBody": "Delete {name} files from disk?", "pv.uninstalling": "Uninstalling…", "pv.uninstalled": "Game uninstalled.",
    "pv.destination": "Install folder", "pv.installStarted": "Installation started.", "pv.installing": "Installing…",
    "pv.installedAt": "Installed · {path}", "pv.owned": "Owned · not installed", "pv.state.installed": "Installed", "pv.state.not_installed": "Not installed",
    "pv.state.installing": "Installing…", "pv.state.failed": "Failed", "pv.state.unknown": "Unknown", "pv.state.queued": "Queued",
    "pv.state.updating": "Updating", "pv.state.repairing": "Repairing",
    "pv.accounts": "Accounts", "pv.accountsHint": "Passwords never pass through Quark: only authorization codes are exchanged.",
    "pv.connect": "Connect", "pv.disconnect": "Disconnect", "pv.disconnectBody": "The local session will be forgotten. Your games and saves stay on disk.",
    "pv.disconnected": "Store disconnected.", "pv.connecting": "Connecting…", "pv.authCode": "Authorization code",
    "pv.codeRequired": "Paste the authorization code.", "pv.openLogin": "Open login page",
    "pv.epicLoginHint": "Sign in on the Legendary page, copy the authorizationCode value and paste it here.",
    "pv.gogLoginHint": "Sign in on GOG, copy the code shown at the end of the return address and paste it here.",
    "pv.steamCreds": "Steam credentials", "pv.steamHint": "Optional: the Web API key and SteamID64 add the owned library and achievements.",
    "pv.itchKey": "itch.io API key", "pv.itchHint": "Create a personal API key on itch.io. It stays in the protected local file.",
    "pv.tools": "Managed tools", "pv.toolsHint": "Legendary, gogdl and Ludusavi are downloaded from official releases and verified with SHA-256.",
    "pv.toolReady": "Installed and ready", "pv.toolMissing": "Not installed", "pv.installTool": "Install", "pv.toolInstalling": "Downloading tool…", "pv.toolDone": "Tool installed.",
    "pv.reload": "Refresh", "pv.connected": "Connected",
    "onboarding.title": "Welcome to Quark", "onboarding.subtitle": "A few choices to get started; everything can be changed later.",
    "onboarding.language": "Language", "onboarding.theme": "Theme", "onboarding.start": "Startup page", "onboarding.install": "Store games folder",
    "onboarding.finish": "Get started", "onboarding.skip": "Later",
    "whatsnew.title": "What's new", "whatsnew.body": "This version modernises the whole interface: list on the left, game page on the right, store, my games, achievements, media and saves.",
    "views.title": "Saved views", "views.save": "Save current view", "views.name": "View name", "views.empty": "No saved view.",
    "views.apply": "Apply", "views.delete": "Delete", "views.saved": "View saved.", "views.deleted": "View deleted.",
    "batch.selected": "{n} selected", "batch.remove": "Remove selection", "batch.removeBody": "Remove {n} game(s) from the library? Files are not deleted.",
    "batch.clear": "Clear selection",
    "settings.providers": "Stores & tools", "settings.savedViews": "Saved views",
    "overlay.title": "In-game overlay", "overlay.show": "Show overlay", "overlay.hide": "Hide overlay", "overlay.hint": "Shift+Backspace opens the panel over the game.",
    "saves.title": "Local saves", "saves.hint": "Ludusavi creates local snapshots before and after play. Nothing is sent without your consent.", "saves.preview": "Preview", "saves.backup": "Create backup", "saves.backups": "Backups", "saves.restore": "Restore",
    "saves.none": "No backup.", "saves.files": "Files", "saves.changed": "Changes", "saves.addFolder": "Add folder",
    "saves.folder": "Save folder", "saves.cloud": "Sync cloud", "saves.cloudDone": "Cloud saves synced.",
    "saves.backupDone": "Backup created.", "saves.restoreDone": "Backup restored.", "saves.confirmRestore": "Restore this backup? Current files will be replaced.",
    "ach.none": "No achievement available.", "ach.locked": "Locked", "ach.unlocked": "Unlocked",
    "installs.title": "Installations", "installs.active": "Active", "installs.activate": "Use this installation",
  },
  es: {
    "x.dllFolder": "Carpeta analizada", "x.dllChange": "Cambiar carpeta", "x.dllScan": "Analizar", "x.dllNone": "No se encontró ninguna DLL en esta carpeta.", "x.dllActive": "Overrides activos", "pv.more": "Cargar más",
    "x.media": "Medios", "x.noDescription": "Sin descripción.", "x.noMedia": "No hay medios disponibles.",
    "x.trailer": "Tráiler", "x.trailerUnavailable": "Reproductor integrado no disponible: el tráiler se puede abrir en Steam.", "art.auto": "Buscar automáticamente", "x.save": "Guardar", "x.cancel": "Cancelar", "x.close": "Cerrar",
    "notes.placeholder": "Tus notas sobre este juego…", "launch.prefix": "Prefijo", "launch.options": "Opciones",
    "launch.env": "Variables de entorno", "launch.anticheat": "Antitrampas",
    "pv.mygames": "Mis juegos", "pv.title": "Biblioteca de tiendas", "pv.subtitle": "Steam, Epic Games, GOG e itch.io en un solo lugar.",
    "pv.loading": "Cargando tiendas…", "pv.emptyHint": "Conecta una tienda en Ajustes para encontrar tus juegos.",
    "pv.empty": "Ningún juego de tienda sincronizado.", "pv.noMatch": "Ningún juego coincide con los filtros.",
    "pv.all": "Todas", "pv.search": "Buscar un juego de tienda…", "pv.filter.all": "Todos", "pv.filter.installed": "Instalados", "pv.filter.missing": "No instalados",
    "pv.syncAll": "Sincronizar todo", "pv.sync": "Sincronizar", "pv.synced": "Bibliotecas sincronizadas.", "pv.syncHint": "Cada tienda se sincroniza de forma independiente.",
    "pv.importSteam": "Importar de Steam", "pv.importSteamHint": "Juegos instalados detectados en las bibliotecas locales de Steam.",
    "pv.import": "Importar", "pv.imported": "{n} juego(s) importado(s).", "pv.noSteam": "No se detectó ningún juego de Steam instalado.", "pv.noneSelected": "Ningún juego seleccionado.", "pv.already": "ya en Quark",
    "pv.open": "Abrir", "pv.adopt": "Añadir a Quark", "pv.adopted": "{name} añadido a la biblioteca.", "pv.install": "Instalar", "pv.installSteam": "Instalar con Steam",
    "pv.steamInstallHint": "Steam se abre en la página del juego y gestiona la descarga.", "pv.openSteam": "Abrir Steam",
    "pv.uninstall": "Desinstalar", "pv.uninstallBody": "¿Eliminar los archivos de {name} del disco?", "pv.uninstalling": "Desinstalando…", "pv.uninstalled": "Juego desinstalado.",
    "pv.destination": "Carpeta de instalación", "pv.installStarted": "Instalación iniciada.", "pv.installing": "Instalando…",
    "pv.installedAt": "Instalado · {path}", "pv.owned": "En propiedad · no instalado", "pv.state.installed": "Instalado", "pv.state.not_installed": "No instalado",
    "pv.state.installing": "Instalando…", "pv.state.failed": "Error", "pv.state.unknown": "Desconocido", "pv.state.queued": "En cola",
    "pv.state.updating": "Actualizando", "pv.state.repairing": "Reparando",
    "pv.accounts": "Cuentas", "pv.accountsHint": "Las contraseñas nunca pasan por Quark: solo se intercambian códigos de autorización.",
    "pv.connect": "Conectar", "pv.disconnect": "Desconectar", "pv.disconnectBody": "Se olvidará la sesión local. Tus juegos y partidas permanecen en el disco.",
    "pv.disconnected": "Tienda desconectada.", "pv.connecting": "Conectando…", "pv.authCode": "Código de autorización",
    "pv.codeRequired": "Pega el código de autorización.", "pv.openLogin": "Abrir página de conexión",
    "pv.epicLoginHint": "Inicia sesión en la página de Legendary, copia el valor authorizationCode y pégalo aquí.",
    "pv.gogLoginHint": "Inicia sesión en GOG, copia el código que aparece al final de la dirección de retorno y pégalo aquí.",
    "pv.steamCreds": "Credenciales de Steam", "pv.steamHint": "Opcional: la clave Web API y el SteamID64 añaden la biblioteca poseída y los logros.",
    "pv.itchKey": "Clave API de itch.io", "pv.itchHint": "Crea una clave API personal en itch.io. Permanece en el archivo local protegido.",
    "pv.tools": "Herramientas gestionadas", "pv.toolsHint": "Legendary, gogdl y Ludusavi se descargan desde sus versiones oficiales y se verifican con SHA-256.",
    "pv.toolReady": "Instalada y lista", "pv.toolMissing": "No instalada", "pv.installTool": "Instalar", "pv.toolInstalling": "Descargando herramienta…", "pv.toolDone": "Herramienta instalada.",
    "pv.reload": "Actualizar", "pv.connected": "Conectado",
    "onboarding.title": "Bienvenido a Quark", "onboarding.subtitle": "Unas elecciones para empezar; todo se puede cambiar después.",
    "onboarding.language": "Idioma", "onboarding.theme": "Tema", "onboarding.start": "Página inicial", "onboarding.install": "Carpeta de juegos de tienda",
    "onboarding.finish": "Empezar", "onboarding.skip": "Más tarde",
    "whatsnew.title": "Novedades", "whatsnew.body": "Esta versión moderniza toda la interfaz: lista a la izquierda, ficha a la derecha, tienda, mis juegos, logros, medios y partidas guardadas.",
    "views.title": "Vistas guardadas", "views.save": "Guardar vista actual", "views.name": "Nombre de la vista", "views.empty": "Ninguna vista guardada.",
    "views.apply": "Aplicar", "views.delete": "Eliminar", "views.saved": "Vista guardada.", "views.deleted": "Vista eliminada.",
    "batch.selected": "{n} seleccionado(s)", "batch.remove": "Quitar selección", "batch.removeBody": "¿Quitar {n} juego(s) de la biblioteca? Los archivos no se eliminan.",
    "batch.clear": "Deseleccionar todo",
    "settings.providers": "Tiendas y herramientas", "settings.savedViews": "Vistas guardadas",
    "overlay.title": "Superposición en el juego", "overlay.show": "Mostrar superposición", "overlay.hide": "Ocultar superposición", "overlay.hint": "Shift+Backspace abre el panel sobre el juego.",
    "saves.title": "Partidas locales", "saves.hint": "Ludusavi crea instantáneas locales antes y después del juego. Nada se envía sin tu permiso.", "saves.preview": "Vista previa", "saves.backup": "Crear copia", "saves.backups": "Copias", "saves.restore": "Restaurar",
    "saves.none": "Ninguna copia.", "saves.files": "Archivos", "saves.changed": "Cambios", "saves.addFolder": "Añadir carpeta",
    "saves.folder": "Carpeta de partidas", "saves.cloud": "Sincronizar nube", "saves.cloudDone": "Partidas en la nube sincronizadas.",
    "saves.backupDone": "Copia creada.", "saves.restoreDone": "Copia restaurada.", "saves.confirmRestore": "¿Restaurar esta copia? Los archivos actuales se reemplazarán.",
    "ach.none": "Ningún logro disponible.", "ach.locked": "Bloqueado", "ach.unlocked": "Desbloqueado",
    "installs.title": "Instalaciones", "installs.active": "Activa", "installs.activate": "Usar esta instalación",
  },
  de: {
    "x.dllFolder": "Analysierter Ordner", "x.dllChange": "Ordner wechseln", "x.dllScan": "Analysieren", "x.dllNone": "Keine DLL in diesem Ordner gefunden.", "x.dllActive": "Aktive Overrides", "pv.more": "Mehr laden",
    "x.media": "Medien", "x.noDescription": "Keine Beschreibung.", "x.noMedia": "Keine Medien verfügbar.",
    "x.trailer": "Trailer", "x.trailerUnavailable": "Integrierter Player nicht verfügbar: der Trailer kann auf Steam geöffnet werden.", "art.auto": "Automatisch suchen", "x.save": "Speichern", "x.cancel": "Abbrechen", "x.close": "Schließen",
    "notes.placeholder": "Deine Notizen zu diesem Spiel…", "launch.prefix": "Präfix", "launch.options": "Optionen",
    "launch.env": "Umgebungsvariablen", "launch.anticheat": "Anti-Cheat",
    "pv.mygames": "Meine Spiele", "pv.title": "Store-Bibliothek", "pv.subtitle": "Steam, Epic Games, GOG und itch.io an einem Ort.",
    "pv.loading": "Stores werden geladen…", "pv.emptyHint": "Verbinde einen Store in den Einstellungen, um deine Spiele zu sehen.",
    "pv.empty": "Noch kein Store-Spiel synchronisiert.", "pv.noMatch": "Kein Spiel entspricht den Filtern.",
    "pv.all": "Alle", "pv.search": "Store-Spiel suchen…", "pv.filter.all": "Alle", "pv.filter.installed": "Installiert", "pv.filter.missing": "Nicht installiert",
    "pv.syncAll": "Alle synchronisieren", "pv.sync": "Synchronisieren", "pv.synced": "Bibliotheken synchronisiert.", "pv.syncHint": "Jeder Store wird unabhängig synchronisiert.",
    "pv.importSteam": "Aus Steam importieren", "pv.importSteamHint": "Installierte Spiele aus lokalen Steam-Bibliotheken.",
    "pv.import": "Importieren", "pv.imported": "{n} Spiel(e) importiert.", "pv.noSteam": "Kein installiertes Steam-Spiel gefunden.", "pv.noneSelected": "Kein Spiel ausgewählt.", "pv.already": "bereits in Quark",
    "pv.open": "Öffnen", "pv.adopt": "Zu Quark hinzufügen", "pv.adopted": "{name} zur Bibliothek hinzugefügt.", "pv.install": "Installieren", "pv.installSteam": "Über Steam installieren",
    "pv.steamInstallHint": "Steam öffnet die Spielseite und übernimmt den Download.", "pv.openSteam": "Steam öffnen",
    "pv.uninstall": "Deinstallieren", "pv.uninstallBody": "Dateien von {name} von der Festplatte löschen?", "pv.uninstalling": "Wird deinstalliert…", "pv.uninstalled": "Spiel deinstalliert.",
    "pv.destination": "Installationsordner", "pv.installStarted": "Installation gestartet.", "pv.installing": "Installation…",
    "pv.installedAt": "Installiert · {path}", "pv.owned": "Im Besitz · nicht installiert", "pv.state.installed": "Installiert", "pv.state.not_installed": "Nicht installiert",
    "pv.state.installing": "Installation…", "pv.state.failed": "Fehlgeschlagen", "pv.state.unknown": "Unbekannt", "pv.state.queued": "Warteschlange",
    "pv.state.updating": "Aktualisierung", "pv.state.repairing": "Reparatur",
    "pv.accounts": "Konten", "pv.accountsHint": "Passwörter laufen nie über Quark: nur Autorisierungscodes werden ausgetauscht.",
    "pv.connect": "Verbinden", "pv.disconnect": "Trennen", "pv.disconnectBody": "Die lokale Sitzung wird vergessen. Spiele und Spielstände bleiben auf der Festplatte.",
    "pv.disconnected": "Store getrennt.", "pv.connecting": "Verbinden…", "pv.authCode": "Autorisierungscode",
    "pv.codeRequired": "Autorisierungscode einfügen.", "pv.openLogin": "Anmeldeseite öffnen",
    "pv.epicLoginHint": "Melde dich auf der Legendary-Seite an, kopiere den authorizationCode und füge ihn hier ein.",
    "pv.gogLoginHint": "Melde dich bei GOG an, kopiere den Code am Ende der Rücksprungadresse und füge ihn hier ein.",
    "pv.steamCreds": "Steam-Zugangsdaten", "pv.steamHint": "Optional: Web-API-Schlüssel und SteamID64 ergänzen die eigene Bibliothek und Erfolge.",
    "pv.itchKey": "itch.io API-Schlüssel", "pv.itchHint": "Erstelle einen persönlichen API-Schlüssel auf itch.io. Er bleibt in der geschützten lokalen Datei.",
    "pv.tools": "Verwaltete Werkzeuge", "pv.toolsHint": "Legendary, gogdl und Ludusavi werden aus offiziellen Releases geladen und per SHA-256 geprüft.",
    "pv.toolReady": "Installiert und bereit", "pv.toolMissing": "Nicht installiert", "pv.installTool": "Installieren", "pv.toolInstalling": "Werkzeug wird geladen…", "pv.toolDone": "Werkzeug installiert.",
    "pv.reload": "Aktualisieren", "pv.connected": "Verbunden",
    "onboarding.title": "Willkommen bei Quark", "onboarding.subtitle": "Ein paar Entscheidungen für den Start; alles ist später änderbar.",
    "onboarding.language": "Sprache", "onboarding.theme": "Design", "onboarding.start": "Startseite", "onboarding.install": "Ordner für Store-Spiele",
    "onboarding.finish": "Loslegen", "onboarding.skip": "Später",
    "whatsnew.title": "Neuigkeiten", "whatsnew.body": "Diese Version modernisiert die gesamte Oberfläche: Liste links, Spieldetails rechts, Store, meine Spiele, Erfolge, Medien und Spielstände.",
    "views.title": "Gespeicherte Ansichten", "views.save": "Aktuelle Ansicht speichern", "views.name": "Name der Ansicht", "views.empty": "Keine gespeicherte Ansicht.",
    "views.apply": "Anwenden", "views.delete": "Löschen", "views.saved": "Ansicht gespeichert.", "views.deleted": "Ansicht gelöscht.",
    "batch.selected": "{n} ausgewählt", "batch.remove": "Auswahl entfernen", "batch.removeBody": "{n} Spiel(e) aus der Bibliothek entfernen? Dateien werden nicht gelöscht.",
    "batch.clear": "Auswahl aufheben",
    "settings.providers": "Stores & Werkzeuge", "settings.savedViews": "Gespeicherte Ansichten",
    "overlay.title": "In-Game-Overlay", "overlay.show": "Overlay anzeigen", "overlay.hide": "Overlay ausblenden", "overlay.hint": "Shift+Backspace öffnet das Panel über dem Spiel.",
    "saves.title": "Lokale Spielstände", "saves.hint": "Ludusavi erstellt lokale Schnappschüsse vor und nach dem Spiel. Nichts wird ohne Zustimmung gesendet.", "saves.preview": "Vorschau", "saves.backup": "Sicherung erstellen", "saves.backups": "Sicherungen", "saves.restore": "Wiederherstellen",
    "saves.none": "Keine Sicherung.", "saves.files": "Dateien", "saves.changed": "Änderungen", "saves.addFolder": "Ordner hinzufügen",
    "saves.folder": "Spielstand-Ordner", "saves.cloud": "Cloud synchronisieren", "saves.cloudDone": "Cloud-Spielstände synchronisiert.",
    "saves.backupDone": "Sicherung erstellt.", "saves.restoreDone": "Sicherung wiederhergestellt.", "saves.confirmRestore": "Diese Sicherung wiederherstellen? Aktuelle Dateien werden ersetzt.",
    "ach.none": "Kein Erfolg verfügbar.", "ach.locked": "Gesperrt", "ach.unlocked": "Freigeschaltet",
    "installs.title": "Installationen", "installs.active": "Aktiv", "installs.activate": "Diese Installation verwenden",
  },
  pt: {
    "x.dllFolder": "Pasta analisada", "x.dllChange": "Mudar de pasta", "x.dllScan": "Analisar", "x.dllNone": "Nenhuma DLL encontrada nesta pasta.", "x.dllActive": "Overrides ativos", "pv.more": "Carregar mais",
    "x.media": "Mídias", "x.noDescription": "Sem descrição.", "x.noMedia": "Nenhuma mídia disponível.",
    "x.trailer": "Trailer", "x.trailerUnavailable": "Reprodutor integrado indisponível: o trailer pode ser aberto na Steam.", "art.auto": "Pesquisar automaticamente", "x.save": "Salvar", "x.cancel": "Cancelar", "x.close": "Fechar",
    "notes.placeholder": "Suas notas sobre este jogo…", "launch.prefix": "Prefixo", "launch.options": "Opções",
    "launch.env": "Variáveis de ambiente", "launch.anticheat": "Antitrapaça",
    "pv.mygames": "Meus jogos", "pv.title": "Biblioteca das lojas", "pv.subtitle": "Steam, Epic Games, GOG e itch.io no mesmo lugar.",
    "pv.loading": "Carregando lojas…", "pv.emptyHint": "Conecte uma loja nos ajustes para encontrar seus jogos.",
    "pv.empty": "Nenhum jogo de loja sincronizado.", "pv.noMatch": "Nenhum jogo corresponde aos filtros.",
    "pv.all": "Todas", "pv.search": "Pesquisar um jogo de loja…", "pv.filter.all": "Todos", "pv.filter.installed": "Instalados", "pv.filter.missing": "Não instalados",
    "pv.syncAll": "Sincronizar tudo", "pv.sync": "Sincronizar", "pv.synced": "Bibliotecas sincronizadas.", "pv.syncHint": "Cada loja sincroniza de forma independente.",
    "pv.importSteam": "Importar do Steam", "pv.importSteamHint": "Jogos instalados detectados nas bibliotecas locais do Steam.",
    "pv.import": "Importar", "pv.imported": "{n} jogo(s) importado(s).", "pv.noSteam": "Nenhum jogo do Steam instalado detectado.", "pv.noneSelected": "Nenhum jogo selecionado.", "pv.already": "já no Quark",
    "pv.open": "Abrir", "pv.adopt": "Adicionar ao Quark", "pv.adopted": "{name} adicionado à biblioteca.", "pv.install": "Instalar", "pv.installSteam": "Instalar pelo Steam",
    "pv.steamInstallHint": "O Steam abre a página do jogo e cuida do download.", "pv.openSteam": "Abrir Steam",
    "pv.uninstall": "Desinstalar", "pv.uninstallBody": "Excluir os arquivos de {name} do disco?", "pv.uninstalling": "Desinstalando…", "pv.uninstalled": "Jogo desinstalado.",
    "pv.destination": "Pasta de instalação", "pv.installStarted": "Instalação iniciada.", "pv.installing": "Instalando…",
    "pv.installedAt": "Instalado · {path}", "pv.owned": "Possuído · não instalado", "pv.state.installed": "Instalado", "pv.state.not_installed": "Não instalado",
    "pv.state.installing": "Instalando…", "pv.state.failed": "Falhou", "pv.state.unknown": "Desconhecido", "pv.state.queued": "Na fila",
    "pv.state.updating": "Atualizando", "pv.state.repairing": "Reparando",
    "pv.accounts": "Contas", "pv.accountsHint": "As senhas nunca passam pelo Quark: apenas códigos de autorização são trocados.",
    "pv.connect": "Conectar", "pv.disconnect": "Desconectar", "pv.disconnectBody": "A sessão local será esquecida. Seus jogos e saves permanecem no disco.",
    "pv.disconnected": "Loja desconectada.", "pv.connecting": "Conectando…", "pv.authCode": "Código de autorização",
    "pv.codeRequired": "Cole o código de autorização.", "pv.openLogin": "Abrir página de login",
    "pv.epicLoginHint": "Entre na página do Legendary, copie o valor authorizationCode e cole aqui.",
    "pv.gogLoginHint": "Entre no GOG, copie o código exibido no fim do endereço de retorno e cole aqui.",
    "pv.steamCreds": "Credenciais do Steam", "pv.steamHint": "Opcional: a chave Web API e o SteamID64 adicionam a biblioteca possuída e as conquistas.",
    "pv.itchKey": "Chave API do itch.io", "pv.itchHint": "Crie uma chave API pessoal no itch.io. Ela fica no arquivo local protegido.",
    "pv.tools": "Ferramentas gerenciadas", "pv.toolsHint": "Legendary, gogdl e Ludusavi são baixados das releases oficiais e verificados com SHA-256.",
    "pv.toolReady": "Instalada e pronta", "pv.toolMissing": "Não instalada", "pv.installTool": "Instalar", "pv.toolInstalling": "Baixando ferramenta…", "pv.toolDone": "Ferramenta instalada.",
    "pv.reload": "Atualizar", "pv.connected": "Conectado",
    "onboarding.title": "Bem-vindo ao Quark", "onboarding.subtitle": "Algumas escolhas para começar; tudo pode ser alterado depois.",
    "onboarding.language": "Idioma", "onboarding.theme": "Tema", "onboarding.start": "Página inicial", "onboarding.install": "Pasta dos jogos de loja",
    "onboarding.finish": "Começar", "onboarding.skip": "Depois",
    "whatsnew.title": "Novidades", "whatsnew.body": "Esta versão moderniza toda a interface: lista à esquerda, ficha à direita, loja, meus jogos, conquistas, mídias e saves.",
    "views.title": "Visualizações salvas", "views.save": "Salvar visualização atual", "views.name": "Nome da visualização", "views.empty": "Nenhuma visualização salva.",
    "views.apply": "Aplicar", "views.delete": "Excluir", "views.saved": "Visualização salva.", "views.deleted": "Visualização excluída.",
    "batch.selected": "{n} selecionado(s)", "batch.remove": "Remover seleção", "batch.removeBody": "Remover {n} jogo(s) da biblioteca? Os arquivos não são excluídos.",
    "batch.clear": "Limpar seleção",
    "settings.providers": "Lojas e ferramentas", "settings.savedViews": "Visualizações salvas",
    "overlay.title": "Overlay no jogo", "overlay.show": "Mostrar overlay", "overlay.hide": "Ocultar overlay", "overlay.hint": "Shift+Backspace abre o painel sobre o jogo.",
    "saves.title": "Saves locais", "saves.hint": "O Ludusavi cria instantâneos locais antes e depois do jogo. Nada é enviado sem o seu consentimento.", "saves.preview": "Prévia", "saves.backup": "Criar backup", "saves.backups": "Backups", "saves.restore": "Restaurar",
    "saves.none": "Nenhum backup.", "saves.files": "Arquivos", "saves.changed": "Alterações", "saves.addFolder": "Adicionar pasta",
    "saves.folder": "Pasta de saves", "saves.cloud": "Sincronizar nuvem", "saves.cloudDone": "Saves na nuvem sincronizados.",
    "saves.backupDone": "Backup criado.", "saves.restoreDone": "Backup restaurado.", "saves.confirmRestore": "Restaurar este backup? Os arquivos atuais serão substituídos.",
    "ach.none": "Nenhuma conquista disponível.", "ach.locked": "Bloqueada", "ach.unlocked": "Desbloqueada",
    "installs.title": "Instalações", "installs.active": "Ativa", "installs.activate": "Usar esta instalação",
  },
};

for (const lang of Object.keys(EXTRA_I18N)) Object.assign(I18N[lang], EXTRA_I18N[lang]);

Object.assign(I18N.es, {
  "x.home": "Inicio", "x.store": "Tienda", "x.mygames": "Mis juegos", "x.inGame": "Jugando", "x.palette": "Ir a un juego o acción…",
  "x.none": "Sin resultados", "x.filters": "Filtros", "x.groupBy": "Agrupar", "x.back": "Biblioteca", "x.news": "Noticias", "x.ach": "Logros", "x.hist": "Historial",
  "x.overview": "Resumen", "x.refresh": "Actualizar", "x.noNews": "No hay noticias para este juego.", "x.noAppId": "Este juego no tiene id de Steam: las noticias y notas no están disponibles.",
  "x.settingsGame": "Ajustes del juego", "x.customize": "Personalizar", "x.logBtn": "Registro de sesión", "x.remove": "Quitar de la biblioteca", "x.weeks": "Últimas 12 semanas",
  "x.main": "Historia principal", "x.extra": "Historia + extras", "x.compl": "Completista", "x.hltbLink": "Ver en HowLongToBeat", "x.search": "Buscar", "x.updates": "Actualizaciones",
  "x.view": "Vista", "x.add": "Añadir un juego", "x.soon": "Próximamente", "x.soonBody": "Esta pantalla llega en el próximo lote del port.", "x.sessions": "Sesiones", "x.allStatuses": "Todos los estados",
  "x.allGenres": "Todos los géneros", "x.allTags": "Todas las etiquetas", "x.favOnly": "Solo favoritos", "x.reset": "Restablecer", "x.continue": "Continuar", "x.favorites": "Favoritos", "x.backlog": "Pendientes",
  "x.save": "Guardar", "x.general": "General", "x.proton": "Proton", "x.env": "Variables", "x.data": "Datos", "x.dll": "DLL overrides", "x.settingsTab": "Ajustes",
  "x.noEnv": "Ninguna variable.", "x.addVar": "Añadir una variable", "x.addDll": "Añadir una DLL", "x.noDll": "Sin overrides.", "x.native": "native", "x.builtin": "builtin",
  "x.updateInstall": "Instalar y reiniciar", "x.updateInstalling": "Descargando y verificando…",
});
Object.assign(I18N.de, {
  "x.home": "Start", "x.store": "Shop", "x.mygames": "Meine Spiele", "x.inGame": "Läuft", "x.palette": "Zu einem Spiel oder einer Aktion springen…",
  "x.none": "Keine Treffer", "x.filters": "Filter", "x.groupBy": "Gruppieren", "x.back": "Bibliothek", "x.news": "Neuigkeiten", "x.ach": "Erfolge", "x.hist": "Verlauf",
  "x.overview": "Übersicht", "x.refresh": "Aktualisieren", "x.noNews": "Keine Neuigkeiten für dieses Spiel.", "x.noAppId": "Dieses Spiel hat keine Steam-ID: Neuigkeiten und Bewertungen sind nicht verfügbar.",
  "x.settingsGame": "Spieleinstellungen", "x.customize": "Anpassen", "x.logBtn": "Sitzungsprotokoll", "x.remove": "Aus der Bibliothek entfernen", "x.weeks": "Letzte 12 Wochen",
  "x.main": "Hauptstory", "x.extra": "Story + Extras", "x.compl": "Komplettist", "x.hltbLink": "Auf HowLongToBeat ansehen", "x.search": "Suchen", "x.updates": "Updates",
  "x.view": "Ansicht", "x.add": "Spiel hinzufügen", "x.soon": "Demnächst", "x.soonBody": "Dieser Bildschirm kommt im nächsten Portierungsschritt.", "x.sessions": "Sitzungen", "x.allStatuses": "Alle Status",
  "x.allGenres": "Alle Genres", "x.allTags": "Alle Tags", "x.favOnly": "Nur Favoriten", "x.reset": "Zurücksetzen", "x.continue": "Weiter", "x.favorites": "Favoriten", "x.backlog": "Warteliste",
  "x.save": "Speichern", "x.general": "Allgemein", "x.proton": "Proton", "x.env": "Variablen", "x.data": "Daten", "x.dll": "DLL overrides", "x.settingsTab": "Einstellungen",
  "x.noEnv": "Keine Variablen.", "x.addVar": "Variable hinzufügen", "x.addDll": "DLL hinzufügen", "x.noDll": "Keine Overrides.", "x.native": "native", "x.builtin": "builtin",
  "x.updateInstall": "Installieren und neu starten", "x.updateInstalling": "Herunterladen und prüfen…",
});
Object.assign(I18N.pt, {
  "x.home": "Início", "x.store": "Loja", "x.mygames": "Meus jogos", "x.inGame": "Jogando", "x.palette": "Ir para um jogo ou ação…",
  "x.none": "Nenhum resultado", "x.filters": "Filtros", "x.groupBy": "Agrupar", "x.back": "Biblioteca", "x.news": "Notícias", "x.ach": "Conquistas", "x.hist": "Histórico",
  "x.overview": "Visão geral", "x.refresh": "Atualizar", "x.noNews": "Nenhuma notícia para este jogo.", "x.noAppId": "Este jogo não tem id da Steam: notícias e notas não estão disponíveis.",
  "x.settingsGame": "Configurações do jogo", "x.customize": "Personalizar", "x.logBtn": "Registro da sessão", "x.remove": "Remover da biblioteca", "x.weeks": "Últimas 12 semanas",
  "x.main": "História principal", "x.extra": "História + extras", "x.compl": "Completista", "x.hltbLink": "Ver no HowLongToBeat", "x.search": "Pesquisar", "x.updates": "Atualizações",
  "x.view": "Visualização", "x.add": "Adicionar um jogo", "x.soon": "Em breve", "x.soonBody": "Esta tela chega no próximo lote do port.", "x.sessions": "Sessões", "x.allStatuses": "Todos os estados",
  "x.allGenres": "Todos os gêneros", "x.allTags": "Todas as tags", "x.favOnly": "Somente favoritos", "x.reset": "Redefinir", "x.continue": "Continuar", "x.favorites": "Favoritos", "x.backlog": "Pendentes",
  "x.save": "Salvar", "x.general": "Geral", "x.proton": "Proton", "x.env": "Variáveis", "x.data": "Dados", "x.dll": "DLL overrides", "x.settingsTab": "Configurações",
  "x.noEnv": "Nenhuma variável.", "x.addVar": "Adicionar uma variável", "x.addDll": "Adicionar uma DLL", "x.noDll": "Nenhum override.", "x.native": "native", "x.builtin": "builtin",
  "x.updateInstall": "Instalar e reiniciar", "x.updateInstalling": "Baixando e verificando…",
});

Object.assign(I18N.fr, { "ach.storeOnly": "Les succès sont disponibles pour les jeux de boutique installés.", "x.noSteamId": "Ce jeu n'a pas d'identifiant Steam.", "x.noGame": "Jeu introuvable.", "x.noStoreInstall": "Aucune installation de boutique vérifiée." });
Object.assign(I18N.en, { "ach.storeOnly": "Achievements are available for installed store games.", "x.noSteamId": "This game has no Steam ID.", "x.noGame": "Game not found.", "x.noStoreInstall": "No verified store installation." });
Object.assign(I18N.es, { "ach.storeOnly": "Los logros están disponibles para los juegos de tienda instalados.", "x.noSteamId": "Este juego no tiene identificador de Steam.", "x.noGame": "Juego no encontrado.", "x.noStoreInstall": "Ninguna instalación de tienda verificada." });
Object.assign(I18N.de, { "ach.storeOnly": "Erfolge sind für installierte Store-Spiele verfügbar.", "x.noSteamId": "Dieses Spiel hat keine Steam-ID.", "x.noGame": "Spiel nicht gefunden.", "x.noStoreInstall": "Keine verifizierte Store-Installation." });
Object.assign(I18N.pt, { "ach.storeOnly": "As conquistas estão disponíveis para jogos de loja instalados.", "x.noSteamId": "Este jogo não tem identificador Steam.", "x.noGame": "Jogo não encontrado.", "x.noStoreInstall": "Nenhuma instalação de loja verificada." });

Object.assign(I18N.fr, { "pv.legendaryMissing": "Legendary n'est pas installé", "pv.legendaryInstalled": "Legendary installé · compte non connecté", "pv.legendarySession": "Session Legendary", "pv.epicDirect": "Connexion directe", "pv.gogMissing": "gogdl n'est pas installé", "pv.gogInstalled": "gogdl installé · compte non connecté", "pv.gogSession": "Session gogdl", "pv.itchSaved": "Clé API personnelle enregistrée", "pv.itchHint": "Clé API personnelle, pour la bibliothèque possédée" });
Object.assign(I18N.en, { "pv.legendaryMissing": "Legendary is not installed", "pv.legendaryInstalled": "Legendary installed · no account connected", "pv.legendarySession": "Legendary session", "pv.epicDirect": "Direct connection", "pv.gogMissing": "gogdl is not installed", "pv.gogInstalled": "gogdl installed · no account connected", "pv.gogSession": "gogdl session", "pv.itchSaved": "Personal API key saved", "pv.itchHint": "Personal API key for your owned library" });
Object.assign(I18N.es, { "pv.legendaryMissing": "Legendary no está instalado", "pv.legendaryInstalled": "Legendary instalado · cuenta no conectada", "pv.legendarySession": "Sesión de Legendary", "pv.epicDirect": "Conexión directa", "pv.gogMissing": "gogdl no está instalado", "pv.gogInstalled": "gogdl instalado · cuenta no conectada", "pv.gogSession": "Sesión de gogdl", "pv.itchSaved": "Clave API personal guardada", "pv.itchHint": "Clave API personal para tu biblioteca" });
Object.assign(I18N.de, { "pv.legendaryMissing": "Legendary ist nicht installiert", "pv.legendaryInstalled": "Legendary installiert · kein Konto verbunden", "pv.legendarySession": "Legendary-Sitzung", "pv.epicDirect": "Direkte Verbindung", "pv.gogMissing": "gogdl ist nicht installiert", "pv.gogInstalled": "gogdl installiert · kein Konto verbunden", "pv.gogSession": "gogdl-Sitzung", "pv.itchSaved": "Persönlicher API-Schlüssel gespeichert", "pv.itchHint": "Persönlicher API-Schlüssel für deine Bibliothek" });
Object.assign(I18N.pt, { "pv.legendaryMissing": "O Legendary não está instalado", "pv.legendaryInstalled": "Legendary instalado · conta não conectada", "pv.legendarySession": "Sessão do Legendary", "pv.epicDirect": "Conexão direta", "pv.gogMissing": "O gogdl não está instalado", "pv.gogInstalled": "gogdl instalado · conta não conectada", "pv.gogSession": "Sessão do gogdl", "pv.itchSaved": "Chave API pessoal salva", "pv.itchHint": "Chave API pessoal para sua biblioteca" });

Object.assign(I18N.fr, { "x.logo": "Logo", "x.icon": "Icône", "art.search": "Rechercher une illustration" });
Object.assign(I18N.en, { "x.logo": "Logo", "x.icon": "Icon", "art.search": "Search artwork" });
Object.assign(I18N.es, { "x.logo": "Logotipo", "x.icon": "Icono", "art.search": "Buscar ilustración" });
Object.assign(I18N.de, { "x.logo": "Logo", "x.icon": "Symbol", "art.search": "Artwork suchen" });
Object.assign(I18N.pt, { "x.logo": "Logotipo", "x.icon": "Ícone", "art.search": "Pesquisar arte" });

Object.assign(I18N.fr, { "st.specials": "Offres spéciales", "st.topSellers": "Meilleures ventes", "st.newReleases": "Nouveautés", "st.comingSoon": "Prochainement", "st.popular": "Populaires sur Steam" });
Object.assign(I18N.en, { "st.specials": "Specials", "st.topSellers": "Top sellers", "st.newReleases": "New releases", "st.comingSoon": "Coming soon", "st.popular": "Popular on Steam" });
Object.assign(I18N.es, { "st.specials": "Ofertas", "st.topSellers": "Más vendidos", "st.newReleases": "Novedades", "st.comingSoon": "Próximamente", "st.popular": "Populares en Steam" });
Object.assign(I18N.de, { "st.specials": "Angebote", "st.topSellers": "Bestseller", "st.newReleases": "Neuerscheinungen", "st.comingSoon": "Demnächst", "st.popular": "Beliebt auf Steam" });
Object.assign(I18N.pt, { "st.specials": "Ofertas", "st.topSellers": "Mais vendidos", "st.newReleases": "Novidades", "st.comingSoon": "Em breve", "st.popular": "Populares na Steam" });

Object.assign(I18N.fr, { "pv.filter.all": "Tous les statuts" });
Object.assign(I18N.en, { "pv.filter.all": "All statuses" });
Object.assign(I18N.es, { "pv.filter.all": "Todos los estados" });
Object.assign(I18N.de, { "pv.filter.all": "Alle Status" });
Object.assign(I18N.pt, { "pv.filter.all": "Todos os estados" });

Object.assign(I18N.fr, {
  "deck.mode": "Mode Deck", "deck.exit": "Quitter le mode Deck", "deck.quit": "Quitter Quark", "deck.menu": "Menu Deck",
  "deck.hint.open": "Ouvrir", "deck.hint.back": "Retour", "deck.hint.play": "Jouer", "deck.hint.favorite": "Favori",
  "deck.hint.tabs": "Onglets", "deck.hint.menu": "Menu",
});
Object.assign(I18N.en, {
  "deck.mode": "Deck mode", "deck.exit": "Exit Deck mode", "deck.quit": "Quit Quark", "deck.menu": "Deck menu",
  "deck.hint.open": "Open", "deck.hint.back": "Back", "deck.hint.play": "Play", "deck.hint.favorite": "Favorite",
  "deck.hint.tabs": "Tabs", "deck.hint.menu": "Menu",
});
Object.assign(I18N.es, {
  "deck.mode": "Modo Deck", "deck.exit": "Salir del modo Deck", "deck.quit": "Salir de Quark", "deck.menu": "Menú Deck",
  "deck.hint.open": "Abrir", "deck.hint.back": "Atrás", "deck.hint.play": "Jugar", "deck.hint.favorite": "Favorito",
  "deck.hint.tabs": "Pestañas", "deck.hint.menu": "Menú",
});
Object.assign(I18N.de, {
  "deck.mode": "Deck-Modus", "deck.exit": "Deck-Modus verlassen", "deck.quit": "Quark beenden", "deck.menu": "Deck-Menü",
  "deck.hint.open": "Öffnen", "deck.hint.back": "Zurück", "deck.hint.play": "Spielen", "deck.hint.favorite": "Favorit",
  "deck.hint.tabs": "Tabs", "deck.hint.menu": "Menü",
});
Object.assign(I18N.pt, {
  "deck.mode": "Modo Deck", "deck.exit": "Sair do modo Deck", "deck.quit": "Sair do Quark", "deck.menu": "Menu Deck",
  "deck.hint.open": "Abrir", "deck.hint.back": "Voltar", "deck.hint.play": "Jogar", "deck.hint.favorite": "Favorito",
  "deck.hint.tabs": "Abas", "deck.hint.menu": "Menu",
});

Object.assign(I18N.fr, {
  "pv.transfers": "Téléchargements", "pv.noTransfers": "Aucun téléchargement en cours.", "pv.cancel": "Annuler",
  "pv.cancelling": "Annulation…", "pv.cancelled": "Téléchargement annulé.", "pv.preparing": "Préparation…",
  "pv.installFailed": "Le téléchargement a échoué.", "pv.installDone": "Installation terminée.",
});
Object.assign(I18N.en, {
  "pv.transfers": "Downloads", "pv.noTransfers": "No download in progress.", "pv.cancel": "Cancel",
  "pv.cancelling": "Cancelling…", "pv.cancelled": "Download cancelled.", "pv.preparing": "Preparing…",
  "pv.installFailed": "The download failed.", "pv.installDone": "Installation complete.",
});
Object.assign(I18N.es, {
  "pv.transfers": "Descargas", "pv.noTransfers": "No hay descargas en curso.", "pv.cancel": "Cancelar",
  "pv.cancelling": "Cancelando…", "pv.cancelled": "Descarga cancelada.", "pv.preparing": "Preparando…",
  "pv.installFailed": "La descarga falló.", "pv.installDone": "Instalación terminada.",
});
Object.assign(I18N.de, {
  "pv.transfers": "Downloads", "pv.noTransfers": "Kein Download läuft.", "pv.cancel": "Abbrechen",
  "pv.cancelling": "Wird abgebrochen…", "pv.cancelled": "Download abgebrochen.", "pv.preparing": "Vorbereitung…",
  "pv.installFailed": "Der Download ist fehlgeschlagen.", "pv.installDone": "Installation abgeschlossen.",
});
Object.assign(I18N.pt, {
  "pv.transfers": "Downloads", "pv.noTransfers": "Nenhum download em andamento.", "pv.cancel": "Cancelar",
  "pv.cancelling": "Cancelando…", "pv.cancelled": "Download cancelado.", "pv.preparing": "Preparando…",
  "pv.installFailed": "O download falhou.", "pv.installDone": "Instalação concluída.",
});

Object.assign(I18N.fr, {
  "scan.title": "Scanner un dossier", "scan.placeholder": "Dossier contenant les jeux", "scan.hint": "Choisissez un dossier : Quark détecte les exécutables et ignore les doublons.",
  "scan.scanning": "Analyse…", "scan.found": "{n} jeu(x) trouvé(s).", "scan.none": "Aucun exécutable trouvé.", "scan.add": "Ajouter la sélection", "scan.pickFirst": "Choisissez d'abord un dossier.", "scan.added": "{n} jeu(x) ajouté(s).",
  "art.choose": "Choisir une image", "art.loading": "Recherche des illustrations…", "art.none": "Aucune illustration trouvée.", "art.cover": "Couverture", "art.hero": "Bannière", "art.logo": "Logo", "art.icon": "Icône",
});
Object.assign(I18N.en, {
  "scan.title": "Scan a folder", "scan.placeholder": "Folder containing games", "scan.hint": "Pick a folder: Quark finds executables and skips duplicates.",
  "scan.scanning": "Scanning…", "scan.found": "{n} game(s) found.", "scan.none": "No executable found.", "scan.add": "Add selected", "scan.pickFirst": "Pick a folder first.", "scan.added": "{n} game(s) added.",
  "art.choose": "Choose an image", "art.loading": "Finding artwork…", "art.none": "No artwork found.", "art.cover": "Cover", "art.hero": "Banner", "art.logo": "Logo", "art.icon": "Icon",
});
Object.assign(I18N.es, {
  "scan.title": "Escanear una carpeta", "scan.placeholder": "Carpeta con los juegos", "scan.hint": "Elige una carpeta: Quark detecta ejecutables y evita duplicados.",
  "scan.scanning": "Analizando…", "scan.found": "{n} juego(s) encontrado(s).", "scan.none": "No se encontró ningún ejecutable.", "scan.add": "Añadir selección", "scan.pickFirst": "Elige primero una carpeta.", "scan.added": "{n} juego(s) añadido(s).",
  "art.choose": "Elegir una imagen", "art.loading": "Buscando ilustraciones…", "art.none": "No se encontró ninguna ilustración.", "art.cover": "Portada", "art.hero": "Banner", "art.logo": "Logo", "art.icon": "Icono",
});
Object.assign(I18N.de, {
  "scan.title": "Ordner scannen", "scan.placeholder": "Ordner mit den Spielen", "scan.hint": "Einen Ordner wählen: Quark findet Programme und überspringt Duplikate.",
  "scan.scanning": "Wird analysiert…", "scan.found": "{n} Spiel(e) gefunden.", "scan.none": "Kein Programm gefunden.", "scan.add": "Auswahl hinzufügen", "scan.pickFirst": "Zuerst einen Ordner wählen.", "scan.added": "{n} Spiel(e) hinzugefügt.",
  "art.choose": "Bild auswählen", "art.loading": "Illustrationen werden gesucht…", "art.none": "Keine Illustration gefunden.", "art.cover": "Cover", "art.hero": "Banner", "art.logo": "Logo", "art.icon": "Icon",
});
Object.assign(I18N.pt, {
  "scan.title": "Digitalizar uma pasta", "scan.placeholder": "Pasta com os jogos", "scan.hint": "Escolha uma pasta: o Quark encontra executáveis e ignora duplicados.",
  "scan.scanning": "A analisar…", "scan.found": "{n} jogo(s) encontrado(s).", "scan.none": "Nenhum executável encontrado.", "scan.add": "Adicionar seleção", "scan.pickFirst": "Escolha primeiro uma pasta.", "scan.added": "{n} jogo(s) adicionado(s).",
  "art.choose": "Escolher uma imagem", "art.loading": "A procurar ilustrações…", "art.none": "Nenhuma ilustração encontrada.", "art.cover": "Capa", "art.hero": "Banner", "art.logo": "Logo", "art.icon": "Ícone",
});

Object.assign(I18N.fr, { "web.title": "Chercher un jeu", "web.placeholder": "Nom du jeu", "web.search": "Rechercher", "web.none": "Aucun résultat.", "web.use": "Utiliser ce nom" });
Object.assign(I18N.en, { "web.title": "Find a game", "web.placeholder": "Game name", "web.search": "Search", "web.none": "No result.", "web.use": "Use this name" });
Object.assign(I18N.es, { "web.title": "Buscar un juego", "web.placeholder": "Nombre del juego", "web.search": "Buscar", "web.none": "Sin resultados.", "web.use": "Usar este nombre" });
Object.assign(I18N.de, { "web.title": "Spiel suchen", "web.placeholder": "Spielname", "web.search": "Suchen", "web.none": "Kein Ergebnis.", "web.use": "Namen übernehmen" });
Object.assign(I18N.pt, { "web.title": "Procurar um jogo", "web.placeholder": "Nome do jogo", "web.search": "Pesquisar", "web.none": "Sem resultados.", "web.use": "Usar este nome" });

Object.assign(I18N.fr, { "batch.favorite": "Favori", "batch.status": "Statut" });
Object.assign(I18N.en, { "batch.favorite": "Favorite", "batch.status": "Status" });
Object.assign(I18N.es, { "batch.favorite": "Favorito", "batch.status": "Estado" });
Object.assign(I18N.de, { "batch.favorite": "Favorit", "batch.status": "Status" });
Object.assign(I18N.pt, { "batch.favorite": "Favorito", "batch.status": "Estado" });

Object.assign(I18N.fr, { "ach.compare": "Comparer", "ach.compareTitle": "Comparer les succès", "ach.compareEmpty": "Aucun jeu de boutique associé à comparer." });
Object.assign(I18N.en, { "ach.compare": "Compare", "ach.compareTitle": "Compare achievements", "ach.compareEmpty": "No linked store game to compare." });
Object.assign(I18N.es, { "ach.compare": "Comparar", "ach.compareTitle": "Comparar logros", "ach.compareEmpty": "Ningún juego de tienda vinculado para comparar." });
Object.assign(I18N.de, { "ach.compare": "Vergleichen", "ach.compareTitle": "Erfolge vergleichen", "ach.compareEmpty": "Kein verknüpftes Store-Spiel zum Vergleichen." });
Object.assign(I18N.pt, { "ach.compare": "Comparar", "ach.compareTitle": "Comparar conquistas", "ach.compareEmpty": "Nenhum jogo de loja associado para comparar." });
