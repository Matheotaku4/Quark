//! Commandes complémentaires de l'interface : réglages et jeux en JSON brut
//! (toutes les options du cœur, sans DTO à maintenir), réseau (actualités,
//! HowLongToBeat, ProtonDB), disque et mises à jour.

use super::{game_dto, lock, persist, AppState, GameDto};
use quark::config::{Game, Settings};
use quark::{artwork, disk, extensions, howlongtobeat, news, proton, protondb, update};
use serde::Serialize;
use serde_json::Value;
use std::path::{Path, PathBuf};
use tauri::State;

/// Fusionne `patch` dans `target` clé par clé (les sous-objets sont remplacés
/// en entier : l'interface renvoie toujours l'objet complet).
fn merge(target: &mut Value, patch: Value) {
    match (target, patch) {
        (Value::Object(target), Value::Object(patch)) => {
            for (key, value) in patch {
                target.insert(key, value);
            }
        }
        (target, patch) => *target = patch,
    }
}

async fn blocking<T: Send + 'static>(
    job: impl FnOnce() -> Result<T, String> + Send + 'static,
) -> Result<T, String> {
    tauri::async_runtime::spawn_blocking(job)
        .await
        .map_err(|error| error.to_string())?
}

// ---------------------------------------------------------------------------
// Réglages et jeux, en JSON
// ---------------------------------------------------------------------------

#[tauri::command]
pub fn get_settings_raw(state: State<'_, AppState>) -> Result<Value, String> {
    serde_json::to_value(&lock(&state.library).settings).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn patch_settings_raw(state: State<'_, AppState>, patch: Value) -> Result<Value, String> {
    let mut library = lock(&state.library);
    let mut current = serde_json::to_value(&library.settings).map_err(|error| error.to_string())?;
    merge(&mut current, patch);
    let next: Settings = serde_json::from_value(current.clone())
        .map_err(|error| format!("Réglage invalide : {error}"))?;
    library.settings = next;
    persist(&state, &mut library)?;
    Ok(current)
}

#[tauri::command]
pub fn get_game_raw(state: State<'_, AppState>, id: String) -> Result<Value, String> {
    let library = lock(&state.library);
    let game = library
        .get(&id)
        .ok_or_else(|| "Jeu introuvable".to_string())?;
    serde_json::to_value(game).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn patch_game_raw(
    state: State<'_, AppState>,
    id: String,
    patch: Value,
) -> Result<GameDto, String> {
    let mut library = lock(&state.library);
    let mut current = {
        let game = library
            .get(&id)
            .ok_or_else(|| "Jeu introuvable".to_string())?;
        serde_json::to_value(game).map_err(|error| error.to_string())?
    };
    merge(&mut current, patch);
    let next: Game =
        serde_json::from_value(current).map_err(|error| format!("Réglage invalide : {error}"))?;
    *library
        .get_mut(&id)
        .ok_or_else(|| "Jeu introuvable".to_string())? = next;
    persist(&state, &mut library)?;
    let settings = library.settings.clone();
    let sessions = lock(&state.sessions);
    let run = sessions.get(&id);
    let game = library
        .get(&id)
        .ok_or_else(|| "Jeu introuvable".to_string())?;
    Ok(game_dto(
        game,
        &settings,
        run.is_some(),
        run.map(|run| run.label.clone()),
    ))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DllCandidateDto {
    pub name: String,
    pub path: String,
    pub relative: String,
}

const DLL_SCAN_DEPTH: usize = 6;
const DLL_SCAN_LIMIT: usize = 400;

/// Liste les DLL visibles dans le dossier du jeu (ou dans un dossier choisi),
/// sous-dossiers compris. Les noms sont dédoublonnés sans tenir compte de la
/// casse : `WINEDLLOVERRIDES` ne peut cibler qu'un module à la fois.
#[tauri::command]
pub fn detect_game_dlls(
    state: State<'_, AppState>,
    id: String,
    folder: Option<String>,
) -> Result<Vec<DllCandidateDto>, String> {
    let root = {
        let library = lock(&state.library);
        let game = library
            .get(&id)
            .ok_or_else(|| "Jeu introuvable".to_string())?;
        folder
            .as_deref()
            .map(str::trim)
            .filter(|path| !path.is_empty())
            .map(PathBuf::from)
            .or_else(|| {
                game.dll_source
                    .as_deref()
                    .map(str::trim)
                    .filter(|path| !path.is_empty())
                    .map(PathBuf::from)
            })
            .or_else(|| game.exe_path().parent().map(Path::to_path_buf))
            .ok_or_else(|| "Dossier du jeu introuvable.".to_string())?
    };
    if !root.is_dir() {
        return Err(format!("Dossier introuvable : {}", root.display()));
    }
    Ok(scan_dlls(&root))
}

fn scan_dlls(root: &Path) -> Vec<DllCandidateDto> {
    let mut out = Vec::new();
    collect_dlls(root, root, 0, &mut out);
    out.sort_by_key(|row| row.relative.to_lowercase());
    out
}

fn collect_dlls(root: &Path, dir: &Path, depth: usize, out: &mut Vec<DllCandidateDto>) {
    if out.len() >= DLL_SCAN_LIMIT {
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        if out.len() >= DLL_SCAN_LIMIT {
            return;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            continue;
        }
        let path = entry.path();
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if file_type.is_symlink() {
            continue;
        }
        if file_type.is_dir() {
            if depth < DLL_SCAN_DEPTH {
                collect_dlls(root, &path, depth + 1, out);
            }
            continue;
        }
        if !path
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("dll"))
        {
            continue;
        }
        let Some(stem) = path
            .file_stem()
            .map(|stem| stem.to_string_lossy().to_string())
        else {
            continue;
        };
        if stem.is_empty() || out.iter().any(|row| row.name.eq_ignore_ascii_case(&stem)) {
            continue;
        }
        out.push(DllCandidateDto {
            name: stem,
            path: path.display().to_string(),
            relative: path
                .strip_prefix(root)
                .unwrap_or(&path)
                .display()
                .to_string(),
        });
    }
}

#[derive(Serialize)]
pub struct RuntimeDto {
    key: String,
    name: String,
    version: Option<String>,
}

/// Détection à chaud : « Détecter à nouveau » dans les réglages.
#[tauri::command]
pub async fn list_runtimes() -> Result<Vec<RuntimeDto>, String> {
    blocking(|| {
        Ok(proton::detect_runtimes()
            .into_iter()
            .filter(|runtime| runtime.has_proton_script)
            .map(|runtime| RuntimeDto {
                key: runtime.key,
                name: runtime.name,
                version: runtime.version,
            })
            .collect())
    })
    .await
}

// ---------------------------------------------------------------------------
// Réseau : actualités, HowLongToBeat, ProtonDB
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct NewsDto {
    title: String,
    url: String,
    feed: String,
    author: String,
    date: i64,
    excerpt: String,
    body: String,
}

#[tauri::command]
pub async fn fetch_news(app_id: u32, count: Option<usize>) -> Result<Vec<NewsDto>, String> {
    blocking(move || {
        let items = news::fetch(&artwork::http(), app_id, count.unwrap_or(8))?;
        Ok(items
            .into_iter()
            .map(|item| NewsDto {
                title: item.title,
                url: item.url,
                feed: item.feed,
                author: item.author,
                date: item.date,
                excerpt: item.excerpt,
                body: item.body,
            })
            .collect())
    })
    .await
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HltbDto {
    url: String,
    main: Option<f64>,
    main_extra: Option<f64>,
    completionist: Option<f64>,
}

#[tauri::command]
pub async fn fetch_hltb(title: String) -> Result<HltbDto, String> {
    blocking(move || {
        let report = howlongtobeat::fetch(&artwork::http(), &title)?;
        Ok(HltbDto {
            url: report.url(),
            main: report.main,
            main_extra: report.main_extra,
            completionist: report.completionist,
        })
    })
    .await
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProtonDbDto {
    tier: String,
    label: String,
    color: String,
    total: u32,
    confidence: String,
}

#[tauri::command]
pub async fn fetch_protondb(app_id: u32) -> Result<ProtonDbDto, String> {
    blocking(move || {
        let report = protondb::fetch(&artwork::http(), app_id)?;
        let (r, g, b) = report.tier.color();
        Ok(ProtonDbDto {
            tier: report.tier.label().to_string(),
            label: report.label(),
            color: format!("#{r:02x}{g:02x}{b:02x}"),
            total: report.total,
            confidence: report.confidence,
        })
    })
    .await
}

// ---------------------------------------------------------------------------
// Disque, mises à jour, liens
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct DiskRowDto {
    label: String,
    path: String,
    bytes: u64,
    human: String,
    detail: String,
}

#[derive(Serialize)]
pub struct DiskDto {
    rows: Vec<DiskRowDto>,
    total: u64,
    human: String,
}

#[tauri::command]
pub async fn disk_report(state: State<'_, AppState>) -> Result<DiskDto, String> {
    let library = lock(&state.library).clone();
    blocking(move || {
        let report = disk::scan(&library);
        Ok(DiskDto {
            total: report.total,
            human: disk::format_bytes(report.total),
            rows: report
                .rows
                .into_iter()
                .map(|row| DiskRowDto {
                    label: row.label,
                    path: row.path.display().to_string(),
                    bytes: row.bytes,
                    human: disk::format_bytes(row.bytes),
                    detail: row.detail,
                })
                .collect(),
        })
    })
    .await
}

#[derive(Serialize)]
pub struct UpdateDto {
    current: String,
    tag: String,
    name: String,
    url: String,
    newer: bool,
}

#[tauri::command]
pub async fn check_update() -> Result<UpdateDto, String> {
    blocking(|| {
        let release = update::latest(&artwork::http())?;
        Ok(UpdateDto {
            current: update::current_version().to_string(),
            newer: update::is_newer(&release.tag_name, update::current_version()),
            tag: release.tag_name,
            name: release.name,
            url: release.html_url,
        })
    })
    .await
}

/// Télécharge la dernière release, vérifie son SHA-256 et son ELF, remplace le
/// binaire courant puis relance Quark. `exec` ne revient pas en cas de succès.
#[tauri::command]
pub async fn install_update() -> Result<(), String> {
    blocking(|| {
        if !update::automatic_update_allowed() {
            return Err("La mise à jour intégrée est désactivée pour cette exécution.".to_string());
        }
        let client = artwork::http();
        let release = update::latest(&client)?;
        if !update::is_newer(&release.tag_name, update::current_version()) {
            return Err("Quark est déjà à jour.".to_string());
        }
        let asset = release
            .archive()
            .ok_or_else(|| "La release ne publie pas d'archive Linux.".to_string())?;
        let target = std::env::current_exe()
            .map_err(|error| format!("Binaire courant introuvable : {error}"))?;
        let directory = quark::config::data_dir().join("updates");
        let staged = update::stage(&client, &release, asset, &directory, &target)?;
        let installed = update::apply(&staged)?;
        if let Err(error) = update::relaunch(&target) {
            let _ = update::rollback(&installed);
            return Err(error);
        }
        Ok(())
    })
    .await
}

/// Ouvre un lien ou un dossier avec l'application par défaut du bureau.
#[tauri::command]
pub fn open_url(target: String) -> Result<(), String> {
    if target.starts_with("http://") || target.starts_with("https://") {
        crate::validators::validate_open_url(&target)?;
        news::open_in_browser(&target)
    } else {
        let path = std::path::Path::new(&target);
        let home = quark::config::home_dir();
        crate::validators::validate_known_path(
            path,
            &[home, "/media".into(), "/mnt".into(), "/run/media".into()],
        )?;
        std::process::Command::new("xdg-open")
            .arg(&target)
            .spawn()
            .map(|_| ())
            .map_err(|error| format!("xdg-open : {error}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(label: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "quark-dll-scan-{label}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn les_dll_des_sous_dossiers_sont_trouvees_et_dedoublonnees() {
        let root = temp_dir("nested");
        std::fs::create_dir_all(root.join("Binaries/Win64")).unwrap();
        std::fs::write(root.join("winmm.dll"), b"dll").unwrap();
        std::fs::write(root.join("Binaries/Win64/winmm.DLL"), b"dll").unwrap();
        std::fs::write(root.join("Binaries/Win64/steam_api64.dll"), b"dll").unwrap();
        std::fs::write(root.join("Binaries/readme.txt"), b"txt").unwrap();

        let found = scan_dlls(&root);
        let names: Vec<_> = found.iter().map(|row| row.name.as_str()).collect();
        assert_eq!(names, ["steam_api64", "winmm"]);
        assert_eq!(found[1].relative, "winmm.dll");
        assert!(found[0].relative.ends_with("steam_api64.dll"));

        std::fs::remove_dir_all(root).unwrap();
    }
}

// ---------------------------------------------------------------------------
// Ajout par scan, recherche de sites et choix d'illustrations
// ---------------------------------------------------------------------------

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanHitDto {
    exe: String,
    title: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SiteHitDto {
    site_id: String,
    site: String,
    title: String,
    url: String,
    detail: String,
    cover: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SiteErrorDto {
    site_id: String,
    site: String,
    message: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SiteSearchDto {
    hits: Vec<SiteHitDto>,
    errors: Vec<SiteErrorDto>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtworkCandidateDto {
    source: String,
    title: String,
    kind: String,
    url: String,
    preview: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtworkCandidatesDto {
    items: Vec<ArtworkCandidateDto>,
    warnings: Vec<String>,
}

#[tauri::command]
pub async fn scan_folder(
    folder: String,
    max_depth: u32,
    limit: u32,
) -> Result<Vec<ScanHitDto>, String> {
    blocking(move || {
        let root = Path::new(&folder);
        crate::validators::validate_scan_folder(root)?;
        let depth = max_depth.clamp(1, 8) as usize;
        let limit = limit.clamp(1, 100) as usize;
        Ok(quark::scan::scan(root, depth, limit)
            .into_iter()
            .map(|item| ScanHitDto {
                exe: item.exe.display().to_string(),
                title: item.title,
            })
            .collect())
    })
    .await
}

#[tauri::command]
pub async fn extensions_add(
    state: State<'_, AppState>,
    url: String,
) -> Result<Vec<extensions::ExtensionStatus>, String> {
    let (sources, path) = extension_snapshot(&state);
    let url = url.trim().to_string();
    let (sources, statuses) = blocking(move || {
        let mut cache = extensions::load_cache_from(&path);
        let mut sources = sources;
        extensions::add_source(&mut cache, &mut sources, &url)?;
        extensions::save_cache_to(&path, &cache)?;
        let statuses = extensions::statuses(&cache, &sources);
        Ok((sources, statuses))
    })
    .await?;
    persist_extension_sources(&state, sources)?;
    Ok(statuses)
}

#[tauri::command]
pub fn extensions_remove(
    state: State<'_, AppState>,
    url: String,
) -> Result<Vec<extensions::ExtensionStatus>, String> {
    let path = extensions::cache_path();
    let mut cache = extensions::load_cache_from(&path);
    let mut sources = lock(&state.library).settings.extension_sources.clone();
    extensions::remove_source(&mut cache, &mut sources, url.trim());
    extensions::save_cache_to(&path, &cache)?;
    let statuses = extensions::statuses(&cache, &sources);
    persist_extension_sources(&state, sources)?;
    Ok(statuses)
}

#[tauri::command]
pub async fn extensions_refresh(
    state: State<'_, AppState>,
    url: Option<String>,
) -> Result<Vec<extensions::ExtensionStatus>, String> {
    let (sources, path) = extension_snapshot(&state);
    let target = url
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    let (sources, statuses) = blocking(move || {
        let mut cache = extensions::load_cache_from(&path);
        let mut sources = sources;
        let urls = match target {
            Some(url) => vec![url],
            None => sources.clone(),
        };
        let mut statuses = Vec::new();
        for url in urls {
            match extensions::refresh_source(&mut cache, &mut sources, &url) {
                Ok(status) => statuses.push(status),
                Err(error) => {
                    let mut status = extensions::statuses(&cache, std::slice::from_ref(&url))
                        .into_iter()
                        .next()
                        .unwrap_or_else(|| extensions::ExtensionStatus {
                            url: url.clone(),
                            id: String::new(),
                            name: String::new(),
                            version: String::new(),
                            site_count: 0,
                            fetched_at: 0,
                            stale: true,
                            error: None,
                        });
                    status.error = Some(error);
                    statuses.push(status);
                }
            }
        }
        extensions::save_cache_to(&path, &cache)?;
        Ok((sources, statuses))
    })
    .await?;
    persist_extension_sources(&state, sources)?;
    Ok(statuses)
}

fn extension_snapshot(state: &AppState) -> (Vec<String>, PathBuf) {
    (
        lock(&state.library).settings.extension_sources.clone(),
        extensions::cache_path(),
    )
}

fn persist_extension_sources(state: &AppState, sources: Vec<String>) -> Result<(), String> {
    let mut library = lock(&state.library);
    library.settings.extension_sources = sources;
    persist(state, &mut library)
}

#[tauri::command]
pub async fn site_search(
    state: State<'_, AppState>,
    query: String,
) -> Result<SiteSearchDto, String> {
    let sources = lock(&state.library).settings.extension_sources.clone();
    blocking(move || {
        let query = query.trim().to_string();
        if query.len() < 2 {
            return Ok(SiteSearchDto {
                hits: Vec::new(),
                errors: Vec::new(),
            });
        }
        let cache = extensions::load_cache_from(&extensions::cache_path());
        let sites = extensions::active_sites(&cache, &sources);
        let mut per_site = Vec::new();
        std::thread::scope(|scope| {
            let mut handles = Vec::new();
            for site in &sites {
                let query = query.clone();
                handles.push((
                    site,
                    scope.spawn(move || quark::game_search::search(&site.site, &query)),
                ));
            }
            for (site, handle) in handles {
                per_site.push((
                    site,
                    handle
                        .join()
                        .unwrap_or_else(|_| Err("Recherche interrompue.".to_string())),
                ));
            }
        });
        let mut hits = Vec::new();
        let mut errors = Vec::new();
        for (site, result) in per_site {
            let site_id = site.uid.clone();
            let label = site.site.label.clone();
            match result {
                Ok(found) => {
                    for hit in found {
                        hits.push(SiteHitDto {
                            site_id: site_id.clone(),
                            site: label.clone(),
                            title: hit.title,
                            url: hit.url,
                            detail: hit.detail,
                            cover: hit.cover,
                        });
                    }
                }
                Err(message) => errors.push(SiteErrorDto {
                    site_id,
                    site: label,
                    message,
                }),
            }
        }
        Ok(SiteSearchDto { hits, errors })
    })
    .await
}

#[tauri::command]
pub async fn artwork_candidates(
    state: State<'_, AppState>,
    id: String,
    query: Option<String>,
) -> Result<ArtworkCandidatesDto, String> {
    let (name, settings) = {
        let library = lock(&state.library);
        let game = library
            .get(&id)
            .ok_or_else(|| "Jeu introuvable.".to_string())?;
        (game.name.clone(), library.settings.clone())
    };
    blocking(move || {
        let query = query
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .unwrap_or(name);
        let client = artwork::http();
        let outcome = artwork::search(&client, &query, &settings);
        let mut items = Vec::new();
        let mut warnings = outcome.warnings.clone();
        for hit in outcome.hits.iter().take(4) {
            let plan = artwork::build_plan(&client, hit, &settings);
            warnings.extend(plan.warnings.iter().cloned());
            for kind in quark::config::AssetKind::ALL {
                for (index, url) in plan.candidates(kind).iter().take(8).enumerate() {
                    if items
                        .iter()
                        .any(|item: &ArtworkCandidateDto| item.url == *url)
                    {
                        continue;
                    }
                    items.push(ArtworkCandidateDto {
                        source: hit.source.label().to_string(),
                        title: hit.title.clone(),
                        kind: kind.stem().to_string(),
                        url: url.clone(),
                        preview: plan.preview(kind, index).map(str::to_string),
                    });
                }
            }
        }
        warnings.sort();
        warnings.dedup();
        Ok(ArtworkCandidatesDto { items, warnings })
    })
    .await
}

#[tauri::command]
pub async fn artwork_choose(
    state: State<'_, AppState>,
    id: String,
    kind: String,
    url: String,
) -> Result<GameDto, String> {
    let kind = match kind.as_str() {
        "cover" => quark::config::AssetKind::Cover,
        "hero" => quark::config::AssetKind::Hero,
        "logo" => quark::config::AssetKind::Logo,
        "icon" => quark::config::AssetKind::Icon,
        _ => return Err("Type d'illustration inconnu.".to_string()),
    };
    let url = url.trim().to_string();
    if !(url.starts_with("https://") || url.starts_with("http://")) {
        return Err("Adresse d'image invalide.".to_string());
    }
    let game_id = id.clone();
    let path = blocking(move || {
        let client = artwork::http();
        artwork::download_asset(&client, &game_id, kind, &url)
    })
    .await?;
    let mut library = lock(&state.library);
    let settings = library.settings.clone();
    {
        let game = library
            .get_mut(&id)
            .ok_or_else(|| "Jeu introuvable.".to_string())?;
        game.artwork.set(kind, path);
    }
    persist(&state, &mut library)?;
    let game = library
        .get(&id)
        .ok_or_else(|| "Jeu introuvable.".to_string())?;
    Ok(game_dto(game, &settings, false, None))
}
