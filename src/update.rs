//! Mise à jour automatique. Au démarrage, Gllia lit la dernière release GitHub ;
//! si elle est plus récente, son exe est téléchargé en arrière-plan, puis
//! remplace celui-ci quand l'utilisateur clique sur « Redémarrer ».
//!
//! Windows refuse d'écraser un exe en cours d'exécution mais accepte de le
//! renommer : l'ancien devient `gllia.exe.old`, supprimé au lancement suivant.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;

use serde::Deserialize;

const LATEST_RELEASE: &str = "https://api.github.com/repos/jabberwockyfool/Gllia/releases/latest";

#[derive(Clone)]
pub enum Step {
    /// Version de développement (`cargo run`) : pas de mise à jour.
    Disabled,
    Checking,
    UpToDate,
    Downloading { version: String, done: u64, total: Option<u64> },
    Ready { version: String, exe: PathBuf },
    Failed(String),
}

pub struct Update {
    step: Arc<Mutex<Step>>,
}

#[derive(Deserialize)]
struct Release {
    tag_name: String,
    assets: Vec<Asset>,
}

#[derive(Deserialize)]
struct Asset {
    name: String,
    browser_download_url: String,
}

impl Update {
    pub fn check(ctx: &egui::Context) -> Self {
        if cfg!(debug_assertions) {
            return Self { step: Arc::new(Mutex::new(Step::Disabled)) };
        }

        let step = Arc::new(Mutex::new(Step::Checking));
        let shared = step.clone();
        let ctx = ctx.clone();

        thread::spawn(move || {
            let set = |new: Step| {
                *shared.lock().unwrap() = new;
                ctx.request_repaint();
            };
            set(match download_newer(&set) {
                Ok(step) => step,
                Err(error) => Step::Failed(error),
            });
            // L'ancien exe d'une mise à jour précédente : son processus a eu le
            // temps de se fermer pendant la vérification.
            if let Ok(exe) = std::env::current_exe() {
                let _ = fs::remove_file(old_path(&exe));
            }
        });

        Self { step }
    }

    pub fn step(&self) -> Step {
        self.step.lock().unwrap().clone()
    }

    pub fn is_busy(&self) -> bool {
        matches!(self.step(), Step::Checking | Step::Downloading { .. })
    }

    /// Remplace l'exe par la nouvelle version et relance Gllia. Ne revient
    /// qu'en cas d'échec.
    pub fn restart(&self) {
        let Step::Ready { exe, .. } = self.step() else {
            return;
        };
        let result = std::env::current_exe()
            .map_err(|e| e.to_string())
            .and_then(|current| {
                replace(&current, &exe)?;
                Command::new(&current).spawn().map_err(|e| format!("impossible de relancer Gllia : {e}"))
            });
        match result {
            Ok(_) => std::process::exit(0),
            Err(error) => *self.step.lock().unwrap() = Step::Failed(error),
        }
    }
}

fn download_newer(set: &dyn Fn(Step)) -> Result<Step, String> {
    let release: Release = ureq::get(LATEST_RELEASE)
        .set("User-Agent", "Gllia")
        .call()
        .map_err(|e| format!("GitHub injoignable : {e}"))?
        .into_json()
        .map_err(|e| format!("réponse de GitHub illisible : {e}"))?;

    if !is_newer(&release.tag_name, env!("CARGO_PKG_VERSION")) {
        return Ok(Step::UpToDate);
    }
    let asset = release
        .assets
        .into_iter()
        .find(|asset| asset.name.to_lowercase().ends_with(".exe"))
        .ok_or("la dernière release n'a pas d'exe")?;

    let version = release.tag_name.trim_start_matches(['v', 'V']).to_owned();
    let exe = crate::install::download(&asset.browser_download_url, |done, total| {
        set(Step::Downloading { version: version.clone(), done, total })
    })?;
    Ok(Step::Ready { version, exe })
}

fn replace(current: &Path, new: &Path) -> Result<(), String> {
    let old = old_path(current);
    let _ = fs::remove_file(&old);
    fs::rename(current, &old).map_err(|e| format!("impossible de remplacer Gllia : {e}"))?;
    if let Err(error) = fs::copy(new, current) {
        let _ = fs::rename(&old, current);
        return Err(format!("impossible de remplacer Gllia : {error}"));
    }
    let _ = fs::remove_file(new);
    Ok(())
}

fn old_path(exe: &Path) -> PathBuf {
    exe.with_extension("exe.old")
}

/// Compare deux versions « v1.2.3 », « V1.0 », « 1.0.0 »… chiffre par chiffre.
fn is_newer(tag: &str, current: &str) -> bool {
    numbers(tag) > numbers(current)
}

fn numbers(version: &str) -> Vec<u64> {
    let mut numbers: Vec<u64> = version
        .trim()
        .trim_start_matches(['v', 'V'])
        .split('.')
        .map(|part| part.chars().take_while(char::is_ascii_digit).collect::<String>().parse().unwrap_or(0))
        .collect();
    // « 1.0 » et « 1.0.0 » sont la même version.
    while numbers.last() == Some(&0) {
        numbers.pop();
    }
    numbers
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compares_versions() {
        assert!(!is_newer("V1.0", "1.0.0"));
        assert!(!is_newer("v0.9.5", "1.0.0"));
        assert!(is_newer("v1.0.1", "1.0.0"));
        assert!(is_newer("V1.1", "1.0.9"));
        assert!(is_newer("2.0", "1.10.0"));
        assert!(!is_newer("1.10.0", "1.10"));
    }

    #[test]
    fn replaces_exe_and_keeps_old_one() {
        let dir = std::env::temp_dir().join("gllia-update-test");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let current = dir.join("gllia.exe");
        let new = dir.join("nouveau.exe");
        fs::write(&current, "ancien").unwrap();
        fs::write(&new, "nouveau").unwrap();

        replace(&current, &new).unwrap();

        assert_eq!(fs::read_to_string(&current).unwrap(), "nouveau");
        assert_eq!(fs::read_to_string(dir.join("gllia.exe.old")).unwrap(), "ancien");
        assert!(!new.exists());
        fs::remove_dir_all(&dir).unwrap();
    }
}
