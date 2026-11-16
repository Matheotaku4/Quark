//! Bibliothèque Epic Games lue directement avec la session OAuth de Quark.
//!
//! Legendary reste utilisé pour installer et lancer les jeux, mais il n'est
//! plus nécessaire pour afficher la bibliothèque : l'API GraphQL du launcher
//! expose les mêmes possessions, avec le titre et les jaquettes.

use super::model::{
    CapabilityState, GameCapabilities, InstallState, Provider, ProviderGame, ProviderGameKey,
};
use crate::net;
use serde_json::{Value, json};
use std::time::Duration;

const GRAPHQL_URL: &str = "https://launcher.store.epicgames.com/graphql";
const MAX_PAGES: u32 = 100;
const QUERY: &str = r#"
query quarkLibrary($locale: String, $cursor: String, $excludeNs: [String]) {
  Library {
    libraryItems(cursor: $cursor, params: {excludeNs: $excludeNs, includeMetadata: true}) {
      records {
        catalogItemId
        namespace
        appName
        sandboxType
        catalogItem(locale: $locale) {
          id
          title
          categories { path }
          keyImages { type url }
          mainGameItem { id }
        }
      }
      responseMetadata { nextCursor }
    }
  }
}
"#;

fn http() -> reqwest::blocking::Client {
    crate::artwork::http()
}

fn gql_page(
    access_token: &str,
    cursor: Option<&str>,
) -> Result<(Vec<ProviderGame>, Option<String>), String> {
    let response = http()
        .post(GRAPHQL_URL)
        .bearer_auth(access_token)
        .header("User-Agent", "UELauncher/Quark")
        .timeout(Duration::from_secs(30))
        .json(&json!({
            "query": QUERY,
            "variables": {
                "locale": "en-US",
                "cursor": cursor,
                "excludeNs": ["ue"],
            }
        }))
        .send()
        .map_err(|_| "Epic injoignable pour la bibliothèque".to_string())?;
    let status = response.status();
    if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN {
        return Err(
            "Epic a refusé la session : reconnecte le compte dans Paramètres → Boutiques & outils"
                .to_string(),
        );
    }
    if !status.is_success() {
        return Err(format!("Epic a répondu {status} pour la bibliothèque"));
    }
    let body = net::text(response, net::MAX_JSON, "bibliothèque Epic")
        .map_err(|_| "bibliothèque Epic illisible".to_string())?;
    let root: Value =
        serde_json::from_str(&body).map_err(|_| "bibliothèque Epic illisible".to_string())?;
    if let Some(errors) = root.get("errors").and_then(Value::as_array) {
        let message = errors
            .iter()
            .find_map(|error| error.get("message").and_then(Value::as_str))
            .unwrap_or("réponse GraphQL en erreur");
        return Err(format!("Epic : {message}"));
    }
    let data = root
        .get("data")
        .and_then(|data| {
            data.get("Launcher")
                .or_else(|| data.get("Library"))
                .and_then(|section| section.get("libraryItems"))
        })
        .ok_or_else(|| "bibliothèque Epic illisible (libraryItems absent)".to_string())?;
    let records = data
        .get("records")
        .and_then(Value::as_array)
        .ok_or_else(|| "bibliothèque Epic illisible (records absents)".to_string())?;
    let mut games = Vec::new();
    for record in records {
        if let Some(game) = parse_record(record) {
            games.push(game);
        }
    }
    let cursor = data
        .get("responseMetadata")
        .and_then(|metadata| metadata.get("nextCursor"))
        .and_then(Value::as_str)
        .map(str::to_string);
    Ok((games, cursor))
}

fn parse_record(record: &Value) -> Option<ProviderGame> {
    let namespace = record.get("namespace")?.as_str()?.trim();
    if namespace.is_empty() || namespace == "ue" {
        return None;
    }
    if record
        .get("sandboxType")
        .and_then(Value::as_str)
        .is_some_and(|kind| kind.eq_ignore_ascii_case("private"))
    {
        return None;
    }
    let app_name = record.get("appName")?.as_str()?.trim();
    if app_name.is_empty() || app_name == "1" {
        return None;
    }
    let catalog = record.get("catalogItem").unwrap_or(&Value::Null);
    // Un DLC n'est pas un jeu lançable séparément : Legendary et Quark le
    // rattachent au jeu principal.
    if catalog
        .get("mainGameItem")
        .is_some_and(|item| !item.is_null())
    {
        return None;
    }
    let categories: Vec<&str> = catalog
        .get("categories")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.get("path").and_then(Value::as_str))
                .collect()
        })
        .unwrap_or_default();
    if categories.contains(&"mods") {
        return None;
    }
    let title = catalog
        .get("title")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|title| !title.is_empty())
        .unwrap_or(app_name)
        .to_string();
    let cover_url = catalog
        .get("keyImages")
        .and_then(Value::as_array)
        .and_then(|images| {
            const PREFERRED: [&str; 5] = [
                "DieselGameBoxTall",
                "OfferImageTall",
                "Thumbnail",
                "DieselGameBox",
                "OfferImageWide",
            ];
            for kind in PREFERRED {
                if let Some(url) = images.iter().find_map(|image| {
                    (image.get("type").and_then(Value::as_str) == Some(kind))
                        .then(|| image.get("url").and_then(Value::as_str))
                        .flatten()
                        .map(str::trim)
                        .filter(|url| url.starts_with("https://") || url.starts_with("http://"))
                        .map(str::to_string)
                }) {
                    return Some(url);
                }
            }
            None
        });
    Some(ProviderGame {
        key: ProviderGameKey {
            provider: Provider::Epic,
            product_id: app_name.to_string(),
        },
        title,
        owned: true,
        cover_url,
        install_state: InstallState::NotInstalled,
        capabilities: GameCapabilities {
            achievements: CapabilityState::Unknown,
            cloud_saves: CapabilityState::Unknown,
        },
        ..Default::default()
    })
}

/// Toute la bibliothèque possédée, page par page. Les doublons d'app name sont
/// fusionnés en gardant la première fiche (la plus récemment acquise).
pub fn library(now: i64) -> Result<Vec<ProviderGame>, String> {
    let tokens = super::epic_account::usable_tokens(now)?;
    let mut games = Vec::new();
    let mut cursor: Option<String> = None;
    for _ in 0..MAX_PAGES {
        let (page, next) = gql_page(&tokens.access_token, cursor.as_deref())?;
        games.extend(page);
        match next {
            Some(next) if !next.is_empty() => cursor = Some(next),
            _ => break,
        }
    }
    let mut seen = std::collections::HashSet::new();
    games.retain(|game| seen.insert(game.key.clone()));
    if games.is_empty() {
        return Err(
            "bibliothèque Epic vide : aucune possession renvoyée par le compte".to_string(),
        );
    }
    for game in &mut games {
        game.last_synced = Some(now);
    }
    games.sort_by_key(|game| game.title.to_lowercase());
    Ok(games)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn une_fiche_epic_garde_le_titre_et_la_jaquette() {
        let record = json!({
            "namespace": "ns",
            "catalogItemId": "cat",
            "appName": "AppName",
            "sandboxType": "PUBLICGAME",
            "catalogItem": {
                "id": "cat",
                "title": "Un jeu",
                "categories": [{"path": "games"}],
                "keyImages": [
                    {"type": "Thumbnail", "url": "https://img.example/thumb.jpg"},
                    {"type": "DieselGameBoxTall", "url": "https://img.example/cover.jpg"}
                ]
            }
        });
        let game = parse_record(&record).unwrap();
        assert_eq!(game.key.product_id, "AppName");
        assert_eq!(game.title, "Un jeu");
        assert_eq!(
            game.cover_url.as_deref(),
            Some("https://img.example/cover.jpg")
        );
    }

    #[test]
    fn les_dlc_les_mods_et_les_objets_ue_sont_ecartes() {
        let dlc = json!({
            "namespace": "ns", "appName": "dlc",
            "catalogItem": {"title": "DLC", "mainGameItem": {"id": "base"}}
        });
        let ue = json!({"namespace": "ue", "appName": "asset", "catalogItem": {"title": "Asset"}});
        let private = json!({
            "namespace": "ns", "appName": "private", "sandboxType": "PRIVATE",
            "catalogItem": {"title": "Private"}
        });
        let r#mod = json!({
            "namespace": "ns", "appName": "mod",
            "catalogItem": {"title": "Mod", "categories": [{"path": "mods"}]}
        });
        assert!(parse_record(&dlc).is_none());
        assert!(parse_record(&ue).is_none());
        assert!(parse_record(&private).is_none());
        assert!(parse_record(&r#mod).is_none());
    }
}
