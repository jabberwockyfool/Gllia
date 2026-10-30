use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::layouts::Layout;
use crate::palette::Appearance;

const SITE: &str = "https://sainteresia.vercel.app";

/// Réglages gardés d'un lancement à l'autre, dans
/// `%APPDATA%\Gllia\config.json`.
#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// `None` tant que le thème n'a jamais été choisi (premier lancement).
    pub layout: Option<Layout>,
    pub appearance: Appearance,
    pub music_volume: f32,
    pub music_muted: bool,
    pub about_transparent: bool,
    /// Une URL ou un chemin vers un fichier local (utile pour tester).
    pub patches_url: String,
    pub members_url: String,
    /// Jeux ajoutés : slug du patch → dossier du jeu.
    pub games: BTreeMap<String, PathBuf>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            layout: None,
            appearance: Appearance::default(),
            music_volume: 0.5,
            music_muted: false,
            about_transparent: true,
            patches_url: format!("{SITE}/api/launcher/patchs"),
            members_url: format!("{SITE}/api/launcher/membres"),
            games: BTreeMap::new(),
        }
    }
}

impl Config {
    pub fn load() -> Self {
        fs::read_to_string(path())
            .ok()
            .and_then(|text| serde_json::from_str(&text).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) {
        let path = path();
        if let Some(dir) = path.parent() {
            let _ = fs::create_dir_all(dir);
        }
        if let Ok(text) = serde_json::to_string_pretty(self) {
            let _ = fs::write(path, text);
        }
    }
}

fn path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_default()
        .join("Gllia")
        .join("config.json")
}
