//! Ce que le launcher récupère du site : la liste des patchs et celle des membres.
//! Les deux sont des tableaux JSON (voir `exemple/`).

use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;

use serde::de::DeserializeOwned;
use serde::Deserialize;

#[derive(Deserialize, Default, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct Patch {
    pub slug: String,
    pub title: String,
    /// "vn" ou "jeu", comme sur le site.
    pub kind: String,
    pub status: String,
    pub progress: u8,
    pub description: String,
    /// Phrase d'accroche (le champ « citation » du site).
    pub quote: String,
    /// Images : URL, ou chemin local pour tester. Vides si absentes.
    pub logo: String,
    pub background: String,
    pub cover: String,
    pub download_url: String,
    /// Fiche du patch sur le site.
    pub page_url: String,
    /// Identifiants servant à retrouver le jeu installé.
    pub steam_app_id: Option<u32>,
    pub gog_id: Option<u64>,
}

impl Patch {
    /// Nom du jeu, sans le « — Patch FR » qui termine les titres du site.
    pub fn game_title(&self) -> &str {
        match self.title.strip_suffix("Patch FR") {
            Some(name) if !name.trim().is_empty() => name.trim_end_matches([' ', '—', '-']),
            _ => &self.title,
        }
    }

    /// Première phrase de la description, la suite étant sur la fiche du site.
    /// Une phrase se termine par « . » suivi d'une majuscule, ce qui ne coupe
    /// pas « (5pb. / MAGES.) ».
    pub fn short_description(&self, max_chars: usize) -> String {
        let text = self.description.trim();
        let first_sentence = text
            .match_indices(". ")
            .find(|(i, _)| text[i + 2..].starts_with(char::is_uppercase))
            .map_or(text, |(i, _)| &text[..=i]);
        cut_at_word(first_sentence, max_chars)
    }

    /// Logo du site, sinon celui de la page Steam du jeu.
    pub fn logo_source(&self) -> String {
        match self.steam_app_id {
            Some(app_id) if self.logo.is_empty() => {
                format!("https://cdn.cloudflare.steamstatic.com/steam/apps/{app_id}/logo.png")
            }
            _ => self.logo.clone(),
        }
    }

    pub fn kind_label(&self) -> &'static str {
        match self.kind.as_str() {
            "jeu" => "Jeu",
            _ => "Visual novel",
        }
    }
}

fn cut_at_word(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_owned();
    }
    let cut: String = text.chars().take(max_chars).collect();
    let cut = cut.rsplit_once(' ').map_or(cut.as_str(), |(start, _)| start);
    format!("{}…", cut.trim_end_matches([',', ';', ':', '.']))
}

#[derive(Deserialize, Default, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct Member {
    pub name: String,
    pub title: String,
    pub roles: Vec<String>,
    pub avatar: String,
}

/// Une donnée chargée en arrière-plan pour ne pas figer la fenêtre.
pub enum Remote<T> {
    Loading(Receiver<Result<T, String>>),
    Ready(T),
    Failed(String),
}

impl<T: DeserializeOwned + Send + 'static> Remote<T> {
    pub fn fetch(source: &str, ctx: &egui::Context) -> Self {
        let (sender, receiver) = mpsc::channel();
        let source = source.to_owned();
        let ctx = ctx.clone();

        thread::spawn(move || {
            let _ = sender.send(load(&source));
            ctx.request_repaint();
        });

        Remote::Loading(receiver)
    }

    /// À appeler à chaque image : passe à `Ready` ou `Failed` dès que le
    /// chargement est terminé. Renvoie `true` à l'image où les données arrivent.
    pub fn poll(&mut self) -> bool {
        let result = match self {
            Remote::Loading(receiver) => receiver.try_recv(),
            _ => return false,
        };

        match result {
            Ok(Ok(data)) => {
                *self = Remote::Ready(data);
                return true;
            }
            Ok(Err(error)) => *self = Remote::Failed(error),
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => {
                *self = Remote::Failed("le chargement s'est interrompu".to_owned())
            }
        }
        false
    }
}

fn load<T: DeserializeOwned>(source: &str) -> Result<T, String> {
    let text = if source.starts_with("http://") || source.starts_with("https://") {
        ureq::get(source)
            .call()
            .map_err(|e| e.to_string())?
            .into_string()
            .map_err(|e| e.to_string())?
    } else {
        std::fs::read_to_string(source).map_err(|e| format!("{source} : {e}"))?
    };

    serde_json::from_str(&text).map_err(|e| format!("JSON invalide : {e}"))
}
