//! Adaptateur GOG limité aux commandes réellement exposées par gogdl 1.4.0.
//!
//! gogdl n'expose pas de commande CLI de découverte/liste de bibliothèque :
//! l'adaptateur ne déduit donc jamais une bibliothèque possédée de `info`.

use super::model::{CapabilityState, InstallState, Provider, ProviderGame};
use super::runner::{CommandOutput, ProviderCommand};
use super::tools::ManagedToolId;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Page d'autorisation GOG utilisée par gogdl (client public de GOG Galaxy).
/// Après la connexion, l'adresse de retour contient `?code=…`.
pub const LOGIN_URL: &str = "https://auth.gog.com/auth?client_id=46899977096215655\
&redirect_uri=https%3A%2F%2Fembed.gog.com%2Fon_login_success%3Forigin%3Dclient\
&response_type=code&layout=galaxy";

/// Fichier de jetons gogdl, séparé de `credentials.json`.
pub fn auth_config_path() -> PathBuf {
    crate::config::data_dir().join("gogdl-auth.json")
}

const CONTENT_SYSTEM_URL: &str = "https://content-system.gog.com";

/// Vrai quand gogdl trouve un build Windows pour ce produit. Un pack Amazon
/// Prime n'en publie aucun : lancer gogdl dans ce cas ne télécharge rien et
/// finit par l'erreur « Game doesn't support content system api ».
pub fn has_windows_content_builds(product_id: &str) -> Result<bool, String> {
    if product_id.is_empty()
        || !product_id
            .chars()
            .all(|character| character.is_ascii_digit())
    {
        return Err("identifiant GOG invalide".to_string());
    }
    let response = crate::artwork::http()
        .get(format!(
            "{CONTENT_SYSTEM_URL}/products/{product_id}/os/windows/builds"
        ))
        .query(&[("generation", "2")])
        .send()
        .map_err(|_| "GOG injoignable pendant la vérification du téléchargement".to_string())?;
    if !response.status().is_success() {
        return Err(format!(
            "GOG a répondu {} pendant la vérification du téléchargement",
            response.status().as_u16()
        ));
    }
    let body = crate::net::text(response, crate::net::MAX_JSON, "builds GOG")
        .map_err(|_| "réponse GOG illisible".to_string())?;
    Ok(parse_windows_build_count(&body)? > 0)
}

fn parse_windows_build_count(body: &str) -> Result<u64, String> {
    let root: serde_json::Value =
        serde_json::from_str(body).map_err(|_| "réponse GOG illisible".to_string())?;
    Ok(root
        .get("total_count")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0))
}

/// gogdl garde un manifeste par jeu sous `heroic_gogdl/manifests`. S'il pointe
/// vers une installation qui n'existe plus, gogdl répond « Nothing to do » et
/// sort en 0 sans rien télécharger. Heroic supprime ce manifeste avant une
/// nouvelle installation ; Quark fait de même.
fn gogdl_base_config_dir() -> PathBuf {
    if let Some(path) = std::env::var_os("GOGDL_CONFIG_PATH").map(PathBuf::from) {
        return path;
    }
    std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .or_else(dirs::config_dir)
        .unwrap_or_else(|| crate::config::home_dir().join(".config"))
}

fn gogdl_manifest_path_in(config_dir: &Path, product_id: &str) -> PathBuf {
    config_dir
        .join("heroic_gogdl")
        .join("manifests")
        .join(product_id)
}

/// Supprime l'état de téléchargement gogdl avant une installation neuve. Sans
/// cela, un ancien manifeste fait croire à gogdl que les fichiers existent
/// encore et aucun dossier n'est créé.
pub fn clear_gogdl_install_manifest(product_id: &str) -> Result<(), String> {
    clear_gogdl_manifest_in(&gogdl_base_config_dir(), product_id)
}

fn clear_gogdl_manifest_in(config_dir: &Path, product_id: &str) -> Result<(), String> {
    if product_id.is_empty()
        || !product_id
            .chars()
            .all(|character| character.is_ascii_digit())
    {
        return Err("identifiant GOG invalide".to_string());
    }
    let path = gogdl_manifest_path_in(config_dir, product_id);
    match std::fs::remove_file(&path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(_) => {
            Err("impossible de réinitialiser le manifeste de téléchargement gogdl".to_string())
        }
    }
}

/// Cherche une installation GOG déjà présente dans la destination choisie.
///
/// Le manifeste `goggame-<id>.info` est la preuve que le dossier appartient
/// bien à ce produit. Sans cette vérification, Quark relançait gogdl sur un jeu
/// déjà installé, supprimait son manifeste de reprise et exigeait de nouveau
/// toute la taille du téléchargement — ce qui échoue sur un disque presque
/// plein alors que les fichiers du jeu sont déjà là.
pub fn existing_install_path(
    destination: &Path,
    folder_hint: &str,
    product_id: &str,
) -> Option<PathBuf> {
    let mut candidates = vec![destination.to_path_buf()];
    let hint = Path::new(folder_hint);
    if !folder_hint.trim().is_empty() && !hint.is_absolute() && hint.components().count() == 1 {
        candidates.push(destination.join(hint));
    }
    if let Ok(entries) = std::fs::read_dir(destination) {
        candidates.extend(
            entries
                .flatten()
                .map(|entry| entry.path())
                .filter(|path| path.is_dir()),
        );
    }
    let mut seen = std::collections::HashSet::new();
    candidates.into_iter().find(|candidate| {
        candidate.is_dir()
            && seen.insert(candidate.clone())
            && is_complete_gog_install(candidate, product_id)
    })
}

/// Vrai seulement si ce dossier est une installation GOG complète et
/// vérifiable. Toute décision qui marque un jeu « installé » (adoption avant
/// gogdl, reprise après un arrêt, synchronisation de bibliothèque, sonde
/// périodique) passe par cette fonction : un dossier partiel ne peut donc
/// jamais être présenté comme jouable.
pub fn is_complete_install(dir: &Path, product_id: &str) -> bool {
    is_complete_gog_install(dir, product_id)
}

/// Un dossier n'est adopté que si son manifeste est complet, que les fichiers
/// de lancement qu'il déclare existent et qu'aucune reprise gogdl/Galaxy n'est
/// en cours. Un
/// simple `goggame-<id>.info` peut en effet être écrit avant la fin du
/// téléchargement, et un installeur `setup.exe` ne prouve pas que le jeu est
/// là. Le manifeste GOG liste ses tâches de lancement : c'est la preuve la plus
/// fiable quand elle est présente.
fn is_complete_gog_install(dir: &Path, product_id: &str) -> bool {
    let Some(info_path) = find_info_file(dir, product_id) else {
        return false;
    };
    let Ok(info) = std::fs::read_to_string(&info_path) else {
        return false;
    };
    let Ok(json) = serde_json::from_str::<serde_json::Value>(&info) else {
        return false;
    };
    if !matches_product_id(&json, product_id) {
        return false;
    }
    if !json
        .get("playTasks")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|tasks| !tasks.is_empty())
    {
        return false;
    }
    if contains_partial_download(dir, 0) {
        return false;
    }
    let declared: Vec<&str> = play_task_paths(&json).collect();
    if !declared.is_empty() {
        // Tous les fichiers de lancement déclarés doivent être là : un seul
        // chemin absent prouve un téléchargement interrompu, même si un autre
        // `.exe` traîne dans le dossier.
        return declared
            .iter()
            .all(|relative| resolve_relative(dir, relative).is_some_and(|path| path.is_file()));
    }
    // Manifests anciens ou sans tâche de fichier : on retombe sur un exécutable
    // hors des dossiers d'installeurs redistribuables.
    contains_executable(dir, 0)
}

fn find_info_file(dir: &Path, product_id: &str) -> Option<PathBuf> {
    let marker = format!("goggame-{product_id}.info");
    std::fs::read_dir(dir).ok().and_then(|entries| {
        entries.flatten().find_map(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .eq_ignore_ascii_case(&marker)
                .then(|| entry.path())
        })
    })
}

fn matches_product_id(json: &serde_json::Value, product_id: &str) -> bool {
    ["gameId", "rootGameId"].iter().any(|key| {
        json.get(*key).is_some_and(|value| match value {
            serde_json::Value::String(value) => value == product_id,
            serde_json::Value::Number(value) => value.to_string() == product_id,
            _ => false,
        })
    })
}

/// Chemins `.exe`/binaires déclarés par les `playTasks` de type fichier.
fn play_task_paths(json: &serde_json::Value) -> impl Iterator<Item = &str> {
    json.get("playTasks")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter(|task| {
            task.get("type")
                .and_then(serde_json::Value::as_str)
                .is_none_or(|kind| kind.eq_ignore_ascii_case("FileTask"))
        })
        .filter_map(|task| task.get("path").and_then(serde_json::Value::as_str))
        .map(str::trim)
        .filter(|path| !path.is_empty())
}

/// Résout un chemin relatif du manifeste sans dépendre de la casse : GOG
/// écrit `Bin/Game.EXE` alors que Wine a pu créer `bin/game.exe`. Les chemins
/// absolus, remontants (`..`) et les dossiers d'installeurs sont refusés.
fn resolve_relative(dir: &Path, relative: &str) -> Option<PathBuf> {
    let parts: Vec<&str> = relative
        .split(['/', '\\'])
        .filter(|part| !part.is_empty() && *part != ".")
        .collect();
    if parts.is_empty()
        || parts
            .iter()
            .any(|part| *part == ".." || part.contains(':') || is_installer_folder(part))
    {
        return None;
    }
    let mut current = dir.to_path_buf();
    for part in parts {
        let found = std::fs::read_dir(&current).ok()?.flatten().find(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .eq_ignore_ascii_case(part)
        })?;
        current.push(found.file_name());
    }
    Some(current)
}

fn is_installer_folder(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "__redist" | "__support" | "commonappdata" | "tmp" | "!downloading"
    )
}

fn contains_executable(dir: &Path, depth: usize) -> bool {
    if depth > 3 {
        return false;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return false;
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_ascii_lowercase();
        let path = entry.path();
        if path.is_dir() {
            // Les installeurs redistribuables ne prouvent jamais que le jeu
            // lui-même a été téléchargé.
            if is_installer_folder(&name) {
                continue;
            }
            if contains_executable(&path, depth + 1) {
                return true;
            }
        } else if path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("exe"))
        {
            return true;
        }
    }
    false
}

/// Marqueurs laissés par un téléchargement interrompu. `gogdl` écrit
/// `.gogdl-resume` ; GOG Galaxy utilise `.info.download` avant de renommer le
/// manifeste final.
fn contains_partial_download(dir: &Path, depth: usize) -> bool {
    if depth > 3 {
        return false;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return false;
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_ascii_lowercase();
        if name == ".gogdl-resume"
            || name == ".gogdl-download-cache"
            || name.ends_with(".info.download")
        {
            return true;
        }
        let path = entry.path();
        if path.is_dir() && contains_partial_download(&path, depth + 1) {
            return true;
        }
    }
    false
}

/// Message d'échec gogdl compréhensible, sans exposer ses tuples de taille.
pub fn install_failure_hint(output: &CommandOutput) -> String {
    let text = format!("{}\n{}", output.stdout, output.stderr).to_ascii_lowercase();
    if text.contains("not enough disk space")
        || text.contains("no space left")
        || text.contains("espace insuffisant")
    {
        "espace disque insuffisant pour gogdl ; libère de la place puis réessaie. ".to_string()
            + "Si le jeu est déjà installé, Quark le conserve."
    } else {
        super::runner::failure_hint(output)
    }
}

#[derive(Debug, Clone)]
pub struct GogAdapter {
    pub executable: PathBuf,
    pub auth_config: PathBuf,
    pub timeout: Duration,
}

impl GogAdapter {
    pub fn new(auth_config: PathBuf) -> Self {
        Self {
            executable: ManagedToolId::Gogdl.installed_path(),
            auth_config,
            timeout: Duration::from_secs(60),
        }
    }

    pub fn command(&self, args: impl IntoIterator<Item = String>) -> ProviderCommand {
        let mut complete = vec![
            "--auth-config-path".into(),
            self.auth_config.display().to_string(),
        ];
        complete.extend(args);
        ProviderCommand {
            program: PathBuf::from("python3"),
            args: std::iter::once(self.executable.display().to_string())
                .chain(complete)
                .map(Into::into)
                .collect(),
            env: vec![],
            cwd: None,
            // gogdl auth output may contain access/refresh tokens; callers that
            // perform auth must set discard_output before running this command.
            secrets: vec![],
            discard_output: false,
        }
    }

    /// Échange le code copié depuis l'adresse de retour GOG contre des jetons.
    ///
    /// gogdl imprime alors les jetons sur stdout et sort avec le code 0 même en
    /// cas de refus : la sortie est donc analysée ici, jamais affichée ni
    /// journalisée.
    pub fn login_with_code(&self, code: &str) -> Result<(), String> {
        let before = std::fs::read(&self.auth_config).ok();
        let mut command = self.command(Self::login_args(code));
        command.discard_output = true;
        command.secrets = vec![code.to_string()];
        let output = super::runner::run(command, self.timeout, None)?;
        if output.exit_code != Some(0) {
            return Err("gogdl a échoué pendant la connexion GOG".to_string());
        }
        let after = std::fs::read(&self.auth_config).ok();
        if super::session_written(before.as_deref(), after.as_deref())
            && Self::auth_config_has_tokens(&self.auth_config)
        {
            Ok(())
        } else {
            Err("gogdl a refusé l'autorisation GOG (code expiré ou invalide)".to_string())
        }
    }

    pub fn is_logged_in() -> bool {
        Self::auth_config_has_tokens(&auth_config_path())
    }

    fn auth_config_has_tokens(path: &Path) -> bool {
        std::fs::read_to_string(path)
            .map(|text| text.contains("access_token") && text.contains("refresh_token"))
            .unwrap_or(false)
    }

    pub fn run(
        &self,
        args: impl IntoIterator<Item = String>,
        cancellation: Option<std::sync::Arc<std::sync::atomic::AtomicBool>>,
    ) -> Result<CommandOutput, String> {
        super::runner::run(self.command(args), self.timeout, cancellation)
    }

    /// Téléchargement avec avancement publié (gogdl écrit le même format que
    /// Legendary : `= Progress: 12.50 4472…/1053…`).
    pub fn run_install_streaming(
        &self,
        args: impl IntoIterator<Item = String>,
        cancellation: Option<std::sync::Arc<std::sync::atomic::AtomicBool>>,
        progress: std::sync::Arc<std::sync::Mutex<super::runner::InstallProgress>>,
    ) -> Result<CommandOutput, String> {
        super::runner::run_streaming(
            self.command(args),
            Duration::from_secs(6 * 60 * 60),
            cancellation,
            progress,
        )
    }

    /// Bibliothèque possédée, lue via l'API publique de GOG avec les jetons
    /// écrits par gogdl (qui ne sait pas lister une bibliothèque).
    pub fn sync_library(&self, fetched_at: i64) -> Result<Vec<super::model::ProviderGame>, String> {
        let tokens = super::gog_library::usable_tokens(&self.auth_config, crate::config::now_ts())?;
        let owned = super::gog_library::library(&tokens.access_token)?;
        Ok(super::gog_library::to_games(&owned, fetched_at))
    }

    pub fn login_args(code: &str) -> Vec<String> {
        vec!["auth".into(), "--code".into(), code.into()]
    }

    /// `--path` est le dossier **parent** : gogdl y ajoute le nom de dossier
    /// publié par `gogdl info` (`folder_name`). DLC et langue sont explicites
    /// pour que rien ne dépende d'un choix interactif.
    pub fn install_args(game_id: &str, destination: &Path, platform: &str) -> Vec<String> {
        vec![
            "download".into(),
            game_id.into(),
            "--path".into(),
            destination.display().to_string(),
            "--platform".into(),
            platform.into(),
            "--lang".into(),
            "en-US".into(),
            "--skip-dlcs".into(),
        ]
    }

    /// Taille et nom de dossier du téléchargement (`gogdl info` sort du JSON).
    pub fn info_args(game_id: &str, platform: &str) -> Vec<String> {
        vec![
            "info".into(),
            game_id.into(),
            "--platform".into(),
            platform.into(),
        ]
    }

    pub fn save_sync_args(
        save_path: &Path,
        game_id: &str,
        timestamp: i64,
        platform: &str,
    ) -> Vec<String> {
        vec![
            "save-sync".into(),
            save_path.display().to_string(),
            game_id.into(),
            "--ts".into(),
            timestamp.to_string(),
            "--platform".into(),
            platform.into(),
        ]
    }
}

/// Taille annoncée par `gogdl info`, quand gogdl la publie.
pub fn install_size_from_info(text: &str) -> Option<u64> {
    super::install_size_from_json(text)
}

/// Taille d'installation demandée à gogdl pour le jeu choisi.
pub fn gog_install_size(game_id: &str) -> Result<Option<u64>, String> {
    let adapter = GogAdapter::new(auth_config_path());
    if !adapter.executable.is_file() {
        return Err("gogdl n'est pas installé".to_string());
    }
    let output = adapter.run(GogAdapter::info_args(game_id, "windows"), None)?;
    if output.exit_code != Some(0) {
        return Err("gogdl n'a pas pu lire la taille du jeu".to_string());
    }
    Ok(install_size_from_info(&output.stdout))
}

/// Nom du dossier d'installation publié par `gogdl info`.
pub fn folder_from_info(text: &str) -> Result<String, String> {
    if super::first_json_value(text).is_none() {
        return Err("réponse gogdl illisible".to_string());
    }
    let root = super::json_object_with_key(text, "folder_name")
        .ok_or_else(|| "gogdl ne publie pas de dossier pour ce jeu".to_string())?;
    root.get("folder_name")
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|name| !name.is_empty() && !name.contains('/') && !name.contains('\0'))
        .map(str::to_string)
        .ok_or_else(|| "gogdl ne publie pas de dossier pour ce jeu".to_string())
}

/// Retrouve le dossier réellement créé par gogdl. `gogdl info` publie un
/// `folder_name`, mais certaines versions ou certains jeux créent un dossier
/// légèrement différent (ponctuation, suffixe de langue, ancien dossier déjà
/// présent). On vérifie donc le dossier annoncé, puis les dossiers voisins qui
/// portent le manifeste GOG du jeu avant de déclarer l'installation échouée.
pub fn resolve_install_path(
    destination: &Path,
    folder_hint: &str,
    product_id: &str,
) -> Option<PathBuf> {
    let exact = destination.join(folder_hint);
    if !folder_hint.trim().is_empty() && is_complete_gog_install(&exact, product_id) {
        return Some(exact);
    }
    // Certaines versions installent directement dans le dossier choisi.
    if is_complete_gog_install(destination, product_id) {
        return Some(destination.to_path_buf());
    }
    // Après un code de sortie nul, gogdl peut avoir laissé un dossier partiel :
    // seuls les dossiers complets sont retenus.
    let mut candidates: Vec<PathBuf> = std::fs::read_dir(destination)
        .ok()?
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_dir() && is_complete_gog_install(path, product_id))
        .collect();
    candidates.sort();
    // Le dossier annoncé par `info` d'abord, puis l'unique dossier complet :
    // jamais un autre jeu installé à côté.
    let hint_key = folder_key(folder_hint);
    if !hint_key.is_empty()
        && let Some(path) = candidates.iter().find(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| folder_key(name) == hint_key)
        })
    {
        return Some(path.clone());
    }
    (candidates.len() == 1).then(|| candidates[0].clone())
}

fn folder_key(name: &str) -> String {
    name.chars()
        .filter(|character| character.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

/// Exécutable le plus plausible dans un dossier GOG installé : le plus court
/// chemin `.exe` sous la racine, sinon un binaire Linux portant le nom du jeu.
pub fn find_executable(root: &Path, title: &str) -> Option<PathBuf> {
    fn walk(dir: &Path, depth: usize, found: &mut Vec<PathBuf>) {
        if depth > 3 || found.len() > 40 {
            return;
        }
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, depth + 1, found);
            } else if path
                .extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| {
                    extension.eq_ignore_ascii_case("exe") || extension.is_empty()
                })
            {
                found.push(path);
            }
        }
    }
    let mut candidates = Vec::new();
    walk(root, 0, &mut candidates);
    let normalized = title.to_lowercase();
    candidates.sort_by_key(|path| path.components().count());
    let by_name = candidates.iter().find(|path| {
        path.file_stem()
            .and_then(|stem| stem.to_str())
            .is_some_and(|stem| {
                let stem = stem.to_lowercase();
                stem.len() > 3 && (normalized.contains(&stem) || stem.contains(&normalized))
            })
    });
    by_name.cloned().or_else(|| {
        candidates
            .into_iter()
            .find(|path| path.extension().is_some())
    })
}

/// Fusionne la bibliothèque GOG rafraîchie sans perdre l'état local.
fn gog_title_key(title: &str) -> String {
    super::gog_library::clean_gog_title(title)
        .chars()
        .filter(|character| character.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

pub fn merge_gog(existing: &[ProviderGame], refreshed: &[ProviderGame]) -> Vec<ProviderGame> {
    let mut merged: Vec<ProviderGame> = existing
        .iter()
        .filter(|game| game.key.provider != Provider::Gog)
        .cloned()
        .collect();
    for incoming in refreshed {
        let mut game = incoming.clone();
        // La déduplication GOG peut garder l'identifiant de la fiche normale
        // alors que l'installation suivie appartient à l'ancienne fiche
        // « Amazon Prime » : on récupère l'état par titre en priorité.
        let previous = existing
            .iter()
            .find(|previous| previous.key == incoming.key)
            .or_else(|| {
                existing
                    .iter()
                    .filter(|previous| {
                        previous.key.provider == Provider::Gog
                            && gog_title_key(&previous.title) == gog_title_key(&incoming.title)
                    })
                    .max_by_key(|previous| {
                        (previous.install_state == InstallState::Installed) as u8
                    })
            });
        if let Some(previous) = previous {
            game.custom_save_paths = previous.custom_save_paths.clone();
            if game.quark_game_id.is_none() {
                game.quark_game_id = previous.quark_game_id.clone();
            }
            if game.cover_url.is_none() {
                game.cover_url = previous.cover_url.clone();
            }
            if game.install_exe.is_none() {
                game.install_exe = previous.install_exe.clone();
            }
            if game.install_root.is_none() {
                game.install_root = previous.install_root.clone();
            }
            let complete = previous.install_path.as_ref().is_some_and(|path| {
                is_complete_install(path, &previous.key.product_id)
                    || (incoming.key.product_id != previous.key.product_id
                        && is_complete_install(path, &incoming.key.product_id))
            });
            if previous.install_state == InstallState::Installed && complete {
                game.install_state = InstallState::Installed;
                game.install_path = previous.install_path.clone();
            } else if matches!(
                previous.install_state,
                InstallState::Installing | InstallState::Queued
            ) {
                game.install_state = previous.install_state;
                game.install_path = previous.install_path.clone();
            }
            if game.capabilities.achievements == CapabilityState::Unknown {
                game.capabilities.achievements = previous.capabilities.achievements.clone();
            }
            if game.capabilities.cloud_saves == CapabilityState::Unknown {
                game.capabilities.cloud_saves = previous.capabilities.cloud_saves.clone();
            }
        }
        merged.push(game);
    }
    merged
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn une_synchro_gog_ne_perd_pas_une_installation_en_cours() {
        let existing = ProviderGame {
            key: crate::providers::ProviderGameKey {
                provider: Provider::Gog,
                product_id: "123".into(),
            },
            title: "Jeu GOG".into(),
            install_state: InstallState::Installing,
            ..Default::default()
        };
        let mut refreshed = existing.clone();
        refreshed.install_state = InstallState::NotInstalled;
        let merged = merge_gog(&[existing], &[refreshed]);
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].install_state, InstallState::Installing);
    }

    #[test]
    fn un_ancien_manifeste_gogdl_est_supprime_avant_une_installation() {
        let root =
            std::env::temp_dir().join(format!("quark-gogdl-manifest-{}", std::process::id()));
        let manifests = root.join("heroic_gogdl").join("manifests");
        std::fs::create_dir_all(&manifests).unwrap();
        let manifest = gogdl_manifest_path_in(&root, "2015389384");
        std::fs::write(&manifest, "{}").unwrap();
        assert_eq!(manifest, manifests.join("2015389384"));
        clear_gogdl_manifest_in(&root, "2015389384").unwrap();
        assert!(!manifest.exists());
        // Une installation neuve n'a pas de manifeste : ce n'est pas une erreur.
        clear_gogdl_manifest_in(&root, "2015389384").unwrap();
        assert!(clear_gogdl_manifest_in(&root, "../jeu").is_err());
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn les_builds_windows_sont_lus_avant_de_lancer_gogdl() {
        assert_eq!(
            parse_windows_build_count(r#"{"total_count":5,"items":[]}"#).unwrap(),
            5
        );
        assert_eq!(
            parse_windows_build_count(r#"{"total_count":0}"#).unwrap(),
            0
        );
        assert_eq!(parse_windows_build_count("{}").unwrap(), 0);
        assert!(parse_windows_build_count("pas du json").is_err());
    }

    #[test]
    #[ignore = "nécessite le réseau GOG"]
    fn le_vrai_pack_na_pas_de_builds_et_le_jeu_en_a() {
        assert!(!has_windows_content_builds("1662043288").unwrap());
        assert!(has_windows_content_builds("2015389384").unwrap());
    }

    #[test]
    fn gogdl_install_keeps_game_id_and_destination_as_separate_arguments() {
        assert_eq!(
            GogAdapter::install_args(
                "game; $(touch x)",
                Path::new("/Games With Space"),
                "windows"
            ),
            vec![
                "download",
                "game; $(touch x)",
                "--path",
                "/Games With Space",
                "--platform",
                "windows",
                "--lang",
                "en-US",
                "--skip-dlcs"
            ]
        );
    }

    #[test]
    fn gogdl_runs_zipapp_through_python_and_places_auth_config_first() {
        let adapter = GogAdapter::new(PathBuf::from("/private/auth config.json"));
        let command = adapter.command(GogAdapter::login_args("code"));
        assert_eq!(command.program, PathBuf::from("python3"));
        assert_eq!(command.args[0], adapter.executable.as_os_str());
        assert_eq!(command.args[1], "--auth-config-path");
        assert_eq!(command.args[2], "/private/auth config.json");
        assert_eq!(command.args[3], "auth");
        assert_eq!(command.args[4], "--code");
        assert_eq!(command.args[5], "code");
        assert!(!command.discard_output);
    }

    #[cfg(unix)]
    #[test]
    fn la_connexion_gog_exige_des_jetons_ecrits_meme_avec_un_code_de_sortie_nul() {
        use std::os::unix::fs::PermissionsExt;
        let root = std::env::temp_dir().join(format!(
            "quark-gog-login-{}-{}",
            std::process::id(),
            crate::config::now_ts()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let auth = root.join("gogdl-auth.json");
        let adapter = GogAdapter {
            executable: root.join("gogdl.pyz"),
            auth_config: auth.clone(),
            timeout: Duration::from_secs(5),
        };

        // gogdl imprime {"error": true} et sort en 0 quand le code est refusé.
        std::fs::write(&adapter.executable, "pass\n").unwrap();
        std::fs::set_permissions(&adapter.executable, std::fs::Permissions::from_mode(0o700))
            .unwrap();
        assert!(adapter.login_with_code("code").is_err());

        std::fs::write(
            &adapter.executable,
            format!(
                "open({:?}, 'w').write('{{\"access_token\":\"a\",\"refresh_token\":\"r\"}}')\n",
                auth.display().to_string()
            ),
        )
        .unwrap();
        std::fs::set_permissions(&adapter.executable, std::fs::Permissions::from_mode(0o700))
            .unwrap();
        assert!(adapter.login_with_code("code").is_ok());
        assert!(GogAdapter::auth_config_has_tokens(&auth));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn une_installation_gog_deja_presente_est_reconnue_avant_de_relancer_gogdl() {
        let root = std::env::temp_dir().join(format!(
            "quark-gog-existing-{}-{}",
            std::process::id(),
            crate::config::now_ts()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("Call of Juarez")).unwrap();
        std::fs::write(
            root.join("Call of Juarez/goggame-123.info"),
            r#"{"gameId":"123","name":"Call of Juarez: Gunslinger","playTasks":[{"type":"FileTask","path":"CoJ.exe"}]}"#,
        )
        .unwrap();
        std::fs::write(root.join("Call of Juarez/CoJ.exe"), b"exe").unwrap();
        std::fs::create_dir_all(root.join("Autre jeu")).unwrap();
        std::fs::write(
            root.join("Autre jeu/goggame-999.info"),
            r#"{"gameId":"999"}"#,
        )
        .unwrap();
        std::fs::write(root.join("Autre jeu/autre.exe"), b"exe").unwrap();

        assert_eq!(
            existing_install_path(&root, "", "123"),
            Some(root.join("Call of Juarez"))
        );
        assert_eq!(
            existing_install_path(&root, "Call of Juarez Prime", "123"),
            Some(root.join("Call of Juarez"))
        );
        assert!(existing_install_path(&root, "", "404").is_none());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn un_dossier_gog_incomplet_n_est_jamais_adopte() {
        let root = std::env::temp_dir().join(format!(
            "quark-gog-incomplete-{}-{}",
            std::process::id(),
            crate::config::now_ts()
        ));
        let _ = std::fs::remove_dir_all(&root);
        let game = root.join("Call of Juarez");
        std::fs::create_dir_all(&game).unwrap();
        std::fs::write(game.join("goggame-123.info"), r#"{"gameId":"123"#).unwrap();
        std::fs::write(game.join("CoJ.exe"), b"exe").unwrap();
        assert!(existing_install_path(&root, "", "123").is_none());

        std::fs::write(
            game.join("goggame-123.info"),
            r#"{"gameId":"123","playTasks":[]}"#,
        )
        .unwrap();
        assert!(existing_install_path(&root, "", "123").is_none());
        std::fs::write(
            game.join("goggame-123.info"),
            r#"{"gameId":"123","playTasks":[{"type":"FileTask","path":"CoJ.exe"}]}"#,
        )
        .unwrap();
        std::fs::write(game.join(".gogdl-resume"), b"resume").unwrap();
        assert!(existing_install_path(&root, "", "123").is_none());

        std::fs::remove_file(game.join(".gogdl-resume")).unwrap();
        std::fs::write(game.join("goggame-123.info.download"), b"partial").unwrap();
        assert!(existing_install_path(&root, "", "123").is_none());

        std::fs::remove_file(game.join("goggame-123.info.download")).unwrap();
        std::fs::remove_file(game.join("CoJ.exe")).unwrap();
        std::fs::create_dir_all(game.join("__redist")).unwrap();
        std::fs::write(game.join("__redist/setup.exe"), b"redist").unwrap();
        assert!(existing_install_path(&root, "", "123").is_none());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn un_echec_de_disque_gogdl_est_explique_sans_tuple_de_taille() {
        let output = CommandOutput {
            stdout: "(4.631216532550752, 'GB') 4972730887\n\
                     Unable to proceed, Not enough disk space\n"
                .to_string(),
            stderr: String::new(),
            exit_code: Some(2),
        };
        let hint = install_failure_hint(&output);
        assert!(hint.contains("espace disque insuffisant"), "{hint}");
        assert!(!hint.contains("4.631216532550752"), "{hint}");
    }

    #[test]
    fn le_dossier_reel_est_retrouve_meme_si_gogdl_a_change_son_nom() {
        let root = std::env::temp_dir().join(format!(
            "quark-gog-install-{}-{}",
            std::process::id(),
            crate::config::now_ts()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("Call of Juarez")).unwrap();
        std::fs::write(
            root.join("Call of Juarez/goggame-123.info"),
            r#"{"gameId":"123","playTasks":[{"type":"FileTask","path":"CoJ.exe"}]}"#,
        )
        .unwrap();
        std::fs::write(root.join("Call of Juarez/CoJ.exe"), b"exe").unwrap();
        // Un autre jeu installé à côté ne doit jamais être choisi.
        std::fs::create_dir_all(root.join("Autre jeu")).unwrap();
        std::fs::write(
            root.join("Autre jeu/goggame-999.info"),
            r#"{"gameId":"999","playTasks":[{"type":"FileTask","path":"autre.exe"}]}"#,
        )
        .unwrap();
        std::fs::write(root.join("Autre jeu/autre.exe"), b"exe").unwrap();
        let found = resolve_install_path(&root, "Call of Juarez Prime", "123").unwrap();
        assert_eq!(found, root.join("Call of Juarez"));
        assert_eq!(
            resolve_install_path(&root, "Call of Juarez", "123").unwrap(),
            root.join("Call of Juarez")
        );
        assert!(resolve_install_path(&root, "Call of Juarez", "404").is_none());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn le_dossier_est_lu_meme_entoure_de_journaux_gogdl() {
        // gogdl 1.4 écrit le JSON au milieu de lignes de journal : l'ancien
        // `from_str` sur toute la sortie échouait avec « réponse illisible ».
        let output = "[INFO] Getting Dependencies repository {\"repository_manifest\": \"x\"}\n\
                      [API] INFO: reading manifest\n\
                      {\"size\": {\"*\": {\"download_size\": 42}}, \"folder_name\": \"Call of Juarez Gunslinger\", \"buildId\": \"7\"}\n\
                      [INFO] Finished";
        assert_eq!(
            folder_from_info(output).unwrap(),
            "Call of Juarez Gunslinger"
        );
        assert!(folder_from_info("[INFO] aucune réponse").is_err());
    }

    #[test]
    fn la_page_de_connexion_gog_est_l_url_officielle_du_client_public() {
        assert!(LOGIN_URL.starts_with("https://auth.gog.com/auth?client_id=46899977096215655"));
        assert!(LOGIN_URL.contains("response_type=code"));
    }

    #[test]
    fn un_manifeste_incomplet_ne_prouve_jamais_une_installation() {
        let root = std::env::temp_dir().join(format!(
            "quark-gog-declared-{}-{}",
            std::process::id(),
            crate::config::now_ts()
        ));
        let _ = std::fs::remove_dir_all(&root);
        let game = root.join("Jeu");
        std::fs::create_dir_all(game.join("__redist")).unwrap();
        std::fs::write(
            game.join("goggame-123.info"),
            r#"{"gameId":"123","playTasks":[{"type":"FileTask","path":"Bin/Game.EXE"}]}"#,
        )
        .unwrap();
        // Un installeur et un redistribuable ne remplacent pas le jeu.
        std::fs::write(game.join("setup.exe"), b"exe").unwrap();
        std::fs::write(game.join("__redist/vcredist.exe"), b"exe").unwrap();
        assert!(!is_complete_install(&game, "123"));
        assert!(resolve_install_path(&root, "Jeu", "123").is_none());
        assert!(existing_install_path(&root, "Jeu", "123").is_none());

        // Le chemin déclaré existe, avec une casse différente : c'est complet.
        std::fs::create_dir_all(game.join("bin")).unwrap();
        std::fs::write(game.join("bin/Game.EXE"), b"exe").unwrap();
        assert!(is_complete_install(&game, "123"));
        assert_eq!(
            resolve_install_path(&root, "Jeu", "123"),
            Some(game.clone())
        );

        // Une tâche qui pointe vers un redistribuable ne suffit pas.
        std::fs::remove_file(game.join("bin/Game.EXE")).unwrap();
        std::fs::write(
            game.join("goggame-123.info"),
            r#"{"gameId":"123","playTasks":[{"type":"FileTask","path":"__redist/vcredist.exe"}]}"#,
        )
        .unwrap();
        assert!(!is_complete_install(&game, "123"));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn une_synchro_gog_ne_marque_pas_installe_un_dossier_incomplet() {
        let root = std::env::temp_dir().join(format!(
            "quark-gog-merge-{}-{}",
            std::process::id(),
            crate::config::now_ts()
        ));
        let _ = std::fs::remove_dir_all(&root);
        let game_dir = root.join("Jeu");
        std::fs::create_dir_all(&game_dir).unwrap();
        std::fs::write(
            game_dir.join("goggame-123.info"),
            r#"{"gameId":"123","playTasks":[{"path":"game.exe"}]}"#,
        )
        .unwrap();
        let existing = ProviderGame {
            key: crate::providers::ProviderGameKey {
                provider: Provider::Gog,
                product_id: "123".into(),
            },
            title: "Jeu".into(),
            install_state: InstallState::Installed,
            install_path: Some(game_dir.clone()),
            ..Default::default()
        };
        let refreshed = ProviderGame {
            key: existing.key.clone(),
            title: "Jeu".into(),
            install_state: InstallState::NotInstalled,
            ..Default::default()
        };
        let merged = merge_gog(
            std::slice::from_ref(&existing),
            std::slice::from_ref(&refreshed),
        );
        assert_eq!(merged[0].install_state, InstallState::NotInstalled);
        assert!(merged[0].install_path.is_none());

        std::fs::write(game_dir.join("game.exe"), b"exe").unwrap();
        let merged = merge_gog(
            std::slice::from_ref(&existing),
            std::slice::from_ref(&refreshed),
        );
        assert_eq!(merged[0].install_state, InstallState::Installed);
        assert_eq!(merged[0].install_path.as_deref(), Some(game_dir.as_path()));
        let _ = std::fs::remove_dir_all(&root);
    }
}
