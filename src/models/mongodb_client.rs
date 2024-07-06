use crate::components::{
    CollectionSelector, Component, ConnectionManager, DatabaseSelector, QueryBuilder, ResultsView,
    StatusBar, Tab,
};
use crate::services::{DatabaseService, QueryService};
use crate::theme::Theme;
use egui::{Align, Frame, Layout, Stroke, Ui};
use std::sync::Arc;

pub struct MongoDBClient {
    connection_manager: ConnectionManager,
    database_selector: DatabaseSelector,
    collection_selector: CollectionSelector,
    query_builder: QueryBuilder,
    results_view: ResultsView,
    status_bar: StatusBar,
    database_service: Arc<DatabaseService>,
    query_service: Arc<QueryService>,
    theme: Arc<Theme>,
    is_dark_mode: bool,
    query_tab: Tab<QueryBuilder>,
    results_tab: Tab<ResultsView>,
}

impl MongoDBClient {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let theme = Arc::new(Theme::google_theme());
        theme.apply(&cc.egui_ctx);

        let mut query_tab = Tab::new("query_tab".to_string(), Arc::clone(&theme));
        query_tab.add_tab(
            "Query".to_string(),
            Box::new(
                |ui: &mut Ui, query_builder: &mut QueryBuilder, _: &Theme, id_prefix: &str| {
                    query_builder.render(ui, id_prefix);
                },
            ),
        );
        query_tab.add_tab(
            "Aggregation".to_string(),
            Box::new(|ui: &mut Ui, _: &mut QueryBuilder, _: &Theme, _: &str| {
                ui.label("Aggregation tab content (to be implemented)");
            }),
        );

        let mut results_tab = Tab::new("results_tab".to_string(), Arc::clone(&theme));
        results_tab.add_tab(
            "Table View".to_string(),
            Box::new(
                |ui: &mut Ui, results_view: &mut ResultsView, _: &Theme, id_prefix: &str| {
                    results_view.render_table(ui, id_prefix);
                },
            ),
        );
        results_tab.add_tab(
            "JSON View".to_string(),
            Box::new(
                |ui: &mut Ui, results_view: &mut ResultsView, _: &Theme, id_prefix: &str| {
                    results_view.render_json(ui, id_prefix);
                },
            ),
        );

        Self {
            connection_manager: ConnectionManager::new(Arc::clone(&theme)),
            database_selector: DatabaseSelector::new(Arc::clone(&theme)),
            collection_selector: CollectionSelector::new(Arc::clone(&theme)),
            query_builder: QueryBuilder::new(Arc::clone(&theme)),
            results_view: ResultsView::new(Arc::clone(&theme)),
            status_bar: StatusBar::new(Arc::clone(&theme)),
            database_service: Arc::new(DatabaseService::new()),
            query_service: Arc::new(QueryService::new()),
            theme,
            is_dark_mode: false,
            query_tab,
            results_tab,
        }
    }

    pub fn render(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                self.render_top_section(ui);
                ui.add_space(10.0);
                self.render_main_section(ui);
                // ui.add_space(10.0);
                // self.render_footer(ui);
            });
        });
    }

    fn render_top_section(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.add_space(20.0);
            self.connection_manager.render(ui, "connection_manager");
            ui.add_space(20.0);
            ui.vertical(|ui| {
                self.database_selector.render(ui, "database_selector");
                ui.add_space(5.0);
                self.collection_selector.render(ui, "collection_selector");
            });
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui
                    .button(if self.is_dark_mode {
                        "Light Theme"
                    } else {
                        "Dark Theme"
                    })
                    .clicked()
                {
                    self.toggle_theme(ui.ctx());
                }
                ui.add_space(20.0);
            });
        });
    }

    fn render_main_section(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                let max_width = ui.available_width().min(600.0); // Set a maximum width of 600 pixels
                ui.set_max_width(max_width);
                self.query_tab.render(ui, &mut self.query_builder);
            });
            ui.add_space(10.0);
            ui.vertical(|ui| {
                ui.set_min_width(ui.available_width());
                self.results_tab.render(ui, &mut self.results_view);
            });
        });
    }

    fn render_footer(&mut self, ui: &mut Ui) {
        Frame::none()
            .stroke(Stroke::new(1.0, self.theme.separator_color))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(10.0);
                    self.status_bar.render(ui, "status_bar");
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui
                            .button(if self.is_dark_mode {
                                "Light Theme"
                            } else {
                                "Dark Theme"
                            })
                            .clicked()
                        {
                            self.toggle_theme(ui.ctx());
                        }
                        ui.add_space(10.0);
                    });
                });
            });
    }

    fn toggle_theme(&mut self, ctx: &egui::Context) {
        self.is_dark_mode = !self.is_dark_mode;
        let new_theme = Arc::new(if self.is_dark_mode {
            Theme::google_dark_theme()
        } else {
            Theme::google_theme()
        });
        self.theme = Arc::clone(&new_theme);
        self.theme.apply(ctx);

        // Update theme for all components
        self.connection_manager.update_theme(Arc::clone(&new_theme));
        self.database_selector.update_theme(Arc::clone(&new_theme));
        self.collection_selector
            .update_theme(Arc::clone(&new_theme));
        self.query_builder.update_theme(Arc::clone(&new_theme));
        self.results_view.update_theme(Arc::clone(&new_theme));
        self.status_bar.update_theme(Arc::clone(&new_theme));
        self.query_tab.update_theme(Arc::clone(&new_theme));
        self.results_tab.update_theme(Arc::clone(&new_theme));
    }

    fn execute_query(&mut self) {
        // Implementation for executing the current query
    }

    fn save_query(&mut self) {
        // Implementation for saving the current query
    }

    fn open_query(&mut self) {
        // Implementation for opening a saved query
    }

    fn new_query_tab(&mut self) {
        // Implementation for creating a new query tab
    }
}

impl eframe::App for MongoDBClient {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if ctx.input(|i| i.key_pressed(egui::Key::F5)) {
            self.execute_query();
        }
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::S)) {
            self.save_query();
        }
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::O)) {
            self.open_query();
        }
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::N)) {
            self.new_query_tab();
        }
        self.render(ctx);
    }
}
