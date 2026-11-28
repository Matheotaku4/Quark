//! Extensions de recherche : manifeste JSON validé, cache local et sites actifs.
//!
//! Une extension ne contient aucun code exécutable : elle décrit des sites et
//! choisit un driver. Le manifeste est hébergé hors de Quark et téléchargé
//! uniquement quand l'utilisateur l'ajoute ou demande une actualisation.

use crate::game_search::{SiteDefinition, validate_site};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

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
        let body = serde_json::to_vec(&manifest(vec![
            builtin_site("same"),
            builtin_site("same"),
        ]))
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
}
