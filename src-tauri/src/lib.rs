mod commands;
pub mod pipeline;
mod store;

use rusqlite::Connection;
use std::sync::Mutex;
use tauri::Manager;

pub struct AppState {
    pub db: Mutex<Connection>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let db_path = app
                .path()
                .app_data_dir()
                .expect("failed to get app data dir")
                .join("conversations.db");
            let conn = Connection::open(&db_path)
                .expect("failed to open database");
            store::db::init_schema(&conn)
                .expect("failed to initialize schema");
            app.manage(AppState { db: Mutex::new(conn) });
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::ingest::parse_zip,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
