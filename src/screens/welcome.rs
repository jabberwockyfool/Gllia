use egui::{vec2, Align2, FontId, Key};

use crate::app::LauncherApp;

/// Le titre glisse vers la droite pendant `SLIDE` secondes, puis le message
/// de bienvenue apparaît. On passe à la suite après `TOTAL` secondes.
const SLIDE: f64 = 1.2;
const FADE: f64 = 0.6;
const TOTAL: f64 = 3.0;
const SLIDE_DISTANCE: f32 = 160.0;

pub fn show(app: &mut LauncherApp, ctx: &egui::Context) {
    let now = ctx.input(|i| i.time);
    let elapsed = now - *app.welcome_started.get_or_insert(now);

    let slide = ease_out((elapsed / SLIDE).min(1.0) as f32);
    let welcome = ((elapsed - SLIDE) / FADE).clamp(0.0, 1.0) as f32;

    egui::CentralPanel::default().show(ctx, |ui| {
        let center = ui.max_rect().center();
        let color = ui.visuals().text_color();
        let painter = ui.painter();

        painter.text(
            center + vec2((slide - 1.0) * SLIDE_DISTANCE, -50.0),
            Align2::CENTER_CENTER,
            "Gllia",
            FontId::proportional(48.0),
            color.gamma_multiply(slide),
        );
        painter.text(
            center + vec2(0.0, 40.0),
            Align2::CENTER_CENTER,
            "Bienvenue sur Gllia !",
            FontId::proportional(26.0),
            color.gamma_multiply(welcome),
        );
    });

    // Un clic ou une touche permet de passer l'animation.
    let skipped = ctx.input(|i| {
        i.pointer.any_click()
            || i.key_pressed(Key::Enter)
            || i.key_pressed(Key::Space)
            || i.key_pressed(Key::Escape)
    });

    if elapsed >= TOTAL || skipped {
        app.leave_welcome();
    } else {
        ctx.request_repaint();
    }
}

fn ease_out(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}
