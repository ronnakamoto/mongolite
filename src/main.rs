use crate::models::MongoDBClient;

mod components;
mod models;
mod services;
mod theme;
mod utils;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Mongolite",
        options,
        Box::new(|cc| Ok(Box::new(MongoDBClient::new(cc)))),
    )
}
