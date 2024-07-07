use crate::theme::Theme;
use egui::Ui;
use std::sync::Arc;

pub trait Component {
    fn render(&mut self, ui: &mut egui::Ui, id_prefix: &str);
    fn update_theme(&mut self, theme: Arc<Theme>);
}

mod collection_selector;
mod connection_manager;
mod database_selector;
mod query_builder;
mod results_view;
mod status_bar;
mod tab;
mod widgets;

pub use collection_selector::CollectionSelector;
pub use connection_manager::ConnectionManager;
pub use database_selector::DatabaseSelector;
pub use query_builder::QueryBuilder;
pub use results_view::ResultsView;
pub use status_bar::StatusBar;
pub use tab::Tab;
pub use widgets::ThemedButton;
