use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;

use serde::Deserialize;

use crate::xdelta;

#[derive(Clone)]
pub enum Step {
    Downloading(u64, Option<u64>),
    Installing,
    /// Patch d'ISO en cours : octets de l'ISO française déjà écrits.
    Patching(u64),
    Done,
    /// ISO française créée, à cet endroit.
    Created(PathBuf),
    Browser,
    Failed(String),
}

pub struct Install {
    pub slug: String,
    step: Arc<Mutex<Step>>,
}

impl Install {
    /// `target` est le dossier du jeu, ou l'ISO d'origine pour un patch d'ISO.
    pub fn start(slug: &str, download_url: String, target: PathBuf, game_title: String, ctx: &egui::Context) -> Self {
        let step = Arc::new(Mutex::new(Step::Downloading(0, None)));
        let shared = step.clone();
        let ctx = ctx.clone();

        thread::spawn(move || {
            let set = |new: Step| {
                *shared.lock().unwrap() = new;
                ctx.request_repaint();
            };
            let result = if xdelta::is_patch_url(&download_url) {
                patch_iso(&download_url, &target, &game_title, &set)
            } else {
                let Some(installer_url) = find_installer(&download_url) else {
                    ctx.open_url(egui::OpenUrl::new_tab(&download_url));
                    set(Step::Browser);
                    return;
                };
                install(&installer_url, &target, &set)
            };
            set(result.unwrap_or_else(Step::Failed));
        });

        Self { slug: slug.to_owned(), step }
    }

    pub fn step(&self) -> Step {
        self.step.lock().unwrap().clone()
    }

    pub fn is_running(&self) -> bool {
        matches!(self.step(), Step::Downloading(..) | Step::Installing | Step::Patching(_))
    }
}

fn install(installer_url: &str, game_folder: &Path, set: &dyn Fn(Step)) -> Result<Step, String> {
    let exe = download(installer_url, |done, total| set(Step::Downloading(done, total)))?;
    set(Step::Installing);
    run(&exe, game_folder)?;
    Ok(Step::Done)
}

fn patch_iso(patch_url: &str, iso: &Path, game_title: &str, set: &dyn Fn(Step)) -> Result<Step, String> {
    let output = xdelta::output_path(iso, game_title);
    if output == iso {
        return Err("cette ISO est déjà patchée : choisissez l'ISO d'origine".to_owned());
    }
    let patch = download(patch_url, |done, total| set(Step::Downloading(done, total)))?;
    set(Step::Patching(0));
    let result = xdelta::apply(iso, &patch, &output, |written| set(Step::Patching(written)));
    let _ = fs::remove_file(&patch);
    result.map(|()| Step::Created(output))
}

fn find_installer(download_url: &str) -> Option<String> {
    let path = download_url.split(['?', '#']).next()?;
    if path.to_lowercase().ends_with(".exe") {
        return Some(download_url.to_owned());
    }

    let repo = path.strip_prefix("https://github.com/")?;
    let mut parts = repo.split('/');
    let (owner, name) = (parts.next()?, parts.next()?);
    if parts.next() != Some("releases") {
        return None;
    }
    latest_github_installer(owner, name)
}

#[derive(Deserialize)]
struct Release {
    draft: bool,
    assets: Vec<Asset>,
}

#[derive(Deserialize)]
struct Asset {
    name: String,
    browser_download_url: String,
}

fn latest_github_installer(owner: &str, name: &str) -> Option<String> {
    let releases: Vec<Release> = ureq::get(&format!("https://api.github.com/repos/{owner}/{name}/releases"))
        .set("User-Agent", "Gllia")
        .call()
        .ok()?
        .into_json()
        .ok()?;
    releases
        .into_iter()
        .filter(|release| !release.draft)
        .flat_map(|release| release.assets)
        .find(|asset| asset.name.to_lowercase().ends_with(".exe"))
        .map(|asset| asset.browser_download_url)
}

pub fn download(url: &str, mut progress: impl FnMut(u64, Option<u64>)) -> Result<PathBuf, String> {
    let response = ureq::get(url).call().map_err(|e| format!("téléchargement impossible : {e}"))?;
    let total = response.header("Content-Length").and_then(|size| size.parse().ok());

    let name = url.split(['?', '#']).next().and_then(|path| path.rsplit('/').next()).unwrap_or("installeur.exe");
    let folder = std::env::temp_dir().join("Gllia");
    fs::create_dir_all(&folder).map_err(|e| e.to_string())?;
    let path = folder.join(name);

    let mut file = File::create(&path).map_err(|e| format!("{} : {e}", path.display()))?;
    let mut reader = response.into_reader();
    let mut buffer = vec![0; 1 << 16];
    let mut done = 0;
    loop {
        let read = reader.read(&mut buffer).map_err(|e| format!("téléchargement interrompu : {e}"))?;
        if read == 0 {
            break;
        }
        file.write_all(&buffer[..read]).map_err(|e| e.to_string())?;
        done += read as u64;
        progress(done, total);
    }
    if total.is_some_and(|total| done < total) {
        return Err("téléchargement incomplet".to_owned());
    }
    Ok(path)
}

fn run(exe: &Path, game_folder: &Path) -> Result<(), String> {
    let mut command = Command::new(exe);
    command.args(["/VERYSILENT", "/SUPPRESSMSGBOXES", "/NORESTART"]);
    command.arg(format!("/DIR={}", game_folder.display()));
    command.arg(if is_writable(game_folder) { "/CURRENTUSER" } else { "/ALLUSERS" });

    let status = command.status().map_err(|e| format!("impossible de lancer l'installeur : {e}"))?;
    let _ = fs::remove_file(exe);
    match status.code() {
        Some(0) => Ok(()),
        Some(2 | 5) => Err("installation annulée".to_owned()),
        Some(code) => Err(format!("l'installeur s'est arrêté (code {code})")),
        None => Err("l'installeur s'est arrêté".to_owned()),
    }
}

fn is_writable(folder: &Path) -> bool {
    let probe = folder.join(".gllia-test");
    let writable = File::create(&probe).is_ok();
    let _ = fs::remove_file(probe);
    writable
}
