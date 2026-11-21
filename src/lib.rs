//! Cœur non graphique de Quark, réutilisé par l'interface Tauri.
//!
//! L'interface egui historique déclare ses propres modules dans `main.rs` et
//! continue de fonctionner telle quelle. Ce fichier n'expose que la logique
//! métier : configuration, boutiques, Proton, sessions, sauvegardes, thèmes.

pub mod archive;
pub mod artwork;
pub mod config;
pub mod disk;
pub mod i18n;
pub mod net;
pub mod process;
pub mod proton;
pub mod providers;
pub mod saves;
pub mod scan;
pub mod session;
pub mod steam_store;
pub mod steamlib;
