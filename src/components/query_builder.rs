use crate::components::Component;
use crate::theme::Theme;
use egui::{RichText, ScrollArea, Ui, Vec2, Widget};
use std::sync::Arc;

use super::ThemedButton;

pub struct QueryBuilder {
    query: String,
    projection: String,
    sort: String,
    theme: Arc<Theme>,
}

impl QueryBuilder {
    pub fn new(theme: Arc<Theme>) -> Self {
        Self {
            query: String::new(),
            projection: String::new(),
            sort: String::new(),
            theme,
        }
    }
}

impl Component for QueryBuilder {
    fn render(&mut self, ui: &mut Ui, id_prefix: &str) {
        ui.vertical(|ui| {
            // Query section
            ui.label(
                RichText::new("Query:")
                    .color(self.theme.text_color)
                    .strong(),
            );
            let query_edit = egui::TextEdit::multiline(&mut self.query)
                .desired_width(ui.available_width())
                .desired_rows(5)
                .id_source(format!("{}_query", id_prefix));
            ui.add(query_edit);

            ui.add_space(10.0);

            // Projection section
            ui.label(
                RichText::new("Projection:")
                    .color(self.theme.text_color)
                    .strong(),
            );
            let projection_edit = egui::TextEdit::singleline(&mut self.projection)
                .desired_width(ui.available_width())
                .id_source(format!("{}_projection", id_prefix));
            ui.add(projection_edit);

            ui.add_space(10.0);

            // Sort section
            ui.label(RichText::new("Sort:").color(self.theme.text_color).strong());
            let sort_edit = egui::TextEdit::singleline(&mut self.sort)
                .desired_width(ui.available_width())
                .id_source(format!("{}_sort", id_prefix));
            ui.add(sort_edit);

            ui.add_space(20.0);

            // Execute button
            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                if ThemedButton::new("Execute Query", Arc::clone(&self.theme))
                    .ui(ui)
                    .clicked()
                {
                    // Execute query logic here
                }
            });
        });
    }

    fn update_theme(&mut self, theme: Arc<Theme>) {
        self.theme = theme;
    }
}
