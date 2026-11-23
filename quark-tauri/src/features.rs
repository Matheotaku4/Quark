//! Commandes complémentaires de l'interface : réglages et jeux en JSON brut
//! (toutes les options du cœur, sans DTO à maintenir), réseau (actualités,
//! HowLongToBeat, ProtonDB), disque et mises à jour.

use super::{game_dto, lock, persist, AppState, GameDto};
use quark::config::{Game, Settings};
use quark::{artwork, disk, howlongtobeat, news, proton, protondb, update};
use serde::Serialize;
use serde_json::Value;
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
