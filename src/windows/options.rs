use egui::{Align2, RichText, Slider};

use crate::app::{LauncherApp, Screen};
use crate::palette::{self, Appearance};

pub fn show(app: &mut LauncherApp, ctx: &egui::Context) {
    let mut open = true;
    let mut change_theme = false;
    let mut reload = false;

    egui::Window::new("Options")
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
        .default_width(420.0)
        .show(ctx, |ui| {
            section(ui, "Musique");
            if app.music.is_available() {
                let volume = ui.add(Slider::new(&mut app.config.music_volume, 0.0..=1.0).text("Volume").show_value(false));
                let muted = ui.checkbox(&mut app.config.music_muted, "Couper la musique");
                if volume.changed() || muted.changed() {
                    app.update_music();
                }
            } else {
                ui.label("Pas de musique : ajoutez musique.ogg ou musique.mp3 dans le dossier assets.");
            }

            section(ui, "Apparence");
            ui.horizontal(|ui| {
                for appearance in Appearance::ALL {
                    if ui.radio_value(&mut app.config.appearance, appearance, appearance.label()).clicked() {
                        palette::apply(ctx, appearance);
                    }
                }
            });
            let layout = app.config.layout.unwrap_or_default();
            ui.label(format!("Thème : {}", layout.name()));
            change_theme = ui.button("Changer de thème").clicked();

            section(ui, "Sources");
            ui.label("Liste des patchs");
            ui.text_edit_singleline(&mut app.config.patches_url);
            ui.label("Liste des membres");
            ui.text_edit_singleline(&mut app.config.members_url);
            reload = ui.button("Recharger").clicked();
        });

    if reload {
        app.reload(ctx);
    }
    if change_theme {
        app.screen = Screen::ThemePicker;
        open = false;
    }
    if !open {
        app.show_options = false;
        app.config.save();
    }
}

fn section(ui: &mut egui::Ui, title: &str) {
    ui.add_space(8.0);
    ui.label(RichText::new(title).size(18.0).strong());
}
