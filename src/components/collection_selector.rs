use crate::components::{Component, ThemedButton};
use crate::theme::Theme;
use egui::{ComboBox, Response, RichText, Ui, Widget};
use std::sync::Arc;

pub struct CollectionSelector {
    selected_collection: String,
    collections: Vec<String>,
    theme: Arc<Theme>,
}

impl CollectionSelector {
    pub fn new(theme: Arc<Theme>) -> Self {
        Self {
            selected_collection: String::new(),
            collections: Vec::new(),
            theme,
        }
    }
}

impl Component for CollectionSelector {
    fn render(&mut self, ui: &mut Ui, id_prefix: &str) {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Collection:")
                    .color(self.theme.text_color)
                    .strong(),
            );
            ComboBox::from_id_source(format!("{}_collection_selector", id_prefix))
                .selected_text(&self.selected_collection)
                .show_ui(ui, |ui| {
                    for collection in &self.collections {
                        ui.selectable_value(
                            &mut self.selected_collection,
                            collection.clone(),
                            collection,
                        );
                    }
                });
            if ThemedButton::new("Refresh", Arc::clone(&self.theme))
                .ui(ui)
                .clicked()
            {
                // Refresh collection list
            }
        });
    }

    fn update_theme(&mut self, theme: Arc<Theme>) {
        self.theme = theme;
    }
}
