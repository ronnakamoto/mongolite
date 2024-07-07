use crate::components::Component;
use crate::theme::Theme;
use egui::{Grid, RichText, ScrollArea, Ui};
use std::sync::Arc;

pub struct ResultsView {
    results: Vec<Vec<String>>,
    theme: Arc<Theme>,
}

impl ResultsView {
    pub fn new(theme: Arc<Theme>) -> Self {
        Self {
            results: Vec::new(),
            theme,
        }
    }
}

impl Component for ResultsView {
    fn render(&mut self, ui: &mut Ui, id_prefix: &str) {
        ui.label(
            RichText::new("Results:")
                .color(self.theme.text_color)
                .strong(),
        );
        self.render_table(ui, id_prefix);
    }

    fn update_theme(&mut self, theme: Arc<Theme>) {
        self.theme = theme;
    }
}

impl ResultsView {
    pub fn render_table(&self, ui: &mut Ui, id_prefix: &str) {
        ScrollArea::both()
            .id_source(format!("{}_table", id_prefix))
            .show(ui, |ui| {
                Grid::new("results_grid").striped(true).show(ui, |ui| {
                    for row in &self.results {
                        for cell in row {
                            ui.label(cell);
                        }
                        ui.end_row();
                    }
                });
            });
    }

    pub fn render_json(&self, ui: &mut Ui, id_prefix: &str) {
        ScrollArea::both()
            .id_source(format!("{}_json", id_prefix))
            .show(ui, |ui| {
                // This is a simplistic JSON representation. In a real application,
                // you'd want to use a proper JSON serialization library.
                let json = self
                    .results
                    .iter()
                    .map(|row| format!("  {}", row.join(", ")))
                    .collect::<Vec<_>>()
                    .join(",\n");
                let json = format!("[\n{}\n]", json);
                ui.code(json);
            });
    }
}
