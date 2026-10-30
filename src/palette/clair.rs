//! Clair sans être blanc : des gris chauds qui ne fatiguent pas les yeux.

use egui::Color32;

use super::Palette;

pub const PALETTE: Palette = Palette {
    dark: false,
    background: Color32::from_rgb(232, 229, 223),
    sidebar: Color32::from_rgb(219, 215, 208),
    panel: Color32::from_rgb(243, 241, 236),
    text: Color32::from_rgb(38, 36, 40),
    accent: Color32::from_rgb(196, 30, 42),
    logo: "logo-clair.png",
};
