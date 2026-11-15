//! Overlay DLL Windows installée à côté du jeu.
//!
//! Le même binaire est un proxy `d3d11.dll` et `dxgi.dll` : il relaie Direct3D
//! 11 et DXGI au système, accroche `Present`/`Present1` et affiche un compteur
//! FPS discret, à la manière de Steam. La couverture DXGI inclut les swap chains
//! Direct3D 12. Il est compilé séparément dans `tools/overlay-dll` (cible
//! `x86_64-pc-windows-gnu`) puis embarqué ici.
//!
//! Quark ne remplace jamais une DLL étrangère : beaucoup de jeux et de mods
//! installent leur propre `d3d11.dll` ou `dxgi.dll`. Seule une DLL portant le
//! marqueur de l'overlay Quark est mise à jour, pour que les mises à jour de
//! Quark rafraîchissent leur propre proxy. En cas de conflit, le lancement
//! continue sans overlay et l'utilisateur est prévenu.

use crate::config::Game;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Noms attendus par le chargeur Windows : les DLL sont résolues depuis le
/// dossier de l'exécutable avant le dossier système.
pub const DLL_NAME: &str = "d3d11.dll";
pub const DXGI_DLL_NAME: &str = "dxgi.dll";
const DLL_NAMES: [&str; 2] = [DLL_NAME, DXGI_DLL_NAME];

/// Copie embarquée, reconstruite par `.github/workflows/release.yml`.
const DLL_BYTES: &[u8] = include_bytes!("../assets/overlay/quark-overlay.dll");

/// Nom de classe de la fenêtre de l'overlay, encodé en UTF-16LE dans la DLL.
/// Il sert uniquement à reconnaître nos propres installations : une DLL
/// système, celle d'un mod ou d'un autre outil ne le contient pas.
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
    /// Un autre `d3d11.dll` ou `dxgi.dll` occupe la place : rien n'a été
    /// modifié.
    Conflict(PathBuf),
}

/// Installe les DLL proxy dans le dossier de l'exécutable actif.
///
/// Un jeu Linux natif n'a pas besoin de proxy Direct3D : l'appelant doit
/// vérifier `looks_windows` avant de demander l'installation. Les deux fichiers
/// sont créés avec `create_new` : une DLL étrangère n'est jamais écrasée, même
/// si une autre instance de Quark écrit au même moment.
pub fn install_for_game(game: &Game) -> Result<InstallOutcome, String> {
    let exe = game.exe_path();
    let directory = exe
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .ok_or_else(|| "dossier du jeu introuvable".to_string())?;
    install_in(directory)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum TargetState {
    Missing,
    Current,
    Managed,
    Foreign,
}

fn inspect_target(target: &Path) -> Result<TargetState, String> {
    match std::fs::symlink_metadata(target) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                return Ok(TargetState::Foreign);
            }
            let existing = std::fs::read(target)
                .map_err(|error| format!("lecture de {} : {error}", target.display()))?;
            if existing == DLL_BYTES {
                Ok(TargetState::Current)
            } else if is_quark_overlay(&existing) {
                Ok(TargetState::Managed)
            } else {
                Ok(TargetState::Foreign)
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(TargetState::Missing),
        Err(error) => Err(format!("accès à {} : {error}", target.display())),
    }
}

fn install_in(directory: &Path) -> Result<InstallOutcome, String> {
    let targets = DLL_NAMES
        .iter()
        .map(|name| {
            let path = directory.join(name);
            inspect_target(&path).map(|state| (path, state))
        })
        .collect::<Result<Vec<_>, _>>()?;
    if let Some((path, TargetState::Foreign)) = targets
        .iter()
        .find(|(_, state)| *state == TargetState::Foreign)
    {
        return Ok(InstallOutcome::Conflict(path.clone()));
    }

    let mut wrote = false;
    for (target, state) in targets {
        match state {
            TargetState::Current => {}
            TargetState::Missing => {
                write_new_dll(&target)?;
                wrote = true;
            }
            TargetState::Managed => {
                replace_managed_dll(&target)?;
                wrote = true;
            }
            TargetState::Foreign => unreachable!("les conflits sont traités avant l'écriture"),
        }
    }
    let path = directory.join(DLL_NAME);
    Ok(if wrote {
        InstallOutcome::Installed(path)
    } else {
        InstallOutcome::UpToDate(path)
    })
}

fn write_new_dll(target: &Path) -> Result<(), String> {
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(target)
        .map_err(|error| format!("création de {} : {error}", target.display()))?;
    let written = file
        .write_all(DLL_BYTES)
        .and_then(|()| file.sync_all())
        .map_err(|error| format!("écriture de {} : {error}", target.display()));
    drop(file);
    if let Err(error) = written {
        let _ = std::fs::remove_file(target);
        return Err(error);
    }
    Ok(())
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

/// Chemin du journal de diagnostic de la DLL, dans le dossier de logs du jeu.
///
/// Wine expose la racine Linux sous `Z:` ; la DLL Windows ne peut pas ouvrir un
/// chemin POSIX. Le fichier est créé par Quark en `0600` avant le lancement,
/// pour ne pas dépendre des permissions par défaut de Wine.
pub fn overlay_log_path(game_id: &str) -> String {
    let path = crate::config::log_dir(game_id).join("overlay.log");
    let path = path.to_string_lossy();
    format!("Z:{}", path.replace('/', "\\"))
}

/// Crée (ou vide) le journal avec des droits propriétaire uniquement. Le jeu
/// Windows n'a alors qu'à l'ouvrir en ajout ; s'il ne peut pas, aucune variable
/// de journal n'est transmise.
fn prepare_overlay_log(game_id: &str) -> bool {
    let path = crate::config::log_dir(game_id).join("overlay.log");
    let Some(directory) = path.parent() else {
        return false;
    };
    if std::fs::create_dir_all(directory).is_err() {
        return false;
    }
    let mut options = std::fs::OpenOptions::new();
    options.create(true).write(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt as _;
        options.mode(0o600);
    }
    let Ok(file) = options.open(&path) else {
        return false;
    };
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        if file
            .set_permissions(std::fs::Permissions::from_mode(0o600))
            .is_err()
        {
            return false;
        }
    }
    true
}

/// Variables de lancement, journal compris. Le journal permet de distinguer
/// « DLL non chargée », « API non couverte » et « swap chain non accrochée »
/// sans demander au joueur de lancer un débogueur Windows.
pub fn launch_environment_for(game: &Game, corner: &str) -> Vec<(String, String)> {
    let mut env = launch_environment(corner);
    if prepare_overlay_log(&game.id) {
        env.push(("QUARK_OVERLAY_LOG".to_string(), overlay_log_path(&game.id)));
    }
    env
}

/// Remplace une variable existante au lieu d'empiler deux valeurs.
pub fn set_env(env: &mut Vec<(String, String)>, name: &str, value: &str) {
    env.retain(|(key, _)| !key.eq_ignore_ascii_case(name));
    env.push((name.to_string(), value.to_string()));
}

/// Force Wine/Proton à charger les `d3d11.dll` et `dxgi.dll` du dossier du jeu
/// avant celles du préfixe. Sans ces overrides, le runtime peut continuer à
/// utiliser ses propres DLL et le proxy ne serait jamais chargé.
pub fn enable_wine_override(env: &mut Vec<(String, String)>) {
    let mut entries: Vec<String> = env
        .iter()
        .find(|(key, _)| key.eq_ignore_ascii_case("WINEDLLOVERRIDES"))
        .map(|(_, value)| {
            value
                .split(';')
                .filter_map(without_overlay_module)
                .collect()
        })
        .unwrap_or_default();
    entries.push("d3d11=n,b".to_string());
    entries.push("dxgi=n,b".to_string());
    set_env(env, "WINEDLLOVERRIDES", &entries.join(";"));
}

/// Filtre une entrée `module[,module...]=ordre` de `WINEDLLOVERRIDES`.
///
/// Les entrées sont séparées par des points-virgules ; les virgules séparent
/// uniquement plusieurs modules qui partagent le même ordre de chargement. Il
/// faut donc retirer `d3d11`/`dxgi` sans confondre une liste de modules avec
/// une nouvelle entrée.
fn without_overlay_module(entry: &str) -> Option<String> {
    let entry = entry.trim();
    if entry.is_empty() {
        return None;
    }
    let Some((modules, order)) = entry.split_once('=') else {
        return Some(entry.to_string());
    };
    let kept: Vec<&str> = modules
        .split(',')
        .map(str::trim)
        .filter(|module| !module.is_empty() && !is_overlay_module(module))
        .collect();
    if kept.is_empty() {
        None
    } else {
        Some(format!("{}={}", kept.join(","), order.trim()))
    }
}

fn is_overlay_module(module: &str) -> bool {
    ["d3d11", "d3d11.dll", "dxgi", "dxgi.dll"]
        .iter()
        .any(|name| module.trim().eq_ignore_ascii_case(name))
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
        assert_eq!(std::fs::read(dir.join(DXGI_DLL_NAME)).unwrap(), DLL_BYTES);
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
        assert_eq!(std::fs::read(dir.join(DXGI_DLL_NAME)).unwrap(), DLL_BYTES);
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
        assert!(!dir.join(DXGI_DLL_NAME).exists());
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn une_dll_dxgi_etrangere_bloque_l_overlay() {
        let dir = temp_dir("conflict-dxgi");
        let path = dir.join(DXGI_DLL_NAME);
        std::fs::write(&path, b"dxgi d'un autre outil").unwrap();
        let outcome = install_in(&dir).unwrap();
        assert_eq!(outcome, InstallOutcome::Conflict(path.clone()));
        assert!(!dir.join(DLL_NAME).exists());
        assert_eq!(std::fs::read(&path).unwrap(), b"dxgi d'un autre outil");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn l_override_wine_preserve_les_autres_dll() {
        let mut env = vec![(
            "WINEDLLOVERRIDES".to_string(),
            "dxvk_config=n;d3d11=b".to_string(),
        )];
        enable_wine_override(&mut env);
        assert_eq!(
            env,
            vec![(
                "WINEDLLOVERRIDES".to_string(),
                "dxvk_config=n;d3d11=n,b;dxgi=n,b".to_string()
            )]
        );
    }

    #[test]
    fn les_entrees_multiples_de_wine_sont_preservees() {
        let mut env = vec![(
            "WINEDLLOVERRIDES".to_string(),
            "comdlg32,shell32=n,b;dxgi=b;foo=n".to_string(),
        )];
        enable_wine_override(&mut env);
        assert_eq!(
            env,
            vec![(
                "WINEDLLOVERRIDES".to_string(),
                "comdlg32,shell32=n,b;foo=n;d3d11=n,b;dxgi=n,b".to_string()
            )]
        );
    }

    #[test]
    fn le_journal_overlay_est_dans_un_chemin_wine_absolu() {
        let path = overlay_log_path("jeu-test");
        assert!(path.starts_with("Z:\\"), "chemin Wine attendu : {path}");
        assert!(
            path.ends_with("overlay.log"),
            "nom du journal attendu : {path}"
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
