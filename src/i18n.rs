//! Traduction de l'interface, sans dépendance externe.
//!
//! Les textes français restent la source dans le code : une chaîne inconnue
//! s'affiche telle quelle, ce qui évite qu'un nouveau libellé disparaisse avant
//! sa traduction. Le catalogue anglais est embarqué dans le binaire, et la
//! langue choisie est appliquée immédiatement à toute l'interface.

use crate::config::UiLanguage;
use eframe::egui::RichText;
use std::collections::HashMap;
use std::sync::LazyLock;
use std::sync::atomic::{AtomicU8, Ordering};

const EN: &str = include_str!("../assets/i18n/en.json");
const ES: &str = include_str!("../assets/i18n/es.json");
const DE: &str = include_str!("../assets/i18n/de.json");
const PT: &str = include_str!("../assets/i18n/pt.json");

static LANGUAGE: AtomicU8 = AtomicU8::new(0);

fn catalog(text: &str) -> HashMap<String, String> {
    let pairs: HashMap<String, String> = serde_json::from_str(text).unwrap_or_default();
    pairs
        .into_iter()
        .map(|(key, value)| (normalize_key(&key), value))
        .collect()
}

static CATALOG_EN: LazyLock<HashMap<String, String>> = LazyLock::new(|| catalog(EN));
static CATALOG_ES: LazyLock<HashMap<String, String>> = LazyLock::new(|| catalog(ES));
static CATALOG_DE: LazyLock<HashMap<String, String>> = LazyLock::new(|| catalog(DE));
static CATALOG_PT: LazyLock<HashMap<String, String>> = LazyLock::new(|| catalog(PT));

pub fn set_language(language: UiLanguage) {
    LANGUAGE.store(
        match language {
            UiLanguage::French => 0,
            UiLanguage::English => 1,
            UiLanguage::Spanish => 2,
            UiLanguage::German => 3,
            UiLanguage::Portuguese => 4,
        },
        Ordering::Relaxed,
    );
}

pub fn language() -> UiLanguage {
    match LANGUAGE.load(Ordering::Relaxed) {
        1 => UiLanguage::English,
        2 => UiLanguage::Spanish,
        3 => UiLanguage::German,
        4 => UiLanguage::Portuguese,
        _ => UiLanguage::French,
    }
}

pub fn is_english() -> bool {
    language() == UiLanguage::English
}

fn current_catalog() -> Option<&'static HashMap<String, String>> {
    match language() {
        UiLanguage::French => None,
        UiLanguage::English => Some(&CATALOG_EN),
        UiLanguage::Spanish => Some(&CATALOG_ES),
        UiLanguage::German => Some(&CATALOG_DE),
        UiLanguage::Portuguese => Some(&CATALOG_PT),
    }
}

/// Normalise les espaces (les chaînes Rust multilignes sont repliées par le
/// compilateur) pour que la même phrase trouve toujours sa traduction.
fn normalize_key(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Traduit un texte déjà construit. En français, la chaîne est renvoyée
/// inchangée ; en anglais, le catalogue est consulté puis quelques formes
/// dynamiques courantes sont adaptées.
pub fn tr(text: &str) -> String {
    if text.is_empty() {
        return text.to_string();
    }
    let Some(catalog) = current_catalog() else {
        return text.to_string();
    };
    let key = normalize_key(text);
    if let Some(value) = catalog.get(&key) {
        return value.clone();
    }
    // Une clé absente d'une traduction secondaire retombe sur l'anglais, puis
    // sur le français : aucun libellé ne disparaît.
    if let Some(value) = CATALOG_EN.get(&key) {
        return value.clone();
    }
    if language() != UiLanguage::French {
        return dynamic_english(&key);
    }
    text.to_string()
}

/// Raccourci pour les textes construits dans l'interface.
pub fn rich(text: impl Into<String>) -> RichText {
    RichText::new(tr(&text.into()))
}

/// Quelques formes produites par `format!` ne peuvent pas figurer telles
/// quelles dans le catalogue. Ces règles restent volontairement conservatrices :
/// elles ne touchent qu'aux mots français isolés, jamais aux noms de jeux.
fn dynamic_english(text: &str) -> String {
    let mut out = text.to_string();
    let rules: [(&str, &str); 63] = [
        ("Aucun jeu sélectionné", "No game selected"),
        ("Aucun jeu ne correspond", "No game matches"),
        ("Aucun résultat", "No results"),
        ("Aucune vue enregistrée", "No saved views"),
        ("Aucune session enregistrée", "No session recorded"),
        ("Aucune sauvegarde", "No backup"),
        ("Aucun tag", "No tags"),
        ("Aucun téléchargement en cours", "No download in progress"),
        ("Aucun runtime trouvé", "No runtime found"),
        ("Aucun artwork", "No artwork"),
        ("Aucun exécutable", "No executable"),
        ("Aucune correspondance", "No match"),
        ("Aucune proposition", "No suggestion"),
        ("Aucune image", "No image"),
        ("Aucune variable", "No variable"),
        ("Aucune archive", "No archive"),
        ("Aucune mise à jour", "No update"),
        ("Aucune erreur", "No error"),
        ("Aucune installation", "No installation"),
        ("Aucune session", "No session"),
        ("Aucune activité", "No activity"),
        ("Aucune nouveauté", "No news"),
        ("Aucune actualité", "No news"),
        ("Dernière session", "Last session"),
        ("Dernière mise à jour", "Last update"),
        ("Temps de jeu", "Playtime"),
        ("temps de jeu", "playtime"),
        ("de jeu", "played"),
        ("en jeu", "in game"),
        ("En jeu", "In game"),
        (" heures", " hours"),
        (" heure", " hour"),
        (" minutes", " minutes"),
        (" seconde", " second"),
        (" secondes", " seconds"),
        ("Tout", "All"),
        ("Tri : ", "Sort: "),
        ("Grouper : ", "Group: "),
        ("correspondance(s)", "match(es)"),
        ("résultat(s)", "result(s)"),
        ("joueurs simultanés", "concurrent players"),
        ("jeu(x)", "game(s)"),
        ("jeux retirés", "games removed"),
        ("succès débloqués", "achievements unlocked"),
        ("sélectionné", "selected"),
        ("Mémoire résidente", "Resident memory"),
        ("Taille à télécharger", "Download size"),
        ("Taille non communiquée", "Size not provided"),
        ("Taille : calcul en cours", "Size: calculating"),
        ("calcul en cours", "calculating"),
        ("Dossier analysé : ", "Scanned folder: "),
        ("Image disque : ", "Disc image: "),
        ("Détecté : ", "Detected: "),
        (
            "Lecture du dossier impossible : ",
            "Could not read folder: ",
        ),
        ("Temps ", "Time "),
        ("Vue «", "View “"),
        (" » appliquée", "” applied"),
        (" » enregistrée", "” saved"),
        (" » supprimée", "” deleted"),
        ("Aucune ", "No "),
        ("Aucun ", "No "),
        ("Aucune", "No"),
        ("Aucun", "No"),
    ];
    for (from, to) in rules {
        if out.contains(from) {
            out = out.replace(from, to);
        }
    }
    // Formes dynamiques très fréquentes : « 7 jeux », « 7 jeu(x) »,
    // « 1 jeu ». On ne remplace jamais « jeu » à l'intérieur d'un titre.
    let words: Vec<&str> = out.split(' ').collect();
    if let Some(first) = words.first().and_then(|word| word.parse::<u64>().ok()) {
        if out.ends_with(" jeux") || out.ends_with(" jeu(x)") {
            out.truncate(out.len() - if out.ends_with(" jeux") { 5 } else { 7 });
            out.push_str(if first == 1 { " game" } else { " games" });
        } else if out.ends_with(" jeu") {
            out.truncate(out.len() - 4);
            out.push_str(if first == 1 { " game" } else { " games" });
        }
    }
    if out.starts_with("1 ")
        && (out.contains(" hours") || out.contains(" minutes") || out.contains(" seconds"))
    {
        out = out.replacen(" hours", " hour", 1);
        out = out.replacen(" minutes", " minute", 1);
        out = out.replacen(" seconds", " second", 1);
    }
    // Formes « N game(s) », « N match(es) », « N result(s) » : le catalogue
    // ne peut pas connaître le nombre, on choisit donc le singulier/pluriel.
    if let Some(count) = out
        .split(' ')
        .next()
        .and_then(|word| word.parse::<u64>().ok())
    {
        for (suffix, singular, plural) in [
            (" game(s)", " game", " games"),
            (" match(es)", " match", " matches"),
            (" result(s)", " result", " results"),
            (" texture(s)", " texture", " textures"),
        ] {
            if let Some(prefix) = out.strip_suffix(suffix) {
                out = format!("{prefix}{}", if count == 1 { singular } else { plural });
                break;
            }
        }
    }
    out
}

/// Résumé concret de la version courante, affiché une fois après une mise à
/// jour. Il est volontairement court : quelques phrases, pas un journal de commit.
pub fn release_summary() -> [&'static str; 6] {
    if !matches!(language(), UiLanguage::French) {
        [
            "The FPS counter is now a Windows DLL dropped next to the game: it hooks Direct3D 11 and draws a Steam-like overlay that hides when the game loses focus.",
            "The in-game settings install the d3d11.dll proxy and enable the counter for Windows games launched with Proton or Wine, without overwriting a foreign DLL.",
            "Epic sign-in is direct: Quark exchanges the authorization code itself and writes the Legendary session, without running legendary auth.",
            "Spanish, German and Portuguese join French and English, with complete interface catalogues and a --lang option.",
            "cargo fmt --check and clippy -D warnings now guard every change, and the overlay DLL is rebuilt in the release workflow.",
            "The existing Quark panel, screenshots, achievements and cloud saves are unchanged; the DLL only adds the FPS counter.",
        ]
    } else {
        [
            "Le compteur FPS est désormais une DLL Windows posée à côté du jeu : elle accroche Direct3D 11 et affiche un overlay façon Steam qui se cache quand le jeu perd le focus.",
            "Les réglages « En jeu » installent le proxy d3d11.dll et activent le compteur pour les jeux Windows lancés avec Proton ou Wine, sans écraser une DLL étrangère.",
            "La connexion Epic est directe : Quark échange lui-même le code d'autorisation et écrit la session Legendary, sans lancer legendary auth.",
            "L'espagnol, l'allemand et le portugais rejoignent le français et l'anglais, avec des catalogues complets et l'option --lang.",
            "cargo fmt --check et clippy -D warnings protègent désormais chaque changement, et la DLL de l'overlay est reconstruite dans le workflow de release.",
            "Le panneau Quark existant, les captures, les succès et les sauvegardes cloud ne changent pas ; la DLL ajoute seulement le compteur FPS.",
        ]
    }
}

pub fn release_title() -> String {
    if is_english() {
        format!("What's new in Quark {}", env!("CARGO_PKG_VERSION"))
    } else {
        format!("Nouveautés de Quark {}", env!("CARGO_PKG_VERSION"))
    }
}

/// Verrou partagé par tous les tests qui lisent ou changent la langue, quel que
/// soit leur module. Sans lui, un test de `game_search` pouvait lire « MB » au
/// lieu de « Mo » pendant qu'un test d'`i18n` passait brièvement en anglais.
#[cfg(test)]
pub(crate) fn language_test_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    LOCK.lock().unwrap_or_else(|poison| poison.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::MutexGuard;

    // La langue est globale au processus : les tests qui la changent doivent
    // s'exécuter l'un après l'autre, sinon l'un lit la langue posée par l'autre.
    fn language_lock() -> MutexGuard<'static, ()> {
        language_test_lock()
    }

    #[test]
    fn les_catalogues_secondaires_sont_complets() {
        let _guard = language_lock();
        let english: HashMap<String, String> = serde_json::from_str(EN).expect("catalogue anglais");
        for (nom, texte) in [("espagnol", ES), ("allemand", DE), ("portugais", PT)] {
            let catalogue: HashMap<String, String> = serde_json::from_str(texte)
                .unwrap_or_else(|error| panic!("catalogue {nom} : {error}"));
            assert_eq!(
                catalogue.keys().collect::<std::collections::BTreeSet<_>>(),
                english.keys().collect::<std::collections::BTreeSet<_>>(),
                "le catalogue {nom} n'a pas les mêmes clés que l'anglais"
            );
        }
    }

    #[test]
    fn les_nouvelles_langues_sont_traduites() {
        let _guard = language_lock();
        set_language(UiLanguage::Spanish);
        assert_eq!(tr("Ajouter un jeu"), "Añadir un juego");
        set_language(UiLanguage::German);
        assert_eq!(tr("Ajouter un jeu"), "Spiel hinzufügen");
        set_language(UiLanguage::Portuguese);
        assert_eq!(tr("Ajouter un jeu"), "Adicionar um jogo");
        set_language(UiLanguage::French);
    }

    #[test]
    fn le_francais_reste_la_source() {
        let _guard = language_lock();
        set_language(UiLanguage::French);
        assert_eq!(tr("Ajouter un jeu"), "Ajouter un jeu");
    }

    #[test]
    fn le_catalogue_traduit_les_libelles_connus() {
        let _guard = language_lock();
        set_language(UiLanguage::English);
        assert_eq!(tr("Ajouter un jeu"), "Add a game");
        assert_eq!(tr("Paramètres"), "Settings");
        set_language(UiLanguage::French);
    }

    #[test]
    fn les_espaces_multilignes_sont_normalises() {
        let _guard = language_lock();
        assert_eq!(normalize_key("  un   texte\n   replié "), "un texte replié");
    }

    #[test]
    fn le_resume_de_version_suit_la_langue() {
        let _guard = language_lock();
        set_language(UiLanguage::French);
        assert!(release_summary()[0].starts_with("Le compteur FPS"));
        assert!(release_summary()[1].starts_with("Les réglages « En jeu »"));
        set_language(UiLanguage::English);
        assert!(release_summary()[0].starts_with("The FPS counter"));
        assert!(release_summary()[1].starts_with("The in-game settings"));
        assert!(release_title().starts_with("What's new"));
        set_language(UiLanguage::French);
    }

    #[test]
    fn les_durees_suivent_la_langue() {
        let _guard = language_lock();
        set_language(UiLanguage::English);
        assert_eq!(crate::config::human_playtime(483_120), "134.2 h");
        set_language(UiLanguage::French);
        assert_eq!(crate::config::human_playtime(483_120), "134,2 h");
    }

    #[test]
    fn une_chaine_inconnue_ne_disparait_pas() {
        let _guard = language_lock();
        set_language(UiLanguage::English);
        assert_eq!(tr("Hollow Knight"), "Hollow Knight");
        assert_eq!(tr("7 jeux"), "7 games");
        assert_eq!(tr("1 jeu"), "1 game");
        set_language(UiLanguage::French);
    }
}
