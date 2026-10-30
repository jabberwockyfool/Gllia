use std::path::PathBuf;

const EMBEDDED: [(&str, &[u8]); 4] = [
    ("icon.png", include_bytes!("../assets/icon.png")),
    ("logo-clair.png", include_bytes!("../assets/logo-clair.png")),
    ("logo-sombre.png", include_bytes!("../assets/logo-sombre.png")),
    ("fonts/florencesans.ttf", include_bytes!("../assets/fonts/florencesans.ttf")),
];

pub fn embedded(name: &str) -> Option<&'static [u8]> {
    EMBEDDED.iter().find(|(file, _)| *file == name).map(|(_, bytes)| *bytes)
}

pub fn image(name: &str) -> Option<egui::Image<'static>> {
    embedded(name).map(|bytes| egui::Image::from_bytes(format!("bytes://{name}"), bytes))
}

/// Cherche un fichier dans `assets/`, d'abord depuis le dossier courant
/// (pratique avec `cargo run`), puis à côté de l'exécutable.
pub fn find(name: &str) -> Option<PathBuf> {
    let next_to_exe = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|dir| dir.join("assets")));

    [Some(PathBuf::from("assets")), next_to_exe]
        .into_iter()
        .flatten()
        .map(|dir| dir.join(name))
        .find(|path| path.is_file())
}

/// URI d'une image venue du catalogue : une URL, ou un chemin local pour les tests.
pub fn source_uri(source: &str) -> Option<String> {
    if source.is_empty() {
        None
    } else if source.starts_with("http://") || source.starts_with("https://") {
        Some(source.to_owned())
    } else {
        Some(format!("file://{source}"))
    }
}
