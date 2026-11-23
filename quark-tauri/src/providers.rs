//! Boutiques, comptes et outils.
//!
//! Cette couche expose le cœur `quark::providers` à l'interface : synchronisation
//! des bibliothèques, connexion des comptes, installation/désinstallation et
//! adoption d'un jeu de boutique dans la bibliothèque Quark.

use super::{game_dto, lock, persist, AppState, GameDto};
use quark::config::{self, AssetKind, Game, Settings};
use quark::providers::{
    self, credentials, epic_account, epic_library, gog, itch, legendary,
    model::{InstallState, Provider, ProviderGame, ProviderGameKey},
    steam, tools,
};
use quark::steamlib;
use serde::Serialize;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, State};

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProviderGameDto {
    provider: String,
    product_id: String,
    title: String,
    owned: bool,
    install_state: String,
    install_path: Option<String>,
    install_root: Option<String>,
    install_exe: Option<String>,
    cover_url: Option<String>,
    quark_game_id: Option<String>,
    last_synced: Option<i64>,
    achievements: String,
    cloud_saves: String,
    can_install: bool,
    can_launch: bool,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AccountDto {
    id: String,
    label: String,
    connected: bool,
    detail: String,
    can_connect: bool,
    can_disconnect: bool,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ToolDto {
    id: String,
    label: String,
    installed: bool,
    version: Option<String>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProvidersDto {
    games: Vec<ProviderGameDto>,
    accounts: Vec<AccountDto>,
    tools: Vec<ToolDto>,
    steam_id: Option<String>,
    error: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SteamCandidateDto {
    app_id: u32,
    name: String,
    dir: String,
    exe: Option<String>,
    owned: bool,
}

fn provider_key(provider: &str, product_id: &str) -> Result<ProviderGameKey, String> {
    let provider: Provider =
        serde_json::from_value(serde_json::Value::String(provider.to_string()))
            .map_err(|_| format!("Boutique inconnue : {provider}"))?;
    Ok(ProviderGameKey {
        provider,
        product_id: product_id.to_string(),
    })
}

fn capability_label(state: &providers::model::CapabilityState) -> String {
    match state {
        providers::model::CapabilityState::Supported => "supported".to_string(),
        providers::model::CapabilityState::Unsupported(reason) => {
            format!("unsupported:{reason}")
        }
        providers::model::CapabilityState::Unknown => "unknown".to_string(),
        providers::model::CapabilityState::RequiresSetup(reason) => {
            format!("setup:{reason}")
        }
    }
}

fn provider_game_dto(
    record: &ProviderGame,
    credentials: &credentials::Credentials,
) -> ProviderGameDto {
    let cover = providers::covers::cached(&record.key)
        .and_then(|path| super::asset_url(Some(&path)))
        .or_else(|| record.cover_url.clone());
    let achievements = match record.key.provider {
        Provider::Steam
            if credentials.steam_web_api_key.is_some() && credentials.steam_id.is_some() =>
        {
            "supported".to_string()
        }
        Provider::Steam => "setup:clé Steam Web API et SteamID64 requis".to_string(),
        Provider::Epic | Provider::Gog => capability_label(&record.capabilities.achievements),
        Provider::Itch => "unsupported:itch.io ne publie pas de succès".to_string(),
    };
    let can_install = match record.key.provider {
        Provider::Steam => true,
        Provider::Epic => tools::ManagedToolId::Legendary.installed_path().is_file(),
        Provider::Gog => tools::ManagedToolId::Gogdl.installed_path().is_file(),
        Provider::Itch => credentials.itch_api_key.is_some(),
    };
    ProviderGameDto {
        provider: format!("{:?}", record.key.provider).to_lowercase(),
        product_id: record.key.product_id.clone(),
        title: record.title.clone(),
        owned: record.owned,
        install_state: serde_json::to_value(record.install_state)
            .ok()
            .and_then(|value| value.as_str().map(str::to_string))
            .unwrap_or_else(|| "unknown".to_string()),
        install_path: record
            .install_path
            .as_ref()
            .map(|path| path.display().to_string()),
        install_root: record
            .install_root
            .as_ref()
            .map(|path| path.display().to_string()),
        install_exe: record
            .install_exe
            .as_ref()
            .map(|path| path.display().to_string()),
        cover_url: cover,
        quark_game_id: record.quark_game_id.clone(),
        last_synced: record.last_synced,
        achievements,
        cloud_saves: capability_label(&record.capabilities.cloud_saves),
        can_install,
        can_launch: record.install_state == InstallState::Installed
            || record.key.provider == Provider::Steam,
    }
}

fn accounts(credentials: &credentials::Credentials) -> Vec<AccountDto> {
    let legendary_ready = tools::ManagedToolId::Legendary.installed_path().is_file();
    let epic_direct = epic_account::connected_account();
    let epic_legendary = legendary_ready && legendary::LegendaryAdapter::is_logged_in();
    let gog_connected = gog::GogAdapter::is_logged_in();
    let steam_connected = credentials
        .steam_web_api_key
        .as_deref()
        .is_some_and(|key| !key.trim().is_empty())
        && credentials.steam_id.is_some();
    vec![
        AccountDto {
            id: "steam".to_string(),
            label: "Steam".to_string(),
            connected: steam_connected,
            detail: if steam_connected {
                format!(
                    "SteamID64 · {}",
                    credentials.steam_id.as_deref().unwrap_or("—")
                )
            } else {
                "Clé Web API et SteamID64 facultatifs, pour la bibliothèque possédée et les succès."
                    .to_string()
            },
            can_connect: true,
            can_disconnect: steam_connected,
        },
        AccountDto {
            id: "epic".to_string(),
            label: "Epic Games".to_string(),
            connected: epic_direct.is_some() || epic_legendary,
            detail: if epic_direct.is_some() {
                format!(
                    "Connexion directe{}",
                    epic_direct
                        .as_ref()
                        .map(|(_, name)| format!(" · {name}"))
                        .unwrap_or_default()
                )
            } else if epic_legendary {
                "Session Legendary".to_string()
            } else if legendary_ready {
                "Legendary installé · compte non connecté".to_string()
            } else {
                "Legendary n'est pas installé".to_string()
            },
            can_connect: legendary_ready || epic_direct.is_none(),
            can_disconnect: epic_direct.is_some() || epic_legendary,
        },
        AccountDto {
            id: "gog".to_string(),
            label: "GOG".to_string(),
            connected: gog_connected,
            detail: if gog_connected {
                "Session gogdl".to_string()
            } else if tools::ManagedToolId::Gogdl.installed_path().is_file() {
                "gogdl installé · compte non connecté".to_string()
            } else {
                "gogdl n'est pas installé".to_string()
            },
            can_connect: tools::ManagedToolId::Gogdl.installed_path().is_file(),
            can_disconnect: gog_connected,
        },
        AccountDto {
            id: "itch".to_string(),
            label: "itch.io".to_string(),
            connected: credentials.itch_api_key.is_some(),
            detail: if credentials.itch_api_key.is_some() {
                "Clé API personnelle enregistrée".to_string()
            } else {
                "Clé API personnelle, pour la bibliothèque possédée".to_string()
            },
            can_connect: true,
            can_disconnect: credentials.itch_api_key.is_some(),
        },
    ]
}

fn tools_dto() -> Vec<ToolDto> {
    tools::ManagedToolId::ALL
        .iter()
        .map(|tool| {
            let path = tool.installed_path();
            ToolDto {
                id: format!("{tool:?}").to_lowercase(),
                label: tool.label().to_string(),
                installed: path.is_file(),
                version: None,
            }
        })
        .collect()
}

fn providers_dto(state: &AppState, error: Option<String>) -> ProvidersDto {
    let library = lock(&state.library);
    let credentials = credentials::load(&config::credentials_path()).unwrap_or_default();
    let mut games = library
        .provider_library
        .games
        .iter()
        .map(|game| provider_game_dto(game, &credentials))
        .collect::<Vec<_>>();
    games.sort_by_key(|game| game.title.to_lowercase());
    ProvidersDto {
        games,
        accounts: accounts(&credentials),
        tools: tools_dto(),
        steam_id: credentials.steam_id.clone(),
        error,
    }
}

#[tauri::command]
pub fn providers_data(state: State<'_, AppState>) -> ProvidersDto {
    providers_dto(&state, None)
}

fn sync_provider(
    provider: Provider,
    existing: &[ProviderGame],
    quark_games: &[Game],
    credentials: &credentials::Credentials,
) -> Result<(Vec<ProviderGame>, Option<String>), String> {
    let now = config::now_ts();
    match provider {
        Provider::Steam => {
            let local = steamlib::installed();
            let (local_records, local_error) = match &local {
                Ok(games) => (steam::installed_games(games, quark_games), None),
                Err(error) => (Vec::new(), Some(error.clone())),
            };
            let owned = match (
                credentials.steam_web_api_key.as_deref(),
                credentials.steam_id.as_deref(),
            ) {
                (Some(key), Some(id)) if !key.trim().is_empty() && steam::valid_steam_id(id) => {
                    Some(steam::owned(&quark::artwork::http(), id, key))
                }
                _ => None,
            };
            let (owned_records, owned_error) = match owned {
                Some(Ok(records)) => (Some(records), None),
                Some(Err(error)) => (None, Some(format!("API Steam : {error}"))),
                None => (None, None),
            };
            let merged = steam::merge_sync(
                existing,
                &local_records,
                owned_records.as_deref(),
                quark_games,
            );
            Ok((merged, local_error.or(owned_error)))
        }
        Provider::Epic => {
            let adapter = legendary::LegendaryAdapter::new();
            if adapter.executable.is_file() && legendary::LegendaryAdapter::is_logged_in() {
                let owned = adapter.sync_owned(now, None)?;
                let installed = adapter.sync_installed(now, None).unwrap_or_default();
                Ok((
                    legendary::merge_epic_with_verified_installs(existing, &owned, &installed),
                    None,
                ))
            } else if epic_account::connected_account().is_some() {
                let owned = epic_library::library(now)?;
                Ok((legendary::merge_epic(existing, &owned), None))
            } else {
                Err("Compte Epic non connecté. Installe Legendary puis connecte le compte, ou utilise la connexion directe par code.".to_string())
            }
        }
        Provider::Gog => {
            let adapter = gog::GogAdapter::new(gog::auth_config_path());
            let owned = adapter.sync_library(now)?;
            Ok((gog::merge_gog(existing, &owned), None))
        }
        Provider::Itch => {
            let key = credentials
                .itch_api_key
                .as_deref()
                .ok_or_else(|| "Ajoute une clé API itch.io dans Boutiques & outils.".to_string())?;
            let owned = itch::library(key, now)?;
            Ok((itch::merge_itch(existing, &owned), None))
        }
    }
}

#[tauri::command]
pub async fn provider_sync(
    state: State<'_, AppState>,
    provider: String,
) -> Result<ProvidersDto, String> {
    let provider: Provider = serde_json::from_value(serde_json::Value::String(provider.clone()))
        .map_err(|_| format!("Boutique inconnue : {provider}"))?;
    let (existing, games, credentials) = {
        let library = lock(&state.library);
        (
            library.provider_library.games.clone(),
            library.games.clone(),
            credentials::load(&config::credentials_path()).unwrap_or_default(),
        )
    };
    let result = tauri::async_runtime::spawn_blocking(move || {
        sync_provider(provider, &existing, &games, &credentials)
    })
    .await
    .map_err(|error| error.to_string())?;
    let (merged, warning) = result?;
    {
        let mut library = lock(&state.library);
        library.provider_library.games = merged;
        persist(&state, &mut library)?;
    }
    Ok(providers_dto(&state, warning))
}

#[tauri::command]
pub async fn provider_login(provider: String, code: String) -> Result<String, String> {
    let provider: Provider = serde_json::from_value(serde_json::Value::String(provider.clone()))
        .map_err(|_| format!("Boutique inconnue : {provider}"))?;
    let code = code.trim().to_string();
    if code.is_empty() || code.len() > 4096 || code.chars().any(char::is_whitespace) {
        return Err("Code d'autorisation invalide.".to_string());
    }
    tauri::async_runtime::spawn_blocking(move || match provider {
        Provider::Epic => epic_account::login_with_code(&code).map(|session| {
            if session.display_name.is_empty() {
                "Compte Epic connecté".to_string()
            } else {
                format!("Compte Epic connecté : {}", session.display_name)
            }
        }),
        Provider::Gog => gog::GogAdapter::new(gog::auth_config_path())
            .login_with_code(&code)
            .map(|()| "Compte GOG connecté".to_string()),
        Provider::Steam | Provider::Itch => {
            Err("Cette boutique utilise un identifiant, pas un code d'autorisation.".to_string())
        }
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub fn provider_disconnect(
    state: State<'_, AppState>,
    provider: String,
) -> Result<ProvidersDto, String> {
    let provider: Provider = serde_json::from_value(serde_json::Value::String(provider.clone()))
        .map_err(|_| format!("Boutique inconnue : {provider}"))?;
    credentials::update(&config::credentials_path(), |credentials| match provider {
        Provider::Steam => {
            credentials.steam_web_api_key = None;
            credentials.steam_id = None;
        }
        Provider::Epic => credentials.epic = None,
        Provider::Gog => credentials.gog = None,
        Provider::Itch => credentials.itch_api_key = None,
    })?;
    {
        let mut library = lock(&state.library);
        if provider == Provider::Itch {
            library
                .provider_library
                .games
                .retain(|game| game.key.provider != Provider::Itch);
            persist(&state, &mut library)?;
        }
    }
    Ok(providers_dto(&state, None))
}

#[tauri::command]
pub fn provider_save_credentials(
    state: State<'_, AppState>,
    provider: String,
    key: Option<String>,
    account: Option<String>,
) -> Result<ProvidersDto, String> {
    let provider: Provider = serde_json::from_value(serde_json::Value::String(provider.clone()))
        .map_err(|_| format!("Boutique inconnue : {provider}"))?;
    let key = key.unwrap_or_default().trim().to_string();
    let account = account.unwrap_or_default().trim().to_string();
    if provider == Provider::Steam {
        if !account.is_empty() && !steam::valid_steam_id(&account) {
            return Err("SteamID64 invalide : 17 chiffres commençant par 7656119.".to_string());
        }
        if key.len() > 512 {
            return Err("Clé Steam Web API invalide.".to_string());
        }
    }
    if provider == Provider::Itch && (key.is_empty() || key.len() > 512) {
        return Err("Clé API itch.io invalide.".to_string());
    }
    credentials::update(&config::credentials_path(), |credentials| match provider {
        Provider::Steam => {
            if !key.is_empty() {
                credentials.steam_web_api_key = Some(key);
            }
            if !account.is_empty() {
                credentials.steam_id = Some(account);
            }
        }
        Provider::Itch => credentials.itch_api_key = Some(key),
        Provider::Epic | Provider::Gog => {}
    })?;
    Ok(providers_dto(&state, None))
}

#[tauri::command]
pub async fn provider_tool_install(
    state: State<'_, AppState>,
    tool: String,
) -> Result<ProvidersDto, String> {
    let managed = match tool.to_ascii_lowercase().as_str() {
        "legendary" => tools::ManagedToolId::Legendary,
        "gogdl" => tools::ManagedToolId::Gogdl,
        "ludusavi" => tools::ManagedToolId::Ludusavi,
        _ => return Err(format!("Outil inconnu : {tool}")),
    };
    let result = tauri::async_runtime::spawn_blocking(move || {
        let client = quark::artwork::http();
        let release = tools::latest_release(&client, managed)?;
        let asset = release.asset(managed)?;
        let dir = config::data_dir().join("tools");
        tools::install_verified(&client, managed, asset, &dir).map(|_| ())
    })
    .await
    .map_err(|error| error.to_string())?;
    result?;
    Ok(providers_dto(&state, None))
}

fn install_destination(settings: &Settings) -> PathBuf {
    settings
        .default_install_dir
        .as_deref()
        .map(config::expand_tilde)
        .filter(|path| !path.trim().is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| config::home_dir().join("Games"))
}

fn update_provider_record(
    state: &AppState,
    key: &ProviderGameKey,
    record: ProviderGame,
) -> Result<(), String> {
    let mut library = lock(&state.library);
    if let Some(existing) = library
        .provider_library
        .games
        .iter_mut()
        .find(|game| &game.key == key)
    {
        *existing = record;
    } else {
        library.provider_library.games.push(record);
    }
    persist(state, &mut library)
}

#[tauri::command]
pub async fn provider_install(
    app: AppHandle,
    state: State<'_, AppState>,
    provider: String,
    product_id: String,
    destination: Option<String>,
) -> Result<ProvidersDto, String> {
    let key = provider_key(&provider, &product_id)?;
    let (record, settings, credentials) = {
        let library = lock(&state.library);
        let record = library
            .provider_library
            .games
            .iter()
            .find(|game| game.key == key)
            .cloned()
            .ok_or_else(|| "Jeu de boutique introuvable.".to_string())?;
        (
            record,
            library.settings.clone(),
            credentials::load(&config::credentials_path()).unwrap_or_default(),
        )
    };
    let destination = destination
        .as_deref()
        .map(config::expand_tilde)
        .filter(|path| !path.trim().is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| install_destination(&settings));

    if key.provider == Provider::Steam {
        let app_id = product_id
            .parse::<u32>()
            .map_err(|_| "AppID Steam invalide.".to_string())?;
        steam::request_install(app_id)?;
        return Ok(providers_dto(
            &state,
            Some("Steam a été ouvert pour installer le jeu.".to_string()),
        ));
    }

    {
        let mut library = lock(&state.library);
        if let Some(current) = library
            .provider_library
            .games
            .iter_mut()
            .find(|game| game.key == key)
        {
            current.install_state = InstallState::Installing;
        }
        persist(&state, &mut library)?;
    }
    let _ = app.emit(
        "quark://provider-install",
        serde_json::json!({ "provider": provider, "productId": product_id, "state": "installing" }),
    );

    let key_for_job = key.clone();
    let record_for_job = record.clone();
    let destination_for_job = destination.clone();
    let result = tauri::async_runtime::spawn_blocking(move || -> Result<ProviderGame, String> {
        std::fs::create_dir_all(&destination_for_job)
            .map_err(|error| format!("Dossier d'installation : {error}"))?;
        match key_for_job.provider {
            Provider::Epic => {
                let adapter = legendary::LegendaryAdapter::new();
                if !adapter.executable.is_file() {
                    return Err("Legendary n'est pas installé.".to_string());
                }
                let progress = Arc::new(Mutex::new(providers::runner::InstallProgress::default()));
                adapter.run_install_streaming(
                    legendary::LegendaryAdapter::install_args(
                        &key_for_job.product_id,
                        &destination_for_job,
                    ),
                    Some(Arc::new(AtomicBool::new(false))),
                    progress,
                )?;
                let installed = adapter.sync_installed(config::now_ts(), None)?;
                let found = installed
                    .into_iter()
                    .find(|game| game.key == key_for_job)
                    .ok_or_else(|| {
                        "Legendary a terminé mais le jeu n'apparaît pas installé.".to_string()
                    })?;
                Ok(found)
            }
            Provider::Gog => {
                let adapter = gog::GogAdapter::new(gog::auth_config_path());
                if !adapter.executable.is_file() {
                    return Err("gogdl n'est pas installé.".to_string());
                }
                let info = adapter.run(
                    gog::GogAdapter::info_args(&key_for_job.product_id, "windows"),
                    None,
                )?;
                if info.exit_code != Some(0) {
                    return Err("gogdl n'a pas pu lire les informations du jeu.".to_string());
                }
                let folder = gog::folder_from_info(&info.stdout)?;
                let progress = Arc::new(Mutex::new(providers::runner::InstallProgress::default()));
                adapter.run_install_streaming(
                    gog::GogAdapter::install_args(
                        &key_for_job.product_id,
                        &destination_for_job,
                        "windows",
                    ),
                    Some(Arc::new(AtomicBool::new(false))),
                    progress,
                )?;
                let install_path = gog::resolve_install_path(
                    &destination_for_job,
                    &folder,
                    &key_for_job.product_id,
                )
                .ok_or_else(|| {
                    "gogdl a terminé sans dossier d'installation complet.".to_string()
                })?;
                let executable = gog::find_executable(&install_path, &record_for_job.title);
                Ok(ProviderGame {
                    key: key_for_job.clone(),
                    title: record_for_job.title.clone(),
                    owned: true,
                    install_root: Some(destination_for_job.clone()),
                    install_path: Some(install_path),
                    install_exe: executable,
                    install_state: InstallState::Installed,
                    cover_url: record_for_job.cover_url.clone(),
                    quark_game_id: record_for_job.quark_game_id.clone(),
                    custom_save_paths: record_for_job.custom_save_paths.clone(),
                    capabilities: record_for_job.capabilities.clone(),
                    last_synced: Some(config::now_ts()),
                })
            }
            Provider::Itch => {
                let api_key = credentials
                    .itch_api_key
                    .as_deref()
                    .ok_or_else(|| "Clé API itch.io absente.".to_string())?;
                let progress = Arc::new(Mutex::new(providers::runner::InstallProgress::default()));
                itch::install(
                    api_key,
                    &record_for_job,
                    &destination_for_job,
                    progress,
                    Arc::new(AtomicBool::new(false)),
                )
            }
            Provider::Steam => Err("Steam gère l'installation.".to_string()),
        }
    })
    .await
    .map_err(|error| error.to_string())?;

    match result {
        Ok(installed) => {
            update_provider_record(&state, &key, installed)?;
            let _ = app.emit(
                "quark://provider-install",
                serde_json::json!({ "provider": provider, "productId": product_id, "state": "installed" }),
            );
            Ok(providers_dto(&state, None))
        }
        Err(error) => {
            let mut library = lock(&state.library);
            if let Some(current) = library
                .provider_library
                .games
                .iter_mut()
                .find(|game| game.key == key)
            {
                current.install_state = InstallState::Failed;
            }
            let _ = persist(&state, &mut library);
            let _ = app.emit(
                "quark://provider-install",
                serde_json::json!({ "provider": provider, "productId": product_id, "state": "failed", "error": error }),
            );
            Err(error)
        }
    }
}

#[tauri::command]
pub fn provider_uninstall(
    state: State<'_, AppState>,
    provider: String,
    product_id: String,
) -> Result<ProvidersDto, String> {
    let key = provider_key(&provider, &product_id)?;
    let record = {
        let library = lock(&state.library);
        library
            .provider_library
            .games
            .iter()
            .find(|game| game.key == key)
            .cloned()
            .ok_or_else(|| "Jeu de boutique introuvable.".to_string())?
    };
    let path = record
        .install_path
        .as_ref()
        .ok_or_else(|| "Dossier d'installation inconnu.".to_string())?;
    let root = record
        .install_root
        .as_ref()
        .ok_or_else(|| "Racine d'installation inconnue.".to_string())?;
    providers::operations::remove_install_directory(path, root)?;
    let mut updated = record;
    updated.install_state = InstallState::NotInstalled;
    updated.install_path = None;
    updated.install_exe = None;
    update_provider_record(&state, &key, updated)?;
    Ok(providers_dto(&state, None))
}

fn link_existing_game(
    library: &mut quark::config::Library,
    record: &ProviderGame,
) -> Option<String> {
    if let Some(id) = record
        .quark_game_id
        .as_deref()
        .filter(|id| library.get(id).is_some())
    {
        return Some(id.to_string());
    }
    let normalized = record.title.trim().to_lowercase();
    let steam_linked = library
        .provider_library
        .games
        .iter()
        .find(|candidate| {
            candidate.key.provider == Provider::Steam
                && candidate.title.trim().to_lowercase() == normalized
        })
        .and_then(|candidate| candidate.quark_game_id.clone())
        .filter(|id| library.get(id).is_some());
    steam_linked.or_else(|| {
        library
            .games
            .iter()
            .find(|game| {
                game.name.trim().to_lowercase() == normalized
                    && (game.exe.trim().is_empty()
                        || record
                            .install_path
                            .as_ref()
                            .is_some_and(|path| game.exe_path().starts_with(path)))
            })
            .map(|game| game.id.clone())
    })
}

#[tauri::command]
pub fn provider_adopt(
    state: State<'_, AppState>,
    provider: String,
    product_id: String,
) -> Result<GameDto, String> {
    let key = provider_key(&provider, &product_id)?;
    let (record, settings) = {
        let library = lock(&state.library);
        (
            library
                .provider_library
                .games
                .iter()
                .find(|game| game.key == key)
                .cloned()
                .ok_or_else(|| "Jeu de boutique introuvable.".to_string())?,
            library.settings.clone(),
        )
    };
    if record.install_state != InstallState::Installed {
        return Err("Ce jeu n'est pas installé.".to_string());
    }
    let mut library = lock(&state.library);
    let existing = link_existing_game(&mut library, &record);
    let id = if let Some(id) = existing {
        if let Some(game) = library.get_mut(&id) {
            if game.exe.trim().is_empty() {
                if let Some(exe) = &record.install_exe {
                    game.exe = exe.display().to_string();
                }
            }
            if game
                .artwork
                .get(AssetKind::Cover)
                .is_none_or(|path| !path.exists())
            {
                if let Some(cover) = providers::covers::cached(&key) {
                    game.artwork.set(AssetKind::Cover, cover);
                }
            }
        }
        id
    } else {
        let mut game = Game::new(
            record.title.clone(),
            record
                .install_exe
                .as_ref()
                .map(|path| path.display().to_string())
                .unwrap_or_default(),
        );
        game.args = settings.default_args.trim().to_string();
        game.prefix = settings
            .default_prefix
            .clone()
            .filter(|prefix| !prefix.trim().is_empty());
        game.anticheat = settings.default_anticheat;
        game.launch = settings.default_launch.clone();
        game.proton_log = settings.default_proton_log;
        game.umu = settings.default_umu;
        game.env = Vec::new();
        if let Some(cover) = providers::covers::cached(&key) {
            game.artwork.set(AssetKind::Cover, cover);
        }
        let id = game.id.clone();
        library.games.push(game);
        id
    };
    if let Some(record) = library
        .provider_library
        .games
        .iter_mut()
        .find(|game| game.key == key)
    {
        record.quark_game_id = Some(id.clone());
    }
    library.migrate_installations();
    library.activate_provider_installation(&id, &key);
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

#[tauri::command]
pub async fn steam_candidates(
    state: State<'_, AppState>,
) -> Result<Vec<SteamCandidateDto>, String> {
    let owned: Vec<u32> = lock(&state.library)
        .games
        .iter()
        .filter_map(|game| game.steam_app_id)
        .collect();
    tauri::async_runtime::spawn_blocking(move || {
        steamlib::installed().map(|games| {
            games
                .into_iter()
                .map(|game| SteamCandidateDto {
                    app_id: game.app_id,
                    name: game.name,
                    dir: game.dir.display().to_string(),
                    exe: game.exe.map(|path| path.display().to_string()),
                    owned: owned.contains(&game.app_id),
                })
                .collect()
        })
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub fn steam_import(state: State<'_, AppState>, app_ids: Vec<u32>) -> Result<Vec<GameDto>, String> {
    let games = tauri::async_runtime::block_on(async { steamlib::installed() })?;
    let mut library = lock(&state.library);
    let settings = library.settings.clone();
    let existing: Vec<u32> = library
        .games
        .iter()
        .filter_map(|game| game.steam_app_id)
        .collect();
    let mut added = Vec::new();
    for candidate in games {
        if !app_ids.contains(&candidate.app_id) || existing.contains(&candidate.app_id) {
            continue;
        }
        let mut game = Game::new(
            candidate.name,
            candidate
                .exe
                .as_ref()
                .map(|path| path.display().to_string())
                .unwrap_or_default(),
        );
        game.steam_app_id = Some(candidate.app_id);
        game.args = settings.default_args.trim().to_string();
        game.prefix = settings
            .default_prefix
            .clone()
            .filter(|prefix| !prefix.trim().is_empty());
        game.anticheat = settings.default_anticheat;
        game.launch = settings.default_launch.clone();
        game.proton_log = settings.default_proton_log;
        game.umu = settings.default_umu;
        game.env = settings.default_env.clone();
        added.push(game);
    }
    library.games.extend(added.iter().cloned());
    persist(&state, &mut library)?;
    let sessions = lock(&state.sessions);
    Ok(added
        .iter()
        .map(|game| {
            let run = sessions.get(&game.id);
            game_dto(
                game,
                &settings,
                run.is_some(),
                run.map(|run| run.label.clone()),
            )
        })
        .collect())
}
