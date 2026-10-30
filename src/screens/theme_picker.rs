use egui::{vec2, Align2, CursorIcon, FontId, Painter, Rect, RichText, Sense, Stroke, StrokeKind, Vec2};

use crate::app::LauncherApp;
use crate::layouts::Layout;
use crate::palette::{self, Palette};

const CARD_SIZE: Vec2 = vec2(220.0, 250.0);
const CARD_GAP: f32 = 40.0;

pub fn show(app: &mut LauncherApp, ctx: &egui::Context) {
    let palette = palette::current(ctx);
    let mut chosen = None;

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.add_space(56.0);
        ui.vertical_centered(|ui| {
            ui.label(RichText::new("Choisissez votre thème").size(36.0).strong());
        });
        ui.add_space(56.0);

        let row_width = Layout::ALL.len() as f32 * (CARD_SIZE.x + CARD_GAP) - CARD_GAP;
        ui.horizontal(|ui| {
            ui.add_space(((ui.available_width() - row_width) / 2.0).max(0.0));
            ui.spacing_mut().item_spacing.x = CARD_GAP;

            for layout in Layout::ALL {
                if card(ui, layout, palette).clicked() {
                    chosen = Some(layout);
                }
            }
        });
    });

    if let Some(layout) = chosen {
        app.select_layout(layout);
    }
}

fn card(ui: &mut egui::Ui, layout: Layout, palette: &Palette) -> egui::Response {
    let ready = layout.is_ready();
    let sense = if ready { Sense::click() } else { Sense::hover() };
    let (rect, response) = ui.allocate_exact_size(CARD_SIZE, sense);
    let painter = ui.painter();

    let border = if ready && response.hovered() {
        Stroke::new(3.0_f32, palette.accent)
    } else {
        Stroke::new(2.0_f32, palette.text.gamma_multiply(0.25))
    };
    painter.rect(rect, 18.0, palette.panel, border, StrokeKind::Inside);

    let preview = Rect::from_min_size(rect.min + vec2(20.0, 20.0), vec2(rect.width() - 40.0, 120.0));
    draw_preview(painter, preview, layout, palette);

    painter.text(
        rect.center_bottom() - vec2(0.0, 76.0),
        Align2::CENTER_CENTER,
        layout.name(),
        FontId::proportional(24.0),
        palette.text,
    );
    painter.text(
        rect.center_bottom() - vec2(0.0, 44.0),
        Align2::CENTER_CENTER,
        if ready { layout.description() } else { "Bientôt" },
        FontId::proportional(14.0),
        palette.text.gamma_multiply(0.7),
    );

    if ready {
        response.on_hover_cursor(CursorIcon::PointingHand)
    } else {
        painter.rect_filled(rect, 18.0, palette.background.gamma_multiply(0.55));
        response
    }
}

/// Miniature schématique de chaque thème.
fn draw_preview(painter: &Painter, rect: Rect, layout: Layout, palette: &Palette) {
    painter.rect_filled(rect, 10.0, palette.background);
    let block = palette.text.gamma_multiply(0.15);
    let inner = rect.shrink(10.0);

    match layout {
        Layout::Steam => {
            let side = Rect::from_min_size(inner.min, vec2(inner.width() * 0.3, inner.height()));
            painter.rect_filled(side, 4.0, palette.sidebar);
            for i in 0..5 {
                let line = Rect::from_min_size(side.min + vec2(6.0, 8.0 + i as f32 * 14.0), vec2(side.width() - 12.0, 6.0));
                painter.rect_filled(line, 3.0, block);
            }
            let button = Rect::from_min_size(inner.right_bottom() - vec2(56.0, 18.0), vec2(56.0, 14.0));
            painter.rect_filled(button, 4.0, palette.accent);
        }
        Layout::Carrousel => {
            let center = inner.center();
            for offset in [-1.0, 1.0] {
                let tile = Rect::from_center_size(center + vec2(offset * 52.0, 0.0), vec2(34.0, 60.0));
                painter.rect_filled(tile, 4.0, block);
            }
            painter.rect_filled(Rect::from_center_size(center, vec2(56.0, 80.0)), 6.0, palette.accent);
        }
        Layout::Bibliotheque => {
            let size = vec2((inner.width() - 16.0) / 3.0, (inner.height() - 8.0) / 2.0);
            for row in 0..2 {
                for column in 0..3 {
                    let min = inner.min + vec2(column as f32 * (size.x + 8.0), row as f32 * (size.y + 8.0));
                    painter.rect_filled(Rect::from_min_size(min, size), 4.0, block);
                }
            }
        }
    }
}
