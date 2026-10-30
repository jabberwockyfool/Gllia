//! Patchs d'ISO au format xdelta (consoles : PS2…). Ils sont appliqués avec
//! xdelta3 3.1.0, intégré à l'exe et extrait dans le dossier temporaire.
//! L'ISO d'origine n'est jamais modifiée : l'ISO française est créée à côté.

use std::fs;
use std::io::Read;
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

const XDELTA3: &[u8] = include_bytes!("../assets/xdelta3.exe");
/// Pas de fenêtre de console pendant le patch.
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// Un patch d'ISO se reconnaît à son lien de téléchargement.
pub fn is_patch_url(url: &str) -> bool {
    let path = url.split(['?', '#']).next().unwrap_or_default().to_lowercase();
    path.ends_with(".xdelta") || path.ends_with(".vcdiff")
}

/// « <dossier de l'ISO>/<nom du jeu> (Patch FR).iso »
pub fn output_path(iso: &Path, game_title: &str) -> PathBuf {
    let name: String = game_title
        .chars()
        .map(|c| if r#"<>:"/\|?*"#.contains(c) { '-' } else { c })
        .collect();
    iso.with_file_name(format!("{} (Patch FR).iso", name.trim()))
}

/// Crée `output` à partir de l'ISO et du patch. `progress` reçoit le nombre
/// d'octets déjà écrits.
pub fn apply(iso: &Path, patch: &Path, output: &Path, progress: impl Fn(u64)) -> Result<(), String> {
    let xdelta3 = extract()?;
    // Écrit à côté sous un autre nom : une ISO incomplète ne remplace jamais la bonne.
    let partial = output.with_extension("iso.part");

    let mut child = Command::new(&xdelta3)
        .arg("-d")
        .arg("-f")
        .arg("-s")
        .arg(iso)
        .arg(patch)
        .arg(&partial)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()
        .map_err(|e| format!("impossible de lancer xdelta3 : {e}"))?;

    let status = loop {
        if let Some(status) = child.try_wait().map_err(|e| e.to_string())? {
            break status;
        }
        progress(fs::metadata(&partial).map(|m| m.len()).unwrap_or(0));
        thread::sleep(Duration::from_millis(300));
    };

    if !status.success() {
        let mut stderr = String::new();
        if let Some(mut pipe) = child.stderr.take() {
            let _ = pipe.read_to_string(&mut stderr);
        }
        let _ = fs::remove_file(&partial);
        return Err(explain(&stderr));
    }

    let _ = fs::remove_file(output);
    fs::rename(&partial, output).map_err(|e| format!("{} : {e}", output.display()))
}

fn extract() -> Result<PathBuf, String> {
    let folder = std::env::temp_dir().join("Gllia");
    fs::create_dir_all(&folder).map_err(|e| e.to_string())?;
    let path = folder.join("xdelta3.exe");
    let up_to_date = fs::metadata(&path).is_ok_and(|m| m.len() == XDELTA3.len() as u64);
    if !up_to_date {
        fs::write(&path, XDELTA3).map_err(|e| format!("impossible d'extraire xdelta3 : {e}"))?;
    }
    Ok(path)
}

/// Traduit les erreurs courantes de xdelta3.
fn explain(stderr: &str) -> String {
    let lower = stderr.to_lowercase();
    if lower.contains("checksum") || lower.contains("source file too short") || lower.contains("invalid input") {
        "cette ISO ne correspond pas au patch : vérifiez que c'est bien l'ISO d'origine indiquée sur la fiche du patch".to_owned()
    } else if lower.contains("no space") {
        "plus assez de place sur le disque".to_owned()
    } else {
        let last = stderr.lines().rev().find(|line| !line.trim().is_empty()).unwrap_or("erreur inconnue");
        format!("xdelta3 : {}", last.trim().trim_start_matches("xdelta3: "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_patch_urls() {
        assert!(is_patch_url("https://github.com/a/b/releases/latest/download/FSR_PatchFR.xdelta"));
        assert!(is_patch_url("https://exemple.fr/patch.XDELTA?v=2"));
        assert!(!is_patch_url("https://github.com/a/b/releases"));
        assert!(!is_patch_url("https://exemple.fr/installeur.exe"));
    }

    #[test]
    fn names_output_next_to_iso() {
        let iso = Path::new(r"D:\Jeux\Hana to Taiyou to Ame to (English v1.0).iso");
        assert_eq!(
            output_path(iso, "Flower, Sun and Rain"),
            Path::new(r"D:\Jeux\Flower, Sun and Rain (Patch FR).iso")
        );
        assert_eq!(output_path(iso, "Steins;Gate: 0"), Path::new(r"D:\Jeux\Steins;Gate- 0 (Patch FR).iso"));
    }

    /// Patch réel, long (plusieurs minutes) : GLLIA_TEST_ISO, GLLIA_TEST_PATCH et
    /// GLLIA_TEST_OUT donnent les chemins. `cargo test -- --ignored`
    #[test]
    #[ignore]
    fn patches_real_iso() {
        let var = |name| PathBuf::from(std::env::var(name).expect(name));
        let output = var("GLLIA_TEST_OUT");
        apply(&var("GLLIA_TEST_ISO"), &var("GLLIA_TEST_PATCH"), &output, |_| {}).unwrap();
        assert!(output.exists());
    }
}
