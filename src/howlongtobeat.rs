//! Temps de jeu HowLongToBeat d'un jeu.
//!
//! HowLongToBeat n'expose pas d'API officielle : le site utilise un endpoint
//! de recherche protégé par un jeton temporaire, lié à l'adresse IP et au
//! User-Agent. On le récupère juste avant la recherche, puis on appelle
//! l'endpoint avec exactement le même User-Agent. Aucune clé n'est demandée.
//!
//! Les temps renvoyés sont des secondes ; un zéro signifie « pas encore de
//! temps soumis », jamais « 0 heure ». L'intégration reste donc facultative :
//! une panne ou un changement de site affiche un message discret dans
//! l'aperçu, sans bloquer la fiche du jeu.

use serde_json::{Value, json};
use std::time::{SystemTime, UNIX_EPOCH};

/// Même User-Agent pour le jeton et la recherche : le serveur le vérifie.
const USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 \
     (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36";

/// Temps HowLongToBeat, en heures (arrondis à une décimale).
#[derive(Debug, Clone, PartialEq)]
pub struct Report {
    pub game_id: u64,
    pub name: String,
    pub main: Option<f64>,
    pub main_extra: Option<f64>,
    pub completionist: Option<f64>,
}

impl Report {
    /// Page HowLongToBeat du jeu.
    pub fn url(&self) -> String {
        format!("https://howlongtobeat.com/game/{}", self.game_id)
    }

    /// Temps le plus utile pour un aperçu : l'histoire principale, sinon
    /// l'histoire avec les extras.
    pub fn headline(&self) -> Option<f64> {
        self.main.or(self.main_extra)
    }

    /// « 51,6 h » ou « 104 h ».
    pub fn hours_label(hours: f64) -> String {
        if hours >= 10.0 {
            format!("{hours:.0} h")
        } else {
            format!("{hours:.1} h").replace('.', ",")
        }
    }
}

#[derive(Debug)]
struct Token {
    value: String,
    hp_key: Option<String>,
    hp_val: Option<String>,
}

#[derive(Debug, Default, serde::Deserialize)]
struct RawGame {
    #[serde(default)]
    game_id: u64,
    #[serde(default)]
    game_name: String,
    #[serde(default)]
    game_type: String,
    #[serde(default)]
    comp_main: f64,
    #[serde(default)]
    comp_plus: f64,
    #[serde(default)]
    comp_100: f64,
    #[serde(default)]
    comp_main_count: u64,
}

#[derive(Debug, Default, serde::Deserialize)]
struct RawSearch {
    #[serde(default)]
    data: Vec<RawGame>,
}

/// Interroge HowLongToBeat pour `title`.
pub fn fetch(client: &reqwest::blocking::Client, title: &str) -> Result<Report, String> {
    let title = title.trim();
    if title.is_empty() {
        return Err("titre vide".to_string());
    }
    // Le site a déjà déplacé son endpoint plusieurs fois. Le chemin actuel est
    // essayé d'abord, puis l'ancien : les deux renvoient le même format.
    let endpoints = [
        ("api/search/site/init", "api/search/site"),
        ("api/find/init", "api/find"),
    ];
    let mut last_error = String::new();
    for (init, endpoint) in endpoints {
        let token = match fetch_token(client, init) {
            Ok(token) => token,
            Err(err) => {
                last_error = err;
                continue;
            }
        };
        match search(client, endpoint, title, &token) {
            Ok(Some(report)) => return Ok(report),
            Ok(None) => return Err("aucun temps trouvé pour ce jeu".to_string()),
            Err(err) => last_error = err,
        }
    }
    Err(if last_error.is_empty() {
        "HowLongToBeat injoignable".to_string()
    } else {
        last_error
    })
}

fn fetch_token(client: &reqwest::blocking::Client, path: &str) -> Result<Token, String> {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0);
    let url = format!("https://howlongtobeat.com/{path}?t={millis}");
    let response = client
        .get(url)
        .header(reqwest::header::USER_AGENT, USER_AGENT)
        .header(reqwest::header::REFERER, "https://howlongtobeat.com/")
        .send()
        .map_err(|err| format!("HowLongToBeat injoignable ({err})"))?;
    if !response.status().is_success() {
        return Err(format!(
            "HowLongToBeat a refusé la connexion ({})",
            response.status().as_u16()
        ));
    }
    let raw: Value = response
        .json()
        .map_err(|err| format!("jeton HowLongToBeat illisible ({err})"))?;
    let value = raw
        .get("token")
        .and_then(Value::as_str)
        .filter(|token| !token.is_empty())
        .ok_or_else(|| "jeton HowLongToBeat absent".to_string())?;
    Ok(Token {
        value: value.to_string(),
        hp_key: raw.get("hpKey").and_then(Value::as_str).map(str::to_string),
        hp_val: raw.get("hpVal").and_then(Value::as_str).map(str::to_string),
    })
}

fn search(
    client: &reqwest::blocking::Client,
    path: &str,
    title: &str,
    token: &Token,
) -> Result<Option<Report>, String> {
    let terms: Vec<&str> = title.split_whitespace().collect();
    let mut payload = json!({
        "searchType": "games",
        "searchTerms": terms,
        "searchPage": 1,
        "size": 12,
        "searchOptions": {
            "games": {
                "userId": 0,
                "platform": "",
                "sortCategory": "popular",
                "rangeCategory": "main",
                "rangeTime": {"min": null, "max": null},
                "gameplay": {"perspective": "", "flow": "", "genre": "", "difficulty": ""},
                "rangeYear": {"min": "", "max": ""},
                "modifier": ""
            },
            "users": {"sortCategory": "postcount"},
            "lists": {"sortCategory": "follows"},
            "filter": "",
            "sort": 0,
            "randomizer": 0
        },
        "useCache": true
    });
    // Les anciennes versions ajoutaient un champ piège dont le nom change à
    // chaque jeton ; il est inutile aujourd'hui, mais reste accepté.
    if let (Some(key), Some(value), Some(object)) =
        (&token.hp_key, &token.hp_val, payload.as_object_mut())
    {
        object.insert(key.clone(), Value::String(value.clone()));
    }

    let mut request = client
        .post(format!("https://howlongtobeat.com/{path}"))
        .header(reqwest::header::USER_AGENT, USER_AGENT)
        .header(reqwest::header::REFERER, "https://howlongtobeat.com/")
        .header(reqwest::header::ORIGIN, "https://howlongtobeat.com")
        .header("x-auth-token", &token.value)
        .json(&payload);
    if let (Some(key), Some(value)) = (&token.hp_key, &token.hp_val) {
        request = request.header("x-hp-key", key).header("x-hp-val", value);
    }
    let response = request
        .send()
        .map_err(|err| format!("recherche HowLongToBeat impossible ({err})"))?;
    if !response.status().is_success() {
        return Err(format!(
            "HowLongToBeat a répondu {}",
            response.status().as_u16()
        ));
    }
    let raw: RawSearch = response
        .json()
        .map_err(|err| format!("réponse HowLongToBeat illisible ({err})"))?;
    Ok(best_match(title, &raw.data))
}

/// Choisit le résultat de base le plus proche du titre demandé.
fn best_match(title: &str, games: &[RawGame]) -> Option<Report> {
    let query = normalize(title);
    let has_base = games
        .iter()
        .any(|game| game.game_type.eq_ignore_ascii_case("game"));
    let mut candidates: Vec<&RawGame> = games
        .iter()
        .filter(|game| !has_base || game.game_type.eq_ignore_ascii_case("game"))
        .collect();
    // Tri stable : à pertinence égale, l'ordre de popularité du site est gardé.
    candidates.sort_by_key(|game| {
        let name = normalize(&game.game_name);
        let relevance = if name == query {
            0
        } else if name.starts_with(&query) {
            1
        } else if name.contains(&query) {
            2
        } else if query.contains(&name) {
            3
        } else {
            9
        };
        let has_times = game.comp_main > 0.0 || game.comp_plus > 0.0 || game.comp_100 > 0.0;
        (relevance, !has_times, game.comp_main_count == 0)
    });
    candidates
        .into_iter()
        .find(|game| !game.game_name.is_empty() && game.game_id > 0)
        .filter(|game| {
            let name = normalize(&game.game_name);
            name == query
                || name.starts_with(&query)
                || name.contains(&query)
                || query.contains(&name)
        })
        .map(|game| Report {
            game_id: game.game_id,
            name: game.game_name.clone(),
            main: hours(game.comp_main),
            main_extra: hours(game.comp_plus),
            completionist: hours(game.comp_100),
        })
}

fn normalize(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

fn hours(seconds: f64) -> Option<f64> {
    (seconds > 0.0).then(|| (seconds / 3600.0 * 10.0).round() / 10.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn game(id: u64, name: &str, kind: &str, main: f64, plus: f64, hundred: f64) -> RawGame {
        RawGame {
            game_id: id,
            game_name: name.to_string(),
            game_type: kind.to_string(),
            comp_main: main * 3600.0,
            comp_plus: plus * 3600.0,
            comp_100: hundred * 3600.0,
            comp_main_count: 10,
        }
    }

    #[test]
    fn les_secondes_deviennent_des_heures() {
        assert_eq!(hours(185_989.0), Some(51.7));
        assert_eq!(hours(0.0), None);
        assert_eq!(Report::hours_label(51.6), "52 h");
        assert_eq!(Report::hours_label(8.3), "8,3 h");
        assert_eq!(Report::hours_label(103.8), "104 h");
    }

    #[test]
    fn le_meilleur_resultat_ignore_les_dlc() {
        let games = vec![
            game(
                1,
                "The Witcher 3: Wild Hunt - Blood and Wine",
                "dlc",
                10.0,
                12.0,
                15.0,
            ),
            game(2, "The Witcher 3: Wild Hunt", "game", 51.6, 103.8, 164.0),
            game(3, "The Witcher 3", "game", 50.0, 100.0, 160.0),
        ];
        let report = best_match("The Witcher 3: Wild Hunt", &games).unwrap();
        assert_eq!(report.game_id, 2);
        assert_eq!(report.main, Some(51.6));
        assert_eq!(report.main_extra, Some(103.8));
        assert_eq!(report.completionist, Some(164.0));
        assert_eq!(report.url(), "https://howlongtobeat.com/game/2");
    }

    #[test]
    fn un_titre_sans_rapport_ne_donne_aucun_temps() {
        let games = vec![game(1, "Un autre jeu", "game", 4.0, 5.0, 6.0)];
        assert!(best_match("Celeste", &games).is_none());
    }
}
