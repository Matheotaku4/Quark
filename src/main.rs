//! Quark — launcher léger pour Linux, en Rust + egui.
//!
//! Utilisation normale : `cargo run --release`
//! Diagnostic (sans interface) : voir `--help`.

mod achievements_bridge;
mod app;
mod archive;
mod artwork;
mod config;
mod disk;
mod fonts;
mod game_search;
mod howlongtobeat;
mod i18n;
mod icon;
mod images;
mod net;
mod news;
mod overlay;
mod overlay_dll;
mod process;
mod proton;
mod protondb;
mod providers;
mod saves;
mod scan;
mod session;
mod session_sync;
mod snapshot;
mod stats;
mod steam_store;
mod steamlib;
mod theme;
mod tray;
mod ui_command;
mod ui_dialogs;
mod ui_home;
mod ui_library;
mod ui_my_games;
mod ui_onboarding;
mod ui_providers;
mod ui_store;
mod ui_views;
mod update;
mod widgets;

use app::QuarkApp;

fn main() -> eframe::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    // Icône seule : ce processus n'affiche aucune fenêtre, il tient la zone de
    // notification et relance Quark au clic.
    if args.first().map(String::as_str) == Some(tray::HELPER_ARG) {
        std::process::exit(tray::run_helper());
    }
    if let Some(code) = cli(&args) {
        std::process::exit(code);
    }
    // Une seule fenêtre à la fois : l'icône peut relancer Quark alors qu'il est
    // déjà ouvert, la seconde instance se retire au lieu d'écrire les réglages
    // en même temps.
    let Some(_window_lock) = tray::lock_gui() else {
        return Ok(());
    };

    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("Quark")
            // Même identifiant que le fichier `assets/quark.desktop` : c'est lui
            // qui associe la fenêtre (Wayland comme X11) et son icône.
            .with_app_id("quark")
            .with_icon(icon::window_icon())
            .with_inner_size([1360.0, 860.0])
            .with_min_inner_size([1024.0, 640.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Quark",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            fonts::install(&cc.egui_ctx);
            let mut app = QuarkApp::new(&cc.egui_ctx);
            app.startup();
            Ok(Box::new(app))
        }),
    )
}

/// Mode diagnostic en ligne de commande (utile sans écran / pour tester le réseau).
fn cli(args: &[String]) -> Option<i32> {
    let command = args.first()?.as_str();
    match command {
        "--help" | "-h" => {
            println!(
                "Quark {}\n\n\
                 USAGE:\n  \
                 quark                    Lance le launcher\n  \
                 quark --doctor           Affiche l'environnement détecté (dossiers, Proton, jeux)\n  \
                 quark --search <nom>     Recherche un jeu sur Steam + SteamGridDB\n  \
                 quark launch <jeu>       Lance un jeu sans ouvrir l'interface\n  \
                 quark list               Liste les jeux de la bibliothèque\n  \
                 quark --fetch <nom>      Télécharge les artworks du premier résultat\n",
                env!("CARGO_PKG_VERSION")
            );
            println!("  quark --scan <dossier>   Liste les jeux détectés dans un dossier");
            println!("  quark --news <nom|appid> Affiche les actualités Steam d'un jeu");
            println!("  quark --update           Vérifie la dernière release GitHub");
            println!("\nVérification du rendu (sans écran, aucun GPU) :");
            print!("{}", snapshot::HELP);
            Some(0)
        }
        "--update" => {
            let client = artwork::http();
            match update::latest(&client) {
                Ok(release) => {
                    println!(
                        "Quark {} · dernière release : {} ({})",
                        update::current_version(),
                        release.tag_name,
                        release.name
                    );
                    if update::is_newer(&release.tag_name, update::current_version()) {
                        println!("mise à jour disponible : {}", release.html_url);
                        match release.archive() {
                            Some(asset) => {
                                let dir = config::data_dir().join("updates");
                                match update::download(&client, &release, asset, &dir) {
                                    Ok(path) => {
                                        println!(
                                            "archive téléchargée : {} ({} octets)",
                                            path.display(),
                                            asset.size
                                        );
                                        println!(
                                            "décompresse-la et remplace le binaire pour installer \
                                             la nouvelle version"
                                        );
                                    }
                                    Err(err) => println!("téléchargement impossible : {err}"),
                                }
                            }
                            None => println!("cette release ne contient pas d'archive"),
                        }
                    } else {
                        println!("à jour.");
                    }
                    Some(0)
                }
                Err(err) => {
                    eprintln!("Vérification impossible : {err}");
                    Some(1)
                }
            }
        }
        "--ui" => {
            let Some(out) = args.get(1).filter(|a| !a.starts_with("--")) else {
                eprintln!("Usage : quark --ui <fichier.png> [options]");
                print!("{}", snapshot::HELP);
                return Some(2);
            };
            let mut opts = snapshot::SnapshotOptions {
                out: std::path::PathBuf::from(out),
                ..snapshot::SnapshotOptions::default()
            };
            let mut index = 2;
            while index < args.len() {
                let key = args[index].clone();
                let value = args.get(index + 1).cloned();
                // Options sans valeur : un seul argument à avancer.
                match key.as_str() {
                    "--info" => {
                        opts.info = true;
                        index += 1;
                        continue;
                    }
                    "--running" => {
                        opts.running = true;
                        index += 1;
                        continue;
                    }
                    "--marked" => {
                        opts.marked = true;
                        index += 1;
                        continue;
                    }
                    "--demo" => {
                        opts.demo = true;
                        index += 1;
                        continue;
                    }
                    "--providers-demo" => {
                        opts.providers_demo = true;
                        index += 1;
                        continue;
                    }
                    "--providers-error" => {
                        opts.providers_demo = true;
                        opts.providers_error = true;
                        index += 1;
                        continue;
                    }
                    _ => {}
                }
                match key.as_str() {
                    "--size" => match value.as_deref().and_then(parse_size) {
                        Some(size) => opts.size = size,
                        None => {
                            eprintln!("--size attend la forme 1360x860");
                            return Some(2);
                        }
                    },
                    "--scale" => match value.as_deref().and_then(|v| v.parse::<f32>().ok()) {
                        Some(scale) => opts.scale = scale,
                        None => {
                            eprintln!("--scale attend un nombre (ex. 1.5)");
                            return Some(2);
                        }
                    },
                    "--frames" => match value.as_deref().and_then(|v| v.parse::<u32>().ok()) {
                        Some(frames) => opts.frames = frames,
                        None => {
                            eprintln!("--frames attend un entier (ex. 20)");
                            return Some(2);
                        }
                    },
                    "--demo-count" => {
                        match value.as_deref().and_then(|v| v.parse::<usize>().ok()) {
                            Some(count) => opts.demo_count = Some(count.clamp(1, 2_000)),
                            None => {
                                eprintln!("--demo-count attend un entier (ex. 660)");
                                return Some(2);
                            }
                        }
                    }
                    "--hover" => match value.as_deref().and_then(parse_point) {
                        Some(point) => opts.hover = Some(point),
                        None => {
                            eprintln!("--hover attend des coordonnées X,Y");
                            return Some(2);
                        }
                    },
                    "--click" => match value.as_deref().and_then(parse_point) {
                        Some(point) => {
                            opts.click = Some(point);
                            opts.hover = Some(point);
                        }
                        None => {
                            eprintln!("--click attend des coordonnées X,Y");
                            return Some(2);
                        }
                    },
                    "--view" => {
                        opts.view = match value.as_deref() {
                            Some("list") => Some(config::ViewMode::List),
                            Some("grid") => Some(config::ViewMode::Grid),
                            _ => {
                                eprintln!("--view attend list ou grid");
                                return Some(2);
                            }
                        }
                    }
                    "--group" => {
                        opts.group = match value.as_deref() {
                            Some("status") => Some(config::GroupBy::Status),
                            Some("store") => Some(config::GroupBy::Store),
                            Some("genre") => Some(config::GroupBy::Genre),
                            Some("year") => Some(config::GroupBy::Year),
                            Some("none") => Some(config::GroupBy::None),
                            _ => {
                                eprintln!("--group attend status, store, genre, year ou none");
                                return Some(2);
                            }
                        }
                    }
                    "--tab" => {
                        opts.tab = match value.as_deref() {
                            Some("library") => Some(app::Tab::Library),
                            Some("store") => Some(app::Tab::Store),
                            Some("my-games") => Some(app::Tab::MyGames),
                            _ => {
                                eprintln!("--tab attend library, store ou my-games");
                                return Some(2);
                            }
                        }
                    }
                    "--theme" => {
                        opts.theme = match value.as_deref() {
                            Some("amoled") => Some(config::ThemeMode::Amoled),
                            Some("graphite") => Some(config::ThemeMode::Graphite),
                            Some("catppuccin-mocha") => Some(config::ThemeMode::CatppuccinMocha),
                            Some("catppuccin-macchiato") => {
                                Some(config::ThemeMode::CatppuccinMacchiato)
                            }
                            Some("catppuccin-frappe") => Some(config::ThemeMode::CatppuccinFrappe),
                            Some("dracula") => Some(config::ThemeMode::Dracula),
                            Some("nord") => Some(config::ThemeMode::Nord),
                            Some("gruvbox") => Some(config::ThemeMode::GruvboxDark),
                            Some("tokyo-night") => Some(config::ThemeMode::TokyoNight),
                            Some("rose-pine") => Some(config::ThemeMode::RosePine),
                            Some("everforest") => Some(config::ThemeMode::Everforest),
                            Some("one-dark") => Some(config::ThemeMode::OneDark),
                            Some("kanagawa") => Some(config::ThemeMode::Kanagawa),
                            _ => {
                                eprintln!(
                                    "thème inconnu (amoled, graphite, catppuccin-mocha, dracula, nord, gruvbox, tokyo-night, rose-pine, everforest, one-dark, kanagawa)"
                                );
                                return Some(2);
                            }
                        }
                    }
                    "--blur" => match value.as_deref().and_then(|v| v.parse::<f32>().ok()) {
                        Some(blur) => opts.hero_blur = Some(blur),
                        None => {
                            eprintln!("--blur attend un nombre (ex. 14)");
                            return Some(2);
                        }
                    },
                    "--lang" => {
                        opts.language = match value.as_deref() {
                            Some("fr") | Some("french") | Some("français") => {
                                Some(config::UiLanguage::French)
                            }
                            Some("en") | Some("english") | Some("anglais") => {
                                Some(config::UiLanguage::English)
                            }
                            Some("es") | Some("spanish") | Some("espagnol") => {
                                Some(config::UiLanguage::Spanish)
                            }
                            Some("de") | Some("german") | Some("allemand") => {
                                Some(config::UiLanguage::German)
                            }
                            Some("pt") | Some("portuguese") | Some("portugais") => {
                                Some(config::UiLanguage::Portuguese)
                            }
                            _ => {
                                eprintln!("--lang attend fr, en, es, de ou pt");
                                return Some(2);
                            }
                        }
                    }
                    "--select" => match value {
                        Some(name) => opts.select = Some(name),
                        None => {
                            eprintln!("--select attend un nom de jeu");
                            return Some(2);
                        }
                    },
                    "--button" => match value.as_deref() {
                        Some("right") => opts.right_click = true,
                        Some("left") => opts.right_click = false,
                        _ => {
                            eprintln!("--button attend left ou right");
                            return Some(2);
                        }
                    },
                    "--key" => match value {
                        Some(key) => opts.key = Some(key),
                        None => {
                            eprintln!(
                                "--key attend une touche (up, down, left, right, enter, escape)"
                            );
                            return Some(2);
                        }
                    },
                    "--home" => {
                        opts.home = true;
                        index += 1;
                        continue;
                    }
                    "--hero-bg" => {
                        opts.hero_background = true;
                        index += 1;
                        continue;
                    }
                    "--dialog" => match value {
                        Some(dialog) => opts.dialog = Some(dialog),
                        None => {
                            eprintln!("--dialog attend un nom de fenêtre");
                            return Some(2);
                        }
                    },
                    _ => {
                        eprintln!("Option inconnue : {key}");
                        print!("{}", snapshot::HELP);
                        return Some(2);
                    }
                }
                index += 2;
            }
            match snapshot::run(&opts) {
                Ok(()) => Some(0),
                Err(err) => {
                    eprintln!("Rendu impossible : {err}");
                    Some(1)
                }
            }
        }
        "--doctor" => {
            let lib = config::Library::load();
            println!("Quark {}", env!("CARGO_PKG_VERSION"));
            println!("config        : {}", config::config_path().display());
            println!("données       : {}", config::data_dir().display());
            println!("artworks      : {}", config::artwork_root().display());
            println!("préfixes      : {}", config::prefix_root().display());
            println!(
                "racine Steam  : {}",
                proton::steam_root()
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|| "introuvable".to_string())
            );
            let runtimes = proton::detect_runtimes();
            println!("runtimes      : {}", runtimes.len());
            for runtime in &runtimes {
                println!("  - {} -> {}", runtime.label(), runtime.path.display());
            }
            if let Some(suggested) = proton::suggested_default(&runtimes) {
                println!("proposé       : {}", suggested.label());
            }
            println!("anti-triche   : {}", proton::anticheat_summary(&runtimes));
            println!("jeux          : {}", lib.games.len());
            for game in &lib.games {
                let state = match proton::resolve(game, &lib.settings, &runtimes) {
                    Ok(resolved) => format!("OK · {}", resolved.label()),
                    Err(err) => format!("ERREUR · {err}"),
                };
                println!("  - {} ({}) [{state}]", game.name, game.exe);
            }
            Some(0)
        }
        "launch" => {
            let Some(query) = args.get(1) else {
                eprintln!("Usage : quark launch <nom du jeu>");
                return Some(2);
            };
            match launch_from_cli(query) {
                Ok(()) => Some(0),
                Err(err) => {
                    eprintln!("{err}");
                    Some(1)
                }
            }
        }
        "list" => {
            let lib = config::Library::load();
            let mut games: Vec<_> = lib.games.iter().collect();
            games.sort_by_key(|game| game.name.to_lowercase());
            for game in games {
                println!(
                    "{:<36} {:<12} {}",
                    game.name,
                    game.status.label(),
                    game.playtime_label()
                );
            }
            Some(0)
        }
        "--search" => {
            let query = args.get(1).cloned().unwrap_or_else(|| {
                eprintln!("Usage : quark --search <nom du jeu>");
                std::process::exit(2);
            });
            let lib = config::Library::load();
            let client = artwork::http();
            let outcome = artwork::search(&client, &query, &lib.settings);
            println!("{} résultat(s) pour « {query} »", outcome.hits.len());
            for hit in &outcome.hits {
                println!(
                    "  - [{}] {} | appid={:?} sgdb={:?} | {}",
                    hit.source.label(),
                    hit.title,
                    hit.steam_app_id,
                    hit.sgdb_id,
                    hit.detail
                );
            }
            for warning in &outcome.warnings {
                println!("  ! {warning}");
            }
            if let Some(hit) = outcome.hits.first() {
                let plan = artwork::build_plan(&client, hit, &lib.settings);
                println!("Plan d'artworks pour « {} » :", hit.title);
                for kind in config::AssetKind::ALL {
                    let candidates = plan.candidates(kind);
                    println!("  {:<12} {} candidat(s)", kind.label(), candidates.len());
                    for url in candidates.iter().take(2) {
                        println!("      {url}");
                    }
                }
                for warning in &plan.warnings {
                    println!("  ! {warning}");
                }
            }
            Some(0)
        }
        "--scan" => {
            let folder = args.get(1).cloned().unwrap_or_else(|| {
                eprintln!("Usage : quark --scan <dossier>");
                std::process::exit(2);
            });
            let lib = config::Library::load();
            let client = artwork::http();
            let found = scan::scan(std::path::Path::new(&folder), 4, 10);
            println!("{} candidat(s) dans {folder}", found.len());
            for item in &found {
                let outcome = artwork::search(&client, &item.title, &lib.settings);
                match artwork::best_hit(&outcome, &item.title) {
                    Some(hit) => println!(
                        "  - {:<28} {:<10} {} [{}]",
                        item.title,
                        hit.title,
                        item.exe.display(),
                        hit.source.label()
                    ),
                    None => println!("  - {:<28} {:<10} {}", item.title, "?", item.exe.display()),
                }
            }
            Some(0)
        }
        "--news" => {
            let arg = args.get(1).cloned().unwrap_or_else(|| {
                eprintln!("Usage : quark --news <appid | nom du jeu>");
                std::process::exit(2);
            });
            let lib = config::Library::load();
            let client = artwork::http();
            let app_id = match arg.parse::<u32>() {
                Ok(id) => id,
                Err(_) => {
                    let outcome = artwork::search(&client, &arg, &lib.settings);
                    match outcome.hits.iter().find_map(|hit| hit.steam_app_id) {
                        Some(id) => id,
                        None => {
                            eprintln!("Aucun appid Steam pour « {arg} »");
                            for warning in &outcome.warnings {
                                eprintln!("  ! {warning}");
                            }
                            return Some(1);
                        }
                    }
                }
            };
            match news::fetch(&client, app_id, news::COUNT) {
                Ok(items) => {
                    println!("{} actualité(s) pour l'appid {app_id}", items.len());
                    for item in &items {
                        println!("  - [{}] {}", config::human_ago(item.date), item.title);
                        println!("      {} · {}", item.feed, item.url);
                        if !item.excerpt.is_empty() {
                            println!("      {}", item.excerpt);
                        }
                    }
                    Some(0)
                }
                Err(err) => {
                    eprintln!("Actualités indisponibles : {err}");
                    Some(1)
                }
            }
        }
        "--fetch" => {
            let query = args.get(1).cloned().unwrap_or_else(|| {
                eprintln!("Usage : quark --fetch <nom du jeu>");
                std::process::exit(2);
            });
            let lib = config::Library::load();
            let client = artwork::http();
            let outcome = artwork::search(&client, &query, &lib.settings);
            let Some(hit) = outcome.hits.first().cloned() else {
                eprintln!("Aucun résultat pour « {query} »");
                for warning in &outcome.warnings {
                    eprintln!("  ! {warning}");
                }
                return Some(1);
            };
            println!("Jeu retenu : {} ({})", hit.title, hit.source.label());
            let dir = std::env::temp_dir().join("quark-artworks").join("cli-test");
            let plan = artwork::build_plan(&client, &hit, &lib.settings);
            let (artwork, warnings) = artwork::apply_plan(&client, &dir, &plan);
            for kind in config::AssetKind::ALL {
                match artwork.get(kind) {
                    Some(path) => {
                        let size = std::fs::read(path)
                            .ok()
                            .and_then(|bytes| artwork::image_size(&bytes))
                            .map(|(w, h)| format!("{w}×{h}"))
                            .unwrap_or_else(|| "?".to_string());
                        println!("  {:<12} {:<12} {}", kind.label(), size, path.display());
                    }
                    None => println!("  {:<12} {:<12} absent", kind.label(), ""),
                }
            }
            for warning in &warnings {
                println!("  ! {warning}");
            }
            println!("Dossier : {}", dir.display());
            Some(0)
        }
        _ => None,
    }
}

/// Jeux dont le nom correspond à la requête : correspondance exacte d'abord,
/// puis sous-chaîne insensible à la casse. Une requête vide ne correspond à rien.
fn matching_games<'a>(games: &'a [config::Game], query: &str) -> Vec<&'a config::Game> {
    let query = query.trim().to_lowercase();
    if query.is_empty() {
        return Vec::new();
    }
    let exact: Vec<&config::Game> = games
        .iter()
        .filter(|game| game.name.to_lowercase() == query)
        .collect();
    if !exact.is_empty() {
        return exact;
    }
    games
        .iter()
        .filter(|game| game.name.to_lowercase().contains(&query))
        .collect()
}

/// Lance un jeu depuis le terminal. La commande reste attachée à la session
/// pour que le journal et les flux du jeu restent ouverts jusqu'à sa fin.
fn launch_from_cli(query: &str) -> Result<(), String> {
    let lib = config::Library::load();
    let matches = matching_games(&lib.games, query);
    match matches.len() {
        0 => {
            return Err(format!(
                "Aucun jeu ne correspond à « {query} ». Utilise `quark list` pour voir la bibliothèque."
            ));
        }
        1 => {}
        _ => {
            let mut message = format!("Plusieurs jeux correspondent à « {query} » :\n");
            for game in &matches {
                message.push_str(&format!("  - {}\n", game.name));
            }
            message.push_str("Précise le nom pour choisir un jeu.");
            return Err(message);
        }
    }
    let game = matches[0];
    let runtimes = proton::detect_runtimes();
    let resolved = proton::resolve(game, &lib.settings, &runtimes)?;
    let mut session = session::start(game, &resolved)
        .map_err(|err| format!("Lancement de {} impossible : {err}", game.name))?;
    println!("{} lancé · {}", game.name, resolved.label());
    println!("journal : {}", session.log.display());
    use std::io::Write as _;
    let _ = std::io::stdout().flush();
    let status = session
        .child
        .wait()
        .map_err(|err| format!("Attente de {} impossible : {err}", game.name))?;
    println!("{} terminé ({status})", game.name);
    Ok(())
}

/// « 1360x860 » -> taille.
fn parse_size(text: &str) -> Option<eframe::egui::Vec2> {
    let (width, height) = text.split_once(['x', 'X'])?;
    Some(eframe::egui::Vec2::new(
        width.trim().parse().ok()?,
        height.trim().parse().ok()?,
    ))
}

/// « 120,340 » -> point.
fn parse_point(text: &str) -> Option<eframe::egui::Pos2> {
    let (x, y) = text.split_once(',')?;
    Some(eframe::egui::Pos2::new(
        x.trim().parse().ok()?,
        y.trim().parse().ok()?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn game(name: &str) -> config::Game {
        config::Game {
            name: name.to_string(),
            ..config::Game::default()
        }
    }

    #[test]
    fn le_lancement_prefere_une_correspondance_exacte() {
        let games = [game("Portal"), game("Portal 2")];
        let matches = matching_games(&games, "portal");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].name, "Portal");
    }

    #[test]
    fn le_lancement_accepte_une_sous_chaine_et_ignore_la_casse() {
        let games = [game("The Witcher 3: Wild Hunt"), game("Celeste")];
        let matches = matching_games(&games, "WITCHER");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].name, "The Witcher 3: Wild Hunt");
    }

    #[test]
    fn le_lancement_signale_les_requetes_vides_ou_inconnues() {
        let games = [game("Portal 2")];
        assert!(matching_games(&games, "  ").is_empty());
        assert!(matching_games(&games, "zzzz").is_empty());
    }
}
