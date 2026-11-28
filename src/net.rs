//! Lecture bornée des réponses HTTP.
//!
//! `reqwest::Response::bytes()` / `.text()` / `.json()` lisent tout le corps
//! avant que l'appelant puisse vérifier sa taille : un serveur hostile ou
//! défaillant peut remplir la mémoire. Ces aides imposent la limite pendant la
//! lecture, puis refusent tout dépassement.

use std::io::Read;
use std::net::IpAddr;

pub const MAX_JSON: u64 = 4 * 1024 * 1024;
pub const MAX_HTML: u64 = 8 * 1024 * 1024;
pub const MAX_IMAGE: u64 = 16 * 1024 * 1024;

/// Lit un corps en refusant plus de `max` octets. `Content-Length` est vérifié
/// quand il est présent, mais ne suffit jamais : le flux reste borné.
pub fn bytes(
    response: reqwest::blocking::Response,
    max: u64,
    label: &str,
) -> Result<Vec<u8>, String> {
    if response.content_length().is_some_and(|length| length > max) {
        return Err(format!("{label} trop volumineux"));
    }
    let mut body = Vec::new();
    response
        .take(max + 1)
        .read_to_end(&mut body)
        .map_err(|_| format!("{label} illisible"))?;
    if body.len() as u64 > max {
        return Err(format!("{label} trop volumineux"));
    }
    Ok(body)
}

pub fn text(
    response: reqwest::blocking::Response,
    max: u64,
    label: &str,
) -> Result<String, String> {
    let body = bytes(response, max, label)?;
    Ok(String::from_utf8_lossy(&body).into_owned())
}

pub fn json<T: serde::de::DeserializeOwned>(
    response: reqwest::blocking::Response,
    max: u64,
    label: &str,
) -> Result<T, String> {
    let body = bytes(response, max, label)?;
    serde_json::from_slice(&body).map_err(|_| format!("{label} illisible"))
}

/// Vérifie qu'une URL distante est bien en HTTPS et ne vise pas le réseau local.
/// Les modèles d'extension (`{query}`, `{slug}`…) sont remplacés par `x` avant
/// l'analyse, sinon `Url::parse` les refuserait.
pub fn validate_public_https(url: &str, allow_placeholders: bool) -> Result<(), String> {
    let candidate = if allow_placeholders {
        strip_placeholders(url)
    } else {
        url.to_string()
    };
    if candidate.len() > 2048 {
        return Err("adresse trop longue".to_string());
    }
    let parsed = reqwest::Url::parse(&candidate).map_err(|_| "adresse invalide".to_string())?;
    if parsed.scheme() != "https" {
        return Err("HTTPS obligatoire".to_string());
    }
    if !parsed.username().is_empty() || parsed.password().is_some() {
        return Err("identifiants interdits dans l'adresse".to_string());
    }
    let host = parsed
        .host_str()
        .ok_or_else(|| "hôte manquant".to_string())?;
    if !is_public_host(host) {
        return Err("hôte local ou privé interdit".to_string());
    }
    Ok(())
}

fn strip_placeholders(url: &str) -> String {
    let mut out = String::with_capacity(url.len());
    let mut inside = false;
    for character in url.chars() {
        match character {
            '{' => inside = true,
            '}' => inside = false,
            _ if !inside => out.push(character),
            _ => {}
        }
    }
    out
}

fn is_public_host(host: &str) -> bool {
    let host = host
        .trim()
        .trim_start_matches('[')
        .trim_end_matches(']')
        .to_ascii_lowercase();
    if host.is_empty() || host == "localhost" || host.ends_with(".localhost") {
        return false;
    }
    let Ok(address) = host.parse::<IpAddr>() else {
        return true;
    };
    match address {
        IpAddr::V4(address) => {
            !(address.is_private()
                || address.is_loopback()
                || address.is_link_local()
                || address.is_unspecified()
                || address.is_broadcast()
                || (address.octets()[0] == 100 && (64..=127).contains(&address.octets()[1])))
        }
        IpAddr::V6(address) => {
            !(address.is_loopback()
                || address.is_unspecified()
                || address.is_unique_local()
                || address.is_unicast_link_local())
        }
    }
}
