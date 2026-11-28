//! Extensions de recherche : manifeste JSON validé, cache local et sites actifs.
//!
//! Une extension ne contient aucun code exécutable : elle décrit des sites et
//! choisit un driver. Le manifeste est hébergé hors de Quark et téléchargé
//! uniquement quand l'utilisateur l'ajoute ou demande une actualisation.

use crate::game_search::{SiteDefinition, validate_site};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Manifeste officiel des sept sites, hébergé par le dépôt d'extension.
pub const OFFICIAL_SOURCE: &str =
    "https://raw.githubusercontent.com/Matheotaku4/Quark-sites-extension/main/quark-sites.json";

pub const SCHEMA_VERSION: u32 = 1;
pub const MAX_MANIFEST_BYTES: u64 = 256 * 1024;
pub const MAX_SITES: usize = 64;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionManifest {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub homepage: Option<String>,
    pub sites: Vec<SiteDefinition>,
}

/// Parse et valide un manifeste. Un seul site invalide rejette le manifeste.
pub fn parse_manifest(bytes: &[u8]) -> Result<ExtensionManifest, String> {
    if bytes.len() as u64 > MAX_MANIFEST_BYTES {
        return Err("manifeste trop volumineux".to_string());
    }
    let manifest: ExtensionManifest =
        serde_json::from_slice(bytes).map_err(|error| format!("manifeste illisible : {error}"))?;
    validate_manifest(&manifest)?;
    Ok(manifest)
}

pub fn validate_manifest(manifest: &ExtensionManifest) -> Result<(), String> {
    if manifest.schema_version != SCHEMA_VERSION {
        return Err(format!(
            "version de schéma inconnue : {}",
            manifest.schema_version
        ));
    }
    if !valid_id(&manifest.id) {
        return Err("identifiant d'extension invalide".to_string());
    }
    if manifest.name.trim().is_empty() || manifest.name.len() > 80 {
        return Err("nom d'extension invalide".to_string());
    }
    if manifest.version.trim().is_empty() || manifest.version.len() > 32 {
        return Err("version d'extension invalide".to_string());
    }
    if let Some(homepage) = &manifest.homepage {
        crate::net::validate_public_https(homepage, false)
            .map_err(|error| format!("page d'accueil de l'extension : {error}"))?;
    }
    if manifest.sites.is_empty() || manifest.sites.len() > MAX_SITES {
        return Err(format!(
            "le manifeste doit contenir entre 1 et {MAX_SITES} sites"
        ));
    }
    let mut ids = HashSet::new();
    for site in &manifest.sites {
        if !ids.insert(site.id.as_str()) {
            return Err(format!("site en double : {}", site.id));
        }
        validate_site(site)?;
    }
    Ok(())
}

fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 64
        && id
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '.' | '_' | '-'))
}

pub const CACHE_VERSION: u32 = 1;
const STALE_AFTER_SECS: i64 = 7 * 24 * 3600;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionCache {
    #[serde(default = "default_cache_version")]
    pub version: u32,
    #[serde(default)]
    pub entries: Vec<CachedExtension>,
}

fn default_cache_version() -> u32 {
    CACHE_VERSION
}

impl Default for ExtensionCache {
    fn default() -> Self {
        Self {
            version: CACHE_VERSION,
            entries: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CachedExtension {
    pub url: String,
    pub fetched_at: i64,
    pub manifest: ExtensionManifest,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionStatus {
    pub url: String,
    pub id: String,
    pub name: String,
    pub version: String,
    pub site_count: usize,
    pub fetched_at: i64,
    pub stale: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ExtensionSite {
    pub uid: String,
    pub extension_id: String,
    pub extension_name: String,
    pub site: SiteDefinition,
}

pub fn cache_path() -> PathBuf {
    crate::config::data_dir().join("extensions.json")
}

/// Charge le cache local. L'extension officielle est livrée dans le binaire
/// pour que la recherche soit disponible immédiatement, même hors ligne.
pub fn load_cache(sources: &[String]) -> ExtensionCache {
    let path = cache_path();
    let mut cache = load_cache_from(&path);
    if !sources.iter().any(|url| url == OFFICIAL_SOURCE)
        || cache
            .entries
            .iter()
            .any(|entry| entry.url == OFFICIAL_SOURCE)
    {
        return cache;
    }
    let Ok(manifest) = official_manifest() else {
        return cache;
    };
    cache.entries.push(CachedExtension {
        url: OFFICIAL_SOURCE.to_string(),
        fetched_at: 0,
        manifest,
    });
    let _ = save_cache_to(&path, &cache);
    cache
}

/// Manifeste des sept sites embarqué avec Quark.
pub fn official_manifest() -> Result<ExtensionManifest, String> {
    parse_manifest(include_bytes!("../docs/extensions/quark-sites.json"))
}

/// Indique si une source doit être retéléchargée au démarrage.
pub fn needs_refresh(cache: &ExtensionCache, url: &str) -> bool {
    cache
        .entries
        .iter()
        .find(|entry| entry.url == url)
        .is_none_or(|entry| entry.fetched_at == 0 || now_ts() - entry.fetched_at > STALE_AFTER_SECS)
}

pub fn load_cache_from(path: &Path) -> ExtensionCache {
    let Ok(text) = std::fs::read_to_string(path) else {
        return ExtensionCache::default();
    };
    let Ok(mut cache) = serde_json::from_str::<ExtensionCache>(&text) else {
        return ExtensionCache::default();
    };
    cache.version = CACHE_VERSION;
    cache
        .entries
        .retain(|entry| validate_manifest(&entry.manifest).is_ok());
    cache
}

pub fn save_cache_to(path: &Path, cache: &ExtensionCache) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let text = serde_json::to_string_pretty(cache).map_err(|error| error.to_string())?;
    let tmp = path.with_extension(format!("tmp-{}", std::process::id()));
    std::fs::write(&tmp, text).map_err(|error| error.to_string())?;
    std::fs::rename(&tmp, path).map_err(|error| {
        let _ = std::fs::remove_file(&tmp);
        error.to_string()
    })
}

pub fn add_manifest(
    cache: &mut ExtensionCache,
    sources: &mut Vec<String>,
    url: &str,
    manifest: ExtensionManifest,
) -> Result<ExtensionStatus, String> {
    crate::net::validate_public_https(url, false)
        .map_err(|error| format!("adresse de l'extension : {error}"))?;
    validate_manifest(&manifest)?;
    if let Some(other) = cache
        .entries
        .iter()
        .find(|entry| entry.url != url && entry.manifest.id == manifest.id)
    {
        return Err(format!(
            "l'extension {} est déjà fournie par {}",
            manifest.id, other.url
        ));
    }
    let fetched_at = now_ts();
    cache.entries.retain(|entry| entry.url != url);
    cache.entries.push(CachedExtension {
        url: url.to_string(),
        fetched_at,
        manifest,
    });
    if !sources.iter().any(|source| source == url) {
        sources.push(url.to_string());
    }
    let entry = cache
        .entries
        .iter()
        .find(|entry| entry.url == url)
        .expect("entrée ajoutée");
    Ok(status_from_entry(entry))
}

pub fn remove_source(cache: &mut ExtensionCache, sources: &mut Vec<String>, url: &str) -> bool {
    let before = sources.len();
    sources.retain(|source| source != url);
    let before_cache = cache.entries.len();
    cache.entries.retain(|entry| entry.url != url);
    sources.len() != before || cache.entries.len() != before_cache
}

pub fn statuses(cache: &ExtensionCache, sources: &[String]) -> Vec<ExtensionStatus> {
    sources
        .iter()
        .map(|url| {
            cache
                .entries
                .iter()
                .find(|entry| entry.url == *url)
                .map(status_from_entry)
                .unwrap_or_else(|| ExtensionStatus {
                    url: url.clone(),
                    id: String::new(),
                    name: String::new(),
                    version: String::new(),
                    site_count: 0,
                    fetched_at: 0,
                    stale: true,
                    error: Some("extension non téléchargée".to_string()),
                })
        })
        .collect()
}

pub fn active_sites(cache: &ExtensionCache, sources: &[String]) -> Vec<ExtensionSite> {
    let mut seen = HashSet::new();
    let mut sites = Vec::new();
    for source in sources {
        let Some(entry) = cache.entries.iter().find(|entry| entry.url == *source) else {
            continue;
        };
        if !seen.insert(entry.manifest.id.clone()) {
            continue;
        }
        for site in &entry.manifest.sites {
            sites.push(ExtensionSite {
                uid: format!("{}/{}", entry.manifest.id, site.id),
                extension_id: entry.manifest.id.clone(),
                extension_name: entry.manifest.name.clone(),
                site: site.clone(),
            });
        }
    }
    sites
}

fn status_from_entry(entry: &CachedExtension) -> ExtensionStatus {
    ExtensionStatus {
        url: entry.url.clone(),
        id: entry.manifest.id.clone(),
        name: entry.manifest.name.clone(),
        version: entry.manifest.version.clone(),
        site_count: entry.manifest.sites.len(),
        fetched_at: entry.fetched_at,
        stale: now_ts() - entry.fetched_at > STALE_AFTER_SECS,
        error: None,
    }
}

fn now_ts() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0)
}

/// Télécharge et valide un manifeste. Aucun cookie, aucun en-tête sensible.
pub fn fetch_manifest(url: &str) -> Result<ExtensionManifest, String> {
    crate::net::validate_public_https(url, false)
        .map_err(|error| format!("adresse de l'extension : {error}"))?;
    let client = reqwest::blocking::Client::builder()
        .user_agent(format!("Quark/{}", env!("CARGO_PKG_VERSION")))
        .timeout(std::time::Duration::from_secs(15))
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()
        .map_err(|error| error.to_string())?;
    let response = client
        .get(url)
        .header(reqwest::header::ACCEPT, "application/json")
        .send()
        .map_err(|_| "extension injoignable".to_string())?;
    if !response.status().is_success() {
        return Err(format!(
            "l'extension a refusé le téléchargement (HTTP {})",
            response.status().as_u16()
        ));
    }
    let final_url = response.url().to_string();
    crate::net::validate_public_https(&final_url, false)
        .map_err(|error| format!("redirection de l'extension : {error}"))?;
    let bytes = crate::net::bytes(response, MAX_MANIFEST_BYTES, "manifeste")?;
    parse_manifest(&bytes)
}

pub fn add_source(
    cache: &mut ExtensionCache,
    sources: &mut Vec<String>,
    url: &str,
) -> Result<ExtensionStatus, String> {
    let manifest = fetch_manifest(url)?;
    add_manifest(cache, sources, url, manifest)
}

pub fn refresh_source(
    cache: &mut ExtensionCache,
    sources: &mut Vec<String>,
    url: &str,
) -> Result<ExtensionStatus, String> {
    let manifest = fetch_manifest(url)?;
    add_manifest(cache, sources, url, manifest)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game_search::{JsonDriver, RequestSpec, SiteDriver};
    use std::collections::BTreeMap;

    fn builtin_site(id: &str) -> SiteDefinition {
        SiteDefinition {
            id: id.to_string(),
            label: "GameBounty".to_string(),
            host: "gamebounty.st".to_string(),
            search_url: "https://gamebounty.st/search?q={query}".to_string(),
            driver: SiteDriver::GameBounty,
        }
    }

    fn manifest(sites: Vec<SiteDefinition>) -> ExtensionManifest {
        ExtensionManifest {
            schema_version: SCHEMA_VERSION,
            id: "quark-sites".to_string(),
            name: "Sites Quark".to_string(),
            version: "1.0.0".to_string(),
            description: String::new(),
            homepage: None,
            sites,
        }
    }

    #[test]
    fn un_manifeste_valide_est_accepte() {
        let body = serde_json::to_vec(&manifest(vec![builtin_site("gamebounty")])).unwrap();
        let parsed = parse_manifest(&body).expect("manifeste valide");
        assert_eq!(parsed.sites.len(), 1);
        assert_eq!(parsed.sites[0].driver, SiteDriver::GameBounty);
    }

    #[test]
    fn un_manifeste_avec_un_doublon_de_site_est_refuse() {
        let body = serde_json::to_vec(&manifest(vec![builtin_site("same"), builtin_site("same")]))
            .unwrap();
        assert!(parse_manifest(&body).unwrap_err().contains("double"));
    }

    #[test]
    fn une_source_non_https_est_refusee() {
        let mut site = builtin_site("gamebounty");
        site.search_url = "http://gamebounty.st/search?q={query}".to_string();
        assert!(
            validate_manifest(&manifest(vec![site]))
                .unwrap_err()
                .contains("HTTPS")
        );
    }

    #[test]
    fn un_hote_prive_est_refuse() {
        for host in ["127.0.0.1", "localhost", "192.168.1.4", "[::1]"] {
            let mut site = builtin_site("gamebounty");
            site.search_url = format!("https://{host}/search?q={{query}}");
            assert!(
                validate_manifest(&manifest(vec![site]))
                    .unwrap_err()
                    .contains("privé")
            );
        }
    }

    #[test]
    fn un_identifiant_invalide_est_refuse() {
        let mut site = builtin_site("Bad Id!");
        site.id = "Bad Id!".to_string();
        assert!(
            validate_manifest(&manifest(vec![site]))
                .unwrap_err()
                .contains("identifiant")
        );
    }

    #[test]
    fn un_manifeste_trop_gros_est_refuse() {
        let body = vec![b' '; MAX_MANIFEST_BYTES as usize + 1];
        assert!(parse_manifest(&body).unwrap_err().contains("volumineux"));
    }

    #[test]
    fn un_driver_inconnu_est_refuse() {
        let body = br#"{
          "schemaVersion": 1, "id": "x", "name": "X", "version": "1",
          "sites": [{
            "id": "x", "label": "X", "host": "example.com",
            "searchUrl": "https://example.com/?q={query}", "driver": "nope"
          }]
        }"#;
        assert!(parse_manifest(body).is_err());
    }

    #[test]
    fn un_driver_json_valide_est_accepte() {
        let site = SiteDefinition {
            id: "example".to_string(),
            label: "Example".to_string(),
            host: "example.com".to_string(),
            search_url: "https://example.com/search?q={query}".to_string(),
            driver: SiteDriver::Json(JsonDriver {
                request: RequestSpec {
                    method: Default::default(),
                    url: "https://example.com/api?q={query}".to_string(),
                    headers: BTreeMap::new(),
                    body: None,
                    encoding: Default::default(),
                },
                list: "data.results".to_string(),
                title: "title".to_string(),
                url: "https://example.com/game/{slug}".to_string(),
                detail: Vec::new(),
                cover: Vec::new(),
                limit: 8,
            }),
        };
        validate_manifest(&manifest(vec![site])).expect("driver json valide");
    }
    fn temp_dir(label: &str) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!(
            "quark-extensions-{label}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&path).unwrap();
        path
    }

    fn manifest_named(id: &str, site_id: &str) -> ExtensionManifest {
        let mut value = manifest(vec![builtin_site(site_id)]);
        value.id = id.to_string();
        value
    }

    const SOURCE_A: &str = "https://raw.githubusercontent.com/example/a/main/quark-sites.json";
    const SOURCE_B: &str = "https://raw.githubusercontent.com/example/b/main/quark-sites.json";

    #[test]
    fn le_cache_se_charge_et_se_sauvegarde() {
        let root = temp_dir("roundtrip");
        let path = root.join("extensions.json");
        let mut cache = ExtensionCache::default();
        let mut sources = Vec::new();
        add_manifest(
            &mut cache,
            &mut sources,
            SOURCE_A,
            manifest_named("quark-a", "gamebounty"),
        )
        .unwrap();
        save_cache_to(&path, &cache).unwrap();
        let loaded = load_cache_from(&path);
        assert_eq!(loaded.entries.len(), 1);
        assert_eq!(loaded.entries[0].manifest.id, "quark-a");
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn un_cache_corrompu_est_ignore() {
        let root = temp_dir("corrupt");
        let path = root.join("extensions.json");
        std::fs::write(&path, "{").unwrap();
        assert!(load_cache_from(&path).entries.is_empty());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn les_sources_actives_sont_filtrees() {
        let mut cache = ExtensionCache::default();
        let mut sources = Vec::new();
        add_manifest(
            &mut cache,
            &mut sources,
            SOURCE_A,
            manifest_named("quark-a", "gamebounty"),
        )
        .unwrap();
        add_manifest(
            &mut cache,
            &mut sources,
            SOURCE_B,
            manifest_named("quark-b", "gamebounty"),
        )
        .unwrap();
        let sites = active_sites(&cache, &[SOURCE_B.to_string()]);
        assert_eq!(sites.len(), 1);
        assert_eq!(sites[0].uid, "quark-b/gamebounty");
    }

    #[test]
    fn deux_extensions_ne_peuvent_pas_partager_le_meme_id() {
        let mut cache = ExtensionCache::default();
        let mut sources = Vec::new();
        add_manifest(
            &mut cache,
            &mut sources,
            SOURCE_A,
            manifest_named("same", "gamebounty"),
        )
        .unwrap();
        let error = add_manifest(
            &mut cache,
            &mut sources,
            SOURCE_B,
            manifest_named("same", "gamebounty"),
        )
        .unwrap_err();
        assert!(error.contains("déjà fournie"));
    }

    #[test]
    fn le_statut_indique_une_extension_perimee() {
        let mut cache = ExtensionCache::default();
        let mut sources = Vec::new();
        add_manifest(
            &mut cache,
            &mut sources,
            SOURCE_A,
            manifest_named("quark-a", "gamebounty"),
        )
        .unwrap();
        cache.entries[0].fetched_at = now_ts() - STALE_AFTER_SECS - 1;
        let status = &statuses(&cache, &sources)[0];
        assert!(status.stale);
        assert_eq!(status.site_count, 1);
    }

    #[test]
    fn remove_source_retire_l_entree_et_la_source() {
        let mut cache = ExtensionCache::default();
        let mut sources = Vec::new();
        add_manifest(
            &mut cache,
            &mut sources,
            SOURCE_A,
            manifest_named("quark-a", "gamebounty"),
        )
        .unwrap();
        assert!(remove_source(&mut cache, &mut sources, SOURCE_A));
        assert!(sources.is_empty());
        assert!(cache.entries.is_empty());
    }

    #[test]
    fn le_manifeste_modele_est_valide() {
        let bytes = include_str!("../docs/extensions/quark-sites.json").as_bytes();
        let manifest = parse_manifest(bytes).expect("manifeste modele");
        assert_eq!(manifest.sites.len(), 7);
        assert_eq!(manifest.sites[0].driver, SiteDriver::GameBounty);
        assert_eq!(manifest.sites[6].driver, SiteDriver::SceneCat);
    }
}
