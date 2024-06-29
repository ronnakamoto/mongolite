use chrono::{DateTime, Utc};
use eframe::egui;
use egui::{Color32, RichText};
use futures_util::TryStreamExt;
use mongodb::{
    bson::{self, doc, Document},
    options::FindOptions,
    Client,
};
use rusqlite::{Connection, Result as SqliteResult};
use serde_json::Value;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::sync::{mpsc, Mutex};

const OUTER_PADDING: f32 = 20.0;
const INNER_PADDING: f32 = 10.0;
const ITEM_SPACING: f32 = 8.0;
const SECTION_SPACING: f32 = 10.0;
const ROUNDED_CORNERS: f32 = 8.0;
const CONTROL_HEIGHT: f32 = 30.0;

#[derive(Clone, PartialEq)]
enum ConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
}

enum Message {
    Connected(Client),
    ConnectionFailed(String),
    DatabasesListed(Vec<String>),
    CollectionsListed(Vec<String>),
    QueryExecuted(Vec<Document>),
    Error(String),
}

#[derive(PartialEq)]
enum ViewMode {
    Table,
    Json,
}

#[derive(Clone, PartialEq)]
enum LeftPanelTab {
    QueryBuilder,
    History,
}

struct QueryHistoryEntry {
    id: i64,
    query: String,
    projection: String,
    sort: String,
    timestamp: i64,
    database: String,
    collection: String,
}

struct QueryHistory {
    conn: Connection,
}

impl QueryHistory {
    fn new() -> SqliteResult<Self> {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("query_history.db");
        println!("Opening database at: {:?}", path);
        let conn = Connection::open(path)?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS query_history (
                id INTEGER PRIMARY KEY,
                database TEXT NOT NULL,
                collection TEXT NOT NULL,
                query TEXT NOT NULL,
                projection TEXT NOT NULL,
                sort TEXT NOT NULL,
                timestamp INTEGER NOT NULL
            )",
            [],
        )?;

        println!("Query history table created");

        Ok(QueryHistory { conn })
    }

    fn add_query(&self, database: &str, collection: &str, query: &str, projection: &str, sort: &str) -> SqliteResult<()> {
        let timestamp = Utc::now().timestamp();
        self.conn.execute(
            "INSERT INTO query_history (database, collection, query, projection, sort, timestamp) VALUES (?, ?, ?, ?, ?, ?)",
            [database, collection, query, projection, sort, &timestamp.to_string()],
        )?;
        println!("Query added to history: {}", query);
        Ok(())
    }

    fn get_queries(&self, database: &str, collection: &str,limit: i64) -> SqliteResult<Vec<QueryHistoryEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, database, collection, query, IFNULL(projection, '') as projection, IFNULL(sort, '') as sort, timestamp 
             FROM query_history WHERE database = ? AND collection = ? ORDER BY timestamp DESC LIMIT ?"
        )?;
        let query_iter = stmt.query_map([database, collection, &limit.to_string()], |row| {
            Ok(QueryHistoryEntry {
                id: row.get(0)?,
                database: row.get(1)?,
                collection: row.get(2)?,
                query: row.get(3)?,
                projection: row.get(4)?,
                sort: row.get(5)?,
                timestamp: row.get(6)?,
            })
        })?;

        let results: Vec<QueryHistoryEntry> = query_iter.collect::<Result<_, _>>()?;
        println!("Retrieved {} queries from history", results.len());
        Ok(results)
    }

    fn reset_database(&self) -> SqliteResult<()> {
        self.conn
            .execute("DROP TABLE IF EXISTS query_history", [])?;
        self.conn.execute(
            "CREATE TABLE query_history (
                id INTEGER PRIMARY KEY,
                query TEXT NOT NULL,
                projection TEXT NOT NULL,
                sort TEXT NOT NULL,
                timestamp INTEGER NOT NULL
            )",
            [],
        )?;
        println!("Database reset successfully");
        Ok(())
    }
}

struct MongoDBClient {
    runtime: Runtime,
    tx: mpsc::Sender<Message>,
    rx: Arc<Mutex<mpsc::Receiver<Message>>>,
    connection_string: String,
    connection_status: ConnectionStatus,
    client: Option<Client>,
    databases: Vec<String>,
    selected_database: String,
    previous_database: String,
    collections: Vec<String>,
    selected_collection: String,
    previous_collection: String,
    query: String,
    projection: String,
    sort: String,
    result: Vec<Document>,
    status_message: String,
    view_mode: ViewMode,
    current_left_tab: LeftPanelTab,
    query_history: QueryHistory,
    history_entries: Vec<QueryHistoryEntry>,
}

impl MongoDBClient {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let (tx, rx) = mpsc::channel(100);
        println!("Initializing MongoDBClient");
        let query_history = QueryHistory::new().expect("Failed to initialize query history");
        Self {
            runtime: Runtime::new().expect("Failed to create Tokio runtime"),
            tx,
            rx: Arc::new(Mutex::new(rx)),
            connection_string: String::new(),
            connection_status: ConnectionStatus::Disconnected,
            client: None,
            databases: Vec::new(),
            selected_database: String::new(),
            previous_database: String::new(),
            collections: Vec::new(),
            selected_collection: String::new(),
            previous_collection: String::new(),
            query: String::new(),
            projection: String::new(),
            sort: String::new(),
            result: Vec::new(),
            status_message: String::new(),
            view_mode: ViewMode::Table,
            current_left_tab: LeftPanelTab::QueryBuilder,
            query_history,
            history_entries: Vec::new(),
        }
    }

    fn connect(&self) {
        let connection_string = self.connection_string.clone();
        let tx = self.tx.clone();

        self.runtime.spawn(async move {
            match Client::with_uri_str(&connection_string).await {
                Ok(client) => {
                    tx.send(Message::Connected(client)).await.unwrap();
                }
                Err(e) => {
                    tx.send(Message::ConnectionFailed(format!(
                        "Connection error: {:?}",
                        e
                    )))
                    .await
                    .unwrap();
                }
            }
        });
    }

    fn list_databases(&self) {
        if let Some(ref client) = self.client {
            let client = client.clone();
            let tx = self.tx.clone();

            self.runtime.spawn(async move {
                match client.list_database_names(None, None).await {
                    Ok(db_names) => {
                        tx.send(Message::DatabasesListed(db_names)).await.unwrap();
                    }
                    Err(e) => {
                        tx.send(Message::Error(format!("Failed to list databases: {:?}", e)))
                            .await
                            .unwrap();
                    }
                }
            });
        }
    }

    fn list_collections(&self, database_name: String) {
        if let Some(ref client) = self.client {
            let client = client.clone();
            let tx = self.tx.clone();

            self.runtime.spawn(async move {
                let db = client.database(&database_name);
                match db.list_collection_names(None).await {
                    Ok(coll_names) => {
                        tx.send(Message::CollectionsListed(coll_names))
                            .await
                            .unwrap();
                    }
                    Err(e) => {
                        tx.send(Message::Error(format!(
                            "Failed to list collections: {:?}",
                            e
                        )))
                        .await
                        .unwrap();
                    }
                }
            });
        }
    }

    fn execute_query(&self) {
        if let Some(ref client) = self.client {
            let client = client.clone();
            let database_name = self.selected_database.clone();
            let collection_name = self.selected_collection.clone();
            let query = self.query.clone();
            let projection = self.projection.clone();
            let sort = self.sort.clone();
            let tx = self.tx.clone();

            self.runtime.spawn(async move {
                let db = client.database(&database_name);
                let collection = db.collection::<Document>(&collection_name);

                let filter = match serde_json::from_str::<Value>(&query) {
                    Ok(v) => bson::to_document(&v).unwrap_or(doc! {}),
                    Err(e) => {
                        tx.send(Message::Error(format!("Invalid query JSON: {:?}", e)))
                            .await
                            .unwrap();
                        return;
                    }
                };

                let projection = if !projection.is_empty() {
                    match serde_json::from_str::<Value>(&projection) {
                        Ok(v) => Some(bson::to_document(&v).unwrap_or(doc! {})),
                        Err(e) => {
                            tx.send(Message::Error(format!("Invalid projection JSON: {:?}", e)))
                                .await
                                .unwrap();
                            return;
                        }
                    }
                } else {
                    None
                };

                let sort = if !sort.is_empty() {
                    match serde_json::from_str::<Value>(&sort) {
                        Ok(v) => Some(bson::to_document(&v).unwrap_or(doc! {})),
                        Err(e) => {
                            tx.send(Message::Error(format!("Invalid sort JSON: {:?}", e)))
                                .await
                                .unwrap();
                            return;
                        }
                    }
                } else {
                    None
                };

                let options = FindOptions::builder()
                    .projection(projection)
                    .sort(sort)
                    .build();

                match collection.find(filter, options).await {
                    Ok(cursor) => match cursor.try_collect().await {
                        Ok(documents) => {
                            tx.send(Message::QueryExecuted(documents)).await.unwrap();
                        }
                        Err(e) => {
                            tx.send(Message::Error(format!(
                                "Failed to collect documents: {:?}",
                                e
                            )))
                            .await
                            .unwrap();
                        }
                    },
                    Err(e) => {
                        tx.send(Message::Error(format!("Query execution error: {:?}", e)))
                            .await
                            .unwrap();
                    }
                }
            });
        }
    }

    fn custom_button(
        &self,
        ui: &mut egui::Ui,
        text: &str,
        accent_color: Color32,
    ) -> egui::Response {
        ui.add_sized(
            [100.0, CONTROL_HEIGHT],
            egui::Button::new(RichText::new(text).color(Color32::WHITE).strong())
                .fill(accent_color)
                .rounding(4.0),
        )
    }

    fn ui_connection_area(&mut self, ui: &mut egui::Ui, accent_color: Color32, bg_color: Color32) {
        egui::Frame::none()
            .fill(bg_color)
            .inner_margin(INNER_PADDING)
            .rounding(ROUNDED_CORNERS)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.add_sized(
                        [100.0, CONTROL_HEIGHT],
                        egui::Label::new(RichText::new("Connection:").strong()),
                    )
                    .on_hover_text("Enter your MongoDB connection string here.");

                    ui.add_sized(
                        [ui.available_width() - 120.0, CONTROL_HEIGHT],
                        egui::TextEdit::singleline(&mut self.connection_string)
                            .hint_text("mongodb://username:password@host:port/database"),
                    )
                    .on_hover_text("Example: mongodb://username:password@host:port/database");

                    if self
                        .custom_button(
                            ui,
                            match self.connection_status {
                                ConnectionStatus::Disconnected => "Connect",
                                ConnectionStatus::Connecting => "Connecting...",
                                ConnectionStatus::Connected => "Disconnect",
                            },
                            accent_color,
                        )
                        .clicked()
                    {
                        match self.connection_status {
                            ConnectionStatus::Disconnected => {
                                self.connect();
                                self.connection_status = ConnectionStatus::Connecting;
                            }
                            ConnectionStatus::Connecting => {}
                            ConnectionStatus::Connected => {
                                self.client = None;
                                self.connection_status = ConnectionStatus::Disconnected;
                                self.databases.clear();
                                self.collections.clear();
                                self.selected_database.clear();
                                self.selected_collection.clear();
                                self.status_message = "Disconnected".to_string();
                            }
                        }
                    }
                });
            });
    }

    fn ui_database_collection_selection(
        &mut self,
        ui: &mut egui::Ui,
        accent_color: Color32,
        bg_color: Color32,
    ) {
        egui::Frame::none()
            .fill(bg_color)
            .inner_margin(INNER_PADDING)
            .rounding(ROUNDED_CORNERS)
            .show(ui, |ui| {
                let mut database_changed = false;
                let mut collection_changed = false;
                ui.horizontal(|ui| {
                    ui.add_sized(
                        [100.0, CONTROL_HEIGHT],
                        egui::Label::new(RichText::new("Database:").strong()),
                    )
                    .on_hover_text("Select the database you want to query.");

                    let mut new_database = self.selected_database.clone();
                    egui::ComboBox::from_id_source("database")
                        .selected_text(&new_database)
                        .width(200.0)
                        .show_ui(ui, |ui| {
                            for db in &self.databases {
                                if ui.selectable_value(&mut new_database, db.clone(), db).clicked() {
                                    database_changed = true;
                                }
                            }
                        });
                    if database_changed {
                        self.previous_database = self.selected_database.clone();
                        self.selected_database = new_database;
                        self.on_database_or_collection_changed();
                    }
                    if self.custom_button(ui, "Refresh", accent_color).clicked() {
                        self.list_databases();
                    }
                });
                ui.add_space(ITEM_SPACING);
                ui.horizontal(|ui| {
                    ui.add_sized(
                        [100.0, CONTROL_HEIGHT],
                        egui::Label::new(RichText::new("Collection:").strong()),
                    )
                    .on_hover_text("Select the collection you want to query.");

                    let mut new_collection = self.selected_collection.clone();
                    egui::ComboBox::from_id_source("collection")
                        .selected_text(&new_collection)
                        .width(200.0)
                        .show_ui(ui, |ui| {
                            for coll in &self.collections {
                                if ui.selectable_value(
                                    &mut new_collection,
                                    coll.clone(),
                                    coll,
                                ).clicked() {
                                    collection_changed = true;
                                }
                            }
                        });
                    if collection_changed {
                        self.previous_collection = self.selected_collection.clone();
                        self.selected_collection = new_collection;
                        self.on_database_or_collection_changed();
                    }
                    if self.custom_button(ui, "Refresh", accent_color).clicked() {
                        self.list_collections(self.selected_database.clone());
                    }
                });
            });
    }

    fn ui_query_builder(&mut self, ui: &mut egui::Ui, accent_color: Color32, bg_color: Color32) {
        egui::Frame::none()
            .fill(bg_color)
            .inner_margin(INNER_PADDING)
            .rounding(ROUNDED_CORNERS)
            .show(ui, |ui| {
                ui.heading(RichText::new("Query Builder").size(18.0));
                ui.add_space(INNER_PADDING);

                ui.vertical(|ui| {
                    ui.label(RichText::new("Query:").strong())
                        .on_hover_text("Enter the MongoDB query in JSON format.");
                    ui.add(
                        egui::TextEdit::multiline(&mut self.query)
                            .desired_width(ui.available_width())
                            .desired_rows(5)
                            .hint_text("Enter query filter, e.g., {\"name\": \"John\"}"),
                    )
                    .on_hover_text("Example: {\"name\": \"John\"}");

                    ui.add_space(ITEM_SPACING);

                    ui.label(RichText::new("Projection:").strong())
                        .on_hover_text("Specify the fields to return in the result.");
                    ui.add(
                        egui::TextEdit::multiline(&mut self.projection)
                            .desired_width(ui.available_width())
                            .desired_rows(2)
                            .hint_text("Specify fields, e.g., {\"name\": 1, \"age\": 1}"),
                    )
                    .on_hover_text("Example: {\"name\": 1, \"age\": 1}");

                    ui.add_space(ITEM_SPACING);

                    ui.label(RichText::new("Sort:").strong())
                        .on_hover_text("Define the sort order for the query results.");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.sort)
                            .desired_width(ui.available_width())
                            .hint_text("Define sort order, e.g., {\"age\": 1}"),
                    )
                    .on_hover_text("Example: {\"age\": 1}");

                    ui.add_space(INNER_PADDING);

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if self
                            .custom_button(ui, "Execute Query", accent_color)
                            .clicked()
                        {
                            self.execute_query_with_validation();
                        }
                    });
                });
            });
    }

    fn ui_results_area(&mut self, ui: &mut egui::Ui, accent_color: Color32, bg_color: Color32) {
        let available_width = ui.available_width();

        egui::Frame::none()
            .fill(bg_color)
            .inner_margin(INNER_PADDING)
            .rounding(ROUNDED_CORNERS)
            .show(ui, |ui| {
                ui.set_min_width(available_width);
                
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.heading(RichText::new("Results").size(18.0).color(accent_color));
                        ui.add_space(10.0);
                        ui.selectable_value(&mut self.view_mode, ViewMode::Table, "Table View");
                        ui.selectable_value(&mut self.view_mode, ViewMode::Json, "JSON View");
                    });
                    ui.add_space(INNER_PADDING);
    
                    if self.result.is_empty() {
                        ui.with_layout(egui::Layout::top_down_justified(egui::Align::Center), |ui| {
                            ui.add_space(20.0);
                            ui.label(RichText::new("No results to display. Execute a query to see results here.")
                                .color(Color32::GRAY)
                                .size(14.0));
                            ui.add_space(20.0);
                        });
                    } else {
                        egui::ScrollArea::vertical()
                            .max_height(ui.available_height() - 40.0)
                            .show(ui, |ui| {
                                match self.view_mode {
                                    ViewMode::Table => self.display_table_view(ui, accent_color),
                                    ViewMode::Json => self.display_json_view(ui, accent_color),
                                }
                            });
                    }
                });
            });
    }

    fn display_table_view(&self, ui: &mut egui::Ui, accent_color: Color32) {
        if let Some(first_doc) = self.result.first() {
            let headers: Vec<&str> = first_doc.keys().map(|s| s.as_str()).collect();

            // Calculate the total width of the table
            let column_width = 150.0; // Adjust this value as needed
            let table_width = column_width * headers.len() as f32;

            egui::ScrollArea::both()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    ui.set_min_width(table_width);

                    egui::Grid::new("results_table")
                        .num_columns(headers.len())
                        .striped(true)
                        .spacing([10.0, 5.0])
                        .min_col_width(column_width)
                        .show(ui, |ui| {
                            // Display headers
                            for header in &headers {
                                ui.label(RichText::new(*header).strong().color(accent_color));
                            }
                            ui.end_row();

                            // Display data
                            for doc in &self.result {
                                for header in &headers {
                                    if let Some(value) = doc.get(*header) {
                                        ui.label(value.to_string());
                                    } else {
                                        ui.label("-");
                                    }
                                }
                                ui.end_row();
                            }
                        });
                });
        }
    }

    fn display_json_view(&self, ui: &mut egui::Ui, accent_color: Color32) {
        for (i, doc) in self.result.iter().enumerate() {
            ui.push_id(i, |ui| {
                let header = egui::CollapsingHeader::new(
                    RichText::new(format!("Document {}", i + 1)).color(accent_color),
                )
                .default_open(false)
                .show(ui, |ui| {
                    let json = serde_json::to_string_pretty(doc)
                        .unwrap_or_else(|_| "Error parsing JSON".to_string());
                    ui.add(
                        egui::TextEdit::multiline(&mut json.as_str())
                            .desired_width(ui.available_width())
                            .desired_rows(10)
                            .code_editor(),
                    );
                });

                if i < self.result.len() - 1 {
                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(8.0);
                }
            });
        }
    }

    fn ui_status_bar(&self, ui: &mut egui::Ui, accent_color: Color32, bg_color: Color32) {
        egui::Frame::none()
            .fill(bg_color)
            .inner_margin(INNER_PADDING)
            .rounding(8.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.add_sized(
                        [60.0, 20.0],
                        egui::Label::new(RichText::new("Status:").strong()),
                    );
                    ui.label(RichText::new(&self.status_message).color(accent_color));
                });
            });
    }

    fn validate_connection_string(&self) -> Result<(), String> {
        if self.connection_string.is_empty() {
            return Err("Connection string cannot be empty".to_string());
        }
        if !self.connection_string.starts_with("mongodb://")
            && !self.connection_string.starts_with("mongodb+srv://")
        {
            return Err(
                "Connection string must start with 'mongodb://' or 'mongodb+srv://'".to_string(),
            );
        }
        Ok(())
    }

    fn validate_query(&self) -> Result<(), String> {
        if self.query.is_empty() {
            return Err("Query cannot be empty".to_string());
        }
        match serde_json::from_str::<Value>(&self.query) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Invalid JSON in query: {}", e)),
        }
    }

    fn validate_projection(&self) -> Result<(), String> {
        if self.projection.is_empty() {
            return Ok(());
        }
        match serde_json::from_str::<Value>(&self.projection) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Invalid JSON in projection: {}", e)),
        }
    }

    fn validate_sort(&self) -> Result<(), String> {
        if self.sort.is_empty() {
            return Ok(());
        }
        match serde_json::from_str::<Value>(&self.sort) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Invalid JSON in sort: {}", e)),
        }
    }

    fn connect_with_validation(&mut self) {
        match self.validate_connection_string() {
            Ok(_) => {
                self.connect();
                self.connection_status = ConnectionStatus::Connecting;
            }
            Err(e) => {
                self.status_message = e;
            }
        }
    }

    fn execute_query_with_validation(&mut self) {
        if let Err(e) = self.validate_query() {
            self.status_message = e;
            return;
        }
        if let Err(e) = self.validate_projection() {
            self.status_message = e;
            return;
        }
        if let Err(e) = self.validate_sort() {
            self.status_message = e;
            return;
        }

        println!("Executing query: {}", self.query);
        println!("Projection: {}", self.projection);
        println!("Sort: {}", self.sort);

        // Save query to history
        match self
            .query_history
            .add_query(&self.selected_database, &self.selected_collection,&self.query, &self.projection, &self.sort)
        {
            Ok(_) => println!("Successfully added query to history"),
            Err(e) => eprintln!("Failed to save query to history: {}", e),
        }

        // Refresh history entries
        self.refresh_history_entries();

        self.execute_query();
    }

    fn refresh_history_entries(&mut self) {
        match self.query_history.get_queries(&self.selected_database, &self.selected_collection,100) {
            Ok(entries) => {
                self.history_entries = entries;
                println!(
                    "Updated history entries: {} items",
                    self.history_entries.len()
                );
            }
            Err(e) => eprintln!("Failed to refresh history entries: {}", e),
        }
    }

    fn on_database_or_collection_changed(&mut self) {
        self.refresh_history_entries();
    }

    fn ui_history(&mut self, ui: &mut egui::Ui, accent_color: Color32, bg_color: Color32) {
        ui.heading(
            RichText::new("Query History")
                .size(24.0)
                .color(accent_color),
        );
        ui.add_space(10.0);

        if self.history_entries.is_empty() {
            ui.label(RichText::new("No history entries available.").italics());
        } else {
            egui::ScrollArea::vertical().show(ui, |ui| {
                for (index, entry) in self.history_entries.iter().enumerate().rev() {
                    let timestamp =
                        DateTime::from_timestamp(entry.timestamp, 0).unwrap();
                    let formatted_time = timestamp.format("%Y-%m-%d %H:%M:%S").to_string();

                    egui::CollapsingHeader::new(format!(
                        "Query {}: {}",
                        self.history_entries.len() - index,
                        &entry.query[..std::cmp::min(50, entry.query.len())]
                    ))
                    .id_source(entry.id)
                    .default_open(false)
                    .show(ui, |ui| {
                        ui.add_space(5.0);
                        ui.label(RichText::new("Timestamp:").strong());
                        ui.label(formatted_time);
                        ui.add_space(5.0);

                        ui.label(RichText::new("Query:").strong());
                        ui.add(
                            egui::TextEdit::multiline(&mut entry.query.as_str())
                                .desired_width(f32::INFINITY)
                                .desired_rows(2)
                                .lock_focus(true)
                                .interactive(false),
                        );

                        if !entry.projection.is_empty() {
                            ui.add_space(5.0);
                            ui.label(RichText::new("Projection:").strong());
                            ui.label(&entry.projection);
                        }

                        if !entry.sort.is_empty() {
                            ui.add_space(5.0);
                            ui.label(RichText::new("Sort:").strong());
                            ui.label(&entry.sort);
                        }

                        ui.add_space(10.0);
                        if ui.button("Load Query").clicked() {
                            self.query = entry.query.clone();
                            self.projection = entry.projection.clone();
                            self.sort = entry.sort.clone();
                            self.current_left_tab = LeftPanelTab::QueryBuilder;
                        }
                    });
                    ui.add_space(5.0);
                    ui.separator();
                }
            });
        }

        ui.add_space(10.0);
        ui.horizontal(|ui| {
            if ui.button("Refresh History").clicked() {
                if let Ok(entries) = self.query_history.get_queries(&self.selected_database, &self.selected_collection, 100) {
                    self.history_entries = entries;
                }
            }
            if ui.button("Clear History").clicked() {
                if let Err(e) = self.query_history.reset_database() {
                    eprintln!("Failed to reset database: {}", e);
                } else {
                    self.history_entries.clear();
                    println!("Database reset and history cleared");
                }
            }
        });
    }

    fn ui_layout(&mut self, ctx: &egui::Context) {
        let accent_color = Color32::from_rgb(15, 157, 88); // Google Green
        let bg_color = Color32::from_rgb(248, 249, 250); // Light Gray
        let text_color = Color32::from_rgb(60, 64, 67); // Dark Gray
        let separator_color = Color32::from_gray(200);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.style_mut().visuals.override_text_color = Some(text_color);
            ui.style_mut().spacing.item_spacing = egui::vec2(ITEM_SPACING, ITEM_SPACING);

            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add_space(OUTER_PADDING);

                // Title
                ui.heading(
                    RichText::new("Mongolite")
                        .color(accent_color)
                        .size(32.0),
                );

                ui.add_space(SECTION_SPACING);

                // Connection Area
                self.ui_connection_area(ui, accent_color, bg_color);

                ui.add_space(SECTION_SPACING);
                ui.painter().hline(
                    ui.min_rect().left()..=ui.min_rect().right(),
                    ui.cursor().min.y,
                    (1.0, separator_color),
                );
                ui.add_space(SECTION_SPACING);

                if self.connection_status == ConnectionStatus::Connected {
                    // Database and Collection Selection
                    self.ui_database_collection_selection(ui, accent_color, bg_color);

                    ui.add_space(SECTION_SPACING);
                    ui.painter().hline(
                        ui.min_rect().left()..=ui.min_rect().right(),
                        ui.cursor().min.y,
                        (1.0, separator_color),
                    );
                    ui.add_space(SECTION_SPACING);

                    // Left panel with tabs and Right panel side by side
                    ui.horizontal(|ui| {
                        let available_width = ui.available_width();
                        let left_column_width = (available_width - ITEM_SPACING) * 0.4; // 40% of available width
                        let right_column_width = (available_width - ITEM_SPACING) * 0.6; // 60% of available width

                        // Left panel with tabs
                        ui.vertical(|ui| {
                            ui.set_max_width(left_column_width);

                            egui::Frame::none()
                                .fill(bg_color)
                                .inner_margin(INNER_PADDING)
                                .rounding(ROUNDED_CORNERS)
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.selectable_value(
                                            &mut self.current_left_tab,
                                            LeftPanelTab::QueryBuilder,
                                            "Query Builder",
                                        );
                                        ui.selectable_value(
                                            &mut self.current_left_tab,
                                            LeftPanelTab::History,
                                            "History",
                                        );
                                    });

                                    ui.add_space(INNER_PADDING);

                                    match self.current_left_tab {
                                        LeftPanelTab::QueryBuilder => {
                                            self.ui_query_builder(ui, accent_color, bg_color)
                                        }
                                        LeftPanelTab::History => {
                                            self.ui_history(ui, accent_color, bg_color)
                                        }
                                    }
                                });
                        });

                        ui.add_space(ITEM_SPACING);

                        // Right panel (Results Area)
                        ui.vertical(|ui| {
                            ui.set_max_width(right_column_width);
                            self.ui_results_area(ui, accent_color, bg_color);
                        });
                    });
                }

                ui.add_space(SECTION_SPACING);

                // Status Bar
                self.ui_status_bar(ui, accent_color, bg_color);

                ui.add_space(OUTER_PADDING);
            });
        });
    }
}

impl eframe::App for MongoDBClient {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Ok(mut rx) = self.rx.try_lock() {
            while let Ok(message) = rx.try_recv() {
                match message {
                    Message::Connected(client) => {
                        self.client = Some(client);
                        self.connection_status = ConnectionStatus::Connected;
                        self.status_message = "Connected successfully".to_string();
                        self.list_databases();
                    }
                    Message::ConnectionFailed(error) => {
                        self.connection_status = ConnectionStatus::Disconnected;
                        self.status_message = error;
                    }
                    Message::DatabasesListed(db_names) => {
                        self.databases = db_names;
                        if !self.databases.is_empty() {
                            self.selected_database = self.databases[0].clone();
                            self.list_collections(self.selected_database.clone());
                        }
                        self.status_message = "Databases listed successfully".to_string();
                    }
                    Message::CollectionsListed(coll_names) => {
                        self.collections = coll_names;
                        if !self.collections.is_empty() {
                            self.selected_collection = self.collections[0].clone();
                        }
                        self.status_message = "Collections listed successfully".to_string();
                    }
                    Message::QueryExecuted(documents) => {
                        self.result = documents;
                        self.status_message = "Query executed successfully".to_string();
                    }
                    Message::Error(error) => {
                        self.status_message = error;
                    }
                }
            }
        }

        self.ui_layout(ctx);

        ctx.request_repaint();
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(800.0, 600.0)),
        ..Default::default()
    };
    eframe::run_native(
        "Mongolite",
        options,
        Box::new(|cc| Box::new(MongoDBClient::new(cc))),
    )
}
