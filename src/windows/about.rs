use egui::{vec2, Align2, Frame, RichText, ScrollArea, Ui};

use crate::app::LauncherApp;
use crate::assets;
use crate::catalogue::{Member, Remote};

const AVATAR_SIZE: f32 = 36.0;

pub fn show(app: &mut LauncherApp, ctx: &egui::Context) {
    let mut open = true;

    let mut fill = ctx.style().visuals.window_fill;
    if app.config.about_transparent {
        fill = fill.gamma_multiply(0.8);
    }

    egui::Window::new("À propos")
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
        .fixed_size([520.0, 380.0])
        .frame(Frame::window(&ctx.style()).fill(fill))
        .show(ctx, |ui| {
            ui.label("Gllia rassemble les patchs de traduction de la team Saint Eresia.");
            ui.add_space(12.0);

            ui.label(RichText::new("L'équipe").size(18.0).strong());
            members(ui, &app.members);

            ui.separator();
            ui.checkbox(&mut app.config.about_transparent, "Transparent");
        });

    if !open {
        app.show_about = false;
        app.config.save();
    }
}

fn members(ui: &mut Ui, members: &Remote<Vec<Member>>) {
    match members {
        Remote::Loading(_) => {
            ui.spinner();
        }
        Remote::Failed(error) => {
            ui.label("Impossible de récupérer la liste des membres.");
            ui.label(RichText::new(error).weak().small());
        }
        Remote::Ready(members) => {
            ScrollArea::vertical().max_height(240.0).show(ui, |ui| {
                for member in members {
                    member_row(ui, member);
                }
            });
        }
    }
}

fn member_row(ui: &mut Ui, member: &Member) {
    ui.horizontal(|ui| {
        match assets::source_uri(&member.avatar) {
            Some(uri) => {
                let avatar = egui::Image::new(uri)
                    .fit_to_exact_size(vec2(AVATAR_SIZE, AVATAR_SIZE))
                    .corner_radius(AVATAR_SIZE / 2.0);
                ui.add(avatar);
            }
            None => ui.add_space(AVATAR_SIZE),
        }

        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(&member.name).strong());
                if !member.title.is_empty() {
                    ui.label(RichText::new(&member.title).weak());
                }
            });
            if !member.roles.is_empty() {
                ui.label(RichText::new(member.roles.join(", ")).small());
            }
        });
    });
    ui.add_space(6.0);
}
