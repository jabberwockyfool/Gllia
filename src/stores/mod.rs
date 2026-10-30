//! Détection des jeux déjà installés par Steam ou GOG.

mod gog;
mod steam;

use std::path::PathBuf;

use crate::catalogue::Patch;

#[derive(Clone, Copy)]
pub enum Store {
    Steam,
    Gog,
}

impl Store {
    pub fn name(self) -> &'static str {
        match self {
            Store::Steam => "Steam",
            Store::Gog => "GOG",
        }
    }
}

pub struct Installed {
    pub store: Store,
    pub path: PathBuf,
}

/// Cherche le jeu d'un patch d'après ses identifiants Steam et GOG.
pub fn find(patch: &Patch) -> Option<Installed> {
    let on_steam = patch
        .steam_app_id
        .and_then(steam::find)
        .map(|path| Installed { store: Store::Steam, path });

    on_steam.or_else(|| {
        patch
            .gog_id
            .and_then(gog::find)
            .map(|path| Installed { store: Store::Gog, path })
    })
}
