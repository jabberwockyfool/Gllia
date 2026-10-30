//! Thème « Liste » : les jeux dans une colonne à gauche, la fiche du jeu
//! sélectionné sur son illustration, comme la bibliothèque Steam.

use egui::{
    pos2, vec2, Align, Button, Color32, Frame, Layout, Margin, ProgressBar, Rect, RichText, ScrollArea, TextEdit,
    Ui, UiBuilder,
};

use super::{paint_cover, paint_horizontal_gradient};
use crate::app::{LauncherApp, VERSION};
use crate::assets;
use crate::catalogue::{Patch, Remote};
use crate::install::Step;
use crate::palette::{self, Palette};

const SIDEBAR_WIDTH: f32 = 250.0;
const LOGO_HEIGHT: f32 = 44.0;
const DETAILS_WIDTH: f32 = 560.0;
const PAGE_MARGIN: f32 = 48.0;
/// La description complète est sur la fiche du site.
const DESCRIPTION_LENGTH: usize = 320;

pub fn show(app: &mut LauncherApp, ctx: &egui::Context) {
    let palette = palette::current(ctx);

    egui::SidePanel::left("jeux")
        .exact_width(SIDEBAR_WIDTH)
        .resizable(false)
        .frame(Frame::NONE.fill(palette.sidebar).inner_margin(Margin::same(16)))
        .show(ctx, |ui| sidebar(app, ui, palette));

    egui::CentralPanel::default()
        .frame(Frame::NONE.fill(palette.background))
        .show(ctx, |ui| game_page(app, ui, palette));
}

fn sidebar(app: &mut LauncherApp, ui: &mut Ui, palette: &Palette) {
    // Version alignée sur le bas du logo.
    let header = vec2(ui.available_width(), LOGO_HEIGHT);
    ui.allocate_ui_with_layout(header, Layout::left_to_right(Align::Max), |ui| {
        match assets::image(palette.logo) {
            Some(logo) => ui.add(logo.max_height(LOGO_HEIGHT)),
            None => ui.label(RichText::new("Gllia").size(24.0).strong()),
        };
        ui.label(RichText::new(VERSION).small().weak());
    });

    ui.add_space(14.0);
    ui.add(
        TextEdit::singleline(&mut app.search)
            .hint_text("Rechercher")
            .desired_width(f32::INFINITY)
            .margin(Margin::symmetric(10, 8)),
    );
    ui.add_space(10.0);

    ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
        ui.horizontal(|ui| {
            if ui.button("À propos").clicked() {
                app.show_about = true;
            }
            if ui.button("Options").clicked() {
                app.show_options = true;
            }
        });
        ui.separator();
        ui.with_layout(Layout::top_down_justified(Align::Min), |ui| game_list(app, ui));
    });
}

fn game_list(app: &mut LauncherApp, ui: &mut Ui) {
    let mut clicked = None;

    ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
        ui.spacing_mut().button_padding = vec2(8.0, 6.0);
        let selected = app.selected_patch().map(|patch| patch.slug.as_str());

        for patch in app.visible_patches() {
            let is_selected = selected == Some(patch.slug.as_str());
            if ui.selectable_label(is_selected, RichText::new(patch.game_title()).size(18.0)).clicked() {
                clicked = Some(patch.slug.clone());
            }
        }
    });

    if clicked.is_some() {
        app.selected = clicked;
    }
}

fn game_page(app: &mut LauncherApp, ui: &mut Ui, palette: &Palette) {
    let error = match &app.patches {
        Remote::Loading(_) => {
            ui.centered_and_justified(|ui| ui.spinner());
            return;
        }
        Remote::Failed(error) => Some(error.clone()),
        Remote::Ready(_) => None,
    };

    if let Some(error) = error {
        ui.vertical_centered(|ui| {
            ui.add_space(ui.available_height() * 0.4);
            ui.label("Impossible de récupérer la liste des jeux.");
            ui.label(RichText::new(error).small().weak());
            if ui.button("Réessayer").clicked() {
                app.reload(ui.ctx());
            }
        });
        return;
    }

    let Some(patch) = app.selected_patch().cloned() else {
        ui.centered_and_justified(|ui| ui.label("Aucun jeu ne correspond à la recherche."));
        return;
    };

    let page = ui.max_rect();
    let left = (page.right() - PAGE_MARGIN - DETAILS_WIDTH).max(page.left() + PAGE_MARGIN);
    let column = Rect::from_min_max(
        pos2(left, page.top() + page.height() * 0.28),
        page.right_bottom() - vec2(PAGE_MARGIN, PAGE_MARGIN),
    );

    if let Some(uri) = assets::source_uri(&patch.background) {
        paint_cover(ui, &uri, page);
        // Voile : l'illustration reste nette à gauche et s'efface sous le texte.
        let veil = palette.background.gamma_multiply(0.88);
        let fade = Rect::from_x_y_ranges(page.left()..=column.left(), page.y_range());
        paint_horizontal_gradient(ui.painter(), fade, Color32::TRANSPARENT, veil);
        let under_text = Rect::from_x_y_ranges(column.left()..=page.right(), page.y_range());
        ui.painter().rect_filled(under_text, 0.0, veil);
    }
    ui.scope_builder(
        UiBuilder::new().max_rect(column).layout(Layout::top_down(Align::Max)),
        |ui| details(app, ui, &patch, palette),
    );
}

fn details(app: &mut LauncherApp, ui: &mut Ui, patch: &Patch, palette: &Palette) {
    // Le titre en texte remplace le logo s'il n'y en a pas ou s'il ne charge pas.
    let logo = assets::source_uri(&patch.logo_source())
        .map(|uri| egui::Image::new(uri).max_height(150.0).max_width(ui.available_width()))
        .filter(|logo| logo.load_for_size(ui.ctx(), ui.available_size()).is_ok());
    match logo {
        Some(logo) => ui.add(logo),
        None => ui.label(RichText::new(patch.game_title()).size(52.0).strong()),
    };
    if !patch.quote.is_empty() {
        ui.label(RichText::new(&patch.quote).size(28.0));
    }
    if !patch.description.is_empty() {
        ui.add_space(4.0);
        ui.label(RichText::new(patch.short_description(DESCRIPTION_LENGTH)).size(16.0));
    }

    ui.add_space(8.0);
    ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
        if !patch.page_url.is_empty() {
            ui.hyperlink_to(RichText::new("Fiche sur le site").small(), &patch.page_url);
            ui.label(RichText::new("·").small());
        }
        ui.label(RichText::new(format!("{} · patch {} · {} %", patch.kind_label(), patch.status, patch.progress)).small());
    });
    ui.add_space(12.0);

    let folder = app.config.games.get(&patch.slug).cloned();
    let install = app.install.as_ref().filter(|install| install.slug == patch.slug).map(|install| install.step());
    let busy = app.install.as_ref().is_some_and(|install| install.is_running());

    ui.scope(|ui| {
        ui.spacing_mut().button_padding = vec2(18.0, 10.0);
        ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
            let can_install = folder.is_some() && !patch.download_url.is_empty() && !busy;
            let label = match install {
                Some(Step::Done) => "Réinstaller le patch",
                _ => "Installer le patch",
            };
            let download = Button::new(RichText::new(label).size(18.0).color(Color32::WHITE)).fill(palette.accent);
            if ui.add_enabled(can_install, download).clicked() {
                app.install_patch(&patch.slug, ui.ctx());
            }

            if folder.is_some() {
                if ui.button(RichText::new("Retirer le jeu").size(18.0)).clicked() {
                    app.remove_game(&patch.slug);
                }
            } else if ui.button(RichText::new("Ajouter le jeu").size(18.0)).clicked() {
                app.add_game(&patch.slug);
            }
        });
    });

    ui.add_space(6.0);
    match install {
        Some(Step::Downloading(done, total)) => {
            let megabytes = |bytes: u64| bytes as f32 / 1_048_576.0;
            let bar = match total {
                Some(total) => ProgressBar::new(done as f32 / total as f32)
                    .text(format!("Téléchargement : {:.0} / {:.0} Mo", megabytes(done), megabytes(total))),
                None => ProgressBar::new(0.0).text(format!("Téléchargement : {:.0} Mo", megabytes(done))),
            };
            ui.add(bar.desired_width(ui.available_width().min(DETAILS_WIDTH)));
        }
        Some(Step::Installing) => {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label("Installation…");
            });
        }
        Some(Step::Done) => {
            ui.label("Patch installé. Lancez le jeu normalement.");
        }
        Some(Step::Browser) => {
            ui.label(RichText::new("Pas d'installeur pour ce patch : la page de téléchargement s'est ouverte dans le navigateur.").small());
        }
        Some(Step::Failed(error)) => {
            ui.label(RichText::new(format!("Échec de l'installation : {error}")).color(ui.visuals().error_fg_color));
        }
        None => {}
    }
    let hint = match (&folder, app.detected.get(&patch.slug)) {
        (Some(folder), _) => format!("Dossier du jeu : {}", folder.display()),
        (None, Some(found)) => format!("Trouvé sur {} : {}", found.store.name(), found.path.display()),
        (None, None) => "Ajoutez le jeu pour pouvoir installer le patch.".to_owned(),
    };
    ui.label(RichText::new(hint).small().weak());
    if patch.download_url.is_empty() {
        ui.label(RichText::new("Le patch n'est pas encore disponible au téléchargement.").small().weak());
    }
}
