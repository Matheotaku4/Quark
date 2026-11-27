//! Validation des entrées des commandes Tauri sensibles.

use std::path::{Path, PathBuf};

pub fn validate_open_url(url: &str) -> Result<(), String> {
    let parsed = url::Url::parse(url).map_err(|_| "URL invalide".to_string())?;
    match parsed.scheme() {
        "https" => Ok(()),
        "http" if matches!(parsed.host_str(), Some("localhost" | "127.0.0.1" | "::1")) => Ok(()),
        _ => Err("Seules les URL HTTPS sont autorisées".to_string()),
    }
}

pub fn validate_executable(path: &Path) -> Result<(), String> {
    let meta = std::fs::symlink_metadata(path).map_err(|_| "Fichier introuvable".to_string())?;
    if !meta.file_type().is_file() {
        return Err("L'exécutable doit être un fichier".to_string());
    }
    if path
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("exe"))
    {
        return Ok(());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if meta.permissions().mode() & 0o111 == 0 {
            return Err("Le fichier n'est pas exécutable".to_string());
        }
    }
    Ok(())
}

pub fn validate_scan_folder(path: &Path) -> Result<(), String> {
    let path = path
        .canonicalize()
        .map_err(|_| "Dossier introuvable".to_string())?;
    let home = quark::config::home_dir()
        .canonicalize()
        .map_err(|_| "Dossier personnel introuvable".to_string())?;
    if path == Path::new("/") || path == home {
        return Err("Dossier trop large".to_string());
    }
    for sensitive in [".ssh", ".gnupg", ".config"] {
        if path.starts_with(home.join(sensitive)) {
            return Err("Dossier sensible refusé".to_string());
        }
    }
    let allowed = [
        home.clone(),
        PathBuf::from("/media"),
        PathBuf::from("/mnt"),
        PathBuf::from("/run/media"),
    ];
    if allowed.iter().any(|root| path.starts_with(root)) {
        Ok(())
    } else {
        Err("Dossier hors des emplacements autorisés".to_string())
    }
}

pub fn validate_known_path(path: &Path, roots: &[PathBuf]) -> Result<(), String> {
    let path = path
        .canonicalize()
        .map_err(|_| "Chemin introuvable".to_string())?;
    if roots
        .iter()
        .any(|root| root.canonicalize().is_ok_and(|root| path.starts_with(root)))
    {
        Ok(())
    } else {
        Err("Chemin hors des dossiers autorisés".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refuse_les_urls_dangereuses() {
        assert!(validate_open_url("file:///etc/passwd").is_err());
        assert!(validate_open_url("javascript:alert(1)").is_err());
        assert!(validate_open_url("https://example.com").is_ok());
    }

    #[test]
    fn refuse_les_dossiers_sensibles() {
        assert!(validate_scan_folder(Path::new("/")).is_err());
        assert!(validate_scan_folder(&quark::config::home_dir()).is_err());
        assert!(validate_scan_folder(&quark::config::home_dir().join(".ssh")).is_err());
        let games = quark::config::home_dir().join(format!(".quark-games-{}", std::process::id()));
        std::fs::create_dir_all(&games).unwrap();
        assert!(validate_scan_folder(&games).is_ok());
        std::fs::remove_dir_all(&games).unwrap();
    }

    #[test]
    fn exige_un_fichier_executable() {
        let dir = std::env::temp_dir().join(format!("quark-val-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("game");
        std::fs::write(&file, b"#!/bin/sh\n").unwrap();
        assert!(validate_executable(&file).is_err());
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
        assert!(validate_executable(&file).is_ok());
    }
}
