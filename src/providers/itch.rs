//! Bibliothèque et installation itch.io.
//!
//! L'API moderne `api.itch.io` liste les clés de téléchargement possédées,
//! puis les fichiers publiés par le développeur. Une clé API personnelle est
//! nécessaire : elle est stockée dans `credentials.json` et n'est jamais
//! affichée ni journalisée.
//!
//! L'installation télécharge l'archive Windows publiée et l'extrait dans un
//! dossier propre au jeu. Les uploads qui ne sont pas des archives (installeur
//! `.exe`, `.dmg`, `.apk`…) restent visibles dans la bibliothèque mais doivent
//! être installés manuellement : Quark ne lance pas un installeur opaque.

use super::model::{
    CapabilityState, GameCapabilities, InstallState, Provider, ProviderGame, ProviderGameKey,
};
use super::runner::InstallProgress;
use crate::net;
use serde_json::Value;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

const API: &str = "https://api.itch.io";
const PAGE_SIZE: u32 = 500;
const MAX_PAGES: u32 = 200;

fn client() -> reqwest::blocking::Client {
    crate::artwork::http()
}

fn api_get(
    api_key: &str,
    path: &str,
    query: &[(&str, String)],
) -> Result<reqwest::blocking::Response, String> {
    let key = valid_key(api_key)?;
    let response = client()
        .get(format!("{API}{path}"))
        .bearer_auth(key)
        .query(query)
        .timeout(Duration::from_secs(30))
        .send()
        .map_err(|_| "itch.io injoignable".to_string())?;
    let status = response.status();
    if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN {
        return Err(
            "itch.io a refusé la clé API : vérifie-la dans Paramètres → Boutiques & outils"
                .to_string(),
        );
    }
    if !status.is_success() {
        return Err(format!("itch.io a répondu {status}"));
    }
    Ok(response)
}

fn valid_key(api_key: &str) -> Result<&str, String> {
    let key = api_key.trim();
    if key.is_empty() || key.len() > 512 || key.chars().any(char::is_whitespace) {
        return Err("clé API itch.io absente ou invalide".to_string());
    }
    Ok(key)
}

fn api_error(body: &str) -> Option<String> {
    let root: Value = serde_json::from_str(body).ok()?;
    let errors = root.get("errors")?.as_array()?;
    let messages: Vec<&str> = errors
        .iter()
        .filter_map(Value::as_str)
        .map(str::trim)
        .filter(|message| !message.is_empty())
        .collect();
    (!messages.is_empty()).then(|| messages.join(" · "))
}

fn read_json(response: reqwest::blocking::Response, label: &str) -> Result<Value, String> {
    let text =
        net::text(response, net::MAX_JSON, label).map_err(|_| format!("{label} illisible"))?;
    if let Some(error) = api_error(&text) {
        return Err(format!("itch.io : {error}"));
    }
    serde_json::from_str(&text).map_err(|_| format!("{label} illisible"))
}

/// Une clé de téléchargement possédée. `download_key_id` est distinct de
/// l'identifiant du jeu : c'est lui qui autorise le téléchargement payant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DownloadKey {
    pub id: u64,
    pub game_id: u64,
    pub title: String,
    pub url: Option<String>,
    pub cover_url: Option<String>,
    pub traits: Vec<String>,
    pub created_at: String,
}

fn numeric_id(value: &Value) -> Option<u64> {
    match value {
        Value::Number(number) => number.as_u64(),
        Value::String(text) => text.trim().parse().ok(),
        _ => None,
    }
}

fn http_url(value: Option<&Value>) -> Option<String> {
    value
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|url| url.starts_with("https://") || url.starts_with("http://"))
        .map(str::to_string)
}

/// Lit une page de `/profile/owned-keys`. Une même boutique peut renvoyer
/// plusieurs clés pour un jeu (achat puis bundle) : la plus récente gagne.
pub fn parse_owned_keys(body: &str) -> Result<Vec<DownloadKey>, String> {
    let root: Value =
        serde_json::from_str(body).map_err(|_| "bibliothèque itch.io illisible".to_string())?;
    let keys = root
        .get("owned_keys")
        .and_then(Value::as_array)
        .ok_or_else(|| "bibliothèque itch.io illisible".to_string())?;
    let mut parsed = Vec::with_capacity(keys.len());
    for entry in keys {
        let Some(id) = numeric_id(entry.get("id").unwrap_or(&Value::Null)) else {
            continue;
        };
        let Some(game_id) = numeric_id(entry.get("game_id").unwrap_or(&Value::Null)) else {
            continue;
        };
        let game = entry.get("game").unwrap_or(&Value::Null);
        let title = game
            .get("title")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .trim()
            .to_string();
        let traits = game
            .get("traits")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default();
        parsed.push(DownloadKey {
            id,
            game_id,
            title: if title.is_empty() {
                format!("Jeu itch.io {game_id}")
            } else {
                title
            },
            url: http_url(game.get("url")),
            cover_url: http_url(game.get("cover_url")),
            traits,
            created_at: entry
                .get("created_at")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
        });
    }
    // Les clés sont déjà triées du plus récent au plus ancien par l'API. On
    // garde la première rencontrée pour chaque jeu.
    let mut seen = std::collections::HashSet::new();
    parsed.retain(|key| seen.insert(key.game_id));
    Ok(parsed)
}

fn owned_keys(api_key: &str, game_id: Option<u64>) -> Result<Vec<DownloadKey>, String> {
    let mut all = Vec::new();
    for page in 1..=MAX_PAGES {
        let mut query = vec![
            ("page", page.to_string()),
            ("per_page", PAGE_SIZE.to_string()),
        ];
        if let Some(game_id) = game_id {
            query.push(("game_ids", game_id.to_string()));
        }
        let response = api_get(api_key, "/profile/owned-keys", &query)?;
        let root = read_json(response, "bibliothèque itch.io")?;
        let body = root.to_string();
        let keys = parse_owned_keys(&body)?;
        if keys.is_empty() {
            break;
        }
        all.extend(keys);
        if game_id.is_some() {
            break;
        }
    }
    let mut seen = std::collections::HashSet::new();
    all.retain(|key| seen.insert(key.game_id));
    Ok(all)
}

pub fn library(api_key: &str, fetched_at: i64) -> Result<Vec<ProviderGame>, String> {
    let keys = owned_keys(api_key, None)?;
    if keys.is_empty() {
        return Err("bibliothèque itch.io vide : aucune clé de téléchargement".to_string());
    }
    Ok(keys
        .into_iter()
        .map(|key| ProviderGame {
            key: ProviderGameKey {
                provider: Provider::Itch,
                product_id: key.game_id.to_string(),
            },
            title: key.title,
            owned: true,
            cover_url: key.cover_url,
            install_state: InstallState::NotInstalled,
            capabilities: GameCapabilities {
                achievements: CapabilityState::Unsupported(
                    "itch.io ne publie pas de succès dans son API".to_string(),
                ),
                cloud_saves: CapabilityState::Unsupported(
                    "itch.io ne fournit pas de sauvegarde cloud".to_string(),
                ),
            },
            last_synced: Some(fetched_at),
            ..Default::default()
        })
        .collect())
}

fn find_key(api_key: &str, game_id: &str) -> Result<DownloadKey, String> {
    let id = game_id
        .trim()
        .parse::<u64>()
        .map_err(|_| "identifiant itch.io invalide".to_string())?;
    owned_keys(api_key, Some(id))?
        .into_iter()
        .find(|key| key.game_id == id)
        .ok_or_else(|| "ce jeu n'est plus dans la bibliothèque itch.io".to_string())
}

/// Un fichier publié par le développeur.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Upload {
    pub id: u64,
    pub filename: String,
    pub size: Option<u64>,
    pub updated_at: String,
    pub traits: Vec<String>,
}

fn trait_names(value: Option<&Value>) -> Vec<String> {
    match value {
        Some(Value::Array(items)) => items
            .iter()
            .filter_map(Value::as_str)
            .map(str::to_string)
            .collect(),
        Some(Value::Object(map)) => map
            .iter()
            .filter(|(_, value)| value.as_bool().unwrap_or(false))
            .map(|(name, _)| name.clone())
            .collect(),
        _ => Vec::new(),
    }
}

pub fn parse_uploads(body: &str) -> Result<Vec<Upload>, String> {
    let root: Value =
        serde_json::from_str(body).map_err(|_| "fichiers itch.io illisibles".to_string())?;
    let uploads = root
        .get("uploads")
        .and_then(Value::as_array)
        .ok_or_else(|| "fichiers itch.io illisibles".to_string())?;
    let mut parsed = Vec::new();
    for entry in uploads {
        let Some(id) = numeric_id(entry.get("id").unwrap_or(&Value::Null)) else {
            continue;
        };
        let filename = entry
            .get("filename")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .trim()
            .to_string();
        if filename.is_empty() {
            continue;
        }
        parsed.push(Upload {
            id,
            filename,
            size: entry.get("size").and_then(Value::as_u64),
            updated_at: entry
                .get("updated_at")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
            traits: trait_names(entry.get("traits")),
        });
    }
    Ok(parsed)
}

fn is_archive(filename: &str) -> bool {
    let lower = filename.to_ascii_lowercase();
    lower.ends_with(".zip")
        || lower.ends_with(".tar.gz")
        || lower.ends_with(".tgz")
        || lower.ends_with(".tar")
}

/// Choisit l'archive Windows la plus récente. Un upload `.exe` n'est jamais
/// exécuté silencieusement : on préfère une archive, ou on explique la limite.
pub fn pick_upload(uploads: &[Upload]) -> Option<&Upload> {
    let mut candidates: Vec<&Upload> = uploads
        .iter()
        .filter(|upload| is_archive(&upload.filename))
        .collect();
    candidates.sort_by(|left, right| {
        let windows_left = left.traits.iter().any(|name| name == "p_windows");
        let windows_right = right.traits.iter().any(|name| name == "p_windows");
        windows_right
            .cmp(&windows_left)
            .then_with(|| right.updated_at.cmp(&left.updated_at))
            .then_with(|| right.size.cmp(&left.size))
    });
    candidates.first().copied()
}

fn fetch_uploads(api_key: &str, key: &DownloadKey) -> Result<Vec<Upload>, String> {
    let response = api_get(
        api_key,
        &format!("/games/{}/uploads", key.game_id),
        &[("download_key_id", key.id.to_string())],
    )?;
    let root = read_json(response, "fichiers itch.io")?;
    parse_uploads(&root.to_string())
}

fn set_progress(
    progress: &Arc<Mutex<InstallProgress>>,
    stage: &str,
    percent: Option<f32>,
    written: Option<u64>,
    total: Option<u64>,
) {
    if let Ok(mut progress) = progress.lock() {
        progress.stage = stage.to_string();
        progress.percent = percent;
        progress.written = written;
        progress.total = total;
    }
}

fn download(
    api_key: &str,
    key: &DownloadKey,
    upload: &Upload,
    destination: &Path,
    progress: &Arc<Mutex<InstallProgress>>,
    cancel: &AtomicBool,
) -> Result<PathBuf, String> {
    let response = api_get(
        api_key,
        &format!("/uploads/{}/download", upload.id),
        &[("download_key_id", key.id.to_string())],
    )?;
    let total = response.content_length().or(upload.size);
    let mut reader = response;
    let temp = destination.join(format!(".quark-itch-{}.part", upload.id));
    let mut file = File::create(&temp)
        .map_err(|_| "impossible de créer le fichier itch.io temporaire".to_string())?;
    let mut written = 0u64;
    let mut buffer = [0u8; 64 * 1024];
    loop {
        if cancel.load(Ordering::Relaxed) {
            let _ = std::fs::remove_file(&temp);
            return Err("installation itch.io annulée".to_string());
        }
        let read = reader
            .read(&mut buffer)
            .map_err(|_| "téléchargement itch.io interrompu".to_string())?;
        if read == 0 {
            break;
        }
        file.write_all(&buffer[..read])
            .map_err(|_| "écriture du téléchargement itch.io impossible".to_string())?;
        written += read as u64;
        let percent = total
            .filter(|total| *total > 0)
            .map(|total| (written as f32 / total as f32 * 100.0).clamp(0.0, 100.0));
        set_progress(progress, "Téléchargement", percent, Some(written), total);
    }
    file.sync_all()
        .map_err(|_| "écriture du téléchargement itch.io impossible".to_string())?;
    Ok(temp)
}

fn extract_zip(archive: &Path, destination: &Path) -> Result<(), String> {
    let file = File::open(archive).map_err(|_| "archive itch.io introuvable".to_string())?;
    let mut zip =
        zip::ZipArchive::new(file).map_err(|_| "archive ZIP itch.io illisible".to_string())?;
    for index in 0..zip.len() {
        let mut entry = zip
            .by_index(index)
            .map_err(|_| "archive ZIP itch.io illisible".to_string())?;
        let Some(relative) = entry.enclosed_name() else {
            return Err("chemin dangereux dans l'archive itch.io".to_string());
        };
        let target = destination.join(relative);
        if entry.is_dir() {
            std::fs::create_dir_all(&target)
                .map_err(|_| "dossier de l'archive itch.io non créable".to_string())?;
            continue;
        }
        if entry
            .unix_mode()
            .is_some_and(|mode| mode & 0o170000 == 0o120000)
        {
            return Err("lien symbolique refusé dans l'archive itch.io".to_string());
        }
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|_| "dossier de l'archive itch.io non créable".to_string())?;
        }
        let mut out = File::create(&target)
            .map_err(|_| "fichier de l'archive itch.io non créable".to_string())?;
        std::io::copy(&mut entry, &mut out)
            .map_err(|_| "extraction de l'archive itch.io impossible".to_string())?;
    }
    Ok(())
}

fn extract_tar(archive: &Path, destination: &Path, gzip: bool) -> Result<(), String> {
    let file = File::open(archive).map_err(|_| "archive itch.io introuvable".to_string())?;
    let reader: Box<dyn Read> = if gzip {
        Box::new(flate2::read::GzDecoder::new(file))
    } else {
        Box::new(file)
    };
    let mut tar = tar::Archive::new(reader);
    let entries = tar
        .entries()
        .map_err(|_| "archive TAR itch.io illisible".to_string())?;
    for entry in entries {
        let mut entry = entry.map_err(|_| "archive TAR itch.io illisible".to_string())?;
        let path = entry
            .path()
            .map_err(|_| "chemin de l'archive itch.io illisible".to_string())?
            .to_path_buf();
        if path.is_absolute()
            || path
                .components()
                .any(|component| matches!(component, std::path::Component::ParentDir))
        {
            return Err("chemin dangereux dans l'archive itch.io".to_string());
        }
        let kind = entry.header().entry_type();
        if kind.is_symlink() || kind.is_hard_link() {
            return Err("lien refusé dans l'archive itch.io".to_string());
        }
        if !kind.is_dir() && !kind.is_file() {
            continue;
        }
        let target = destination.join(&path);
        if kind.is_dir() {
            std::fs::create_dir_all(&target)
                .map_err(|_| "dossier de l'archive itch.io non créable".to_string())?;
            continue;
        }
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|_| "dossier de l'archive itch.io non créable".to_string())?;
        }
        entry
            .unpack(&target)
            .map_err(|_| "extraction de l'archive itch.io impossible".to_string())?;
    }
    Ok(())
}

fn extract(archive: &Path, destination: &Path) -> Result<(), String> {
    let name = archive
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    if name.ends_with(".zip") {
        extract_zip(archive, destination)
    } else if name.ends_with(".tar.gz") || name.ends_with(".tgz") {
        extract_tar(archive, destination, true)
    } else if name.ends_with(".tar") {
        extract_tar(archive, destination, false)
    } else {
        Err("format d'archive itch.io non pris en charge".to_string())
    }
}

fn safe_folder(name: &str, game_id: &str) -> String {
    let mut folder: String = name
        .chars()
        .map(|character| {
            if character.is_alphanumeric() || matches!(character, ' ' | '-' | '_' | '.' | '(' | ')')
            {
                character
            } else {
                '_'
            }
        })
        .collect();
    folder = folder.trim().trim_matches('.').to_string();
    if folder.is_empty() {
        folder = format!("itch-{game_id}");
    }
    folder.truncate(80);
    folder
}

fn unique_folder(destination: &Path, folder: &str) -> PathBuf {
    let candidate = destination.join(folder);
    if !candidate.exists() {
        return candidate;
    }
    for suffix in 2..1000 {
        let candidate = destination.join(format!("{folder} ({suffix})"));
        if !candidate.exists() {
            return candidate;
        }
    }
    destination.join(format!("{folder} (nouveau)"))
}

/// Exécutable le plus probable : un `.exe` à faible profondeur, sinon un
/// binaire exécutable portant le nom du jeu.
pub fn find_executable(root: &Path, title: &str) -> Option<PathBuf> {
    fn walk(dir: &Path, depth: usize, found: &mut Vec<PathBuf>) {
        if depth > 4 || found.len() > 200 {
            return;
        }
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, depth + 1, found);
            } else {
                found.push(path);
            }
        }
    }
    let mut files = Vec::new();
    walk(root, 0, &mut files);
    let normalized = title.to_lowercase();
    let mut candidates: Vec<PathBuf> = files
        .into_iter()
        .filter(|path| {
            if path
                .extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| extension.eq_ignore_ascii_case("exe"))
            {
                return true;
            }
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                path.metadata()
                    .map(|metadata| metadata.permissions().mode() & 0o111 != 0)
                    .unwrap_or(false)
            }
            #[cfg(not(unix))]
            false
        })
        .collect();
    candidates.sort_by_key(|path| path.components().count());
    candidates
        .iter()
        .find(|path| {
            path.file_stem()
                .and_then(|stem| stem.to_str())
                .is_some_and(|stem| {
                    let stem = stem.to_lowercase();
                    stem.len() > 3 && (normalized.contains(&stem) || stem.contains(&normalized))
                })
        })
        .cloned()
        .or_else(|| candidates.into_iter().next())
}

/// Télécharge et extrait le jeu. Le dossier choisi reçoit un sous-dossier au
/// nom du jeu, comme gogdl ; aucune suppression récursive n'est faite ici.
pub fn install(
    api_key: &str,
    game: &ProviderGame,
    destination: &Path,
    progress: Arc<Mutex<InstallProgress>>,
    cancel: Arc<AtomicBool>,
) -> Result<ProviderGame, String> {
    let key = find_key(api_key, &game.key.product_id)?;
    let uploads = fetch_uploads(api_key, &key)?;
    let upload = pick_upload(&uploads).ok_or_else(|| {
        "itch.io ne publie aucune archive téléchargeable pour ce jeu (installe-le manuellement depuis sa page)"
            .to_string()
    })?;
    std::fs::create_dir_all(destination)
        .map_err(|_| "dossier de destination itch.io invalide".to_string())?;
    let install_path = unique_folder(destination, &safe_folder(&game.title, &game.key.product_id));
    std::fs::create_dir_all(&install_path)
        .map_err(|_| "dossier d'installation itch.io non créable".to_string())?;
    set_progress(&progress, "Téléchargement", Some(0.0), Some(0), upload.size);
    let archive = match download(api_key, &key, upload, destination, &progress, &cancel) {
        Ok(path) => path,
        Err(error) => {
            let _ = std::fs::remove_dir_all(&install_path);
            return Err(error);
        }
    };
    set_progress(&progress, "Extraction", None, None, None);
    let result = extract(&archive, &install_path);
    let _ = std::fs::remove_file(&archive);
    if let Err(error) = result {
        let _ = std::fs::remove_dir_all(&install_path);
        return Err(error);
    }
    let executable = find_executable(&install_path, &game.title);
    Ok(ProviderGame {
        key: game.key.clone(),
        title: game.title.clone(),
        owned: true,
        install_root: Some(destination.to_path_buf()),
        install_path: Some(install_path),
        install_exe: executable,
        install_state: InstallState::Installed,
        cover_url: game.cover_url.clone(),
        quark_game_id: None,
        custom_save_paths: Vec::new(),
        capabilities: game.capabilities.clone(),
        last_synced: Some(crate::config::now_ts()),
    })
}

/// Fusionne la bibliothèque rafraîchie sans perdre les installations locales.
pub fn merge_itch(existing: &[ProviderGame], refreshed: &[ProviderGame]) -> Vec<ProviderGame> {
    let mut merged: Vec<ProviderGame> = existing
        .iter()
        .filter(|game| game.key.provider != Provider::Itch)
        .cloned()
        .collect();
    for incoming in refreshed {
        let mut game = incoming.clone();
        if let Some(previous) = existing
            .iter()
            .find(|previous| previous.key == incoming.key)
            .or_else(|| {
                existing.iter().find(|previous| {
                    previous.key.provider == Provider::Itch
                        && previous
                            .title
                            .trim()
                            .eq_ignore_ascii_case(incoming.title.trim())
                })
            })
        {
            game.quark_game_id = previous.quark_game_id.clone();
            game.custom_save_paths = previous.custom_save_paths.clone();
            if game.cover_url.is_none() {
                game.cover_url = previous.cover_url.clone();
            }
            if game.install_exe.is_none() {
                game.install_exe = previous.install_exe.clone();
            }
            if game.install_root.is_none() {
                game.install_root = previous.install_root.clone();
            }
            if previous.install_state == InstallState::Installed
                && previous
                    .install_path
                    .as_ref()
                    .is_some_and(|path| path.is_dir())
            {
                game.install_state = InstallState::Installed;
                game.install_path = previous.install_path.clone();
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
    fn une_bibliotheque_deduplique_les_cles_et_garde_la_plus_recente() {
        let body = r#"{
            "owned_keys": [
                {"id": 10, "game_id": 42, "created_at": "2024-02-01T00:00:00Z",
                 "game": {"title": "Premier", "url": "https://studio.itch.io/premier",
                          "cover_url": "https://img.itch.zone/premier.png", "traits": ["p_windows"]}},
                {"id": 9, "game_id": 42, "created_at": "2023-01-01T00:00:00Z",
                 "game": {"title": "Premier", "traits": []}},
                {"id": 11, "game_id": 43, "created_at": "2024-03-01T00:00:00Z",
                 "game": {"title": "Deuxième"}}
            ]
        }"#;
        let keys = parse_owned_keys(body).unwrap();
        assert_eq!(keys.len(), 2);
        assert_eq!(keys[0].id, 10);
        assert_eq!(keys[0].game_id, 42);
        assert_eq!(keys[1].game_id, 43);
        assert_eq!(
            keys[0].cover_url.as_deref(),
            Some("https://img.itch.zone/premier.png")
        );
    }

    #[test]
    fn les_uploads_windows_en_archive_sont_preferes() {
        let uploads = vec![
            Upload {
                id: 1,
                filename: "jeu-linux.tar.gz".into(),
                size: Some(20),
                updated_at: "2024-01-01T00:00:00Z".into(),
                traits: vec!["p_linux".into()],
            },
            Upload {
                id: 2,
                filename: "jeu-windows.zip".into(),
                size: Some(10),
                updated_at: "2023-01-01T00:00:00Z".into(),
                traits: vec!["p_windows".into()],
            },
            Upload {
                id: 3,
                filename: "installeur.exe".into(),
                size: Some(30),
                updated_at: "2025-01-01T00:00:00Z".into(),
                traits: vec!["p_windows".into()],
            },
        ];
        assert_eq!(pick_upload(&uploads).unwrap().id, 2);
    }

    #[test]
    fn un_nom_de_dossier_ne_peut_pas_sortir_de_la_destination() {
        assert_eq!(safe_folder("../Mon Jeu", "42"), "_Mon Jeu");
        assert_eq!(safe_folder("", "42"), "itch-42");
        assert!(unique_folder(Path::new("/tmp"), "quark-itch-test").is_absolute());
    }

    #[test]
    fn les_erreurs_itch_ne_contiennent_pas_de_cle() {
        let body = r#"{"errors":["authentication required"]}"#;
        assert_eq!(api_error(body).as_deref(), Some("authentication required"));
    }
}
