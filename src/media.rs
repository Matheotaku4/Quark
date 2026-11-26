//! Bande-annonce et captures d'un jeu, lues dans la fiche publique Steam.
//!
//! La fiche jeu affiche les mêmes médias que la page Steam : la bande-annonce
//! en premier, puis les captures. Seules les URL sont conservées ici, jamais
//! les octets.

use reqwest::blocking::Client;
use serde::Deserialize;

/// Nombre de captures affichées : assez pour la grille, sans charger toute la
/// galerie d'un jeu qui en publie des centaines.
pub const MAX_SCREENSHOTS: usize = 12;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GameMedia {
    pub trailers: Vec<Trailer>,
    pub screenshots: Vec<Photo>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Trailer {
    pub title: String,
    /// Vignette 16:9 affichée avant lecture.
    pub thumbnail: String,
    /// Flux lisible par le lecteur intégré : HLS, DASH, MP4 ou WebM.
    pub video: String,
    /// Fiche Steam, utilisée si le lecteur intégré n'est pas disponible.
    pub page: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Photo {
    pub thumbnail: String,
    pub full: String,
}

/// Récupère la bande-annonce et les captures d'un appid Steam.
pub fn fetch(
    client: &Client,
    app_id: u32,
    language: &str,
    country: &str,
) -> Result<GameMedia, String> {
    let response = client
        .get("https://store.steampowered.com/api/appdetails")
        .query(&[
            ("appids", app_id.to_string()),
            ("l", language.to_string()),
            ("cc", country.to_string()),
        ])
        .send()
        .map_err(|_| "connexion aux médias Steam impossible".to_string())?;
    if !response.status().is_success() {
        return Err(format!(
            "Steam Store a répondu {}",
            response.status().as_u16()
        ));
    }
    let body = crate::net::text(response, crate::net::MAX_JSON, "médias Steam")?;
    parse(&body, app_id)
}

fn parse(body: &str, app_id: u32) -> Result<GameMedia, String> {
    #[derive(Deserialize)]
    struct Envelope {
        #[serde(default)]
        success: bool,
        #[serde(default)]
        data: Option<Data>,
    }
    #[derive(Deserialize, Default, Clone)]
    struct Data {
        #[serde(default)]
        movies: Vec<Movie>,
        #[serde(default)]
        screenshots: Vec<Screenshot>,
    }
    #[derive(Deserialize, Default, Clone)]
    struct Movie {
        #[serde(default)]
        name: String,
        #[serde(default)]
        thumbnail: String,
        #[serde(default)]
        highlight: bool,
        #[serde(default)]
        mp4: Video,
        #[serde(default)]
        webm: Video,
        /// Steam sert désormais les bandes-annonces en HLS/DASH.
        #[serde(default)]
        hls_h264: String,
        #[serde(default)]
        dash_h264: String,
    }
    #[derive(Deserialize, Default, Clone)]
    struct Video {
        #[serde(default, rename = "480")]
        small: String,
        #[serde(default)]
        max: String,
    }
    #[derive(Deserialize, Default, Clone)]
    struct Screenshot {
        #[serde(default)]
        path_thumbnail: String,
        #[serde(default)]
        path_full: String,
    }

    let envelopes: std::collections::HashMap<String, Envelope> =
        serde_json::from_str(body).map_err(|_| "médias Steam illisibles".to_string())?;
    let envelope = envelopes
        .get(&app_id.to_string())
        .ok_or_else(|| format!("aucune fiche Steam pour l'appid {app_id}"))?;
    if !envelope.success {
        return Err("Steam n'a pas renvoyé de fiche pour ce jeu".to_string());
    }
    let data = envelope.data.clone().unwrap_or_default();

    let page = format!("https://store.steampowered.com/app/{app_id}/");
    let mut trailers: Vec<Trailer> = data
        .movies
        .iter()
        .filter_map(|movie| {
            // MP4/WebM d'abord : WebKitGTK les lit sans greffon HLS. Les
            // bandes-annonces récentes n'exposent parfois que HLS/DASH, gardés
            // en secours.
            let direct = first_url(&movie.mp4.max, &movie.mp4.small)
                .or_else(|| first_url(&movie.webm.max, &movie.webm.small));
            let streaming = {
                let hls = https_or_empty(&movie.hls_h264);
                if hls.is_empty() {
                    https_or_empty(&movie.dash_h264)
                } else {
                    hls
                }
            };
            let video = direct.or_else(|| (!streaming.is_empty()).then_some(streaming))?;
            let thumbnail = https_or_empty(&movie.thumbnail);
            if thumbnail.is_empty() {
                return None;
            }
            Some(Trailer {
                title: movie.name.trim().to_string(),
                thumbnail,
                video,
                page: page.clone(),
            })
        })
        .collect();
    // La bande-annonce mise en avant par Steam passe devant les autres.
    if let Some(index) = data
        .movies
        .iter()
        .position(|movie| movie.highlight)
        .filter(|index| *index < trailers.len())
    {
        trailers.swap(0, index);
    }

    let screenshots = data
        .screenshots
        .into_iter()
        .filter_map(|shot| {
            let thumbnail = https_or_empty(&shot.path_thumbnail);
            let full = https_or_empty(&shot.path_full);
            if thumbnail.is_empty() || full.is_empty() {
                return None;
            }
            Some(Photo { thumbnail, full })
        })
        .take(MAX_SCREENSHOTS)
        .collect();

    Ok(GameMedia {
        trailers,
        screenshots,
    })
}

fn first_url(max: &str, small: &str) -> Option<String> {
    let max = https_or_empty(max);
    if !max.is_empty() {
        return Some(max);
    }
    let small = https_or_empty(small);
    (!small.is_empty()).then_some(small)
}

fn https_or_empty(url: &str) -> String {
    let url = url.trim();
    if url.starts_with("https://") || url.starts_with("http://") {
        url.to_string()
    } else {
        String::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn la_bande_annonce_passe_avant_les_photos() {
        let body = r#"{
          "730": {
            "success": true,
            "data": {
              "movies": [
                {"name": "Trailer", "thumbnail": "https://cdn.test/trailer.jpg", "highlight": true,
                 "mp4": {"480": "https://cdn.test/trailer-480.mp4", "max": "https://cdn.test/trailer.mp4"},
                 "webm": {"480": "https://cdn.test/trailer-480.webm", "max": "https://cdn.test/trailer.webm"}}
              ],
              "screenshots": [
                {"path_thumbnail": "https://cdn.test/shot-1-thumb.jpg", "path_full": "https://cdn.test/shot-1.jpg"},
                {"path_thumbnail": "https://cdn.test/shot-2-thumb.jpg", "path_full": "https://cdn.test/shot-2.jpg"}
              ]
            }
          }
        }"#;
        let media = parse(body, 730).unwrap();
        assert_eq!(media.trailers.len(), 1);
        assert_eq!(media.trailers[0].video, "https://cdn.test/trailer.mp4");
        assert_eq!(
            media.trailers[0].page,
            "https://store.steampowered.com/app/730/"
        );
        assert_eq!(media.screenshots.len(), 2);
        assert_eq!(media.screenshots[0].full, "https://cdn.test/shot-1.jpg");
    }

    #[test]
    fn le_mp4_est_prefere_au_hls() {
        let body = r#"{
          "30": {"success": true, "data": {
            "movies": [{"name": "Trailer", "thumbnail": "https://cdn.test/trailer.jpg",
                        "hls_h264": "https://video.test/master.m3u8",
                        "mp4": {"max": "https://cdn.test/trailer.mp4"}}],
            "screenshots": []
          }}
        }"#;
        let media = parse(body, 30).unwrap();
        assert_eq!(media.trailers[0].video, "https://cdn.test/trailer.mp4");
    }

    #[test]
    fn une_video_hls_reste_lisible_dans_le_lecteur() {
        let body = r#"{
          "20": {"success": true, "data": {
            "movies": [{"name": "Trailer", "thumbnail": "https://cdn.test/trailer.jpg",
                        "hls_h264": "https://video.test/master.m3u8"}],
            "screenshots": []
          }}
        }"#;
        let media = parse(body, 20).unwrap();
        assert_eq!(media.trailers.len(), 1);
        assert_eq!(media.trailers[0].video, "https://video.test/master.m3u8");
        assert_eq!(
            media.trailers[0].page,
            "https://store.steampowered.com/app/20/"
        );
    }

    #[test]
    fn les_medias_invalides_ou_absents_sont_ignores() {
        let body = r#"{
          "10": {"success": true, "data": {
            "movies": [{"name": "Sans vidéo", "thumbnail": "https://cdn.test/x.jpg", "mp4": {}, "webm": {}}],
            "screenshots": [{"path_thumbnail": "javascript:alert(1)", "path_full": "https://cdn.test/x.jpg"}]
          }}
        }"#;
        let media = parse(body, 10).unwrap();
        assert!(media.trailers.is_empty());
        assert!(media.screenshots.is_empty());
        assert!(parse(r#"{"10":{"success":false}}"#, 10).is_err());
    }

    #[test]
    #[ignore = "nécessite le réseau Steam"]
    fn le_vrai_jeu_a_une_bande_annonce_et_des_photos() {
        let media = fetch(&crate::artwork::http(), 292030, "french", "fr").unwrap();
        assert!(!media.trailers.is_empty());
        assert!(!media.screenshots.is_empty());
        assert!(media.trailers[0].video.starts_with("https://"));
        assert!(media.screenshots[0].thumbnail.starts_with("https://"));
    }
}
