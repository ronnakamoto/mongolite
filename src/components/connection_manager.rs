use crate::components::{Component, ThemedButton};
use crate::models::ConnectionProfile;
use crate::theme::Theme;
use egui::{Response, RichText, Ui, Widget};
use std::sync::Arc;

pub struct ConnectionManager {
    connection_string: String,
    profiles: Vec<ConnectionProfile>,
    theme: Arc<Theme>,
}

impl ConnectionManager {
    pub fn new(theme: Arc<Theme>) -> Self {
        Self {
            connection_string: String::new(),
            profiles: Vec::new(),
            theme,
        }
    }
}

impl Component for ConnectionManager {
    fn render(&mut self, ui: &mut Ui, _id_prefix: &str) {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Connection:")
                    .color(self.theme.text_color)
                    .strong(),
            );
            ui.text_edit_singleline(&mut self.connection_string);

            if ThemedButton::new("Connect", Arc::clone(&self.theme))
                .ui(ui)
                .clicked()
            {
                // Handle connection logic
            }

            if ThemedButton::new("Manage Connections", Arc::clone(&self.theme))
                .ui(ui)
                .clicked()
            {
                // Open connection manager
            }
        });
    }

    fn update_theme(&mut self, theme: Arc<Theme>) {
        self.theme = theme;
    }
}
