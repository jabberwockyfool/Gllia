use egui::Color32;

use super::Palette;

pub const PALETTE: Palette = Palette {
    dark: true,
    background: Color32::from_rgb(26, 27, 31),
    sidebar: Color32::from_rgb(19, 20, 23),
    panel: Color32::from_rgb(37, 38, 43),
    text: Color32::from_rgb(232, 230, 226),
    accent: Color32::from_rgb(228, 58, 68),
    logo: "logo-sombre.png",
};
