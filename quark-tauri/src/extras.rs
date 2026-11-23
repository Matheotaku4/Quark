//! Succès, médias, sauvegardes et illustrations.
//!
//! Le cœur expose déjà toutes les opérations ; cette couche les rend appelables
//! depuis la fiche d'un jeu et depuis les réglages.

use super::{game_dto, lock, persist, AppState, GameDto};
use quark::config::{self, AssetKind};
use quark::providers::{
    self, achievements, cloud_saves, credentials, gog, gog_library, legendary,
    model::{Provider, ProviderGame},
    steam,
};
use quark::saves::{BackupInfo, BackupPhase, BackupPreview, SaveIdentity, SaveManager};
use quark::{artwork, media};
use serde::Serialize;
use std::path::PathBuf;
use tauri::State;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AchievementRowDto {
    key: String,
    name: String,
    description: String,
    unlocked: bool,
    progress: f32,
    session: bool,
    rarity: Option<f32>,
    icon_url: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AchievementsDto {
    total: u32,
    unlocked: u32,
    rows: Vec<AchievementRowDto>,
    provider: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrailerDto {
    title: String,
    thumbnail: String,
    video: String,
    page: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PhotoDto {
    thumbnail: String,
    full: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaDto {
    trailers: Vec<TrailerDto>,
    screenshots: Vec<PhotoDto>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveFileDto {
    path: String,
    change: Option<String>,
    bytes: Option<u64>,
    failed: bool,
    ignored: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SavePreviewDto {
    title: String,
    files: Vec<SaveFileDto>,
    errors: Vec<String>,
    changed_games: Option<usize>,
    processed_games: Option<usize>,
    processed_bytes: Option<u64>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveBackupDto {
    id: String,
    when: String,
    locked: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveBackupResultDto {
    title: String,
    phase: String,
    changed_games: usize,
    processed_bytes: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CloudSyncDto {
    message: String,
    timestamp: Option<i64>,
}

fn provider_record(library: &config::Library, game_id: &str) -> Option<ProviderGame> {
    library
        .provider_library
        .games
        .iter()
        .find(|record| {
            record.quark_game_id.as_deref() == Some(game_id)
                && record.install_state == providers::model::InstallState::Installed
                && record
                    .install_path
                    .as_ref()
                    .is_some_and(|path| path.is_dir())
        })
        .cloned()
}

fn save_identity(library: &config::Library, game_id: &str) -> Result<SaveIdentity, String> {
    let game = library
        .get(game_id)
        .ok_or_else(|| "Jeu introuvable.".to_string())?;
    let record = provider_record(library, game_id).ok_or_else(|| {
        "Les sauvegardes locales demandent une installation de boutique vérifiée.".to_string()
    })?;
    let install_path = record
        .install_path
        .clone()
        .filter(|path| path.is_dir())
        .ok_or_else(|| "Chemin d'installation introuvable.".to_string())?;
    let wine_prefix = Some(game.wine_prefix_path(&library.settings))
        .filter(|prefix| prefix.join("drive_c").is_dir());
    Ok(SaveIdentity {
        provider_key: record.key.clone(),
        install_path: Some(install_path),
        wine_prefix,
        ludusavi_title: record.title.clone(),
        custom_paths: record.custom_save_paths.clone(),
    })
}

fn preview_dto(preview: BackupPreview) -> SavePreviewDto {
    SavePreviewDto {
        title: preview.title,
        files: preview
            .files
            .into_iter()
            .map(|file| SaveFileDto {
                path: file.path,
                change: file.change,
                bytes: file.bytes,
                failed: file.failed,
                ignored: file.ignored,
            })
            .collect(),
        errors: preview.errors,
        changed_games: preview.changed_games,
        processed_games: preview.processed_games,
        processed_bytes: preview.processed_bytes,
    }
}

#[tauri::command]
pub async fn game_achievements(
    state: State<'_, AppState>,
    id: String,
) -> Result<AchievementsDto, String> {
    let (record, steam_id, credentials) = {
        let library = lock(&state.library);
        let game = library
            .get(&id)
            .ok_or_else(|| "Jeu introuvable.".to_string())?;
        let record = provider_record(&library, &id);
        let credentials = credentials::load(&config::credentials_path()).unwrap_or_default();
        (record, game.steam_app_id, credentials)
    };
    let record = record.ok_or_else(|| {
        "Les succès sont disponibles pour les jeux de boutique installés.".to_string()
    })?;
    let provider = record.key.provider;
    let product = record.key.product_id.clone();
    let result = tauri::async_runtime::spawn_blocking(
        move || -> Result<achievements::AchievementInfo, String> {
            let mut info = match provider {
                Provider::Steam => {
                    let app_id =
                        steam_id.ok_or_else(|| "Ce jeu n'a pas d'AppID Steam.".to_string())?;
                    let key = credentials.steam_web_api_key.as_deref().ok_or_else(|| {
                        "Ajoute une clé Steam Web API dans Boutiques & outils.".to_string()
                    })?;
                    let steam_id = credentials.steam_id.as_deref().ok_or_else(|| {
                        "Ajoute ton SteamID64 dans Boutiques & outils.".to_string()
                    })?;
                    steam::fetch_achievements(&app_id.to_string(), key, steam_id)?
                }
                Provider::Epic => legendary::epic_achievements(&product)?,
                Provider::Gog => {
                    let tokens =
                        gog_library::usable_tokens(&gog::auth_config_path(), config::now_ts())?;
                    gog_library::fetch_achievements(
                        &product,
                        &tokens.access_token,
                        &tokens.user_id,
                    )?
                }
                Provider::Itch => {
                    return Err("itch.io ne publie pas de succès.".to_string());
                }
            };
            achievements::cache_icons(&mut info, provider, &product);
            info.normalize();
            Ok(info)
        },
    )
    .await
    .map_err(|error| error.to_string())??;
    Ok(AchievementsDto {
        total: result.total,
        unlocked: result.unlocked,
        provider: format!("{provider:?}").to_lowercase(),
        rows: result
            .rows
            .into_iter()
            .map(|row| AchievementRowDto {
                key: row.key,
                name: row.name,
                description: row.description,
                unlocked: row.unlocked,
                progress: row.progress,
                session: row.session,
                rarity: row.rarity,
                icon_url: row
                    .icon_path
                    .as_ref()
                    .and_then(|path| super::asset_url(Some(path)))
                    .or(row.icon_url),
            })
            .collect(),
    })
}

#[tauri::command]
pub async fn game_playtime(state: State<'_, AppState>, id: String) -> Result<u64, String> {
    let record = {
        let library = lock(&state.library);
        provider_record(&library, &id)
            .ok_or_else(|| "Aucune installation de boutique vérifiée.".to_string())?
    };
    tauri::async_runtime::spawn_blocking(move || providers::playtime::fetch(&record.key))
        .await
        .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn game_media(state: State<'_, AppState>, id: String) -> Result<MediaDto, String> {
    let (app_id, language, country) = {
        let library = lock(&state.library);
        let game = library
            .get(&id)
            .ok_or_else(|| "Jeu introuvable.".to_string())?;
        (
            game.steam_app_id
                .ok_or_else(|| "Ce jeu n'a pas d'AppID Steam.".to_string())?,
            library.settings.language.clone(),
            library.settings.country.clone(),
        )
    };
    let result = tauri::async_runtime::spawn_blocking(move || {
        media::fetch(&artwork::http(), app_id, &language, &country)
    })
    .await
    .map_err(|error| error.to_string())??;
    Ok(MediaDto {
        trailers: result
            .trailers
            .into_iter()
            .map(|trailer| TrailerDto {
                title: trailer.title,
                thumbnail: trailer.thumbnail,
                video: trailer.video,
                page: trailer.page,
            })
            .collect(),
        screenshots: result
            .screenshots
            .into_iter()
            .map(|photo| PhotoDto {
                thumbnail: photo.thumbnail,
                full: photo.full,
            })
            .collect(),
    })
}

#[tauri::command]
pub async fn save_preview(
    state: State<'_, AppState>,
    id: String,
) -> Result<SavePreviewDto, String> {
    let identity = {
        let library = lock(&state.library);
        save_identity(&library, &id)?
    };
    let result =
        tauri::async_runtime::spawn_blocking(move || SaveManager::new().preview(&identity))
            .await
            .map_err(|error| error.to_string())??;
    Ok(preview_dto(result))
}

#[tauri::command]
pub async fn save_backups(
    state: State<'_, AppState>,
    id: String,
) -> Result<Vec<SaveBackupDto>, String> {
    let identity = {
        let library = lock(&state.library);
        save_identity(&library, &id)?
    };
    let result =
        tauri::async_runtime::spawn_blocking(move || SaveManager::new().backups(&identity))
            .await
            .map_err(|error| error.to_string())??;
    Ok(result
        .into_iter()
        .map(|backup: BackupInfo| SaveBackupDto {
            id: backup.id,
            when: backup.when,
            locked: backup.locked,
        })
        .collect())
}

#[tauri::command]
pub async fn save_backup(
    state: State<'_, AppState>,
    id: String,
    phase: String,
) -> Result<SaveBackupResultDto, String> {
    let identity = {
        let library = lock(&state.library);
        save_identity(&library, &id)?
    };
    let phase = match phase.as_str() {
        "after" => BackupPhase::AfterExit,
        _ => BackupPhase::BeforeLaunch,
    };
    let result = tauri::async_runtime::spawn_blocking(move || {
        SaveManager::new().backup(&identity, phase, true)
    })
    .await
    .map_err(|error| error.to_string())??;
    Ok(SaveBackupResultDto {
        title: result.title,
        phase: format!("{:?}", result.phase).to_lowercase(),
        changed_games: result.changed_games,
        processed_bytes: result.processed_bytes,
    })
}

#[tauri::command]
pub async fn save_restore_preview(
    state: State<'_, AppState>,
    id: String,
    backup_id: String,
) -> Result<SavePreviewDto, String> {
    let identity = {
        let library = lock(&state.library);
        save_identity(&library, &id)?
    };
    let result = tauri::async_runtime::spawn_blocking(move || {
        SaveManager::new().restore_preview(&identity, &backup_id)
    })
    .await
    .map_err(|error| error.to_string())??;
    Ok(preview_dto(result))
}

#[tauri::command]
pub async fn save_restore(
    state: State<'_, AppState>,
    id: String,
    backup_id: String,
    expected_paths: Vec<String>,
) -> Result<(), String> {
    let identity = {
        let library = lock(&state.library);
        save_identity(&library, &id)?
    };
    tauri::async_runtime::spawn_blocking(move || {
        SaveManager::new().restore(&identity, &backup_id, true, &expected_paths)
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub fn save_add_folder(state: State<'_, AppState>, id: String, path: String) -> Result<(), String> {
    let path = PathBuf::from(config::expand_tilde(&path));
    if !path.is_absolute() {
        return Err("Le dossier de sauvegarde doit être absolu.".to_string());
    }
    let mut library = lock(&state.library);
    let record = library
        .provider_library
        .games
        .iter_mut()
        .find(|record| record.quark_game_id.as_deref() == Some(&id))
        .ok_or_else(|| "Ce jeu n'a pas d'installation de boutique.".to_string())?;
    if !record.custom_save_paths.contains(&path) {
        record.custom_save_paths.push(path);
    }
    persist(&state, &mut library)
}

#[tauri::command]
pub fn save_remove_folder(
    state: State<'_, AppState>,
    id: String,
    path: String,
) -> Result<(), String> {
    let path = PathBuf::from(config::expand_tilde(&path));
    let mut library = lock(&state.library);
    let record = library
        .provider_library
        .games
        .iter_mut()
        .find(|record| record.quark_game_id.as_deref() == Some(&id))
        .ok_or_else(|| "Ce jeu n'a pas d'installation de boutique.".to_string())?;
    record
        .custom_save_paths
        .retain(|candidate| candidate != &path);
    persist(&state, &mut library)
}

#[tauri::command]
pub async fn cloud_sync(state: State<'_, AppState>, id: String) -> Result<CloudSyncDto, String> {
    let (record, save_path, timestamp) = {
        let library = lock(&state.library);
        let record = provider_record(&library, &id)
            .ok_or_else(|| "Aucune installation de boutique vérifiée.".to_string())?;
        let game = library.get(&id);
        let save_path = cloud_saves::save_path_for(game, &record);
        let timestamp = library
            .installation_for_provider(&id, &record.key)
            .and_then(|installation| installation.save_sync_ts);
        (record, save_path, timestamp)
    };
    let record_for_job = record.clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        cloud_saves::sync(&record_for_job, save_path.as_deref(), timestamp)
    })
    .await
    .map_err(|error| error.to_string())??;
    {
        let mut library = lock(&state.library);
        if let Some(installation) = library.installation_for_provider_mut(&id, &record.key) {
            installation.last_synced = Some(config::now_ts());
            if let Some(timestamp) = result.timestamp {
                installation.save_sync_ts = Some(timestamp);
            }
        }
        persist(&state, &mut library)?;
    }
    Ok(CloudSyncDto {
        message: result.message,
        timestamp: result.timestamp,
    })
}

#[tauri::command]
pub async fn artwork_fetch(
    state: State<'_, AppState>,
    id: String,
    query: Option<String>,
) -> Result<GameDto, String> {
    let (mut game, settings) = {
        let library = lock(&state.library);
        (
            library
                .get(&id)
                .cloned()
                .ok_or_else(|| "Jeu introuvable.".to_string())?,
            library.settings.clone(),
        )
    };
    let query = query
        .map(|query| query.trim().to_string())
        .filter(|query| !query.is_empty())
        .unwrap_or_else(|| game.name.clone());
    let settings_for_job = settings.clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        let client = artwork::http();
        let outcome = artwork::search(&client, &query, &settings_for_job);
        let hit = artwork::best_hit(&outcome, &query)
            .ok_or_else(|| "Aucune illustration trouvée.".to_string())?;
        let (art, warnings) = artwork::fetch(&client, &game.id, &hit, &settings_for_job);
        let metadata = hit
            .steam_app_id
            .and_then(|app_id| artwork::metadata(&client, app_id, &settings_for_job).ok());
        game.steam_app_id = hit.steam_app_id;
        game.sgdb_id = hit.sgdb_id;
        game.artwork = art;
        if let Some(metadata) = metadata {
            if game.developer.trim().is_empty() {
                game.developer = metadata.developer;
            }
            if game.genres.is_empty() {
                game.genres = metadata.genres;
            }
            if game.released.trim().is_empty() {
                game.released = metadata.released;
            }
            if game.description.trim().is_empty() {
                game.description = metadata.description;
            }
        }
        Ok::<_, String>((game, warnings))
    })
    .await
    .map_err(|error| error.to_string())??;
    let (game, _warnings) = result;
    let mut library = lock(&state.library);
    if let Some(current) = library.get_mut(&id) {
        *current = game.clone();
    }
    persist(&state, &mut library)?;
    let sessions = lock(&state.sessions);
    let run = sessions.get(&id);
    Ok(game_dto(
        &game,
        &settings,
        run.is_some(),
        run.map(|run| run.label.clone()),
    ))
}

#[tauri::command]
pub fn game_remove_artwork(
    state: State<'_, AppState>,
    id: String,
    kind: String,
) -> Result<GameDto, String> {
    let kind: AssetKind = serde_json::from_value(serde_json::Value::String(kind))
        .map_err(|_| "Type d'illustration inconnu.".to_string())?;
    let mut library = lock(&state.library);
    let settings = library.settings.clone();
    if let Some(game) = library.get_mut(&id) {
        if let Some(path) = game.artwork.get(kind) {
            let _ = std::fs::remove_file(path);
        }
        game.artwork.clear(kind);
    }
    persist(&state, &mut library)?;
    let game = library
        .get(&id)
        .ok_or_else(|| "Jeu introuvable.".to_string())?;
    let sessions = lock(&state.sessions);
    let run = sessions.get(&id);
    Ok(game_dto(
        game,
        &settings,
        run.is_some(),
        run.map(|run| run.label.clone()),
    ))
}
