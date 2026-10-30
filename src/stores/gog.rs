//! GOG Galaxy et les installateurs GOG inscrivent chaque jeu dans le registre,
//! sous son identifiant GOG, avec son dossier dans la valeur `path`.

use std::path::PathBuf;

use winreg::enums::HKEY_LOCAL_MACHINE;
use winreg::RegKey;

pub fn find(game_id: u64) -> Option<PathBuf> {
    let key = RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey(format!(r"SOFTWARE\WOW6432Node\GOG.com\Games\{game_id}"))
        .ok()?;
    let path: String = key.get_value("path").ok()?;

    let path = PathBuf::from(path);
    path.is_dir().then_some(path)
}
