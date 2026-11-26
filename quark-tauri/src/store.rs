//! Boutique Steam : catalogue en cache, actualisation, recherche et recommandations.
//! Reprend `quark::steam_store` tel quel ; l'interface ne fait que de l'affichage.

use super::{lock, AppState};
use base64::Engine;
use quark::providers::credentials;
use quark::steam_store::{self, StoreItem, StoreSnapshot};
use quark::{artwork, config};
use serde::Serialize;
use tauri::State;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PriceDto {
    currency: String,
    initial: u64,
    final_price: u64,
    discount_percent: Option<u8>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemDto {
    app_id: u32,
    name: String,
    price: Option<PriceDto>,
    artwork_url: Option<String>,
    store_url: String,
    linux_available: Option<bool>,
    players_peak: Option<u32>,
}

#[derive(Serialize)]
pub struct SectionDto {
    title: String,
    items: Vec<ItemDto>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StorePageDto {
    fetched_at: i64,
    stale: bool,
    sections: Vec<SectionDto>,
    recommendations: Vec<ItemDto>,
    recommendation_source: Option<String>,
    notice: Option<String>,
}

fn item_dto(item: StoreItem) -> ItemDto {
    ItemDto {
        app_id: item.app_id,
        name: item.name,
        price: item.price.map(|price| PriceDto {
            currency: price.currency,
            initial: price.initial,
            final_price: price.final_price,
            discount_percent: price.discount_percent,
        }),
        artwork_url: item.artwork_url,
        store_url: item.store_url,
        linux_available: item.linux_available,
        players_peak: item.players_peak,
    }
}

fn page_dto(
    snapshot: StoreSnapshot,
    games: &[config::Game],
    steam_library: Option<&[steam_store::OwnedSteamGame]>,
    notice: Option<String>,
) -> StorePageDto {
    let (recommendations, source) =
        steam_store::recommendations_for(Some(&snapshot), games, steam_library);
    StorePageDto {
        fetched_at: snapshot.fetched_at,
        stale: snapshot.is_stale(config::now_ts()),
        sections: snapshot
            .sections
            .into_iter()
            .map(|section| SectionDto {
                title: section.title,
                items: section.items.into_iter().map(item_dto).collect(),
            })
            .collect(),
        recommendations: recommendations.into_iter().map(item_dto).collect(),
        recommendation_source: source,
        notice,
    }
}

fn demo_art(name: &str, hue: u16) -> String {
    let letter = name.chars().next().unwrap_or('Q');
    let svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="460" height="215"><defs><linearGradient id="g" x1="0" y1="0" x2="1" y2="1"><stop stop-color="hsl({hue},45%,34%)"/><stop offset="1" stop-color="hsl({},45%,10%)"/></linearGradient></defs><rect width="460" height="215" fill="url(#g)"/><text x="28" y="158" font-family="sans-serif" font-size="132" fill="rgba(255,255,255,.16)">{letter}</text></svg>"#,
        (hue + 40) % 360
    );
    format!(
        "data:image/svg+xml;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(svg)
    )
}

fn demo_item(
    app_id: u32,
    name: &str,
    hue: u16,
    initial: u64,
    discount: u8,
    players: u32,
) -> StoreItem {
    StoreItem {
        app_id,
        name: name.to_string(),
        price: Some(steam_store::StorePrice {
            currency: "EUR".to_string(),
            initial,
            final_price: initial * u64::from(100 - discount) / 100,
            discount_percent: (discount > 0).then_some(discount),
        }),
        artwork_url: Some(demo_art(name, hue)),
        store_url: format!("https://store.steampowered.com/app/{app_id}"),
        linux_available: Some(true),
        players_peak: Some(players),
    }
}

fn demo_snapshot() -> StoreSnapshot {
    let items = [
        (1145360, "Hades", 285, 2499, 40, 24_000),
        (367520, "Hollow Knight", 190, 1499, 0, 18_000),
        (1086940, "Baldur's Gate 3", 25, 5999, 10, 120_000),
        (1091500, "Cyberpunk 2077", 50, 5999, 50, 45_000),
        (1245620, "Elden Ring", 42, 5999, 30, 38_000),
        (413150, "Stardew Valley", 105, 1399, 0, 52_000),
        (427520, "Factorio", 28, 3200, 0, 14_000),
        (504230, "Celeste", 210, 1999, 75, 6_000),
        (632470, "Disco Elysium", 330, 3999, 80, 8_500),
        (1262350, "SIGNALIS", 350, 1999, 40, 3_200),
        (753640, "Outer Wilds", 265, 2499, 0, 5_400),
        (553420, "TUNIC", 85, 2999, 30, 2_800),
    ]
    .into_iter()
    .map(|(id, name, hue, price, discount, players)| {
        demo_item(id, name, hue, price, discount, players)
    })
    .collect::<Vec<_>>();
    StoreSnapshot {
        fetched_at: config::now_ts(),
        sections: vec![
            steam_store::StoreSection {
                title: "Meilleures ventes".to_string(),
                items: items[..6].to_vec(),
            },
            steam_store::StoreSection {
                title: "Offres spéciales".to_string(),
                items: items[3..9].to_vec(),
            },
            steam_store::StoreSection {
                title: "Nouveautés".to_string(),
                items: items[6..].to_vec(),
            },
        ],
    }
}

/// Catalogue enregistré lors de la dernière actualisation (affichage immédiat).
#[tauri::command]
pub fn store_load(state: State<'_, AppState>) -> Option<StorePageDto> {
    let games = lock(&state.library).games.clone();
    if state.demo {
        return Some(page_dto(demo_snapshot(), &games, None, None));
    }
    let snapshot = steam_store::load_cache(&steam_store::cache_path())
        .ok()
        .flatten()?;
    Some(page_dto(snapshot, &games, None, None))
}

/// Recharge les catégories, les jeux les plus joués et, si une clé Web API est
/// enregistrée, la bibliothèque Steam du compte (pour mieux recommander).
#[tauri::command]
pub async fn store_refresh(state: State<'_, AppState>) -> Result<StorePageDto, String> {
    if state.demo {
        let games = lock(&state.library).games.clone();
        return Ok(page_dto(demo_snapshot(), &games, None, None));
    }
    let (games, language, country) = {
        let library = lock(&state.library);
        (
            library.games.clone(),
            library.settings.language.clone(),
            library.settings.country.clone(),
        )
    };
    tauri::async_runtime::spawn_blocking(move || {
        let creds = credentials::load(&config::credentials_path()).unwrap_or_default();
        let fetch = steam_store::fetch_all(
            &artwork::http(),
            &language,
            &country,
            creds.steam_web_api_key.as_deref(),
            creds.steam_id.as_deref(),
        )?;
        let _ = steam_store::save_cache_atomic(&steam_store::cache_path(), &fetch.snapshot);
        let notice = fetch.popular_error.or(fetch.steam_library_error);
        Ok(page_dto(
            fetch.snapshot,
            &games,
            fetch.steam_library.as_deref(),
            notice,
        ))
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn store_search(
    state: State<'_, AppState>,
    query: String,
) -> Result<Vec<ItemDto>, String> {
    let (language, country) = {
        let library = lock(&state.library);
        (
            library.settings.language.clone(),
            library.settings.country.clone(),
        )
    };
    tauri::async_runtime::spawn_blocking(move || {
        steam_store::search(&artwork::http(), query.trim(), &language, &country)
            .map(|items| items.into_iter().map(item_dto).collect())
    })
    .await
    .map_err(|error| error.to_string())?
}
