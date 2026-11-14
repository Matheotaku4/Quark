//! Overlay DLL Windows installée à côté du jeu.
//!
//! La DLL est un proxy `d3d11.dll` : elle relaie les points d'entrée Direct3D
//! 11 au système, accroche `IDXGISwapChain::Present` et affiche un compteur FPS
//! discret, à la manière de Steam. Elle est compilée séparément dans
//! `tools/overlay-dll` (cible `x86_64-pc-windows-gnu`) puis embarquée ici.
//!
//! Quark ne remplace jamais un `d3d11.dll` étranger : beaucoup de jeux et de
//! mods en installent un. Seule une DLL portant le marqueur de l'overlay Quark
//! est mise à jour, pour que les mises à jour de Quark rafraîchissent leur
//! propre proxy. En cas de conflit, le lancement continue sans overlay et
//! l'utilisateur est prévenu.

use crate::config::Game;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Nom attendu par le chargeur Windows : la DLL est résolue depuis le dossier
/// de l'exécutable avant le dossier système.
pub const DLL_NAME: &str = "d3d11.dll";

/// Copie embarquée, reconstruite par `.github/workflows/release.yml`.
const DLL_BYTES: &[u8] = include_bytes!("../assets/overlay/quark-overlay.dll");

/// Nom de classe de la fenêtre de l'overlay, encodé en UTF-16LE dans la DLL.
/// Il sert uniquement à reconnaître nos propres installations : un
/// `d3d11.dll` système, celui d'un mod ou d'un autre outil ne le contient pas.
const OVERLAY_MARKER: &[u8] = b"Q\0u\0a\0r\0k\0O\0v\0e\0r\0l\0a\0y\0";

fn is_quark_overlay(bytes: &[u8]) -> bool {
    bytes
        .windows(OVERLAY_MARKER.len())
        .any(|window| window == OVERLAY_MARKER)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallOutcome {
    /// DLL écrite dans le dossier du jeu.
    Installed(PathBuf),
    /// Une DLL identique était déjà en place.
    UpToDate(PathBuf),
    /// Un autre `d3d11.dll` occupe la place : rien n'a été modifié.
    Conflict(PathBuf),
}

/// Installe la DLL dans le dossier de l'exécutable actif.
///
/// Un jeu Linux natif n'a pas besoin de proxy Direct3D : l'appelant doit
/// vérifier `looks_windows` avant de demander l'installation. Le fichier est
/// créé avec `create_new` : un `d3d11.dll` existant n'est jamais écrasé, même
/// si une autre instance de Quark écrit au même moment.
pub fn install_for_game(game: &Game) -> Result<InstallOutcome, String> {
    let exe = game.exe_path();
    let directory = exe
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .ok_or_else(|| "dossier du jeu introuvable".to_string())?;
    install_in(directory)
}

fn install_in(directory: &Path) -> Result<InstallOutcome, String> {
    let target = directory.join(DLL_NAME);
    match std::fs::symlink_metadata(&target) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                return Ok(InstallOutcome::Conflict(target));
            }
            let existing = std::fs::read(&target)
                .map_err(|error| format!("lecture de {} : {error}", target.display()))?;
            if existing == DLL_BYTES {
                return Ok(InstallOutcome::UpToDate(target));
            }
            if is_quark_overlay(&existing) {
                replace_managed_dll(&target)?;
                return Ok(InstallOutcome::Installed(target));
            }
            return Ok(InstallOutcome::Conflict(target));
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            return Err(format!("accès à {} : {error}", target.display()));
        }
    }

    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&target)
        .map_err(|error| format!("création de {} : {error}", target.display()))?;
    let written = file
        .write_all(DLL_BYTES)
        .and_then(|()| file.sync_all())
        .map_err(|error| format!("écriture de {} : {error}", target.display()));
    drop(file);
    if let Err(error) = written {
        let _ = std::fs::remove_file(&target);
        return Err(error);
    }
    Ok(InstallOutcome::Installed(target))
}

/// Remplace une installation que Quark a lui-même créée. Le fichier temporaire
/// est écrit puis renommé pour ne jamais laisser une DLL à moitié copiée.
fn replace_managed_dll(target: &Path) -> Result<(), String> {
    let temp = target.with_extension("dll.quark-overlay.tmp");
    std::fs::write(&temp, DLL_BYTES)
        .map_err(|error| format!("écriture de {} : {error}", temp.display()))?;
    std::fs::rename(&temp, target).map_err(|error| {
        let _ = std::fs::remove_file(&temp);
        format!("remplacement de {} : {error}", target.display())
    })
}

/// Variables lues par la DLL. Elles sont posées sur le processus du jeu, donc
/// visibles par le proxy même quand Quark est fermé.
pub fn launch_environment(corner: &str) -> Vec<(String, String)> {
    vec![
        ("QUARK_OVERLAY_FPS".to_string(), "1".to_string()),
        ("QUARK_OVERLAY_CORNER".to_string(), corner.to_string()),
    ]
}

/// Remplace une variable existante au lieu d'empiler deux valeurs.
pub fn set_env(env: &mut Vec<(String, String)>, name: &str, value: &str) {
    env.retain(|(key, _)| !key.eq_ignore_ascii_case(name));
    env.push((name.to_string(), value.to_string()));
}

/// Force Wine/Proton à charger le `d3d11.dll` du dossier du jeu avant celui du
/// préfixe. Sans cet override, le runtime peut continuer à utiliser sa propre
/// DLL et le proxy ne serait jamais chargé.
pub fn enable_wine_override(env: &mut Vec<(String, String)>) {
    let mut entries: Vec<String> = env
        .iter()
        .find(|(key, _)| key.eq_ignore_ascii_case("WINEDLLOVERRIDES"))
        .map(|(_, value)| {
            value
                .split(',')
                .map(str::trim)
                .filter(|entry| !entry.is_empty())
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default();
    entries.retain(|entry| !entry.to_ascii_lowercase().starts_with("d3d11="));
    entries.push("d3d11=n,b".to_string());
    set_env(env, "WINEDLLOVERRIDES", &entries.join(","));
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "quark-overlay-dll-{label}-{}-{nonce}",
            std::process::id()
        ));
        std::fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn la_dll_est_installee_puis_reconnue() {
        let dir = temp_dir("install");
        let first = install_in(&dir).unwrap();
        assert!(matches!(first, InstallOutcome::Installed(_)));
        assert_eq!(std::fs::read(dir.join(DLL_NAME)).unwrap(), DLL_BYTES);
        let second = install_in(&dir).unwrap();
        assert!(matches!(second, InstallOutcome::UpToDate(_)));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn une_ancienne_dll_quark_est_mise_a_jour() {
        let dir = temp_dir("update");
        let path = dir.join(DLL_NAME);
        let mut old = OVERLAY_MARKER.to_vec();
        old.extend_from_slice(b"ancienne version");
        std::fs::write(&path, &old).unwrap();
        let outcome = install_in(&dir).unwrap();
        assert_eq!(outcome, InstallOutcome::Installed(path.clone()));
        assert_eq!(std::fs::read(&path).unwrap(), DLL_BYTES);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn une_dll_etrangere_n_est_jamais_ecrasee() {
        let dir = temp_dir("conflict");
        let path = dir.join(DLL_NAME);
        std::fs::write(&path, b"un autre proxy").unwrap();
        let outcome = install_in(&dir).unwrap();
        assert_eq!(outcome, InstallOutcome::Conflict(path.clone()));
        assert_eq!(std::fs::read(&path).unwrap(), b"un autre proxy");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn l_override_wine_preserve_les_autres_dll() {
        let mut env = vec![(
            "WINEDLLOVERRIDES".to_string(),
            "d3d11=b,dxvk_config=n".to_string(),
        )];
        enable_wine_override(&mut env);
        assert_eq!(
            env,
            vec![(
                "WINEDLLOVERRIDES".to_string(),
                "dxvk_config=n,d3d11=n,b".to_string()
            )]
        );
    }

    #[test]
    fn les_variables_remplacent_une_valeur_precedente() {
        let mut env = vec![
            ("QUARK_OVERLAY_FPS".to_string(), "0".to_string()),
            ("AUTRE".to_string(), "x".to_string()),
        ];
        for (name, value) in launch_environment("top-left") {
            set_env(&mut env, &name, &value);
        }
        assert_eq!(
            env,
            vec![
                ("AUTRE".to_string(), "x".to_string()),
                ("QUARK_OVERLAY_FPS".to_string(), "1".to_string()),
                ("QUARK_OVERLAY_CORNER".to_string(), "top-left".to_string()),
            ]
        );
    }
}
