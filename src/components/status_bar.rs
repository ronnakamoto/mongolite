use crate::components::Component;
use crate::theme::Theme;
use egui::{RichText, Ui};
use std::sync::Arc;

pub struct StatusBar {
    status: String,
    theme: Arc<Theme>,
}

impl StatusBar {
    pub fn new(theme: Arc<Theme>) -> Self {
        Self {
            status: String::new(),
            theme,
        }
    }

    pub fn set_status(&mut self, status: String) {
        self.status = status;
    }
}

impl Component for StatusBar {
    fn render(&mut self, ui: &mut Ui, _id_prefix: &str) {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Status:")
                    .color(self.theme.text_color)
                    .strong(),
            );
            ui.label(RichText::new(&self.status).color(self.theme.text_color));
        });
    }

    fn update_theme(&mut self, theme: Arc<Theme>) {
        self.theme = theme;
    }
}
