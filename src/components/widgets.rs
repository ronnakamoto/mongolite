use crate::theme::Theme;
use egui::{Response, Ui, Widget};
use std::sync::Arc;

pub struct ThemedButton<'a> {
    text: &'a str,
    theme: Arc<Theme>,
}

impl<'a> ThemedButton<'a> {
    pub fn new(text: &'a str, theme: Arc<Theme>) -> Self {
        Self { text, theme }
    }
}

impl<'a> Widget for ThemedButton<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.add(
            egui::Button::new(egui::RichText::new(self.text).color(self.theme.bg_color))
                .fill(self.theme.accent_color)
                .rounding(self.theme.button_rounding),
        )
    }
}
