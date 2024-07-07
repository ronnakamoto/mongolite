use egui::{Color32, Rounding, Stroke, Vec2};

pub struct Theme {
    pub accent_color: Color32,
    pub bg_color: Color32,
    pub text_color: Color32,
    pub danger_color: Color32,
    pub separator_color: Color32,
    pub button_rounding: Rounding,
    pub frame_rounding: Rounding,
    pub window_rounding: Rounding,
    pub spacing: Vec2,
    pub button_padding: Vec2,
    pub frame_stroke: Stroke,
}

impl Theme {
    pub fn google_theme() -> Self {
        Self {
            accent_color: Color32::from_rgb(15, 157, 88), // Google Green
            bg_color: Color32::from_rgb(248, 249, 250),   // Light Gray
            text_color: Color32::from_rgb(60, 64, 67),    // Dark Gray
            danger_color: Color32::from_rgb(234, 67, 53), // Google Red
            separator_color: Color32::from_gray(200),
            button_rounding: Rounding::same(4.0),
            frame_rounding: Rounding::same(4.0),
            window_rounding: Rounding::same(8.0),
            spacing: Vec2::new(8.0, 8.0),
            button_padding: Vec2::new(8.0, 4.0),
            frame_stroke: Stroke::new(1.0, Color32::from_gray(180)),
        }
    }

    pub fn google_dark_theme() -> Self {
        Self {
            accent_color: Color32::from_rgb(15, 157, 88), // Google Green
            bg_color: Color32::from_rgb(32, 33, 36),      // Dark Gray
            text_color: Color32::from_rgb(232, 234, 237), // Light Gray
            danger_color: Color32::from_rgb(234, 67, 53), // Google Red
            separator_color: Color32::from_gray(100),
            // Other fields remain the same
            ..Self::google_theme()
        }
    }

    pub fn apply(&self, ctx: &egui::Context) {
        let mut style = (*ctx.style()).clone();
        style.spacing.item_spacing = self.spacing;
        style.spacing.button_padding = self.button_padding;

        let mut visuals = style.visuals.clone();
        visuals.override_text_color = Some(self.text_color);
        visuals.widgets.noninteractive.bg_fill = self.bg_color;
        visuals.widgets.inactive.bg_fill = self.bg_color;
        visuals.widgets.hovered.bg_fill = self.accent_color.linear_multiply(0.8);
        visuals.widgets.active.bg_fill = self.accent_color;
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, self.text_color);
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, self.text_color);
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.5, Color32::WHITE);
        visuals.widgets.active.fg_stroke = Stroke::new(2.0, Color32::WHITE);
        visuals.widgets.noninteractive.rounding = self.frame_rounding;
        visuals.widgets.inactive.rounding = self.frame_rounding;
        visuals.widgets.hovered.rounding = self.frame_rounding;
        visuals.widgets.active.rounding = self.frame_rounding;
        visuals.window_rounding = self.window_rounding;
        visuals.popup_shadow.spread = 8.0;

        style.visuals = visuals;
        ctx.set_style(style);
    }
}
