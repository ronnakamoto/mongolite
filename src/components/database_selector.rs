use crate::components::{Component, ThemedButton};
use crate::theme::Theme;
use egui::{ComboBox, RichText, Ui, Widget};
use std::sync::Arc;

pub struct DatabaseSelector {
    selected_database: String,
    databases: Vec<String>,
    theme: Arc<Theme>,
}

impl DatabaseSelector {
    pub fn new(theme: Arc<Theme>) -> Self {
        Self {
            selected_database: String::new(),
            databases: Vec::new(),
            theme,
        }
    }
}

impl Component for DatabaseSelector {
    fn render(&mut self, ui: &mut Ui, _id_prefix: &str) {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Database:")
                    .color(self.theme.text_color)
                    .strong(),
            );
            ui.add_space(10.0);
            ComboBox::from_id_source("database_selector")
                .width(290.0)
                .selected_text(&self.selected_database)
                .show_ui(ui, |ui| {
                    for db in &self.databases {
                        ui.selectable_value(&mut self.selected_database, db.clone(), db);
                    }
                });
            if ThemedButton::new("Refresh", Arc::clone(&self.theme))
                .ui(ui)
                .clicked()
            {
                // Refresh database list
            }
        });
    }

    fn update_theme(&mut self, theme: Arc<Theme>) {
        self.theme = theme;
    }
}
