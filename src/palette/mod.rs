//! Couleurs du launcher : une palette claire et une sombre, chacune dans son
//! fichier. L'apparence (clair, sombre ou comme Windows) se règle dans les options.

mod clair;
mod sombre;

use egui::{Color32, CornerRadius, Stroke, ThemePreference, Visuals};
use serde::{Deserialize, Serialize};

pub struct Palette {
    pub dark: bool,
    pub background: Color32,
    pub sidebar: Color32,
    /// Fond des fenêtres, cartes et champs.
    pub panel: Color32,
    pub text: Color32,
    pub accent: Color32,
    /// Logo du dossier assets qui se lit bien sur ce fond.
    pub logo: &'static str,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum Appearance {
    #[default]
    Clair,
    Sombre,
    Systeme,
}

impl Appearance {
    pub const ALL: [Appearance; 3] = [Appearance::Clair, Appearance::Sombre, Appearance::Systeme];

    pub fn label(self) -> &'static str {
        match self {
            Appearance::Clair => "Clair",
            Appearance::Sombre => "Sombre",
            Appearance::Systeme => "Comme Windows",
        }
    }
}

pub fn apply(ctx: &egui::Context, appearance: Appearance) {
    ctx.set_visuals_of(egui::Theme::Light, clair::PALETTE.visuals());
    ctx.set_visuals_of(egui::Theme::Dark, sombre::PALETTE.visuals());
    ctx.set_theme(match appearance {
        Appearance::Clair => ThemePreference::Light,
        Appearance::Sombre => ThemePreference::Dark,
        Appearance::Systeme => ThemePreference::System,
    });
}

/// La palette affichée en ce moment (utile en mode « Comme Windows »).
pub fn current(ctx: &egui::Context) -> &'static Palette {
    if ctx.style().visuals.dark_mode {
        &sombre::PALETTE
    } else {
        &clair::PALETTE
    }
}

impl Palette {
    fn visuals(&self) -> Visuals {
        let mut visuals = if self.dark { Visuals::dark() } else { Visuals::light() };

        visuals.override_text_color = Some(self.text);
        visuals.panel_fill = self.background;
        visuals.window_fill = self.panel;
        visuals.window_corner_radius = CornerRadius::same(16);
        // Fond des barres de progression et des champs de texte.
        visuals.extreme_bg_color = self.panel;
        visuals.faint_bg_color = self.panel;
        visuals.hyperlink_color = self.accent;
        visuals.selection.bg_fill = self.accent.gamma_multiply(0.35);
        visuals.selection.stroke = Stroke::new(1.0_f32, self.text);

        // Boutons teintés de la couleur d'accent ; cases à cocher sur le fond.
        visuals.widgets.inactive.weak_bg_fill = self.accent.gamma_multiply(0.18);
        visuals.widgets.inactive.bg_fill = self.background;
        visuals.widgets.hovered.weak_bg_fill = self.accent.gamma_multiply(0.4);
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, self.accent);
        visuals.widgets.active.weak_bg_fill = self.accent.gamma_multiply(0.6);
        for widget in [
            &mut visuals.widgets.inactive,
            &mut visuals.widgets.hovered,
            &mut visuals.widgets.active,
        ] {
            widget.corner_radius = CornerRadius::same(8);
        }

        visuals
    }
}
