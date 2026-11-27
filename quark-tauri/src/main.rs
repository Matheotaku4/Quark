//! Interface Tauri de Quark.
//!
//! Le cœur métier (configuration, Proton, sessions, boutiques) est celui de
//! `quark` : cette couche ne fait que l'exposer à une interface web locale.
//! Aucun réseau n'est nécessaire au rendu ; les images d'artwork passent par
//! le protocole local `quarkimg://`.

mod extras;
mod features;
mod gamepad;
mod providers;
mod screenshot;
mod store;
mod themes;
mod validators;

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;
use quark::config::{
    self, AssetKind, Game, GameStatus, Library, PlaySession, ProtonMode, Settings, ThemeMode,
    UiLanguage,
};
use quark::proton;
use quark::providers as core_providers;
use quark::session::{self, Session};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};
use std::time::Duration;
use tauri::http::{header, Response, StatusCode};
use tauri::{AppHandle, Emitter, Manager, RunEvent, State};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};
use themes::THEMES;

// ---------------------------------------------------------------------------
// État partagé
// ---------------------------------------------------------------------------

struct Running {
    session: Session,
    started_ts: i64,
    label: String,
    name: String,
}

pub(crate) struct InstallJob {
    pub cancel: Arc<AtomicBool>,
}

struct AppState {
    library: Mutex<Library>,
    sessions: Mutex<HashMap<String, Running>>,
    installs: Mutex<HashMap<String, InstallJob>>,
    demo: bool,
    deck_demo: bool,
    demo_root: Option<PathBuf>,
    quitting: AtomicBool,
    tray: Mutex<Option<ksni::blocking::Handle<QuarkTray>>>,
    tray_ready: AtomicBool,
    tray_visible: Arc<AtomicBool>,
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poison| poison.into_inner())
}

fn tray_available() -> bool {
    std::env::var_os("DBUS_SESSION_BUS_ADDRESS").is_some()
}

fn tray_labels(language: UiLanguage) -> (&'static str, &'static str) {
    match language {
        UiLanguage::French => ("Ouvrir Quark", "Fermer Quark"),
        UiLanguage::English => ("Open Quark", "Close Quark"),
        UiLanguage::Spanish => ("Abrir Quark", "Cerrar Quark"),
        UiLanguage::German => ("Quark öffnen", "Quark schließen"),
        UiLanguage::Portuguese => ("Abrir o Quark", "Fechar o Quark"),
        UiLanguage::Russian => ("Открыть Quark", "Закрыть Quark"),
    }
}

struct QuarkTray {
    app: AppHandle,
    visible: Arc<AtomicBool>,
}

impl ksni::Tray for QuarkTray {
    fn id(&self) -> String {
        "quark".to_string()
    }

    fn title(&self) -> String {
        "Quark".to_string()
    }

    fn status(&self) -> ksni::Status {
        if self.visible.load(Ordering::Relaxed) {
            ksni::Status::Active
        } else {
            ksni::Status::Passive
        }
    }

    fn icon_pixmap(&self) -> Vec<ksni::Icon> {
        let rgba = include_bytes!("../icons/icon.rgba");
        let mut data = Vec::with_capacity(rgba.len());
        for pixel in rgba.as_chunks::<4>().0 {
            data.extend_from_slice(&[pixel[3], pixel[0], pixel[1], pixel[2]]);
        }
        vec![ksni::Icon {
            width: 128,
            height: 128,
            data,
        }]
    }

    fn activate(&mut self, _x: i32, _y: i32) {
        let app = self.app.clone();
        run_on_main(&self.app, move || toggle_main_window(&app));
    }

    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        use ksni::menu::StandardItem;
        let (open, quit) = tray_labels(
            lock(&self.app.state::<AppState>().library)
                .settings
                .ui_language,
        );
        vec![
            StandardItem {
                label: open.to_string(),
                icon_name: "window-new".to_string(),
                activate: Box::new(|this: &mut Self| {
                    let app = this.app.clone();
                    run_on_main(&this.app, move || {
                        let _ = show_main_window(&app);
                    });
                }),
                ..Default::default()
            }
            .into(),
            ksni::MenuItem::Separator,
            StandardItem {
                label: quit.to_string(),
                icon_name: "application-exit".to_string(),
                activate: Box::new(|this: &mut Self| {
                    let app = this.app.clone();
                    run_on_main(&this.app, move || {
                        app.state::<AppState>()
                            .quitting
                            .store(true, Ordering::Relaxed);
                        app.exit(0);
                    });
                }),
                ..Default::default()
            }
            .into(),
        ]
    }
}

fn run_on_main(app: &AppHandle, action: impl FnOnce() + Send + 'static) {
    if let Err(error) = app.run_on_main_thread(action) {
        eprintln!("Action de la fenêtre indisponible : {error}");
    }
}

fn spawn_tray(app: &AppHandle) -> Result<(), String> {
    let state = app.state::<AppState>();
    if lock(&state.tray).is_some() {
        return Ok(());
    }
    let tray = QuarkTray {
        app: app.clone(),
        visible: state.tray_visible.clone(),
    };
    let handle = ksni::blocking::TrayMethods::spawn(tray)
        .map_err(|error| format!("Icône de notification indisponible : {error}"))?;
    *lock(&state.tray) = Some(handle);
    state.tray_ready.store(true, Ordering::Relaxed);
    Ok(())
}

/// Notification bureau via le service XDG. La fenêtre au premier plan est
/// ignorée : l'utilisateur voit déjà le résultat dans Quark.
fn send_notification(title: &str, body: &str) {
    let _ = std::process::Command::new("notify-send")
        .args(["--app-name=Quark", "--urgency=normal", title, body])
        .spawn();
}

fn notify_desktop(app: &AppHandle, title: &str, body: &str) {
    let focused = app
        .get_webview_window("main")
        .and_then(|window| window.is_focused().ok())
        .unwrap_or(false);
    if !focused {
        send_notification(title, body);
    }
}

// ---------------------------------------------------------------------------
// Lancement depuis le terminal, sans interface
// ---------------------------------------------------------------------------

fn title_key(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .filter(|character| character.is_alphanumeric())
        .collect()
}

fn cli_find_game<'a>(library: &'a Library, query: &str) -> Result<&'a Game, String> {
    let wanted = title_key(query);
    if wanted.is_empty() {
        return Err("Indiquez le nom d'un jeu.".to_string());
    }
    let exact: Vec<&Game> = library
        .games
        .iter()
        .filter(|game| title_key(&game.name) == wanted)
        .collect();
    if let [game] = exact.as_slice() {
        return Ok(game);
    }
    let matches: Vec<&Game> = library
        .games
        .iter()
        .filter(|game| title_key(&game.name).contains(&wanted))
        .collect();
    match matches.as_slice() {
        [game] => Ok(game),
        [] => Err(format!("Aucun jeu ne correspond à « {query} ».")),
        many => {
            let names = many
                .iter()
                .map(|game| format!("  - {}", game.name))
                .collect::<Vec<_>>()
                .join("\n");
            Err(format!(
                "Plusieurs jeux correspondent à « {query} » :\n{names}"
            ))
        }
    }
}

fn process_alive(pid: i32) -> bool {
    if pid <= 0 {
        return false;
    }
    let Ok(stat) = std::fs::read_to_string(format!("/proc/{pid}/stat")) else {
        return false;
    };
    // Un enfant adopté par init reste visible en état zombie : il est terminé.
    // Le nom du processus peut contenir des espaces, d'où la lecture après « ) ».
    stat.rsplit_once(')')
        .is_none_or(|(_, rest)| !rest.trim_start().starts_with('Z'))
}

fn cli_record_session(id: &str, started: i64, pid: i32) -> Result<(), String> {
    while process_alive(pid) {
        std::thread::sleep(Duration::from_secs(2));
    }
    let mut library = Library::load();
    let settings = library.settings.clone();
    let secs = (config::now_ts() - started).max(1) as u64;
    let name = library
        .get(id)
        .map(|game| game.name.clone())
        .unwrap_or_else(|| "Quark".to_string());
    if let Some(game) = library.get_mut(id) {
        game.record_session(started, secs);
    }
    library.save()?;
    if settings.notify_session_done {
        let duration = if secs >= 3600 {
            format!("{} h {:02}", secs / 3600, (secs % 3600) / 60)
        } else {
            format!("{} min", secs / 60)
        };
        send_notification(
            &quark::i18n::tr("Partie terminée"),
            &format!("{name} · {duration}"),
        );
    }
    Ok(())
}

fn cli_launch(query: &str) -> Result<(), String> {
    let library = Library::load();
    let game = cli_find_game(&library, query)?.clone();
    let settings = library.settings.clone();
    drop(library);
    let mut game = game;
    game.normalize_installations();
    if game.exe_missing() {
        return Err(format!(
            "{} : exécutable introuvable ({}).",
            game.name,
            if game.exe.trim().is_empty() {
                "aucun chemin défini".to_string()
            } else {
                game.exe.clone()
            }
        ));
    }
    let resolved = proton::resolve_with(
        &game,
        &settings,
        &proton::detect_runtimes(),
        proton::Tools::detect(),
    )?;
    if let Some(prefix) = &resolved.prefix {
        std::fs::create_dir_all(prefix).map_err(|error| format!("Préfixe Proton : {error}"))?;
    }
    let diagnostics =
        core_providers::connection_diagnostics(None, Some(&game.wine_prefix_path(&settings)));
    let session = session::start(&game, &resolved, &diagnostics)?;
    let pid = session.pid;
    let started = session.started_ts;
    let executable = std::env::current_exe()
        .map_err(|error| format!("Binaire courant introuvable : {error}"))?;
    std::process::Command::new(executable)
        .arg("--record-session")
        .arg(&game.id)
        .arg(started.to_string())
        .arg(pid.to_string())
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|error| format!("Suivi de session impossible : {error}"))?;
    println!("Lancement de {}…", game.name);
    Ok(())
}

fn cli_list() -> Result<(), String> {
    let library = Library::load();
    let mut names: Vec<&str> = library
        .games
        .iter()
        .map(|game| game.name.as_str())
        .collect();
    names.sort_by_key(|name| name.to_lowercase());
    for name in names {
        println!("{name}");
    }
    Ok(())
}

#[cfg(test)]
mod cli_tests {
    use super::*;

    fn library(names: &[&str]) -> Library {
        Library {
            games: names
                .iter()
                .map(|name| Game {
                    id: name.to_lowercase().replace(' ', "-"),
                    name: (*name).to_string(),
                    ..Game::default()
                })
                .collect(),
            ..Library::default()
        }
    }

    #[test]
    fn un_titre_est_reconnu_sans_la_ponctuation() {
        let library = library(&["Half-Life 2"]);
        assert_eq!(
            cli_find_game(&library, "half life 2").unwrap().id,
            "half-life-2"
        );
    }

    #[test]
    fn un_titre_ambigu_liste_les_candidats() {
        let library = library(&["Portal", "Portal 2"]);
        let error = cli_find_game(&library, "port").unwrap_err();
        assert!(error.contains("Portal"));
        assert!(error.contains("Portal 2"));
    }
}

fn cli_main(args: &[String]) -> Option<i32> {
    match args.first().map(String::as_str) {
        Some("launch") => {
            let query = args[1..].join(" ");
            match cli_launch(&query) {
                Ok(()) => Some(0),
                Err(error) => {
                    eprintln!("{error}");
                    Some(1)
                }
            }
        }
        Some("list") => match cli_list() {
            Ok(()) => Some(0),
            Err(error) => {
                eprintln!("{error}");
                Some(1)
            }
        },
        Some("--record-session") if args.len() >= 4 => {
            let id = &args[1];
            let started = args[2].parse::<i64>().unwrap_or_else(|_| config::now_ts());
            let pid = args[3].parse::<i32>().unwrap_or(0);
            match cli_record_session(id, started, pid) {
                Ok(()) => Some(0),
                Err(error) => {
                    eprintln!("{error}");
                    Some(1)
                }
            }
        }
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Objets envoyés à l'interface
// ---------------------------------------------------------------------------

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GameDto {
    id: String,
    name: String,
    exe: String,
    args: String,
    status: String,
    status_color: String,
    favorite: bool,
    tags: Vec<String>,
    developer: String,
    genres: Vec<String>,
    released: String,
    description: String,
    notes: String,
    playtime_secs: u64,
    last_played: Option<i64>,
    added_at: i64,
    steam_app_id: Option<u32>,
    provider: String,
    source_label: String,
    is_native: bool,
    exe_missing: bool,
    installed: bool,
    installations: Vec<InstallationDto>,
    cover_url: Option<String>,
    hero_url: Option<String>,
    logo_url: Option<String>,
    icon_url: Option<String>,
    monogram: String,
    hue: u16,
    proton: String,
    umu: bool,
    anticheat: String,
    prefix: String,
    launch: LaunchDto,
    env: Vec<EnvDto>,
    sessions: Vec<SessionDto>,
    running: bool,
    running_label: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct InstallationDto {
    id: String,
    label: String,
    source: String,
    exe: String,
    path: Option<String>,
    installed: bool,
    active: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LaunchDto {
    gamemode: bool,
    mangohud: bool,
    gamescope: bool,
    gamescope_width: u32,
    gamescope_height: u32,
    gamescope_fsr: bool,
    gamescope_fps: u32,
    prime: bool,
    wrapper: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EnvDto {
    name: String,
    value: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SessionDto {
    start: i64,
    secs: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SettingsDto {
    theme: String,
    ui_language: String,
    compact: bool,
    start_page: String,
    view: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ThemeDto {
    id: &'static str,
    label: &'static str,
    vars: ThemeVars,
}

#[derive(Serialize)]
struct ThemeVars {
    bg: &'static str,
    panel: &'static str,
    surface: &'static str,
    #[serde(rename = "surface-hover")]
    surface_hover: &'static str,
    border: &'static str,
    #[serde(rename = "row-selected")]
    row_selected: &'static str,
    text: &'static str,
    #[serde(rename = "text-dim")]
    text_dim: &'static str,
    #[serde(rename = "text-faint")]
    text_faint: &'static str,
    accent: &'static str,
    #[serde(rename = "accent-hover")]
    accent_hover: &'static str,
    #[serde(rename = "accent-pressed")]
    accent_pressed: &'static str,
    danger: &'static str,
    header: &'static str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct StatusDto {
    id: String,
    color: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LanguageDto {
    id: String,
    label: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RunningDto {
    id: String,
    name: String,
    label: String,
    elapsed: u64,
    stopping: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Bootstrap {
    games: Vec<GameDto>,
    settings: SettingsDto,
    themes: Vec<ThemeDto>,
    statuses: Vec<StatusDto>,
    languages: Vec<LanguageDto>,
    running: Vec<RunningDto>,
    config_path: String,
    data_dir: String,
    version: String,
    memory_mb: Option<u64>,
    demo: bool,
    deck: bool,
    load_error: Option<String>,
}

// ---------------------------------------------------------------------------
// Conversions
// ---------------------------------------------------------------------------

fn snake<T: Serialize>(value: &T) -> String {
    serde_json::to_value(value)
        .ok()
        .and_then(|value| value.as_str().map(str::to_owned))
        .unwrap_or_default()
}

fn hue_for(name: &str) -> u16 {
    let mut hash: u32 = 2_166_136_261;
    for byte in name.bytes() {
        hash ^= u32::from(byte);
        hash = hash.wrapping_mul(16_777_619);
    }
    (hash % 360) as u16
}

fn asset_url(path: Option<&PathBuf>) -> Option<String> {
    let path = path?;
    if !path.is_file() {
        return None;
    }
    let encoded = URL_SAFE_NO_PAD.encode(path.to_string_lossy().as_bytes());
    Some(format!("quarkimg://localhost/{encoded}"))
}

fn proton_label(mode: &ProtonMode) -> String {
    match mode {
        ProtonMode::Default => "Auto".to_string(),
        ProtonMode::Native => "Natif".to_string(),
        ProtonMode::Runtime(key) => key.clone(),
    }
}

fn game_dto(
    game: &Game,
    settings: &Settings,
    running: bool,
    running_label: Option<String>,
) -> GameDto {
    let active = game.active_installation();
    let provider = active
        .and_then(|installation| installation.provider)
        .map(|provider| snake(&provider))
        .unwrap_or_else(|| "local".to_string());
    let source_label = active
        .map(|installation| installation.source_label().to_string())
        .unwrap_or_else(|| "Locale".to_string());
    let status_color = game.status.color();
    let installations = game
        .installations
        .iter()
        .map(|installation| InstallationDto {
            id: installation.id.clone(),
            label: installation.label.clone(),
            source: installation.source_label().to_string(),
            exe: installation.exe.clone(),
            path: installation.path.clone(),
            installed: installation.installed,
            active: active.is_some_and(|current| current.id == installation.id),
        })
        .collect::<Vec<_>>();
    let installed = game
        .installations
        .iter()
        .any(|installation| installation.installed)
        || !game.exe_missing();
    let mut sessions = game.sessions.clone();
    sessions.sort_by_key(|session| std::cmp::Reverse(session.start));
    let sessions = sessions
        .iter()
        .take(120)
        .map(|session| SessionDto {
            start: session.start,
            secs: session.secs,
        })
        .collect::<Vec<_>>();
    GameDto {
        id: game.id.clone(),
        name: game.name.clone(),
        exe: game.exe.clone(),
        args: game.args.clone(),
        status: snake(&game.status),
        status_color: format!(
            "#{:02x}{:02x}{:02x}",
            status_color.0, status_color.1, status_color.2
        ),
        favorite: game.favorite,
        tags: game.tags.clone(),
        developer: game.developer.clone(),
        genres: game.genres.clone(),
        released: game.released.clone(),
        description: game.description.clone(),
        notes: game.notes.clone(),
        playtime_secs: game.playtime_secs,
        last_played: game.last_played,
        added_at: game.added_at,
        steam_app_id: game.steam_app_id,
        provider,
        source_label,
        is_native: game.is_native(),
        exe_missing: game.exe_missing(),
        installed,
        installations,
        cover_url: asset_url(game.artwork.cover.as_ref()),
        hero_url: asset_url(game.artwork.hero.as_ref()),
        logo_url: asset_url(game.artwork.logo.as_ref()),
        icon_url: asset_url(game.artwork.icon.as_ref()),
        monogram: game.monogram_letter(),
        hue: hue_for(&game.name),
        proton: proton_label(&game.proton),
        umu: game.umu,
        anticheat: snake(&game.anticheat),
        prefix: game.prefix_path(settings).display().to_string(),
        launch: LaunchDto {
            gamemode: game.launch.gamemode,
            mangohud: game.launch.mangohud,
            gamescope: game.launch.gamescope,
            gamescope_width: game.launch.gamescope_width,
            gamescope_height: game.launch.gamescope_height,
            gamescope_fsr: game.launch.gamescope_fsr,
            gamescope_fps: game.launch.gamescope_fps,
            prime: game.launch.prime,
            wrapper: game.launch.wrapper.clone(),
        },
        env: game
            .env
            .iter()
            .map(|variable| EnvDto {
                name: variable.name.clone(),
                value: variable.value.clone(),
            })
            .collect(),
        sessions,
        running,
        running_label,
    }
}

fn settings_dto(settings: &Settings) -> SettingsDto {
    SettingsDto {
        theme: snake(&settings.theme),
        ui_language: snake(&settings.ui_language),
        compact: settings.ui_compact,
        start_page: snake(&settings.start_page),
        view: snake(&settings.view),
    }
}

fn theme_dtos() -> Vec<ThemeDto> {
    THEMES
        .iter()
        .map(|theme| ThemeDto {
            id: theme.id,
            label: theme.label,
            vars: ThemeVars {
                bg: theme.bg,
                panel: theme.panel,
                surface: theme.surface,
                surface_hover: theme.surface_hover,
                border: theme.border,
                row_selected: theme.row_selected,
                text: theme.text,
                text_dim: theme.text_dim,
                text_faint: theme.text_faint,
                accent: theme.accent,
                accent_hover: theme.accent_hover,
                accent_pressed: theme.accent_pressed,
                danger: theme.danger,
                header: theme.header,
            },
        })
        .collect()
}

fn running_dtos(state: &AppState) -> Vec<RunningDto> {
    lock(&state.sessions)
        .iter()
        .map(|(id, run)| RunningDto {
            id: id.clone(),
            name: run.name.clone(),
            label: run.label.clone(),
            elapsed: run.session.started.elapsed().as_secs(),
            stopping: run.session.stopping.is_some(),
        })
        .collect()
}

fn bootstrap(state: &AppState) -> Bootstrap {
    let library = lock(&state.library);
    let settings = library.settings.clone();
    let running = running_dtos(state);
    let running_map: HashMap<&str, &RunningDto> =
        running.iter().map(|run| (run.id.as_str(), run)).collect();
    let games = library
        .games
        .iter()
        .map(|game| {
            let run = running_map.get(game.id.as_str());
            game_dto(
                game,
                &settings,
                run.is_some(),
                run.map(|run| run.label.clone()),
            )
        })
        .collect();
    Bootstrap {
        games,
        settings: settings_dto(&settings),
        themes: theme_dtos(),
        statuses: GameStatus::ALL
            .iter()
            .map(|status| {
                let color = status.color();
                StatusDto {
                    id: snake(status),
                    color: format!("#{:02x}{:02x}{:02x}", color.0, color.1, color.2),
                }
            })
            .collect(),
        languages: UiLanguage::ALL
            .iter()
            .map(|language| LanguageDto {
                id: snake(language),
                label: language.label(),
            })
            .collect(),
        running,
        config_path: config::config_dir().display().to_string(),
        data_dir: config::data_dir().display().to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        memory_mb: config::resident_mb(),
        demo: state.demo,
        deck: state.deck_demo,
        load_error: library.load_error.clone(),
    }
}

fn persist(state: &AppState, library: &mut Library) -> Result<(), String> {
    if state.demo {
        return Ok(());
    }
    library.save()
}

fn parse_language(value: &str) -> Option<UiLanguage> {
    match value.trim().to_ascii_lowercase().as_str() {
        "french" | "fr" => Some(UiLanguage::French),
        "english" | "en" => Some(UiLanguage::English),
        "spanish" | "es" => Some(UiLanguage::Spanish),
        "german" | "de" => Some(UiLanguage::German),
        "portuguese" | "pt" => Some(UiLanguage::Portuguese),
        "russian" | "ru" => Some(UiLanguage::Russian),
        _ => None,
    }
}

fn parse_theme(value: &str) -> Option<ThemeMode> {
    serde_json::from_value(serde_json::Value::String(value.to_string())).ok()
}

fn parse_status(value: &str) -> Option<GameStatus> {
    serde_json::from_value(serde_json::Value::String(value.to_string())).ok()
}

// ---------------------------------------------------------------------------
// Commandes
// ---------------------------------------------------------------------------

#[tauri::command]
fn bootstrap_data(state: State<'_, AppState>) -> Bootstrap {
    bootstrap(&state)
}

#[tauri::command]
fn running_games(state: State<'_, AppState>) -> Vec<RunningDto> {
    running_dtos(&state)
}

#[tauri::command]
fn get_game(state: State<'_, AppState>, id: String) -> Result<GameDto, String> {
    let library = lock(&state.library);
    let settings = library.settings.clone();
    let game = library
        .get(&id)
        .ok_or_else(|| "Jeu introuvable".to_string())?;
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
fn launch_game(app: AppHandle, state: State<'_, AppState>, id: String) -> Result<GameDto, String> {
    if state.demo {
        return Err("Le lancement est désactivé en mode démonstration.".to_string());
    }
    if lock(&state.sessions).contains_key(&id) {
        return Err("Ce jeu est déjà en cours.".to_string());
    }
    let (mut game, settings) = {
        let library = lock(&state.library);
        (
            library
                .get(&id)
                .cloned()
                .ok_or_else(|| "Jeu introuvable".to_string())?,
            library.settings.clone(),
        )
    };
    game.normalize_installations();
    if game.exe_missing() {
        return Err(format!(
            "{} : exécutable introuvable ({}).",
            game.name,
            if game.exe.trim().is_empty() {
                "aucun chemin défini".to_string()
            } else {
                game.exe.clone()
            }
        ));
    }
    // Redétecte les runtimes à chaque lancement : Steam peut en avoir ajouté
    // ou renommé un depuis le démarrage de Quark.
    let runtimes = proton::detect_runtimes();
    let resolved = proton::resolve_with(&game, &settings, &runtimes, proton::Tools::detect())?;
    for warning in &resolved.warnings {
        let _ = app.emit(
            "quark://notice",
            serde_json::json!({ "message": warning, "error": false }),
        );
    }
    if let Some(prefix) = &resolved.prefix {
        std::fs::create_dir_all(prefix).map_err(|error| format!("Préfixe Proton : {error}"))?;
    }
    let diagnostics =
        core_providers::connection_diagnostics(None, Some(&game.wine_prefix_path(&settings)));
    let session = session::start(&game, &resolved, &diagnostics)?;
    let label = resolved.label();
    lock(&state.sessions).insert(
        id.clone(),
        Running {
            session,
            started_ts: config::now_ts(),
            label: label.clone(),
            name: game.name.clone(),
        },
    );
    let _ = app.emit(
        "quark://session-started",
        serde_json::json!({ "id": id, "label": label }),
    );
    get_game(state, id)
}

#[tauri::command]
fn stop_game(app: AppHandle, state: State<'_, AppState>, id: String) -> Result<RunningDto, String> {
    let mut sessions = lock(&state.sessions);
    let run = sessions
        .get_mut(&id)
        .ok_or_else(|| "Ce jeu n'est pas en cours.".to_string())?;
    session::stop(&mut run.session);
    let dto = RunningDto {
        id: id.clone(),
        name: run.name.clone(),
        label: run.label.clone(),
        elapsed: run.session.started.elapsed().as_secs(),
        stopping: run.session.stopping.is_some(),
    };
    drop(sessions);
    let _ = app.emit("quark://session-stopping", serde_json::json!({ "id": id }));
    Ok(dto)
}

#[tauri::command]
fn set_favorite(state: State<'_, AppState>, id: String, favorite: bool) -> Result<GameDto, String> {
    let mut library = lock(&state.library);
    let settings = library.settings.clone();
    {
        let game = library
            .get_mut(&id)
            .ok_or_else(|| "Jeu introuvable".to_string())?;
        game.favorite = favorite;
    }
    persist(&state, &mut library)?;
    Ok(game_dto(library.get(&id).unwrap(), &settings, false, None))
}

#[tauri::command]
fn set_status(state: State<'_, AppState>, id: String, status: String) -> Result<GameDto, String> {
    let status = parse_status(&status).ok_or_else(|| "Statut inconnu".to_string())?;
    let mut library = lock(&state.library);
    let settings = library.settings.clone();
    {
        let game = library
            .get_mut(&id)
            .ok_or_else(|| "Jeu introuvable".to_string())?;
        game.status = status;
    }
    persist(&state, &mut library)?;
    Ok(game_dto(library.get(&id).unwrap(), &settings, false, None))
}

#[tauri::command]
fn set_notes(state: State<'_, AppState>, id: String, notes: String) -> Result<(), String> {
    let mut library = lock(&state.library);
    {
        let game = library
            .get_mut(&id)
            .ok_or_else(|| "Jeu introuvable".to_string())?;
        game.notes = notes;
    }
    persist(&state, &mut library)
}

#[tauri::command]
fn add_tag(state: State<'_, AppState>, id: String, tag: String) -> Result<GameDto, String> {
    let tag = tag.trim().to_string();
    if tag.is_empty() {
        return Err("Tag vide".to_string());
    }
    let mut library = lock(&state.library);
    let settings = library.settings.clone();
    {
        let game = library
            .get_mut(&id)
            .ok_or_else(|| "Jeu introuvable".to_string())?;
        if !game
            .tags
            .iter()
            .any(|existing| existing.eq_ignore_ascii_case(&tag))
        {
            game.tags.push(tag);
        }
    }
    persist(&state, &mut library)?;
    Ok(game_dto(library.get(&id).unwrap(), &settings, false, None))
}

#[tauri::command]
fn remove_tag(state: State<'_, AppState>, id: String, tag: String) -> Result<GameDto, String> {
    let mut library = lock(&state.library);
    let settings = library.settings.clone();
    {
        let game = library
            .get_mut(&id)
            .ok_or_else(|| "Jeu introuvable".to_string())?;
        game.tags.retain(|existing| existing != &tag);
    }
    persist(&state, &mut library)?;
    Ok(game_dto(library.get(&id).unwrap(), &settings, false, None))
}

#[tauri::command]
fn update_game(
    state: State<'_, AppState>,
    id: String,
    name: Option<String>,
    exe: Option<String>,
    args: Option<String>,
) -> Result<GameDto, String> {
    let mut library = lock(&state.library);
    let settings = library.settings.clone();
    {
        let game = library
            .get_mut(&id)
            .ok_or_else(|| "Jeu introuvable".to_string())?;
        if let Some(name) = name {
            let name = name.trim().to_string();
            if !name.is_empty() {
                game.name = name;
            }
        }
        if let Some(exe) = exe {
            let exe = exe.trim().to_string();
            if !exe.is_empty() {
                game.exe = exe;
            }
        }
        if let Some(args) = args {
            game.args = args;
        }
        game.normalize_installations();
    }
    persist(&state, &mut library)?;
    Ok(game_dto(library.get(&id).unwrap(), &settings, false, None))
}

#[tauri::command]
fn add_game(state: State<'_, AppState>, name: String, exe: String) -> Result<GameDto, String> {
    let exe = exe.trim().to_string();
    if exe.is_empty() {
        return Err("Choisissez un exécutable.".to_string());
    }
    validators::validate_executable(Path::new(&exe))?;
    let name = if name.trim().is_empty() {
        Path::new(&exe)
            .file_stem()
            .map(|stem| stem.to_string_lossy().into_owned())
            .unwrap_or_else(|| "Nouveau jeu".to_string())
    } else {
        name.trim().to_string()
    };
    let mut library = lock(&state.library);
    let settings = library.settings.clone();
    let mut game = Game::new(name, exe);
    game.args = settings.default_args.clone();
    game.proton = settings
        .default_proton
        .clone()
        .map(ProtonMode::Runtime)
        .unwrap_or_default();
    game.prefix = settings.default_prefix.clone();
    game.anticheat = settings.default_anticheat;
    game.launch = settings.default_launch.clone();
    game.umu = settings.default_umu;
    game.proton_log = settings.default_proton_log;
    game.env = settings.default_env.clone();
    game.normalize_installations();
    let id = game.id.clone();
    library.games.push(game);
    persist(&state, &mut library)?;
    Ok(game_dto(library.get(&id).unwrap(), &settings, false, None))
}

#[tauri::command]
fn remove_games(state: State<'_, AppState>, ids: Vec<String>) -> Result<usize, String> {
    let running = lock(&state.sessions);
    let mut library = lock(&state.library);
    let before = library.games.len();
    library
        .games
        .retain(|game| !ids.contains(&game.id) || running.contains_key(&game.id));
    let removed = before - library.games.len();
    persist(&state, &mut library)?;
    Ok(removed)
}

#[tauri::command]
fn activate_installation(
    state: State<'_, AppState>,
    id: String,
    installation_id: String,
) -> Result<GameDto, String> {
    let mut library = lock(&state.library);
    let settings = library.settings.clone();
    {
        let game = library
            .get_mut(&id)
            .ok_or_else(|| "Jeu introuvable".to_string())?;
        if !game.activate_installation(&installation_id) {
            return Err("Installation introuvable.".to_string());
        }
    }
    persist(&state, &mut library)?;
    let game = library
        .get(&id)
        .ok_or_else(|| "Jeu introuvable".to_string())?;
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
fn remove_game(state: State<'_, AppState>, id: String) -> Result<(), String> {
    if lock(&state.sessions).contains_key(&id) {
        return Err("Arrêtez le jeu avant de le retirer.".to_string());
    }
    let mut library = lock(&state.library);
    library.games.retain(|game| game.id != id);
    persist(&state, &mut library)
}

#[tauri::command]
fn open_folder(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let library = lock(&state.library);
    let game = library
        .get(&id)
        .ok_or_else(|| "Jeu introuvable".to_string())?;
    let directory = game
        .exe_path()
        .parent()
        .filter(|path| path.is_dir())
        .map(Path::to_path_buf)
        .or_else(|| {
            game.active_installation()
                .and_then(|installation| installation.path.as_deref())
                .map(PathBuf::from)
                .filter(|path| path.is_dir())
        })
        .ok_or_else(|| "Dossier introuvable.".to_string())?;
    session::open_folder(&directory)
}

#[tauri::command]
fn open_config_folder() -> Result<(), String> {
    session::open_folder(&config::config_dir())
}

#[tauri::command]
fn read_log(state: State<'_, AppState>, id: String) -> Result<String, String> {
    let library = lock(&state.library);
    let game = library
        .get(&id)
        .ok_or_else(|| "Jeu introuvable".to_string())?;
    let (lines, truncated) = session::read_log_tail(&config::session_log(&game.id), 240)?;
    let mut text = String::new();
    if truncated {
        text.push_str("…\n");
    }
    text.push_str(&lines.join("\n"));
    Ok(text)
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
struct SettingsPatch {
    theme: Option<String>,
    ui_language: Option<String>,
    compact: Option<bool>,
    start_page: Option<String>,
    view: Option<String>,
}

#[tauri::command]
fn save_settings(state: State<'_, AppState>, patch: SettingsPatch) -> Result<SettingsDto, String> {
    let mut library = lock(&state.library);
    {
        let settings = &mut library.settings;
        if let Some(theme) = patch.theme {
            settings.theme = parse_theme(&theme).ok_or_else(|| "Thème inconnu".to_string())?;
        }
        if let Some(language) = patch.ui_language {
            let language =
                parse_language(&language).ok_or_else(|| "Langue inconnue".to_string())?;
            settings.ui_language = language;
            quark::i18n::set_language(language);
        }
        if let Some(compact) = patch.compact {
            settings.ui_compact = compact;
        }
        if let Some(start_page) = patch.start_page {
            settings.start_page = serde_json::from_value(serde_json::Value::String(start_page))
                .map_err(|_| "Page de démarrage inconnue".to_string())?;
        }
        if let Some(view) = patch.view {
            settings.view = serde_json::from_value(serde_json::Value::String(view))
                .map_err(|_| "Vue inconnue".to_string())?;
        }
    }
    let dto = settings_dto(&library.settings);
    persist(&state, &mut library)?;
    Ok(dto)
}

#[tauri::command]
fn reload_library(state: State<'_, AppState>) -> Result<Bootstrap, String> {
    if state.demo {
        return Ok(bootstrap(&state));
    }
    let library = Library::load();
    quark::i18n::set_language(library.settings.ui_language);
    *lock(&state.library) = library;
    Ok(bootstrap(&state))
}

#[tauri::command]
async fn pick_folder() -> Option<String> {
    tauri::async_runtime::spawn_blocking(|| {
        rfd::FileDialog::new()
            .set_title("Choisir un dossier")
            .pick_folder()
            .map(|path| path.display().to_string())
    })
    .await
    .ok()
    .flatten()
}

#[tauri::command]
async fn pick_executable() -> Option<String> {
    tauri::async_runtime::spawn_blocking(|| {
        rfd::FileDialog::new()
            .set_title("Choisir l'exécutable du jeu")
            .pick_file()
            .map(|path| path.display().to_string())
    })
    .await
    .ok()
    .flatten()
}

#[tauri::command]
async fn pick_artwork(app: AppHandle, id: String, kind: String) -> Result<GameDto, String> {
    let path = tauri::async_runtime::spawn_blocking(|| {
        rfd::FileDialog::new()
            .set_title("Choisir une image")
            .add_filter("Images", &["png", "jpg", "jpeg", "webp"])
            .pick_file()
    })
    .await
    .map_err(|error| error.to_string())?
    .ok_or_else(|| "Aucune image choisie.".to_string())?;
    let kind = match kind.as_str() {
        "cover" => AssetKind::Cover,
        "hero" => AssetKind::Hero,
        "logo" => AssetKind::Logo,
        "icon" => AssetKind::Icon,
        _ => return Err("Type d'image inconnu.".to_string()),
    };
    let state = app.state::<AppState>();
    let mut library = lock(&state.library);
    let settings = library.settings.clone();
    let game_id = library
        .get(&id)
        .ok_or_else(|| "Jeu introuvable".to_string())?
        .id
        .clone();
    let directory = config::artwork_root().join(&game_id);
    let imported = quark::artwork::import_file(&path, &directory, kind)?;
    {
        let game = library
            .get_mut(&id)
            .ok_or_else(|| "Jeu introuvable".to_string())?;
        game.artwork.set(kind, imported);
    }
    persist(&state, &mut library)?;
    Ok(game_dto(library.get(&id).unwrap(), &settings, false, None))
}

// ---------------------------------------------------------------------------
// Fenêtre sans décorations natives
// ---------------------------------------------------------------------------

#[tauri::command]
fn win_set_fullscreen(app: AppHandle, active: bool) -> Result<(), String> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "Fenêtre principale introuvable.".to_string())?;
    window
        .set_fullscreen(active)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn win_minimize(window: tauri::Window) {
    let _ = window.minimize();
}

#[tauri::command]
fn win_toggle_maximize(window: tauri::Window) {
    if window.is_maximized().unwrap_or(false) {
        let _ = window.unmaximize();
    } else {
        let _ = window.maximize();
    }
}

fn normalized_shortcut(shortcut: &str) -> String {
    shortcut
        .trim()
        .replace("Retour arrière", "Backspace")
        .replace("Retour arriere", "Backspace")
        .replace("Maj", "Shift")
        .replace("BackSpace", "Backspace")
}

fn configured_shortcut(shortcut: &str) -> Option<Shortcut> {
    normalized_shortcut(shortcut).parse().ok()
}

fn shortcut_matches(actual: &Shortcut, configured: &str) -> bool {
    configured_shortcut(configured).is_some_and(|shortcut| &shortcut == actual)
}

fn shortcut_matches_with_fallback(actual: &Shortcut, configured: &str, fallback: &str) -> bool {
    shortcut_matches(actual, configured)
        || (configured_shortcut(configured).is_none()
            && !configured.trim().is_empty()
            && shortcut_matches(actual, fallback))
}

fn register_one_shortcut(app: &AppHandle, raw: &str, fallback: &str) -> Result<(), String> {
    let value = normalized_shortcut(raw);
    if value.is_empty() {
        return Ok(());
    }
    if value.parse::<Shortcut>().is_err() {
        let fallback_value = normalized_shortcut(fallback);
        let _ = app.global_shortcut().register(fallback_value.as_str());
        return Err(format!(
            "Raccourci invalide « {raw} », remplacé par {fallback}"
        ));
    }
    app.global_shortcut()
        .register(value.as_str())
        .map_err(|error| format!("Raccourci indisponible « {raw} » : {error}"))
}

fn register_shortcuts(app: &AppHandle, settings: &Settings) -> Result<(), String> {
    let _ = app.global_shortcut().unregister_all();
    let mut errors = Vec::new();
    if !settings.screenshot_key.trim().is_empty() {
        if let Err(error) = register_one_shortcut(app, &settings.screenshot_key, "F12") {
            errors.push(error);
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join(" · "))
    }
}

#[tauri::command]
fn refresh_shortcuts(app: AppHandle) -> Result<(), String> {
    let settings = {
        let state = app.state::<AppState>();
        let settings = lock(&state.library).settings.clone();
        settings
    };
    register_shortcuts(&app, &settings)
}

fn capture_active_session(app: &AppHandle) {
    let name = {
        let state = app.state::<AppState>();
        let sessions = lock(&state.sessions);
        sessions
            .values()
            .next()
            .map(|run| run.name.clone())
            .unwrap_or_else(|| "jeu".to_string())
    };
    let path = screenshot::path_for(&name);
    match screenshot::capture(&path) {
        Ok(()) => {
            screenshot::play_shutter();
            let _ = app.emit(
                "quark://screenshot-saved",
                serde_json::json!({ "path": path.display().to_string(), "game": name }),
            );
        }
        Err(error) => {
            let _ = app.emit(
                "quark://notice",
                serde_json::json!({
                    "message": format!("Capture d'écran impossible : {error}"),
                    "error": true
                }),
            );
        }
    }
}

#[tauri::command]
fn tray_visibility(app: AppHandle, visible: bool) -> Result<(), String> {
    let state = app.state::<AppState>();
    state.tray_visible.store(visible, Ordering::Relaxed);
    if visible && lock(&state.tray).is_none() && tray_available() {
        spawn_tray(&app)?;
    }
    let tray = { lock(&state.tray).clone() };
    if let Some(tray) = tray {
        tray.update(|_| {});
    }
    Ok(())
}

/// Clic sur l'icône : affiche la fenêtre si elle est absente ou réduite, et la
/// referme comme la croix si elle est déjà au premier plan. La fenêtre est
/// détruite (pas seulement masquée) pour libérer la WebView ; l'icône reste.
fn toggle_main_window(app: &AppHandle) {
    let Some(window) = app.get_webview_window("main") else {
        let _ = show_main_window(app);
        return;
    };
    let visible = window.is_visible().unwrap_or(false);
    let minimized = window.is_minimized().unwrap_or(false);
    let tray_on = lock(&app.state::<AppState>().library).settings.tray_icon;
    if visible && !minimized && tray_on {
        let _ = window.destroy();
    } else {
        let _ = show_main_window(app);
    }
}

fn show_main_window(app: &AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.show().map_err(|error| error.to_string())?;
        let _ = window.unminimize();
        let _ = window.set_focus();
        return Ok(());
    }
    let config = app
        .config()
        .app
        .windows
        .iter()
        .find(|window| window.label == "main")
        .cloned()
        .ok_or_else(|| "Configuration de la fenêtre principale introuvable.".to_string())?;
    let window = tauri::WebviewWindowBuilder::from_config(app, &config)
        .map_err(|error| error.to_string())?
        .build()
        .map_err(|error| error.to_string())?;
    let icon =
        tauri::image::Image::new_owned(include_bytes!("../icons/icon.rgba").to_vec(), 128, 128);
    let _ = window.set_icon(icon);
    window.show().map_err(|error| error.to_string())?;
    let _ = window.set_focus();
    Ok(())
}

#[tauri::command]
fn win_close(window: tauri::Window) {
    let state = window.app_handle().state::<AppState>();
    if lock(&state.library).settings.tray_icon && state.tray_ready.load(Ordering::Relaxed) {
        // Détruire la WebView libère la mémoire du moteur ; la fenêtre est
        // reconstruite depuis tauri.conf.json au clic sur l'icône.
        let _ = window.destroy();
    } else {
        let _ = window.close();
    }
}

#[tauri::command]
fn win_start_drag(window: tauri::Window) {
    let _ = window.start_dragging();
}

#[tauri::command]
fn win_start_resize(window: tauri::Window, direction: String) {
    use tauri_runtime::ResizeDirection;
    let direction = match direction.as_str() {
        "e" => ResizeDirection::East,
        "n" => ResizeDirection::North,
        "ne" => ResizeDirection::NorthEast,
        "nw" => ResizeDirection::NorthWest,
        "s" => ResizeDirection::South,
        "se" => ResizeDirection::SouthEast,
        "sw" => ResizeDirection::SouthWest,
        "w" => ResizeDirection::West,
        _ => return,
    };
    let _ = window.start_resize_dragging(direction);
}

// ---------------------------------------------------------------------------
// Protocole d'images local
// ---------------------------------------------------------------------------

fn serve_asset(state: &AppState, request: tauri::http::Request<Vec<u8>>) -> Response<Vec<u8>> {
    let encoded = request.uri().path().trim_start_matches('/');
    let Ok(bytes) = URL_SAFE_NO_PAD.decode(encoded) else {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Vec::new())
            .unwrap();
    };
    let Ok(text) = String::from_utf8(bytes) else {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Vec::new())
            .unwrap();
    };
    let path = PathBuf::from(text);
    let Some(canonical) = std::fs::canonicalize(&path).ok() else {
        return Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Vec::new())
            .unwrap();
    };
    let allowed = std::fs::canonicalize(config::artwork_root())
        .ok()
        .is_some_and(|root| canonical.starts_with(root))
        || state
            .demo_root
            .as_ref()
            .and_then(|root| std::fs::canonicalize(root).ok())
            .is_some_and(|root| canonical.starts_with(root));
    if !allowed {
        return Response::builder()
            .status(StatusCode::FORBIDDEN)
            .body(Vec::new())
            .unwrap();
    }
    let Ok(data) = std::fs::read(&canonical) else {
        return Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Vec::new())
            .unwrap();
    };
    let mime = match canonical
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        _ => "application/octet-stream",
    };
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, mime)
        .header(header::CACHE_CONTROL, "no-cache")
        .body(data)
        .unwrap()
}

// ---------------------------------------------------------------------------
// Démonstration (captures, revue de l'interface)
// ---------------------------------------------------------------------------

struct DemoSpec {
    name: &'static str,
    developer: &'static str,
    genres: &'static [&'static str],
    released: &'static str,
    description: &'static str,
    status: GameStatus,
    favorite: bool,
    hours: f32,
    days_ago: i64,
    tags: &'static [&'static str],
    notes: &'static str,
}

const DEMO: &[DemoSpec] = &[
    DemoSpec {
        name: "Hollow Knight",
        developer: "Team Cherry",
        genres: &["Metroidvania", "Action", "Aventure"],
        released: "24 février 2017",
        description: "Descendez dans les ruines de Hallownest, un royaume souterrain rongé par un mal ancien. Explorez des galeries interconnectées, affrontez des gardiens oubliés et découvrez pourquoi tant de voyageurs ont été appelés sous terre.",
        status: GameStatus::Playing,
        favorite: true,
        hours: 42.5,
        days_ago: 1,
        tags: &["coop", "exploration", "difficile"],
        notes: "Reprendre au Colisée des Fous. Il reste deux masques à trouver dans les Jardins de la Reine.",
    },
    DemoSpec {
        name: "Celeste",
        developer: "Extremely OK Games",
        genres: &["Plateforme", "Précision"],
        released: "25 janvier 2018",
        description: "Aidez Madeline à gravir la montagne Celeste. Un jeu de plateforme exigeant sur le geste, porté par une bande-son de Lena Raine et un récit sur l'anxiété et la persévérance.",
        status: GameStatus::Finished,
        favorite: true,
        hours: 18.0,
        days_ago: 26,
        tags: &["difficile", "speedrun"],
        notes: "Face B terminée. Il reste les fraises dorées.",
    },
    DemoSpec {
        name: "Hades",
        developer: "Supergiant Games",
        genres: &["Roguelike", "Action", "Narratif"],
        released: "17 septembre 2020",
        description: "Fuyez les Enfers encore et encore. Chaque tentative fait avancer l'histoire, débloque de nouvelles armes et resserre les liens avec les dieux de l'Olympe.",
        status: GameStatus::Playing,
        favorite: false,
        hours: 31.2,
        days_ago: 3,
        tags: &["roguelike", "manette"],
        notes: "",
    },
    DemoSpec {
        name: "Disco Elysium",
        developer: "ZA/UM",
        genres: &["RPG", "Narratif", "Enquête"],
        released: "15 octobre 2019",
        description: "Un détective amnésique se réveille dans une ville au bord de l'effondrement. Faites parler les voix de son esprit, menez l'enquête et décidez de ce qu'il reste de lui.",
        status: GameStatus::Backlog,
        favorite: true,
        hours: 0.0,
        days_ago: 0,
        tags: &["histoire", "à commencer"],
        notes: "À lancer quand j'aurai une soirée calme devant moi.",
    },
    DemoSpec {
        name: "Outer Wilds",
        developer: "Mobius Digital",
        genres: &["Aventure", "Exploration", "Mystère"],
        released: "28 mai 2019",
        description: "Votre système solaire tourne en boucle depuis 22 minutes. Explorez des planètes vivantes, lisez les traces d'une civilisation disparue et comprenez ce qui se répète.",
        status: GameStatus::Finished,
        favorite: true,
        hours: 22.4,
        days_ago: 64,
        tags: &["exploration", "sans spoiler"],
        notes: "Ne rien lire avant de l'avoir fini. Le DLC reste à faire.",
    },
    DemoSpec {
        name: "Factorio",
        developer: "Wube Software",
        genres: &["Simulation", "Stratégie", "Automatisation"],
        released: "14 août 2020",
        description: "Construisez une usine toujours plus grande sur une planète hostile. Extraire, transformer, transporter, automatiser — puis recommencer, en plus grand.",
        status: GameStatus::Playing,
        favorite: false,
        hours: 128.6,
        days_ago: 2,
        tags: &["base", "long terme"],
        notes: "Base principale : 1 200 spm. Passer au nucléaire.",
    },
    DemoSpec {
        name: "Stardew Valley",
        developer: "ConcernedApe",
        genres: &["Simulation", "Gestion", "Détente"],
        released: "26 février 2016",
        description: "Reprenez la ferme de votre grand-père. Cultivez, pêchez, explorez les mines et apprenez à connaître les habitants de la vallée, au rythme des saisons.",
        status: GameStatus::Playing,
        favorite: false,
        hours: 76.1,
        days_ago: 5,
        tags: &["détente", "coop"],
        notes: "Fermer la saison d'automne. Communauté terminée à 80 %.",
    },
    DemoSpec {
        name: "Elden Ring",
        developer: "FromSoftware",
        genres: &["Action", "RPG", "Monde ouvert"],
        released: "25 février 2022",
        description: "L'Anneau ancien a été brisé. Parcourez les Terres intermédiaires, forgez votre propre chemin et affrontez des demi-dieux dans un monde ouvert signé FromSoftware et George R. R. Martin.",
        status: GameStatus::Backlog,
        favorite: false,
        hours: 0.0,
        days_ago: 0,
        tags: &["difficile", "monde ouvert"],
        notes: "",
    },
    DemoSpec {
        name: "Baldur's Gate 3",
        developer: "Larian Studios",
        genres: &["RPG", "Tactique", "Coopération"],
        released: "3 août 2023",
        description: "Rassemblez votre groupe, résistez au parasite et traversez une aventure à embranchements. Chaque choix pèse, chaque combat se prépare, et le multijoueur change la donne.",
        status: GameStatus::Playing,
        favorite: true,
        hours: 64.8,
        days_ago: 1,
        tags: &["coop", "histoire", "long terme"],
        notes: "Acte II. Garder Astarion dans l'équipe pour la quête du palais.",
    },
    DemoSpec {
        name: "Ori and the Blind Forest",
        developer: "Moon Studios",
        genres: &["Metroidvania", "Plateforme"],
        released: "11 mars 2015",
        description: "Un conte visuel et musical dans une forêt qui se meurt. Ori court, saute et restaure la lumière, porté par une direction artistique peinte à la main.",
        status: GameStatus::Finished,
        favorite: false,
        hours: 12.3,
        days_ago: 120,
        tags: &["visuel", "musique"],
        notes: "",
    },
    DemoSpec {
        name: "Slay the Spire",
        developer: "Mega Crit",
        genres: &["Deckbuilding", "Roguelike", "Stratégie"],
        released: "23 janvier 2019",
        description: "Construisez votre deck, gravissez la Flèche et adaptez-vous à chaque rencontre. Un roguelike de cartes dont chaque partie raconte une stratégie différente.",
        status: GameStatus::Playing,
        favorite: false,
        hours: 54.7,
        days_ago: 4,
        tags: &["cartes", "roguelike"],
        notes: "Montée 18 avec la Silencieuse.",
    },
    DemoSpec {
        name: "Cyberpunk 2077",
        developer: "CD Projekt Red",
        genres: &["RPG", "Action", "Monde ouvert"],
        released: "10 décembre 2020",
        description: "Night City, ses implants, ses gangs et ses promesses. Incarnez V, poursuivez un implant volé et décidez de ce que vous êtes prêt à sacrifier.",
        status: GameStatus::Dropped,
        favorite: false,
        hours: 9.4,
        days_ago: 210,
        tags: &["monde ouvert"],
        notes: "Abandonné après l'acte I. À retenter avec le mode Phantom Liberty.",
    },
    DemoSpec {
        name: "A Short Hike",
        developer: "Adam Robinson-Yu",
        genres: &["Aventure", "Détente", "Exploration"],
        released: "30 juillet 2019",
        description: "Une île, des sentiers, des mouettes et un sommet à atteindre. Un jeu court et chaleureux où l'on prend son temps, sans pression.",
        status: GameStatus::Finished,
        favorite: false,
        hours: 3.1,
        days_ago: 45,
        tags: &["court", "détente"],
        notes: "",
    },
    DemoSpec {
        name: "Signalis",
        developer: "rose-engine",
        genres: &["Horreur", "Survival", "Mystère"],
        released: "27 octobre 2022",
        description: "Une technicienne se réveille dans une station abandonnée, à la recherche de son partenaire. Survival horror à l'ambiance rétro, entre mémoire et répétition.",
        status: GameStatus::Backlog,
        favorite: false,
        hours: 0.0,
        days_ago: 0,
        tags: &["horreur", "court"],
        notes: "",
    },
];

fn escape_xml(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn slug(text: &str) -> String {
    text.chars()
        .map(|character| {
            if character.is_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect()
}

fn write_demo_art(root: &Path, name: &str, hue: u16) -> (PathBuf, PathBuf) {
    let dir = root.join(format!("{hue}-{}", slug(name)));
    let _ = std::fs::create_dir_all(&dir);
    let title = escape_xml(name);
    let initial = escape_xml(
        &name
            .chars()
            .next()
            .unwrap_or('Q')
            .to_uppercase()
            .to_string(),
    );
    let cover = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 600 900">
<defs>
<linearGradient id="g" x1="0" y1="0" x2="1" y2="1">
<stop offset="0" stop-color="hsl({hue}, 62%, 16%)"/>
<stop offset="0.55" stop-color="hsl({hue2}, 55%, 9%)"/>
<stop offset="1" stop-color="#05060a"/>
</linearGradient>
<radialGradient id="r" cx="0.82" cy="0.16" r="0.75">
<stop offset="0" stop-color="hsl({hue}, 95%, 62%)" stop-opacity="0.85"/>
<stop offset="1" stop-color="hsl({hue}, 95%, 62%)" stop-opacity="0"/>
</radialGradient>
<filter id="n"><feTurbulence type="fractalNoise" baseFrequency="0.9" numOctaves="2"/><feColorMatrix type="saturate" values="0"/><feComponentTransfer><feFuncA type="linear" slope="0.045"/></feComponentTransfer></filter>
</defs>
<rect width="600" height="900" fill="url(#g)"/>
<rect width="600" height="900" fill="url(#r)"/>
<circle cx="470" cy="180" r="250" fill="none" stroke="hsl({hue}, 90%, 72%)" stroke-opacity="0.18" stroke-width="1.5"/>
<circle cx="470" cy="180" r="168" fill="none" stroke="hsl({hue}, 90%, 72%)" stroke-opacity="0.12"/>
<circle cx="470" cy="180" r="92" fill="hsl({hue}, 90%, 60%)" fill-opacity="0.10"/>
<text x="300" y="520" text-anchor="middle" font-family="Archivo, DejaVu Sans, sans-serif" font-size="360" font-weight="800" fill="#ffffff" fill-opacity="0.10">{initial}</text>
<rect x="44" y="700" width="512" height="3" fill="hsl({hue}, 90%, 70%)" fill-opacity="0.35"/>
<text x="44" y="758" font-family="Archivo, DejaVu Sans, sans-serif" font-size="30" font-weight="600" fill="#ffffff" fill-opacity="0.82">{title}</text>
<rect width="600" height="900" filter="url(#n)"/>
</svg>"##,
        hue = hue,
        hue2 = (hue + 42) % 360,
        initial = initial,
        title = title,
    );
    let hero = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1920 720">
<defs>
<linearGradient id="g" x1="0" y1="0" x2="1" y2="0.7">
<stop offset="0" stop-color="hsl({hue}, 55%, 13%)"/>
<stop offset="0.5" stop-color="hsl({hue2}, 48%, 8%)"/>
<stop offset="1" stop-color="#04050a"/>
</linearGradient>
<radialGradient id="r" cx="0.76" cy="0.2" r="0.7">
<stop offset="0" stop-color="hsl({hue}, 92%, 58%)" stop-opacity="0.55"/>
<stop offset="1" stop-color="hsl({hue}, 92%, 58%)" stop-opacity="0"/>
</radialGradient>
<filter id="n"><feTurbulence type="fractalNoise" baseFrequency="0.8" numOctaves="2"/><feColorMatrix type="saturate" values="0"/><feComponentTransfer><feFuncA type="linear" slope="0.04"/></feComponentTransfer></filter>
</defs>
<rect width="1920" height="720" fill="url(#g)"/>
<rect width="1920" height="720" fill="url(#r)"/>
<circle cx="1500" cy="120" r="430" fill="none" stroke="hsl({hue}, 92%, 70%)" stroke-opacity="0.14" stroke-width="2"/>
<circle cx="1500" cy="120" r="300" fill="none" stroke="hsl({hue}, 92%, 70%)" stroke-opacity="0.10"/>
<path d="M0 560 C 380 480, 640 650, 980 540 S 1600 430, 1920 520 L1920 720 L0 720 Z" fill="#03040a" fill-opacity="0.45"/>
<text x="1370" y="600" font-family="Archivo, DejaVu Sans, sans-serif" font-size="520" font-weight="800" fill="#ffffff" fill-opacity="0.07">{initial}</text>
<rect width="1920" height="720" filter="url(#n)"/>
</svg>"##,
        hue = hue,
        hue2 = (hue + 42) % 360,
        initial = initial,
    );
    let cover_path = dir.join("cover.svg");
    let hero_path = dir.join("hero.svg");
    let _ = std::fs::write(&cover_path, cover);
    let _ = std::fs::write(&hero_path, hero);
    (cover_path, hero_path)
}

fn demo_library(root: &Path, with_games: bool) -> Library {
    let mut library = Library::default();
    library.settings.onboarding_done = std::env::var_os("QUARK_DEMO_ONBOARDING").is_none();
    library.settings.last_seen_version = env!("CARGO_PKG_VERSION").to_string();
    library.settings.ui_language = std::env::var("QUARK_DEMO_LANG")
        .ok()
        .and_then(|value| parse_language(&value))
        .unwrap_or(UiLanguage::French);
    library.settings.theme = ThemeMode::Amoled;
    library.settings.ui_compact = false;
    if !with_games {
        return library;
    }
    let _ = std::fs::remove_dir_all(root);
    let _ = std::fs::create_dir_all(root);
    let now = config::now_ts();
    let provider_cycle = [
        quark::providers::model::Provider::Steam,
        quark::providers::model::Provider::Epic,
        quark::providers::model::Provider::Gog,
        quark::providers::model::Provider::Itch,
    ];
    for (index, spec) in DEMO.iter().enumerate() {
        let mut game = Game::new(spec.name, "/bin/true");
        game.developer = spec.developer.to_string();
        game.genres = spec.genres.iter().map(|genre| genre.to_string()).collect();
        game.released = spec.released.to_string();
        game.description = spec.description.to_string();
        game.status = spec.status;
        game.favorite = spec.favorite;
        game.tags = spec.tags.iter().map(|tag| tag.to_string()).collect();
        game.notes = spec.notes.to_string();
        game.playtime_secs = (spec.hours * 3600.0) as u64;
        game.last_played = if spec.hours > 0.0 {
            Some(now - spec.days_ago * 86_400)
        } else {
            None
        };
        game.added_at = now - (spec.days_ago + 30) * 86_400;
        if game.playtime_secs > 0 {
            let count = 5u64;
            let part = game.playtime_secs / count;
            for index in 0..count {
                game.sessions.push(PlaySession {
                    start: now - (spec.days_ago + index as i64 * 2) * 86_400,
                    secs: part.max(60),
                });
            }
        }
        let (cover, hero) = write_demo_art(root, spec.name, hue_for(spec.name));
        game.artwork.cover = Some(cover.clone());
        game.artwork.hero = Some(hero);
        game.normalize_installations();
        let provider = provider_cycle[index % provider_cycle.len()];
        let installed = index < 8;
        let cover_url = asset_url(Some(&cover));
        library
            .provider_library
            .games
            .push(quark::providers::model::ProviderGame {
                key: quark::providers::model::ProviderGameKey {
                    provider,
                    product_id: format!("demo-{index}"),
                },
                title: spec.name.to_string(),
                owned: true,
                install_root: installed.then(|| PathBuf::from("/bin")),
                install_path: installed.then(|| PathBuf::from("/bin")),
                install_exe: installed.then(|| PathBuf::from("/bin/true")),
                install_state: if installed {
                    quark::providers::model::InstallState::Installed
                } else {
                    quark::providers::model::InstallState::NotInstalled
                },
                cover_url,
                quark_game_id: installed.then(|| game.id.clone()),
                custom_save_paths: Vec::new(),
                capabilities: quark::providers::model::GameCapabilities::default(),
                last_synced: Some(now),
            });
        library.games.push(game);
    }
    library
}

// ---------------------------------------------------------------------------
// Surveillance des sessions
// ---------------------------------------------------------------------------

fn spawn_monitor(app: AppHandle) {
    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_millis(1000));
        let state = app.state::<AppState>();
        let mut ended = Vec::new();
        {
            let mut sessions = lock(&state.sessions);
            let mut remove = Vec::new();
            for (id, run) in sessions.iter_mut() {
                if run.session.should_escalate() {
                    session::force_kill(&run.session);
                    run.session.escalated();
                }
                match run.session.child.try_wait() {
                    Ok(Some(_)) => remove.push(id.clone()),
                    Ok(None) => {
                        if run.session.should_abandon() {
                            remove.push(id.clone());
                        }
                    }
                    Err(_) => remove.push(id.clone()),
                }
            }
            for id in remove {
                if let Some(run) = sessions.remove(&id) {
                    ended.push((
                        id,
                        run.started_ts,
                        run.session.started.elapsed().as_secs().max(1),
                    ));
                }
            }
        }
        for (id, started, secs) in ended {
            let mut library = lock(&state.library);
            if let Some(game) = library.get_mut(&id) {
                game.record_session(started, secs);
            }
            if !state.demo {
                let _ = library.save();
            }
            let settings = library.settings.clone();
            let dto = library
                .get(&id)
                .map(|game| game_dto(game, &settings, false, None));
            let name = dto
                .as_ref()
                .map(|game| game.name.clone())
                .unwrap_or_default();
            drop(library);
            let _ = app.emit(
                "quark://session-ended",
                serde_json::json!({ "id": id, "secs": secs, "game": dto }),
            );
            if settings.notify_session_done {
                let duration = if secs >= 3600 {
                    format!("{} h {:02}", secs / 3600, (secs % 3600) / 60)
                } else {
                    format!("{} min", secs / 60)
                };
                notify_desktop(
                    &app,
                    &quark::i18n::tr("Partie terminée"),
                    &format!("{name} · {duration}"),
                );
            }
        }
    });
}

fn finalize_sessions(app: &AppHandle) {
    let state = app.state::<AppState>();
    let mut ended = Vec::new();
    {
        let mut sessions = lock(&state.sessions);
        for (id, run) in sessions.drain() {
            ended.push((
                id,
                run.started_ts,
                run.session.started.elapsed().as_secs().max(1),
            ));
        }
    }
    if ended.is_empty() {
        return;
    }
    let mut library = lock(&state.library);
    for (id, started, secs) in ended {
        if let Some(game) = library.get_mut(&id) {
            game.record_session(started, secs);
        }
    }
    if !state.demo {
        let _ = library.save();
    }
}

// ---------------------------------------------------------------------------
// Point d'entrée
// ---------------------------------------------------------------------------

fn is_wayland_session() -> bool {
    #[cfg(target_os = "linux")]
    {
        std::env::var_os("WAYLAND_DISPLAY").is_some()
            || std::env::var("XDG_SESSION_TYPE")
                .is_ok_and(|value| value.eq_ignore_ascii_case("wayland"))
    }
    #[cfg(not(target_os = "linux"))]
    {
        false
    }
}

#[cfg(target_os = "linux")]
fn configure_wayland_rendering() {
    if !is_wayland_session() {
        return;
    }
    // WebKitGTK's DMA-BUF renderer and accelerated compositing both rely on
    // damage tracking that some Wayland compositors mishandle, leaving stale
    // pixels until the cursor repaints them. Fall back to full-surface paints.
    for key in [
        "WEBKIT_DISABLE_DMABUF_RENDERER",
        "WEBKIT_DISABLE_COMPOSITING_MODE",
    ] {
        if std::env::var_os(key).is_none() {
            // SAFETY: called before GTK or any other thread starts.
            unsafe { std::env::set_var(key, "1") };
        }
    }
}

static INSTANCE_APP: OnceLock<AppHandle> = OnceLock::new();

/// Une seule instance graphique : la seconde demande à la première d'afficher
/// sa fenêtre puis quitte, au lieu d'ouvrir une deuxième WebView.
fn claim_single_instance() -> bool {
    let path = config::config_dir().join("quark.sock");
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let listener = match UnixListener::bind(&path) {
        Ok(listener) => listener,
        Err(_) => {
            if let Ok(mut stream) = UnixStream::connect(&path) {
                let _ = stream.write_all(b"show\n");
                return false;
            }
            let _ = std::fs::remove_file(&path);
            match UnixListener::bind(&path) {
                Ok(listener) => listener,
                Err(_) => return true,
            }
        }
    };
    let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600));
    std::thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            let mut request = [0u8; 16];
            let mut stream = stream;
            let _ = stream.read(&mut request);
            for _ in 0..50 {
                if let Some(app) = INSTANCE_APP.get() {
                    let target = app.clone();
                    let _ = app.run_on_main_thread(move || {
                        let _ = show_main_window(&target);
                    });
                    break;
                }
                std::thread::sleep(Duration::from_millis(100));
            }
        }
    });
    true
}

fn main() {
    #[cfg(target_os = "linux")]
    configure_wayland_rendering();
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args
        .iter()
        .any(|arg| arg == "--help" || arg == "-h" || arg == "--version")
    {
        println!("Quark {}", env!("CARGO_PKG_VERSION"));
        println!("Usage: quark [--demo | --demo-empty | --deck]");
        println!("       quark launch <jeu>");
        println!("       quark list");
        return;
    }
    if let Some(code) = cli_main(&args) {
        std::process::exit(code);
    }
    if !claim_single_instance() {
        return;
    }
    let demo = args
        .iter()
        .any(|arg| arg == "--demo" || arg == "--demo-empty");
    let empty = args.iter().any(|arg| arg == "--demo-empty");
    let demo_root = demo.then(|| std::env::temp_dir().join("quark-tauri-demo"));
    let library = match &demo_root {
        Some(root) => demo_library(root, !empty),
        None => Library::load(),
    };
    quark::i18n::set_language(library.settings.ui_language);
    let tray_visible = Arc::new(AtomicBool::new(library.settings.tray_icon));
    let state = AppState {
        library: Mutex::new(library),
        sessions: Mutex::new(HashMap::new()),
        installs: Mutex::new(HashMap::new()),
        demo,
        deck_demo: args.iter().any(|arg| arg == "--deck"),
        demo_root,
        quitting: AtomicBool::new(false),
        tray: Mutex::new(None),
        tray_ready: AtomicBool::new(false),
        tray_visible,
    };

    let app = tauri::Builder::default()
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, shortcut, event| {
                    if event.state != ShortcutState::Pressed {
                        return;
                    }
                    let state = app.state::<AppState>();
                    if lock(&state.sessions).is_empty() {
                        return;
                    }
                    let settings = lock(&state.library).settings.clone();
                    if shortcut_matches_with_fallback(shortcut, &settings.screenshot_key, "F12") {
                        std::thread::spawn({
                            let app = app.clone();
                            move || capture_active_session(&app)
                        });
                    }
                })
                .build(),
        )
        .manage(state)
        .register_uri_scheme_protocol("quarkimg", |ctx, request| {
            let state = ctx.app_handle().state::<AppState>();
            serve_asset(&state, request)
        })
        .invoke_handler(tauri::generate_handler![
            bootstrap_data,
            running_games,
            get_game,
            launch_game,
            stop_game,
            set_favorite,
            set_status,
            set_notes,
            add_tag,
            remove_tag,
            update_game,
            add_game,
            remove_game,
            remove_games,
            activate_installation,
            open_folder,
            open_config_folder,
            read_log,
            save_settings,
            reload_library,
            pick_executable,
            pick_folder,
            pick_artwork,
            win_minimize,
            win_toggle_maximize,
            win_close,
            win_start_drag,
            win_start_resize,
            refresh_shortcuts,
            tray_visibility,
            gamepad::deck_status,
            win_set_fullscreen,
            features::get_settings_raw,
            features::patch_settings_raw,
            features::get_game_raw,
            features::patch_game_raw,
            features::detect_game_dlls,
            features::list_runtimes,
            features::fetch_news,
            features::fetch_hltb,
            features::fetch_protondb,
            features::disk_report,
            features::check_update,
            features::install_update,
            features::open_url,
            store::store_load,
            store::store_refresh,
            store::store_search,
            providers::providers_data,
            providers::provider_ensure_covers,
            providers::provider_sync,
            providers::provider_login,
            providers::provider_disconnect,
            providers::provider_save_credentials,
            providers::provider_tool_install,
            providers::provider_install,
            providers::provider_cancel,
            providers::provider_uninstall,
            providers::provider_adopt,
            providers::steam_candidates,
            providers::steam_import,
            features::scan_folder,
            features::site_search,
            features::artwork_candidates,
            features::artwork_choose,
            extras::game_achievements,
            extras::achievements_compare,
            extras::game_playtime,
            extras::game_media,
            extras::save_preview,
            extras::save_backups,
            extras::save_backup,
            extras::save_restore_preview,
            extras::save_restore,
            extras::save_add_folder,
            extras::save_remove_folder,
            extras::cloud_sync,
            extras::artwork_fetch,
            extras::game_remove_artwork,
        ])
        .setup(|app| {
            let _ = INSTANCE_APP.set(app.handle().clone());
            let icon = tauri::image::Image::new_owned(
                include_bytes!("../icons/icon.rgba").to_vec(),
                128,
                128,
            );
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_icon(icon.clone());
            }
            let tray_on = app.state::<AppState>().tray_visible.load(Ordering::Relaxed);
            if tray_on && tray_available() {
                if let Err(error) = spawn_tray(app.handle()) {
                    eprintln!("{error}");
                }
            }
            let settings = {
                let state = app.state::<AppState>();
                let settings = lock(&state.library).settings.clone();
                settings
            };
            let _ = register_shortcuts(app.handle(), &settings);
            spawn_monitor(app.handle().clone());
            gamepad::spawn(app.handle().clone());
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("Quark n'a pas pu démarrer");

    app.run(|app, event| {
        if let RunEvent::ExitRequested { api, .. } = event {
            let state = app.state::<AppState>();
            let keep_alive = !state.quitting.load(Ordering::Relaxed)
                && lock(&state.library).settings.tray_icon
                && state.tray_ready.load(Ordering::Relaxed);
            if keep_alive {
                api.prevent_exit();
            } else {
                finalize_sessions(app);
            }
        }
    });
}
