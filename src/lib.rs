//! Cœur métier de Quark, consommé par l'interface Tauri.
//!
//! Ce fichier n'expose que la logique non graphique : configuration, boutiques,
//! Proton, sessions et sauvegardes.

pub mod achievements_bridge;
pub mod archive;
pub mod artwork;
pub mod config;
pub mod disk;
pub mod game_search;
pub mod howlongtobeat;
pub mod i18n;
pub mod media;
pub mod net;
pub mod news;
pub mod process;
pub mod proton;
pub mod protondb;
pub mod providers;
pub mod saves;
pub mod scan;
pub mod session;
pub mod stats;
pub mod steam_store;
pub mod steamlib;
pub mod update;
