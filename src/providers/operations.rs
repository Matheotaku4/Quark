//! Provider title matching and installation lifecycle state.

use super::model::{InstallState, ManualGameGroup, ProviderGame, ProviderGameKey};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub enum OperationState {
    Queued,
    Running {
        progress: Option<f32>,
        message: String,
    },
    Succeeded {
        install_path: PathBuf,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct InstallOperation {
    pub key: ProviderGameKey,
    pub destination: PathBuf,
    pub state: OperationState,
}

impl InstallOperation {
    pub fn new(key: ProviderGameKey, destination: PathBuf) -> Self {
        Self {
            key,
            destination,
            state: OperationState::Queued,
        }
    }

    pub fn start(&mut self) {
        self.state = OperationState::Running {
            progress: None,
            message: "Installation en cours".into(),
        };
    }

    pub fn complete_verified(&mut self, install_path: PathBuf) -> Result<(), String> {
        if !matches!(self.state, OperationState::Running { .. }) {
            return Err("installation non confirmée ou non démarrée".into());
        }
        if !install_path.is_absolute() || !install_path.is_dir() {
            return Err("dossier d'installation absent ou invalide".into());
        }
        let destination = self
            .destination
            .canonicalize()
            .map_err(|_| "destination d'installation absente ou invalide".to_string())?;
        let install_path = install_path
            .canonicalize()
            .map_err(|_| "dossier d'installation absent ou invalide".to_string())?;
        if !install_path.starts_with(&destination) {
            return Err("installation détectée hors du dossier de destination choisi".into());
        }
        self.state = OperationState::Succeeded { install_path };
        Ok(())
    }
}

/// Remet d'aplomb les états laissés par une installation interrompue.
///
/// Aucun téléchargement ne survit à la fermeture de Quark : un état
/// transitoire au démarrage est donc périmé. Si le dossier existe encore, le
/// jeu est réellement installé ; sinon il redevient disponible au
/// téléchargement. Sans cela, un jeu Epic présent sur le disque restait
/// bloqué sur « Installation… » pour toujours.
pub fn recover_interrupted_installs(games: &mut [ProviderGame]) -> usize {
    let mut recovered = 0;
    for game in games {
        if matches!(
            game.install_state,
            InstallState::Installing
                | InstallState::Queued
                | InstallState::Updating
                | InstallState::Repairing
        ) {
            let installed = game.install_path.as_ref().is_some_and(|path| {
                crate::providers::install_directory_is_complete(&game.key, path)
            });
            game.install_state = if installed {
                InstallState::Installed
            } else {
                InstallState::NotInstalled
            };
            if !installed {
                game.install_path = None;
            }
            recovered += 1;
        }
    }
    recovered
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchCandidate {
    pub normalized_title: String,
    pub members: Vec<ProviderGameKey>,
}

pub fn match_candidates(games: &[ProviderGame]) -> Vec<MatchCandidate> {
    let mut grouped: std::collections::BTreeMap<String, Vec<ProviderGameKey>> =
        std::collections::BTreeMap::new();
    for game in games {
        let title = normalize_title(&game.title);
        if !title.is_empty() {
            grouped.entry(title).or_default().push(game.key.clone());
        }
    }
    grouped
        .into_iter()
        .filter_map(|(normalized_title, members)| {
            (members.len() > 1).then_some(MatchCandidate {
                normalized_title,
                members,
            })
        })
        .collect()
}

pub fn normalize_title(title: &str) -> String {
    title
        .chars()
        .flat_map(char::to_lowercase)
        .filter(|character| character.is_alphanumeric())
        .collect()
}

pub fn merge_group(members: &[ProviderGameKey]) -> Result<ManualGameGroup, String> {
    if members.len() < 2 {
        return Err("une fusion manuelle nécessite au moins deux jeux".into());
    }
    let unique: std::collections::HashSet<_> = members.iter().collect();
    if unique.len() != members.len() {
        return Err("un même jeu ne peut pas apparaître deux fois dans un groupe".into());
    }
    Ok(ManualGameGroup {
        members: members.to_vec(),
    })
}

pub fn split_member(group: &mut ManualGameGroup, member: &ProviderGameKey) {
    group.members.retain(|candidate| candidate != member);
}

/// Supprime le dossier d'une installation de boutique après confirmation.
///
/// La vérification est volontairement stricte : un dossier racine, le dossier
/// personnel ou un chemin hors de la racine choisie ne sont jamais supprimés.
/// Les fichiers de sauvegarde et le préfixe Proton vivent ailleurs et restent
/// intacts.
pub fn remove_install_directory(
    path: &std::path::Path,
    install_root: &std::path::Path,
) -> Result<(), String> {
    if !path.is_absolute() || !install_root.is_absolute() {
        return Err("chemin d'installation ou racine non absolu".to_string());
    }
    let metadata = std::fs::symlink_metadata(path)
        .map_err(|_| "dossier d'installation introuvable".to_string())?;
    if metadata.file_type().is_symlink() {
        return Err("suppression refusée : le dossier est un lien symbolique".to_string());
    }
    let canonical = path
        .canonicalize()
        .map_err(|_| "dossier d'installation introuvable".to_string())?;
    let root = install_root
        .canonicalize()
        .map_err(|_| "racine d'installation introuvable".to_string())?;
    if !canonical.is_dir() || !root.is_dir() {
        return Err("chemin d'installation invalide".to_string());
    }
    // Le dossier doit être strictement sous la racine choisie à l'installation.
    // Un chemin fournisseur corrompu ne peut donc plus viser Documents ou $HOME.
    if canonical == root || !canonical.starts_with(&root) {
        return Err("suppression refusée : dossier hors de la racine d'installation".to_string());
    }
    if canonical.components().count() < 3 {
        return Err("suppression refusée : dossier trop haut dans l'arborescence".to_string());
    }
    std::fs::remove_dir_all(&canonical)
        .map_err(|_| "suppression du dossier d'installation impossible".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::model::{InstallState, Provider, ProviderGame, ProviderGameKey};

    fn record(provider: Provider, id: &str, title: &str) -> ProviderGame {
        ProviderGame {
            key: ProviderGameKey {
                provider,
                product_id: id.into(),
            },
            title: title.into(),
            owned: true,
            install_path: None,
            install_root: None,
            custom_save_paths: Vec::new(),
            cover_url: None,
            install_exe: None,
            install_state: InstallState::NotInstalled,
            quark_game_id: None,
            capabilities: Default::default(),
            last_synced: None,
        }
    }

    #[test]
    fn exact_normalized_titles_match_but_near_matches_remain_separate() {
        let games = vec![
            record(Provider::Steam, "1", "The Witcher 3: Wild Hunt"),
            record(Provider::Epic, "witcher", "the witcher 3 wild hunt"),
            record(Provider::Gog, "witcher-gog", "The Witcher 3"),
            record(Provider::Epic, "portal", "Portal 2"),
        ];
        let groups = match_candidates(&games);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].members.len(), 2);
        assert_eq!(groups[0].members[0].product_id, "1");
        assert_eq!(groups[0].members[1].product_id, "witcher");
    }

    #[test]
    fn manual_merge_rejects_duplicate_keys_and_split_removes_only_selected_member() {
        let first = ProviderGameKey {
            provider: Provider::Steam,
            product_id: "1".into(),
        };
        let second = ProviderGameKey {
            provider: Provider::Epic,
            product_id: "2".into(),
        };
        assert!(merge_group(&[first.clone(), first.clone()]).is_err());
        let mut group = merge_group(&[first.clone(), second.clone()]).unwrap();
        split_member(&mut group, &first);
        assert_eq!(group.members, vec![second]);
        split_member(
            &mut group,
            &ProviderGameKey {
                provider: Provider::Gog,
                product_id: "missing".into(),
            },
        );
        assert_eq!(group.members.len(), 1);
    }

    #[test]
    fn success_requires_running_operation_and_an_existing_install_directory() {
        let key = ProviderGameKey {
            provider: Provider::Epic,
            product_id: "app".into(),
        };
        let mut operation = InstallOperation::new(key, std::env::temp_dir());
        let directory =
            std::env::temp_dir().join(format!("quark-install-ok-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        assert!(operation.complete_verified(directory.clone()).is_err());
        operation.start();
        operation.complete_verified(directory.clone()).unwrap();
        assert_eq!(
            operation.state,
            OperationState::Succeeded {
                install_path: directory.clone().canonicalize().unwrap()
            }
        );
        let missing = directory.join("not-installed");
        assert!(operation.complete_verified(missing).is_err());
        let _ = std::fs::remove_dir_all(directory);
    }

    #[test]
    fn une_installation_interrompue_est_reclassee_au_demarrage() {
        let root =
            std::env::temp_dir().join(format!("quark-install-recover-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let mut installed = record(Provider::Epic, "installed", "Déjà là");
        installed.install_state = InstallState::Installing;
        installed.install_path = Some(root.clone());
        let mut missing = record(Provider::Gog, "missing", "Absent");
        missing.install_state = InstallState::Queued;
        missing.install_path = Some(root.join("disparu"));
        let untouched = record(Provider::Steam, "kept", "Intact");
        let mut games = vec![installed, missing, untouched];

        assert_eq!(recover_interrupted_installs(&mut games), 2);
        // Le dossier existe : le jeu est réellement installé.
        assert_eq!(games[0].install_state, InstallState::Installed);
        assert_eq!(games[0].install_path.as_deref(), Some(root.as_path()));
        // Le dossier a disparu : le jeu redevient installable.
        assert_eq!(games[1].install_state, InstallState::NotInstalled);
        assert!(games[1].install_path.is_none());
        assert_eq!(games[2].install_state, InstallState::NotInstalled);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn installation_is_rejected_when_verified_path_is_outside_selected_destination() {
        let root = std::env::temp_dir().join(format!("quark-install-root-{}", std::process::id()));
        let outside =
            std::env::temp_dir().join(format!("quark-install-outside-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        std::fs::create_dir_all(&outside).unwrap();
        let key = ProviderGameKey {
            provider: Provider::Epic,
            product_id: "app".into(),
        };
        let mut operation = InstallOperation::new(key, root.clone());
        operation.start();
        assert!(operation.complete_verified(outside.clone()).is_err());
        assert!(matches!(operation.state, OperationState::Running { .. }));
        let _ = std::fs::remove_dir_all(root);
        let _ = std::fs::remove_dir_all(outside);
    }

    #[test]
    fn la_suppression_d_un_dossier_installe_refuse_les_chemins_dangereux() {
        let root = std::env::temp_dir().join(format!("quark-remove-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        assert!(remove_install_directory(std::path::Path::new("relative/path"), &root).is_err());
        assert!(remove_install_directory(std::path::Path::new("/"), &root).is_err());
        assert!(remove_install_directory(&crate::config::home_dir(), &root).is_err());
        let game = root.join("Jeu");
        std::fs::create_dir_all(&game).unwrap();
        std::fs::write(game.join("data.bin"), b"x").unwrap();
        assert!(remove_install_directory(&game, &root).is_ok());
        assert!(!game.exists());
        let _ = std::fs::remove_dir_all(&root);
        assert!(remove_install_directory(&game, &root).is_err());
    }

    #[test]
    fn une_reprise_gog_exige_un_manifeste_complet() {
        let root = std::env::temp_dir().join(format!(
            "quark-install-recover-gog-{}-{}",
            std::process::id(),
            crate::config::now_ts()
        ));
        std::fs::create_dir_all(&root).unwrap();
        let mut incomplete = record(Provider::Gog, "123", "Jeu GOG");
        incomplete.install_state = InstallState::Installing;
        incomplete.install_path = Some(root.clone());
        let mut games = vec![incomplete];
        assert_eq!(recover_interrupted_installs(&mut games), 1);
        assert_eq!(games[0].install_state, InstallState::NotInstalled);
        assert!(games[0].install_path.is_none());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[cfg(unix)]
    #[test]
    fn la_suppression_refuse_un_lien_symbolique_et_une_racine_differente() {
        use std::os::unix::fs::symlink;
        let root = std::env::temp_dir().join(format!(
            "quark-remove-links-{}-{}",
            std::process::id(),
            crate::config::now_ts()
        ));
        let outside = root.join("ailleurs");
        std::fs::create_dir_all(&outside).unwrap();
        let link = root.join("Jeu");
        symlink(&outside, &link).unwrap();
        assert!(remove_install_directory(&link, &root).is_err());
        assert!(outside.exists());
        let other = std::env::temp_dir().join(format!(
            "quark-remove-other-{}-{}",
            std::process::id(),
            crate::config::now_ts()
        ));
        std::fs::create_dir_all(&other).unwrap();
        let game = other.join("Jeu");
        std::fs::create_dir_all(&game).unwrap();
        assert!(remove_install_directory(&game, &root).is_err());
        assert!(game.exists());
        let _ = std::fs::remove_dir_all(&root);
        let _ = std::fs::remove_dir_all(&other);
    }
}
