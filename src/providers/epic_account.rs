//! Connexion directe au compte Epic Games.
//!
//! Epic n'expose pas de device-flow pour le client du launcher : la connexion
//! se fait par code d'autorisation, comme dans Legendary, mais Quark échange
//! lui-même le code contre une session OAuth. Le JSON complet est écrit dans
//! le `user.json` de Legendary : la bibliothèque, l'installation et les
//! paramètres de lancement continuent donc de fonctionner sans que
//! l'utilisateur ait à lancer `legendary auth`.
//!
//! Les jetons ne sont jamais affichés ni journalisés. Le fichier de session est
//! créé avec les droits du compte (0600 sous Unix).

use crate::providers::credentials::{self, ProviderTokens};
use serde::Deserialize;
use std::path::Path;
use std::time::Duration;

const TOKEN_URL: &str =
    "https://account-public-service-prod03.ol.epicgames.com/account/api/oauth/token";
/// Client public du launcher Epic, celui qu'utilise Legendary. Ce n'est pas un
/// secret utilisateur : il identifie l'application demandeuse.
const CLIENT_ID: &str = "34a02cf8f4414e29b15921876da36f9a";
const CLIENT_SECRET: &str = "daafbccc737745039dffe53d94fc76cf";

pub struct Session {
    pub access_token: String,
    pub refresh_token: String,
    pub account_id: String,
    pub display_name: String,
    pub expires_at: i64,
    /// Réponse OAuth complète : c'est elle que Legendary relit dans user.json.
    raw: serde_json::Value,
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: String,
    account_id: String,
    #[serde(rename = "displayName", default)]
    display_name: String,
    #[serde(default)]
    expires_at: String,
    #[serde(default)]
    expires_in: i64,
}

#[derive(Deserialize)]
struct OAuthError {
    #[serde(default, rename = "errorCode")]
    error_code: String,
    #[serde(default, rename = "errorMessage")]
    error_message: String,
    #[serde(default, rename = "continuationUrl")]
    continuation_url: String,
}

/// Échange un `authorizationCode` copié depuis la page Epic.
pub fn login_with_code(code: &str) -> Result<Session, String> {
    let code = code.trim();
    if code.is_empty() || code.len() > 4096 || code.chars().any(char::is_whitespace) {
        return Err("code d'autorisation Epic invalide".to_string());
    }
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|_| "client HTTP Epic indisponible".to_string())?;
    let response = client
        .post(TOKEN_URL)
        .basic_auth(CLIENT_ID, Some(CLIENT_SECRET))
        .header("User-Agent", "Quark/1.5 (Epic direct login)")
        .form(&[
            ("grant_type", "authorization_code"),
            ("code", code),
            ("token_type", "eg1"),
        ])
        .send()
        .map_err(|_| "connexion à Epic impossible (réseau)".to_string())?;

    let status = response.status();
    let bytes = response
        .bytes()
        .map_err(|_| "réponse Epic illisible".to_string())?;
    if !status.is_success() {
        return Err(oauth_error(&bytes));
    }
    let value: serde_json::Value = serde_json::from_slice(&bytes)
        .map_err(|_| "réponse Epic illisible (JSON invalide)".to_string())?;
    let parsed: TokenResponse = serde_json::from_value(value.clone())
        .map_err(|_| "réponse Epic incomplète (session absente)".to_string())?;
    if parsed.access_token.is_empty()
        || parsed.refresh_token.is_empty()
        || parsed.account_id.is_empty()
    {
        return Err("réponse Epic incomplète (session absente)".to_string());
    }
    let expires_at = expires_at(&parsed);
    let session = Session {
        access_token: parsed.access_token,
        refresh_token: parsed.refresh_token,
        account_id: parsed.account_id,
        display_name: parsed.display_name,
        expires_at,
        raw: value,
    };
    // Le fichier Legendary est la source de vérité pour la suite : s'il ne
    // peut pas être écrit, la session ne sert à rien et l'erreur est explicite.
    write_legendary_session(&session)?;
    remember_in_credentials(&session);
    Ok(session)
}

/// Écrit la réponse OAuth telle quelle dans `user.json`, comme Legendary.
fn write_legendary_session(session: &Session) -> Result<(), String> {
    let path = crate::providers::legendary::LegendaryAdapter::user_config_path();
    write_private_atomic(
        &path,
        &serde_json::to_vec_pretty(&session.raw).unwrap_or_default(),
    )
}

fn write_private_atomic(path: &Path, bytes: &[u8]) -> Result<(), String> {
    use std::io::Write;
    let parent = path
        .parent()
        .ok_or_else(|| "dossier de session Epic introuvable".to_string())?;
    std::fs::create_dir_all(parent)
        .map_err(|_| "dossier de session Epic inaccessible".to_string())?;
    let temp = parent.join(format!(".quark-epic-session-{}.tmp", std::process::id()));
    #[cfg(unix)]
    let file = {
        use std::os::unix::fs::OpenOptionsExt;
        std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(&temp)
    };
    #[cfg(not(unix))]
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&temp);
    let mut file = file.map_err(|_| "session Epic non enregistrable".to_string())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        let _ = file.set_permissions(std::fs::Permissions::from_mode(0o600));
    }
    file.write_all(bytes)
        .and_then(|()| file.sync_all())
        .map_err(|_| "session Epic non enregistrable".to_string())?;
    drop(file);
    std::fs::rename(&temp, path).map_err(|_| "session Epic non enregistrable".to_string())
}

/// Copie la session dans `credentials.json` pour que Quark puisse afficher le
/// compte sans relire le fichier interne de Legendary. Un échec d'écriture ne
/// fait pas échouer la connexion : `user.json` a déjà été écrit.
fn remember_in_credentials(session: &Session) {
    let path = crate::config::credentials_path();
    let Ok(mut credentials) = credentials::load(&path) else {
        return;
    };
    credentials.epic = Some(ProviderTokens {
        access_token: session.access_token.clone(),
        refresh_token: session.refresh_token.clone(),
        expires_at: session.expires_at,
        account_id: session.account_id.clone(),
        display_name: session.display_name.clone(),
    });
    let _ = credentials::save_atomic(&path, &credentials);
}

/// Compte Epic enregistré, sans exposer les jetons. La session reste utilisable
/// par Legendary même si Quark a été fermé entre-temps.
pub fn connected_account() -> Option<(String, String)> {
    let credentials = credentials::load(&crate::config::credentials_path()).ok()?;
    let epic = credentials.epic?;
    if epic.account_id.is_empty() {
        return None;
    }
    Some((epic.account_id, epic.display_name))
}

fn expires_at(response: &TokenResponse) -> i64 {
    if let Ok(date) = chrono_like_parse(&response.expires_at) {
        return date;
    }
    crate::config::now_ts() + response.expires_in.max(0)
}

/// Parse la date ISO-8601 `2026-11-14T13:00:00.000Z` sans dépendance
/// supplémentaire. Une date invalide retombe sur `expires_in`.
fn chrono_like_parse(text: &str) -> Result<i64, ()> {
    if text.len() < 19 {
        return Err(());
    }
    let parse = |range: std::ops::Range<usize>| text.get(range).and_then(|s| s.parse::<i64>().ok());
    let year = parse(0..4).ok_or(())?;
    let month = parse(5..7).ok_or(())?;
    let day = parse(8..10).ok_or(())?;
    let hour = parse(11..13).ok_or(())?;
    let minute = parse(14..16).ok_or(())?;
    let second = parse(17..19).ok_or(())?;
    if !(1970..=9999).contains(&year)
        || !(1..=12).contains(&month)
        || !(1..=31).contains(&day)
        || hour > 23
        || minute > 59
        || second > 60
    {
        return Err(());
    }
    // Algorithme de Howard Hinnant, valable pour toutes les dates modernes.
    let y = if month <= 2 { year - 1 } else { year };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = (month + 9) % 12;
    let doy = (153 * mp + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146097 + doe - 719468;
    Ok(days * 86_400 + hour * 3_600 + minute * 60 + second)
}

fn oauth_error(bytes: &[u8]) -> String {
    let Ok(error) = serde_json::from_slice::<OAuthError>(bytes) else {
        return "Epic a refusé la connexion".to_string();
    };
    if error.error_code == "errors.com.epicgames.oauth.corrective_action_required"
        && !error.continuation_url.is_empty()
    {
        return format!(
            "Epic demande une action sur le compte : {}",
            error.continuation_url
        );
    }
    if !error.error_message.is_empty() {
        return format!("Epic a refusé la connexion : {}", error.error_message);
    }
    "Epic a refusé la connexion (code expiré ou invalide)".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn une_reponse_oauth_complete_donne_une_session() {
        let text = r#"{
            "access_token": "eg1~access",
            "refresh_token": "eg1~refresh",
            "account_id": "abc123",
            "displayName": "Joueur",
            "expires_at": "2026-11-14T17:00:00.000Z",
            "expires_in": 14400
        }"#;
        let parsed: TokenResponse = serde_json::from_str(text).unwrap();
        assert_eq!(parsed.account_id, "abc123");
        assert_eq!(expires_at(&parsed), 1_794_675_600);
    }

    #[test]
    fn une_date_iso_invalide_retombe_sur_expires_in() {
        let text = r#"{
            "access_token": "a", "refresh_token": "r", "account_id": "id",
            "expires_at": "demain", "expires_in": 60
        }"#;
        let parsed: TokenResponse = serde_json::from_str(text).unwrap();
        assert!(expires_at(&parsed) >= crate::config::now_ts() + 59);
    }

    #[test]
    fn le_message_d_erreur_epic_ne_contient_pas_de_jeton() {
        let message = oauth_error(
            br#"{"errorCode":"errors.com.epicgames.oauth.corrective_action_required",
                 "errorMessage":"action needed","continuationUrl":"https://epic.example/continue"}"#,
        );
        assert!(message.contains("https://epic.example/continue"));
        assert!(!message.contains("access_token"));
    }
}
