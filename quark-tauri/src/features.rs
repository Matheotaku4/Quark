//! Commandes complémentaires de l'interface : réglages et jeux en JSON brut
//! (toutes les options du cœur, sans DTO à maintenir), réseau (actualités,
//! HowLongToBeat, ProtonDB), disque et mises à jour.

use super::{game_dto, lock, persist, AppState, GameDto};
use quark::config::{Game, Settings};
use quark::{artwork, disk, howlongtobeat, news, proton, protondb, update};
use serde::Serialize;
use serde_json::Value;
use std::path::{Path, PathBuf};
use tauri::State;

/// Fusionne `patch` dans `target` clé par clé (les sous-objets sont remplacés
/// en entier : l'interface renvoie toujours l'objet complet).
fn merge(target: &mut Value, patch: Value) {
    match (target, patch) {
        (Value::Object(target), Value::Object(patch)) => {
            for (key, value) in patch {
                target.insert(key, value);
            }
        }
        (target, patch) => *target = patch,
    }
}

async fn blocking<T: Send + 'static>(
    job: impl FnOnce() -> Result<T, String> + Send + 'static,
) -> Result<T, String> {
    tauri::async_runtime::spawn_blocking(job)
        .await
        .map_err(|error| error.to_string())?
}

// ---------------------------------------------------------------------------
// Réglages et jeux, en JSON
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn get_settings_raw(state: State<'_, AppState>) -> Result<Value, String> {
    serde_json::to_value(&lock(&state.library).settings).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn patch_settings_raw(state: State<'_, AppState>, patch: Value) -> Result<Value, String> {
    let mut library = lock(&state.library);
    let mut current = serde_json::to_value(&library.settings).map_err(|error| error.to_string())?;
    merge(&mut current, patch);
    let next: Settings = serde_json::from_value(current.clone())
        .map_err(|error| format!("Réglage invalide : {error}"))?;
    library.settings = next;
    persist(&state, &mut library)?;
    Ok(current)
}

#[tauri::command]
pub fn get_game_raw(state: State<'_, AppState>, id: String) -> Result<Value, String> {
    let library = lock(&state.library);
    let game = library
        .get(&id)
        .ok_or_else(|| "Jeu introuvable".to_string())?;
    serde_json::to_value(game).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn patch_game_raw(
    state: State<'_, AppState>,
    id: String,
    patch: Value,
) -> Result<GameDto, String> {
    let mut library = lock(&state.library);
    let mut current = {
        let game = library
            .get(&id)
            .ok_or_else(|| "Jeu introuvable".to_string())?;
        serde_json::to_value(game).map_err(|error| error.to_string())?
    };
    merge(&mut current, patch);
    let next: Game =
        serde_json::from_value(current).map_err(|error| format!("Réglage invalide : {error}"))?;
    *library
        .get_mut(&id)
        .ok_or_else(|| "Jeu introuvable".to_string())? = next;
    persist(&state, &mut library)?;
    let settings = library.settings.clone();
    let sessions = lock(&state.sessions);
    let run = sessions.get(&id);
    let game = library
        .get(&id)
        .ok_or_else(|| "Jeu introuvable".to_string())?;
    Ok(game_dto(
        game,
        &settings,
        run.is_some(),
        run.map(|run| run.label.clone()),
    ))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DllCandidateDto {
    pub name: String,
    pub path: String,
    pub relative: String,
}

const DLL_SCAN_DEPTH: usize = 6;
const DLL_SCAN_LIMIT: usize = 400;

/// Liste les DLL visibles dans le dossier du jeu (ou dans un dossier choisi),
/// sous-dossiers compris. Les noms sont dédoublonnés sans tenir compte de la
/// casse : `WINEDLLOVERRIDES` ne peut cibler qu'un module à la fois.
#[tauri::command]
pub fn detect_game_dlls(
    state: State<'_, AppState>,
    id: String,
    folder: Option<String>,
) -> Result<Vec<DllCandidateDto>, String> {
    let root = {
        let library = lock(&state.library);
        let game = library
            .get(&id)
            .ok_or_else(|| "Jeu introuvable".to_string())?;
        folder
            .as_deref()
            .map(str::trim)
            .filter(|path| !path.is_empty())
            .map(PathBuf::from)
            .or_else(|| {
                game.dll_source
                    .as_deref()
                    .map(str::trim)
                    .filter(|path| !path.is_empty())
                    .map(PathBuf::from)
            })
            .or_else(|| game.exe_path().parent().map(Path::to_path_buf))
            .ok_or_else(|| "Dossier du jeu introuvable.".to_string())?
    };
    if !root.is_dir() {
        return Err(format!("Dossier introuvable : {}", root.display()));
    }
    Ok(scan_dlls(&root))
}

fn scan_dlls(root: &Path) -> Vec<DllCandidateDto> {
    let mut out = Vec::new();
    collect_dlls(root, root, 0, &mut out);
    out.sort_by_key(|row| row.relative.to_lowercase());
    out
}

fn collect_dlls(root: &Path, dir: &Path, depth: usize, out: &mut Vec<DllCandidateDto>) {
    if out.len() >= DLL_SCAN_LIMIT {
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        if out.len() >= DLL_SCAN_LIMIT {
            return;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            continue;
        }
        let path = entry.path();
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if file_type.is_symlink() {
            continue;
        }
        if file_type.is_dir() {
            if depth < DLL_SCAN_DEPTH {
                collect_dlls(root, &path, depth + 1, out);
            }
            continue;
        }
        if !path
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("dll"))
        {
            continue;
        }
        let Some(stem) = path.file_stem().map(|stem| stem.to_string_lossy().to_string()) else {
            continue;
        };
        if stem.is_empty() || out.iter().any(|row| row.name.eq_ignore_ascii_case(&stem)) {
            continue;
        }
        out.push(DllCandidateDto {
            name: stem,
            path: path.display().to_string(),
            relative: path
                .strip_prefix(root)
                .unwrap_or(&path)
                .display()
                .to_string(),
        });
    }
}

#[derive(Serialize)]
pub struct RuntimeDto {
    key: String,
    name: String,
    version: Option<String>,
}

/// Détection à chaud : « Détecter à nouveau » dans les réglages.
#[tauri::command]
pub async fn list_runtimes() -> Result<Vec<RuntimeDto>, String> {
    blocking(|| {
        Ok(proton::detect_runtimes()
            .into_iter()
            .filter(|runtime| runtime.has_proton_script)
            .map(|runtime| RuntimeDto {
                key: runtime.key,
                name: runtime.name,
                version: runtime.version,
            })
            .collect())
    })
    .await
}

// ---------------------------------------------------------------------------
// Réseau : actualités, HowLongToBeat, ProtonDB
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct NewsDto {
    title: String,
    url: String,
    feed: String,
    author: String,
    date: i64,
    excerpt: String,
    body: String,
}

#[tauri::command]
pub async fn fetch_news(app_id: u32, count: Option<usize>) -> Result<Vec<NewsDto>, String> {
    blocking(move || {
        let items = news::fetch(&artwork::http(), app_id, count.unwrap_or(8))?;
        Ok(items
            .into_iter()
            .map(|item| NewsDto {
                title: item.title,
                url: item.url,
                feed: item.feed,
                author: item.author,
                date: item.date,
                excerpt: item.excerpt,
                body: item.body,
            })
            .collect())
    })
    .await
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HltbDto {
    url: String,
    main: Option<f64>,
    main_extra: Option<f64>,
    completionist: Option<f64>,
}

#[tauri::command]
pub async fn fetch_hltb(title: String) -> Result<HltbDto, String> {
    blocking(move || {
        let report = howlongtobeat::fetch(&artwork::http(), &title)?;
        Ok(HltbDto {
            url: report.url(),
            main: report.main,
            main_extra: report.main_extra,
            completionist: report.completionist,
        })
    })
    .await
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProtonDbDto {
    tier: String,
    label: String,
    color: String,
    total: u32,
    confidence: String,
}

#[tauri::command]
pub async fn fetch_protondb(app_id: u32) -> Result<ProtonDbDto, String> {
    blocking(move || {
        let report = protondb::fetch(&artwork::http(), app_id)?;
        let (r, g, b) = report.tier.color();
        Ok(ProtonDbDto {
            tier: report.tier.label().to_string(),
            label: report.label(),
            color: format!("#{r:02x}{g:02x}{b:02x}"),
            total: report.total,
            confidence: report.confidence,
        })
    })
    .await
}

// ---------------------------------------------------------------------------
// Disque, mises à jour, liens
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct DiskRowDto {
    label: String,
    path: String,
    bytes: u64,
    human: String,
    detail: String,
}

#[derive(Serialize)]
pub struct DiskDto {
    rows: Vec<DiskRowDto>,
    total: u64,
    human: String,
}

#[tauri::command]
pub async fn disk_report(state: State<'_, AppState>) -> Result<DiskDto, String> {
    let library = lock(&state.library).clone();
    blocking(move || {
        let report = disk::scan(&library);
        Ok(DiskDto {
            total: report.total,
            human: disk::format_bytes(report.total),
            rows: report
                .rows
                .into_iter()
                .map(|row| DiskRowDto {
                    label: row.label,
                    path: row.path.display().to_string(),
                    bytes: row.bytes,
                    human: disk::format_bytes(row.bytes),
                    detail: row.detail,
                })
                .collect(),
        })
    })
    .await
}

#[derive(Serialize)]
pub struct UpdateDto {
    current: String,
    tag: String,
    name: String,
    url: String,
    newer: bool,
}

#[tauri::command]
pub async fn check_update() -> Result<UpdateDto, String> {
    blocking(|| {
        let release = update::latest(&artwork::http())?;
        Ok(UpdateDto {
            current: update::current_version().to_string(),
            newer: update::is_newer(&release.tag_name, update::current_version()),
            tag: release.tag_name,
            name: release.name,
            url: release.html_url,
        })
    })
    .await
}

/// Télécharge la dernière release, vérifie son SHA-256 et son ELF, remplace le
/// binaire courant puis relance Quark. `exec` ne revient pas en cas de succès.
#[tauri::command]
pub async fn install_update() -> Result<(), String> {
    blocking(|| {
        if !update::automatic_update_allowed() {
            return Err(
                "La mise à jour intégrée est désactivée pour cette exécution.".to_string(),
            );
        }
        let client = artwork::http();
        let release = update::latest(&client)?;
        if !update::is_newer(&release.tag_name, update::current_version()) {
            return Err("Quark est déjà à jour.".to_string());
        }
        let asset = release
            .archive()
            .ok_or_else(|| "La release ne publie pas d'archive Linux.".to_string())?;
        let target = std::env::current_exe()
            .map_err(|error| format!("Binaire courant introuvable : {error}"))?;
        let directory = quark::config::data_dir().join("updates");
        let staged = update::stage(&client, &release, asset, &directory, &target)?;
        let installed = update::apply(&staged)?;
        if let Err(error) = update::relaunch(&target) {
            let _ = update::rollback(&installed);
            return Err(error);
        }
        Ok(())
    })
    .await
}

/// Ouvre un lien ou un dossier avec l'application par défaut du bureau.
#[tauri::command]
pub fn open_url(target: String) -> Result<(), String> {
    if target.starts_with("http://") || target.starts_with("https://") {
        news::open_in_browser(&target)
    } else if std::path::Path::new(&target).exists() {
        std::process::Command::new("xdg-open")
            .arg(&target)
            .spawn()
            .map(|_| ())
            .map_err(|error| format!("xdg-open : {error}"))
    } else {
        Err("Chemin introuvable".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(label: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "quark-dll-scan-{label}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn les_dll_des_sous_dossiers_sont_trouvees_et_dedoublonnees() {
        let root = temp_dir("nested");
        std::fs::create_dir_all(root.join("Binaries/Win64")).unwrap();
        std::fs::write(root.join("winmm.dll"), b"dll").unwrap();
        std::fs::write(root.join("Binaries/Win64/winmm.DLL"), b"dll").unwrap();
        std::fs::write(root.join("Binaries/Win64/steam_api64.dll"), b"dll").unwrap();
        std::fs::write(root.join("Binaries/readme.txt"), b"txt").unwrap();

        let found = scan_dlls(&root);
        let names: Vec<_> = found.iter().map(|row| row.name.as_str()).collect();
        assert_eq!(names, ["steam_api64", "winmm"]);
        assert_eq!(found[1].relative, "winmm.dll");
        assert!(found[0].relative.ends_with("steam_api64.dll"));

        std::fs::remove_dir_all(root).unwrap();
    }
}
