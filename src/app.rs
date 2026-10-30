use std::collections::HashMap;
use std::sync::mpsc::Receiver;

use crate::catalogue::{Member, Patch, Remote};
use crate::config::Config;
use crate::install::Install;
use crate::layouts::Layout;
use crate::music::Music;
use crate::stores::{self, Installed};
use crate::{link, palette, screens, windows};

/// Affiché sous le logo.
pub const VERSION: &str = concat!("a-", env!("CARGO_PKG_VERSION"));

/// L'écran affiché en plein dans la fenêtre. Les fenêtres « À propos » et
/// « Options » ne sont pas des écrans : elles s'ouvrent par-dessus.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Welcome,
    ThemePicker,
    Home,
}

pub struct LauncherApp {
    pub config: Config,
    pub screen: Screen,
    pub patches: Remote<Vec<Patch>>,
    pub members: Remote<Vec<Member>>,
    /// Jeux trouvés sur Steam ou GOG, par slug de patch.
    pub detected: HashMap<String, Installed>,
    /// Slug du jeu sélectionné dans la liste.
    pub selected: Option<String>,
    pub search: String,
    pub music: Music,
    pub show_about: bool,
    pub show_options: bool,
    /// Moment (en secondes, horloge d'egui) où l'écran d'accueil est apparu.
    pub welcome_started: Option<f64>,
    pub install: Option<Install>,
    pub link: Option<String>,
    pub links: Option<Receiver<String>>,
}

impl LauncherApp {
    pub fn new(ctx: &egui::Context, link: Option<String>) -> Self {
        let config = Config::load();
        palette::apply(ctx, config.appearance);

        Self {
            patches: Remote::fetch(&config.patches_url, ctx),
            members: Remote::fetch(&config.members_url, ctx),
            detected: HashMap::new(),
            selected: None,
            search: String::new(),
            music: Music::start(config.music_volume, config.music_muted),
            config,
            screen: Screen::Welcome,
            show_about: false,
            show_options: false,
            welcome_started: None,
            install: None,
            link,
            links: link::listen(ctx),
        }
    }

    /// Après l'animation d'accueil : choix du thème au premier lancement,
    /// sinon directement l'écran principal.
    pub fn leave_welcome(&mut self) {
        self.screen = if self.config.layout.is_some() {
            Screen::Home
        } else {
            Screen::ThemePicker
        };
    }

    pub fn select_layout(&mut self, layout: Layout) {
        self.config.layout = Some(layout);
        self.config.save();
        self.screen = Screen::Home;
    }

    pub fn update_music(&self) {
        self.music.set_volume(self.config.music_volume, self.config.music_muted);
    }

    pub fn reload(&mut self, ctx: &egui::Context) {
        self.patches = Remote::fetch(&self.config.patches_url, ctx);
        self.members = Remote::fetch(&self.config.members_url, ctx);
    }

    /// Les patchs dont le titre correspond à la recherche.
    pub fn visible_patches(&self) -> Vec<&Patch> {
        let Remote::Ready(patches) = &self.patches else {
            return Vec::new();
        };
        let search = self.search.to_lowercase();
        patches
            .iter()
            .filter(|patch| patch.title.to_lowercase().contains(&search))
            .collect()
    }

    /// Le patch sélectionné, ou le premier de la liste.
    pub fn selected_patch(&self) -> Option<&Patch> {
        let visible = self.visible_patches();
        visible
            .iter()
            .find(|patch| self.selected.as_deref() == Some(patch.slug.as_str()))
            .or(visible.first())
            .copied()
    }

    /// « Ajouter le jeu » : prend le dossier trouvé sur Steam ou GOG, sinon
    /// demande à l'utilisateur où est installé le jeu.
    pub fn add_game(&mut self, slug: &str) {
        let folder = match self.detected.get(slug) {
            Some(found) => Some(found.path.clone()),
            None => rfd::FileDialog::new()
                .set_title("Où est installé le jeu ?")
                .pick_folder(),
        };

        if let Some(folder) = folder {
            self.config.games.insert(slug.to_owned(), folder);
            self.config.save();
        }
    }

    pub fn remove_game(&mut self, slug: &str) {
        self.config.games.remove(slug);
        self.config.save();
    }

    pub fn install_patch(&mut self, slug: &str, ctx: &egui::Context) {
        if self.install.as_ref().is_some_and(Install::is_running) {
            return;
        }
        let Remote::Ready(patches) = &self.patches else {
            return;
        };
        let Some(download_url) = patches
            .iter()
            .find(|patch| patch.slug == slug)
            .map(|patch| patch.download_url.clone())
            .filter(|url| !url.is_empty())
        else {
            return;
        };

        if !self.config.games.contains_key(slug) {
            self.add_game(slug);
        }
        if let Some(folder) = self.config.games.get(slug).cloned() {
            self.install = Some(Install::start(slug, download_url, folder, ctx));
        }
    }

    fn open_link(&mut self, ctx: &egui::Context) {
        if let Some(links) = &self.links {
            if let Ok(slug) = links.try_recv() {
                self.link = Some(slug);
                ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
            }
        }
        if !matches!(self.patches, Remote::Ready(_)) {
            return;
        }
        let Some(slug) = self.link.take() else {
            return;
        };

        if self.screen == Screen::Welcome {
            self.leave_welcome();
        }
        self.search.clear();
        self.selected = Some(slug.clone());
        self.install_patch(&slug, ctx);
    }

    fn detect_games(&mut self) {
        let Remote::Ready(patches) = &self.patches else {
            return;
        };
        self.detected = patches
            .iter()
            .filter_map(|patch| stores::find(patch).map(|found| (patch.slug.clone(), found)))
            .collect();
    }
}

impl eframe::App for LauncherApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.patches.poll() {
            self.detect_games();
        }
        self.members.poll();
        self.open_link(ctx);

        match self.screen {
            Screen::Welcome => screens::welcome::show(self, ctx),
            Screen::ThemePicker => screens::theme_picker::show(self, ctx),
            Screen::Home => self.config.layout.unwrap_or_default().show(self, ctx),
        }

        if self.show_about {
            windows::about::show(self, ctx);
        }
        if self.show_options {
            windows::options::show(self, ctx);
        }
    }
}
