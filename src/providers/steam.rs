//! Adaptateur Steam : manifestes locaux et accès optionnel à GetOwnedGames.

use crate::config::Game;
use crate::providers::achievements::rarity_percent;
use crate::providers::{
    AchievementInfo, AchievementRow, InstallState, Provider, ProviderGame, ProviderGameKey,
};
use reqwest::blocking::Client;
use serde::Deserialize;
use std::collections::HashMap;

const OWNED_GAMES_URL: &str = "https://api.steampowered.com/IPlayerService/GetOwnedGames/v1/";
const PLAYER_ACHIEVEMENTS_URL: &str =
    "https://api.steampowered.com/ISteamUserStats/GetPlayerAchievements/v1/";
const SCHEMA_URL: &str = "https://api.steampowered.com/ISteamUserStats/GetSchemaForGame/v2/";
const GLOBAL_PERCENTAGES_URL: &str =
    "https://api.steampowered.com/ISteamUserStats/GetGlobalAchievementPercentagesForApp/v2/";
const MAX_RESPONSE_BYTES: u64 = 8 * 1024 * 1024;

#[derive(Deserialize)]
struct OwnedResponse {
    response: OwnedPayload,
}

#[derive(Deserialize)]
struct OwnedPayload {
    game_count: usize,
    #[serde(default)]
    games: Vec<OwnedEntry>,
}

#[derive(Deserialize)]
struct OwnedEntry {
    appid: u32,
    #[serde(default)]
    name: String,
}

/// Couverture verticale publiée par Steam, sans clé ni API : le CDN public sert
/// la même image que la bibliothèque Steam.
pub fn cover_url(app_id: u32) -> String {
    format!("https://cdn.cloudflare.steamstatic.com/steam/apps/{app_id}/library_600x900.jpg")
}

/// Convertit les jeux découverts sur disque en enregistrements provider.
/// La fiche Quark n'est ni modifiée ni remplacée ; le lien est uniquement AppID.
pub fn installed_games(
    steam_games: &[crate::steamlib::SteamGame],
    quark_games: &[Game],
) -> Vec<ProviderGame> {
    let mut result: Vec<ProviderGame> = Vec::new();
    for steam in steam_games {
        if steam.app_id == 0
            || result
                .iter()
                .any(|game| game.key.product_id == steam.app_id.to_string())
        {
            continue;
        }
        let appid = steam.app_id.to_string();
        let linked = quark_games
            .iter()
            .find(|game| game.steam_app_id == Some(steam.app_id));
        result.push(ProviderGame {
            key: ProviderGameKey {
                provider: Provider::Steam,
                product_id: appid,
            },
            title: steam.name.clone(),
            owned: true,
            install_root: steam.dir.parent().map(std::path::Path::to_path_buf),
            install_path: Some(steam.dir.clone()),
            custom_save_paths: Vec::new(),
            cover_url: Some(cover_url(steam.app_id)),
            install_exe: None,
            install_state: InstallState::Installed,
            quark_game_id: linked.map(|game| game.id.clone()),
            capabilities: Default::default(),
            last_synced: Some(crate::config::now_ts()),
        });
    }
    result.sort_by_key(|left| left.title.to_lowercase());
    result
}

/// Parse l'API utilisateur de Steam. `GetOwnedGames` renvoie parfois une liste
/// vide lorsque la confidentialité masque les détails ; cette réponse n'est
/// pas considérée comme une bibliothèque confirmée vide.
pub fn parse_owned(json: &str) -> Result<Vec<ProviderGame>, String> {
    if json.len() as u64 > MAX_RESPONSE_BYTES {
        return Err("réponse de bibliothèque Steam trop volumineuse".to_string());
    }
    let payload: OwnedResponse = serde_json::from_str(json)
        .map_err(|_| "réponse de bibliothèque Steam invalide".to_string())?;
    if payload.response.game_count == 0 || payload.response.games.is_empty() {
        return Err(
            "bibliothèque Steam vide ou détails privés : résultat non vérifiable".to_string(),
        );
    }
    if payload.response.game_count != payload.response.games.len() {
        return Err("réponse de bibliothèque Steam incomplète".to_string());
    }
    let mut result = Vec::with_capacity(payload.response.games.len());
    for entry in payload.response.games {
        if entry.appid == 0 {
            return Err("AppID nul dans la bibliothèque Steam".to_string());
        }
        let title = entry.name.trim();
        if title.is_empty() {
            return Err("nom de jeu absent dans la bibliothèque Steam".to_string());
        }
        if result
            .iter()
            .any(|game: &ProviderGame| game.key.product_id == entry.appid.to_string())
        {
            continue;
        }
        result.push(ProviderGame {
            key: ProviderGameKey {
                provider: Provider::Steam,
                product_id: entry.appid.to_string(),
            },
            title: title.to_string(),
            owned: true,
            install_path: None,
            install_root: None,
            custom_save_paths: Vec::new(),
            cover_url: Some(cover_url(entry.appid)),
            install_exe: None,
            install_state: InstallState::Unknown,
            quark_game_id: None,
            capabilities: Default::default(),
            last_synced: Some(crate::config::now_ts()),
        });
    }
    Ok(result)
}

/// Fusionne les résultats Steam avec les enregistrements déjà stockés.
/// Une réponse API absente/échouée (`owned == None`) ne supprime jamais la
/// dernière bibliothèque connue. Les autres boutiques ne sont jamais modifiées.
pub fn merge_sync(
    existing: &[ProviderGame],
    local: &[ProviderGame],
    owned: Option<&[ProviderGame]>,
    quark_games: &[Game],
) -> Vec<ProviderGame> {
    let mut merged: std::collections::BTreeMap<String, ProviderGame> = existing
        .iter()
        .filter(|game| game.key.provider != Provider::Steam)
        .cloned()
        .map(|game| {
            (
                format!("{:?}:{}", game.key.provider, game.key.product_id),
                game,
            )
        })
        .collect();
    let mut steam: std::collections::BTreeMap<String, ProviderGame> =
        std::collections::BTreeMap::new();

    let sources: Vec<&ProviderGame> = match owned {
        Some(records) => records.iter().chain(local.iter()).collect(),
        None => existing
            .iter()
            .filter(|game| game.key.provider == Provider::Steam)
            .chain(local.iter())
            .collect(),
    };
    for incoming in sources {
        if incoming.key.provider != Provider::Steam {
            continue;
        }
        let appid = &incoming.key.product_id;
        let mut record = incoming.clone();
        if let Some(old) = existing
            .iter()
            .find(|old| old.key.provider == Provider::Steam && old.key.product_id == *appid)
        {
            let old_path_alive = old.install_path.as_ref().is_some_and(|path| path.is_dir());
            if record.install_path.is_none() && old_path_alive {
                record.install_path = old.install_path.clone();
            }
            if record.install_state != InstallState::Installed
                && old.install_state == InstallState::Installed
                && old_path_alive
            {
                record.install_state = old.install_state;
            }
            if record.quark_game_id.is_none() {
                record.quark_game_id = old.quark_game_id.clone();
            }
            if record.cover_url.is_none() {
                record.cover_url = old.cover_url.clone();
            }
            if record.install_exe.is_none() {
                record.install_exe = old.install_exe.clone();
            }
            let mut custom_paths = old.custom_save_paths.clone();
            for path in &record.custom_save_paths {
                if !custom_paths.contains(path) {
                    custom_paths.push(path.clone());
                }
            }
            record.custom_save_paths = custom_paths;
            if record.last_synced.is_none() {
                record.last_synced = old.last_synced;
            }
        }
        if let Some(game) = quark_games.iter().find(|game| {
            game.steam_app_id
                .and_then(|id| (id.to_string() == *appid).then_some(id))
                .is_some()
        }) {
            record.quark_game_id = Some(game.id.clone());
        }
        match steam.get(appid) {
            Some(previous)
                if previous.install_state == InstallState::Installed
                    && record.install_state != InstallState::Installed
                    && previous
                        .install_path
                        .as_ref()
                        .is_some_and(|path| path.is_dir()) => {}
            _ => {
                steam.insert(appid.clone(), record);
            }
        }
    }
    merged.extend(
        steam
            .into_iter()
            .map(|(id, game)| (format!("Steam:{id}"), game)),
    );
    merged.into_values().collect()
}

/// Appelle uniquement le host HTTPS public, avec une clé utilisateur locale.
pub fn owned(client: &Client, steam_id: &str, api_key: &str) -> Result<Vec<ProviderGame>, String> {
    if !valid_steam_id(steam_id) {
        return Err("SteamID64 invalide".to_string());
    }
    if api_key.trim().is_empty() {
        return Err("clé Steam Web API absente".to_string());
    }
    let response = client
        .get(OWNED_GAMES_URL)
        .query(&[
            ("key", api_key),
            ("steamid", steam_id),
            ("include_appinfo", "1"),
            ("include_played_free_games", "1"),
        ])
        .send()
        .map_err(|_| "connexion à Steam Web API impossible".to_string())?;
    if !response.status().is_success() {
        return Err(format!(
            "Steam Web API a répondu {}",
            response.status().as_u16()
        ));
    }
    let mut bytes = Vec::new();
    use std::io::Read;
    response
        .take(MAX_RESPONSE_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "lecture de la bibliothèque Steam impossible".to_string())?;
    if bytes.len() as u64 > MAX_RESPONSE_BYTES {
        return Err("réponse de bibliothèque Steam trop volumineuse".to_string());
    }
    let json = std::str::from_utf8(&bytes).map_err(|_| "réponse Steam non UTF-8".to_string())?;
    parse_owned(json)
}

pub fn valid_steam_id(id: &str) -> bool {
    id.len() == 17
        && id.bytes().all(|byte| byte.is_ascii_digit())
        && id
            .parse::<u64>()
            .is_ok_and(|value| value >= 76_561_197_960_265_728)
}

pub fn install_uri(app_id: u32) -> Result<String, String> {
    if app_id == 0 {
        return Err("AppID Steam invalide".to_string());
    }
    Ok(format!("steam://install/{app_id}"))
}

#[derive(Debug, Default, Deserialize)]
struct PlayerResponse {
    #[serde(default)]
    playerstats: Option<PlayerStats>,
}

#[derive(Debug, Default, Deserialize)]
struct PlayerStats {
    #[serde(default)]
    success: bool,
    #[serde(default)]
    achievements: Vec<PlayerAchievement>,
    #[serde(default)]
    error: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct PlayerAchievement {
    #[serde(default)]
    apiname: String,
    #[serde(default)]
    achieved: u8,
}

#[derive(Debug, Default, Deserialize)]
struct SchemaResponse {
    #[serde(default)]
    game: Option<SchemaGame>,
}

#[derive(Debug, Default, Deserialize)]
struct SchemaGame {
    #[serde(default, rename = "availableGameStats")]
    available_game_stats: Option<SchemaStats>,
}

#[derive(Debug, Default, Deserialize)]
struct SchemaStats {
    #[serde(default)]
    achievements: Vec<SchemaAchievement>,
}

#[derive(Debug, Default, Deserialize)]
struct SchemaAchievement {
    #[serde(default)]
    name: String,
    #[serde(default, rename = "displayName")]
    display_name: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    hidden: Option<serde_json::Value>,
    #[serde(default)]
    icon: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct GlobalResponse {
    #[serde(default, rename = "achievementpercentages")]
    achievement_percentages: Option<GlobalStats>,
}

#[derive(Debug, Default, Deserialize)]
struct GlobalStats {
    #[serde(default)]
    achievements: Vec<GlobalAchievement>,
}

#[derive(Debug, Default, Deserialize)]
struct GlobalAchievement {
    #[serde(default)]
    name: String,
    #[serde(default)]
    percent: Option<serde_json::Value>,
}

fn schema_hidden(value: Option<&serde_json::Value>) -> bool {
    match value {
        Some(serde_json::Value::Bool(hidden)) => *hidden,
        Some(serde_json::Value::Number(number)) => number.as_u64().is_some_and(|value| value != 0),
        _ => false,
    }
}

/// Assemble les succès du joueur avec le schéma du jeu et les pourcentages
/// globaux. Les trois réponses Steam sont séparées ; cette fonction pure
/// permet de tester les cas où le schéma ou la rareté manquent.
pub fn parse_achievements(
    player_json: &str,
    schema_json: Option<&str>,
    global_json: Option<&str>,
) -> Result<AchievementInfo, String> {
    let player: PlayerResponse = serde_json::from_str(player_json)
        .map_err(|_| "réponse de succès Steam illisible".to_string())?;
    let stats = player
        .playerstats
        .ok_or_else(|| "Steam n'a pas renvoyé les succès de ce jeu".to_string())?;
    if !stats.success {
        return Err(stats
            .error
            .filter(|error| !error.trim().is_empty())
            .unwrap_or_else(|| "Steam n'expose pas les succès de ce jeu".to_string()));
    }

    let schema = schema_json
        .and_then(|json| serde_json::from_str::<SchemaResponse>(json).ok())
        .and_then(|response| response.game)
        .and_then(|game| game.available_game_stats)
        .map(|stats| {
            stats
                .achievements
                .into_iter()
                .filter(|entry| !entry.name.trim().is_empty())
                .map(|entry| (entry.name.to_lowercase(), entry))
                .collect::<HashMap<_, _>>()
        })
        .unwrap_or_default();
    let global = global_json
        .and_then(|json| serde_json::from_str::<GlobalResponse>(json).ok())
        .and_then(|response| response.achievement_percentages)
        .map(|stats| {
            stats
                .achievements
                .into_iter()
                .filter(|entry| !entry.name.trim().is_empty())
                .filter_map(|entry| {
                    rarity_percent(entry.percent.as_ref())
                        .map(|percent| (entry.name.to_lowercase(), percent))
                })
                .collect::<HashMap<_, _>>()
        })
        .unwrap_or_default();

    let rows = stats
        .achievements
        .into_iter()
        .filter(|achievement| !achievement.apiname.trim().is_empty())
        .map(|achievement| {
            let schema_entry = schema.get(&achievement.apiname.to_lowercase());
            let hidden = schema_entry
                .map(|entry| schema_hidden(entry.hidden.as_ref()))
                .unwrap_or(false);
            let description = schema_entry
                .map(|entry| entry.description.trim().to_string())
                .filter(|description| !description.is_empty())
                .unwrap_or_else(|| {
                    if hidden {
                        "Succès masqué".to_string()
                    } else {
                        String::new()
                    }
                });
            AchievementRow {
                key: achievement.apiname.clone(),
                id: String::new(),
                name: schema_entry
                    .map(|entry| entry.display_name.trim().to_string())
                    .filter(|name| !name.is_empty())
                    .unwrap_or_else(|| achievement.apiname.clone()),
                description,
                unlocked: achievement.achieved != 0,
                progress: if achievement.achieved != 0 { 1.0 } else { 0.0 },
                session: false,
                rarity: global.get(&achievement.apiname.to_lowercase()).copied(),
                icon_url: schema_entry.and_then(|entry| entry.icon.clone()),
                icon_path: None,
            }
        })
        .collect::<Vec<_>>();
    let mut info = AchievementInfo {
        total: rows.len() as u32,
        unlocked: 0,
        rows,
    };
    info.normalize();
    Ok(info)
}

fn steam_get(
    client: &Client,
    url: &str,
    query: &[(&str, &str)],
    label: &str,
) -> Result<String, String> {
    let response = client
        .get(url)
        .query(query)
        .send()
        .map_err(|_| format!("connexion à {label} impossible"))?;
    let status = response.status();
    if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN {
        return Err("Steam Web API refuse la clé ou le profil est privé".to_string());
    }
    if !status.is_success() {
        return Err(format!("{label} a répondu {}", status.as_u16()));
    }
    crate::net::text(response, MAX_RESPONSE_BYTES, label)
}

/// Lit les succès Steam avec la clé Web et le SteamID64 déjà configurés pour
/// la bibliothèque. Le schéma et la rareté sont facultatifs : un jeu sans
/// schéma public garde quand même ses noms d'API et ses déblocages.
pub fn fetch_achievements(
    app_id: &str,
    api_key: &str,
    steam_id: &str,
) -> Result<AchievementInfo, String> {
    let app_id = app_id
        .parse::<u32>()
        .ok()
        .filter(|id| *id != 0)
        .ok_or_else(|| "AppID Steam invalide".to_string())?;
    if !valid_steam_id(steam_id) {
        return Err("SteamID64 invalide".to_string());
    }
    if api_key.trim().is_empty() {
        return Err("clé Steam Web API absente".to_string());
    }
    let app_id_text = app_id.to_string();
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .map_err(|_| "client HTTP Steam indisponible".to_string())?;
    let player = steam_get(
        &client,
        PLAYER_ACHIEVEMENTS_URL,
        &[
            ("key", api_key),
            ("steamid", steam_id),
            ("appid", &app_id_text),
            ("l", "english"),
        ],
        "succès Steam",
    )?;
    let schema = steam_get(
        &client,
        SCHEMA_URL,
        &[("key", api_key), ("appid", &app_id_text), ("l", "english")],
        "schéma des succès Steam",
    )
    .ok();
    let global = steam_get(
        &client,
        GLOBAL_PERCENTAGES_URL,
        &[("gameid", &app_id_text)],
        "rareté des succès Steam",
    )
    .ok();
    parse_achievements(&player, schema.as_deref(), global.as_deref())
}

/// Demande au client Steam installé de démarrer l'installation.
/// Le succès de l'ouverture ne constitue pas une preuve d'installation.
pub fn request_install(app_id: u32) -> Result<(), String> {
    let uri = install_uri(app_id)?;
    let mut command = std::process::Command::new("xdg-open");
    command
        .arg(uri)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());
    crate::process::spawn_and_reap(&mut command)
        .map_err(|_| "impossible d'ouvrir Steam (xdg-open indisponible)".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn jeux_installes_sont_lies_par_appid_sans_ecraser_la_fiche_quark() {
        let mut quark = Game::new("Renommé dans Quark", "/games/portal2/portal2.exe");
        quark.steam_app_id = Some(620);
        quark.artwork.hero = Some(PathBuf::from("/artwork/custom-hero.jpg"));
        let duplicates = vec![
            crate::steamlib::SteamGame {
                app_id: 620,
                name: "Portal 2".to_string(),
                dir: PathBuf::from("/steam/common/Portal 2"),
                exe: None,
            },
            crate::steamlib::SteamGame {
                app_id: 620,
                name: "Portal 2 duplicate".to_string(),
                dir: PathBuf::from("/other/Portal 2"),
                exe: None,
            },
        ];
        let result = installed_games(&duplicates, &[quark.clone()]);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].key.provider, Provider::Steam);
        assert_eq!(result[0].key.product_id, "620");
        assert_eq!(result[0].title, "Portal 2");
        assert_eq!(
            result[0].install_path,
            Some(PathBuf::from("/steam/common/Portal 2"))
        );
        assert_eq!(result[0].install_state, InstallState::Installed);
        assert_eq!(result[0].quark_game_id.as_deref(), Some(quark.id.as_str()));
        assert_eq!(quark.name, "Renommé dans Quark");
        assert_eq!(
            quark.artwork.hero,
            Some(PathBuf::from("/artwork/custom-hero.jpg"))
        );
    }

    #[test]
    fn api_steam_parse_les_jeux_et_refuse_la_reponse_privee_ambigue() {
        let json = r#"{"response":{"game_count":1,"games":[{"appid":620,"name":"Portal 2","playtime_forever":42}]}}"#;
        let games = parse_owned(json).unwrap();
        assert_eq!(games.len(), 1);
        assert_eq!(games[0].key.product_id, "620");
        assert_eq!(games[0].title, "Portal 2");
        assert!(games[0].owned);
        assert_eq!(games[0].install_state, InstallState::Unknown);
        let ambiguous = r#"{"response":{"game_count":0,"games":[]}}"#;
        assert!(parse_owned(ambiguous).is_err());
        assert!(parse_owned(r#"{"response":{}}"#).is_err());
    }

    #[test]
    fn echec_api_preserve_la_bibliotheque_connue_et_les_autres_fournisseurs() {
        let steam = ProviderGame {
            key: ProviderGameKey {
                provider: Provider::Steam,
                product_id: "620".into(),
            },
            title: "Portal 2".into(),
            owned: true,
            install_path: Some(PathBuf::from("/steam/Portal 2")),
            install_root: None,
            custom_save_paths: Vec::new(),
            cover_url: None,
            install_exe: None,
            install_state: InstallState::Installed,
            quark_game_id: None,
            capabilities: Default::default(),
            last_synced: Some(1),
        };
        let epic = ProviderGame {
            key: ProviderGameKey {
                provider: Provider::Epic,
                product_id: "epic-portal".into(),
            },
            title: "Portal".into(),
            owned: true,
            install_path: None,
            install_root: None,
            custom_save_paths: Vec::new(),
            cover_url: None,
            install_exe: None,
            install_state: InstallState::Unknown,
            quark_game_id: None,
            capabilities: Default::default(),
            last_synced: None,
        };
        let merged = merge_sync(&[steam.clone(), epic.clone()], &[], None, &[]);
        assert_eq!(merged.len(), 2);
        assert!(
            merged
                .iter()
                .any(|game| game.key == steam.key && game.install_path == steam.install_path)
        );
        assert!(merged.iter().any(|game| game.key == epic.key));
    }

    #[test]
    fn une_synchro_steam_preserve_les_dossiers_de_sauvegarde_personnalises() {
        let save = PathBuf::from("/home/joueur/sauvegardes/portal-2");
        let existing = ProviderGame {
            key: ProviderGameKey {
                provider: Provider::Steam,
                product_id: "620".into(),
            },
            title: "Portal 2".into(),
            owned: true,
            custom_save_paths: vec![save.clone()],
            install_state: InstallState::Installed,
            last_synced: Some(1),
            ..Default::default()
        };
        let refreshed = parse_owned(
            r#"{"response":{"game_count":1,"games":[{"appid":620,"name":"Portal 2"}]}}"#,
        )
        .unwrap();
        // La fusion doit tenir que la liste possédée soit disponible ou non.
        for owned in [Some(refreshed.as_slice()), None] {
            let merged = merge_sync(std::slice::from_ref(&existing), &[], owned, &[]);
            assert_eq!(merged.len(), 1);
            assert_eq!(merged[0].custom_save_paths, vec![save.clone()]);
        }
    }

    #[test]
    fn une_synchro_steam_oublie_un_jeu_installe_qui_a_disparu_du_disque() {
        let dir = std::env::temp_dir().join(format!(
            "quark-steam-merge-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let existing = ProviderGame {
            key: ProviderGameKey {
                provider: Provider::Steam,
                product_id: "620".into(),
            },
            title: "Portal 2".into(),
            owned: true,
            install_path: Some(dir.clone()),
            install_root: None,
            custom_save_paths: Vec::new(),
            cover_url: None,
            install_exe: None,
            install_state: InstallState::Installed,
            quark_game_id: None,
            capabilities: Default::default(),
            last_synced: Some(1),
        };
        let refreshed = parse_owned(
            r#"{"response":{"game_count":1,"games":[{"appid":620,"name":"Portal 2"}]}}"#,
        )
        .unwrap();
        let merged = merge_sync(
            std::slice::from_ref(&existing),
            &[],
            Some(refreshed.as_slice()),
            &[],
        );
        assert_eq!(merged[0].install_state, InstallState::Installed);
        assert_eq!(merged[0].install_path.as_deref(), Some(dir.as_path()));

        std::fs::remove_dir_all(&dir).unwrap();
        let merged = merge_sync(&[existing], &[], Some(refreshed.as_slice()), &[]);
        assert_eq!(merged[0].install_state, InstallState::Unknown);
        assert!(merged[0].install_path.is_none());
    }

    #[test]
    fn les_succes_steam_fusionnent_schema_raretes_et_deblocages() {
        let player = r#"{"playerstats":{"success":true,"achievements":[
            {"apiname":"ACH_WIN","achieved":1,"unlocktime":1700000000},
            {"apiname":"ACH_SECRET","achieved":0,"unlocktime":0}
        ]}}"#;
        let schema = r#"{"game":{"availableGameStats":{"achievements":[
            {"name":"ACH_WIN","displayName":"Victoire","description":"Terminer le jeu","hidden":0,"icon":"https://cdn/win.jpg"},
            {"name":"ACH_SECRET","displayName":"Secret","description":"","hidden":1}
        ]}}}"#;
        let global = r#"{"achievementpercentages":{"achievements":[
            {"name":"ACH_WIN","percent":42.5},
            {"name":"ACH_SECRET","percent":"3,2"}
        ]}}"#;
        let info = parse_achievements(player, Some(schema), Some(global)).unwrap();
        assert_eq!(info.total, 2);
        assert_eq!(info.unlocked, 1);
        let win = info.rows.iter().find(|row| row.key == "ACH_WIN").unwrap();
        assert_eq!(win.name, "Victoire");
        assert!(win.unlocked);
        assert_eq!(win.rarity, Some(42.5));
        assert_eq!(win.icon_url.as_deref(), Some("https://cdn/win.jpg"));
        let secret = info
            .rows
            .iter()
            .find(|row| row.key == "ACH_SECRET")
            .unwrap();
        assert_eq!(secret.description, "Succès masqué");
        assert_eq!(secret.rarity, Some(3.2));
    }

    #[test]
    fn les_succes_steam_signalent_un_jeu_sans_statistiques() {
        let response = r#"{"playerstats":{"success":false,"error":"Requested app has no stats"}}"#;
        let error = parse_achievements(response, None, None).unwrap_err();
        assert!(error.contains("no stats"));
        assert!(
            parse_achievements(r#"{"playerstats":{"success":true}}"#, None, None)
                .unwrap()
                .rows
                .is_empty()
        );
    }

    #[test]
    fn steamid_et_uri_installation_sont_valides_separement() {
        assert!(valid_steam_id("76561198000000000"));
        assert!(!valid_steam_id("123"));
        assert!(!valid_steam_id("7656119800000000x"));
        assert!(valid_steam_id("76561197960265728"));
        assert_eq!(install_uri(620).unwrap(), "steam://install/620");
        assert!(install_uri(0).is_err());
    }
}
