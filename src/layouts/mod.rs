//! Les « thèmes » de l'écran principal : chacun présente les jeux à sa façon,
//! un fichier par thème. Les couleurs, elles, sont dans `palette`.

mod steam;

use egui::{pos2, Color32, Mesh, Painter, Rect};
use serde::{Deserialize, Serialize};

use crate::app::LauncherApp;

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum Layout {
    #[default]
    Steam,
    Carrousel,
    Bibliotheque,
}

impl Layout {
    pub const ALL: [Layout; 3] = [Layout::Steam, Layout::Carrousel, Layout::Bibliotheque];

    pub fn name(self) -> &'static str {
        match self {
            Layout::Steam => "Liste",
            Layout::Carrousel => "Carrousel",
            Layout::Bibliotheque => "Bibliothèque",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Layout::Steam => "Les jeux à gauche, comme Steam.",
            Layout::Carrousel => "Les jeux défilent, comme sur Wii.",
            Layout::Bibliotheque => "Une grille de jaquettes.",
        }
    }

    /// Les thèmes pas encore codés sont grisés sur l'écran de choix.
    pub fn is_ready(self) -> bool {
        matches!(self, Layout::Steam)
    }

    pub fn show(self, app: &mut LauncherApp, ctx: &egui::Context) {
        match self {
            Layout::Steam => steam::show(app, ctx),
            // Pas encore faits : on retombe sur la liste.
            Layout::Carrousel | Layout::Bibliotheque => steam::show(app, ctx),
        }
    }
}

/// Dessine une image en remplissant tout `rect` sans la déformer : ce qui
/// dépasse est coupé, comme `object-fit: cover` en CSS.
pub fn paint_cover(ui: &egui::Ui, uri: &str, rect: Rect) {
    let image = egui::Image::new(uri.to_owned());
    let Ok(egui::load::TexturePoll::Ready { texture }) = image.load_for_size(ui.ctx(), rect.size()) else {
        return;
    };

    let scale = (rect.width() / texture.size.x).max(rect.height() / texture.size.y);
    let visible = rect.size() / scale / texture.size;
    let uv = Rect::from_center_size(pos2(0.5, 0.5), visible);
    image.uv(uv).paint_at(ui, rect);
}

pub fn paint_horizontal_gradient(painter: &Painter, rect: Rect, left: Color32, right: Color32) {
    let mut mesh = Mesh::default();
    mesh.colored_vertex(rect.left_top(), left);
    mesh.colored_vertex(rect.right_top(), right);
    mesh.colored_vertex(rect.right_bottom(), right);
    mesh.colored_vertex(rect.left_bottom(), left);
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(0, 2, 3);
    painter.add(mesh);
}
