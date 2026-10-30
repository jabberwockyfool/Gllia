use std::fs;
use std::path::PathBuf;

use winreg::enums::HKEY_CURRENT_USER;
use winreg::RegKey;

pub fn find(app_id: u32) -> Option<PathBuf> {
    libraries().into_iter().find_map(|library| {
        let steamapps = library.join("steamapps");
        let manifest = fs::read_to_string(steamapps.join(format!("appmanifest_{app_id}.acf"))).ok()?;
        let folder = vdf_values(&manifest, "installdir").into_iter().next()?;

        let path = steamapps.join("common").join(folder);
        path.is_dir().then_some(path)
    })
}

fn libraries() -> Vec<PathBuf> {
    let Some(steam) = steam_folder() else {
        return Vec::new();
    };

    let mut libraries = vec![steam.clone()];
    if let Ok(text) = fs::read_to_string(steam.join("steamapps").join("libraryfolders.vdf")) {
        libraries.extend(vdf_values(&text, "path").into_iter().map(PathBuf::from));
    }
    libraries
}

fn steam_folder() -> Option<PathBuf> {
    let key = RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey(r"Software\Valve\Steam")
        .ok()?;
    let path: String = key.get_value("SteamPath").ok()?;
    Some(PathBuf::from(path))
}

fn vdf_values(text: &str, key: &str) -> Vec<String> {
    text.lines()
        .filter_map(|line| {
            let fields: Vec<&str> = line.split('"').map(str::trim).filter(|f| !f.is_empty()).collect();
            match fields.as_slice() {
                [name, value] if *name == key => Some(value.replace("\\\\", "\\")),
                _ => None,
            }
        })
        .collect()
}
