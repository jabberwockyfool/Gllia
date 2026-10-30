use std::sync::Arc;

use egui::{FontData, FontDefinitions, FontFamily};

/// Police des maquettes.
const MAIN_FONT: &str = "fonts/florencesans.ttf";

pub fn install(ctx: &egui::Context) {
    let Some(bytes) = crate::assets::embedded(MAIN_FONT) else {
        return;
    };

    let mut fonts = FontDefinitions::default();
    fonts
        .font_data
        .insert("florencesans".to_owned(), Arc::new(FontData::from_static(bytes)));
    fonts
        .families
        .entry(FontFamily::Proportional)
        .or_default()
        .insert(0, "florencesans".to_owned());

    ctx.set_fonts(fonts);
}
