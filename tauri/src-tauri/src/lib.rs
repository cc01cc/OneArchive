// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
pub mod mod_archive;
pub mod mod_database;
pub mod mod_extract;

use mod_database::database::Database;
use mod_database::traits::{InitializationOperations, RootOperations};
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::State;

// Using Mutex to wrap database connection for use in Tauri state
// Using HashMap to store multiple database connections, key is connection name
struct DatabaseState {
    connections: Mutex<HashMap<String, Database>>,
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn init_database(db_state: State<DatabaseState>, db_path: &str, name: &str) -> Result<(), String> {
    let db = Database::new(db_path).map_err(|e| e.to_string())?;
    let mut connections = db_state.connections.lock().unwrap();
    connections.insert(name.to_string(), db);
    Ok(())
}

#[tauri::command]
fn add_root_directory(
    db_state: State<DatabaseState>,
    connection_name: &str,
    root_path: &str,
    root_name: &str,
    status: &str,
) -> Result<i64, String> {
    let connections = db_state.connections.lock().unwrap();
    let db = connections
        .get(connection_name)
        .ok_or("Database connection not found")?;
    // 使用完全限定语法调用 trait 方法
    <Database as RootOperations>::add_root_directory(db, root_path, root_name, status)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn scan_directory(
    db_state: State<DatabaseState>,
    connection_name: &str,
    path: &str,
) -> Result<(), String> {
    let connections = db_state.connections.lock().unwrap();
    let db = connections
        .get(connection_name)
        .ok_or("Database connection not found")?;

    // 使用完全限定语法调用 trait 方法
    <Database as InitializationOperations>::initialize_tables(db, &db.conn)
        .map_err(|e| e.to_string())?;

    // 这里应该添加实际的扫描逻辑
    // 暂时只做示例
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(DatabaseState {
            connections: Mutex::new(HashMap::new()),
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            init_database,
            add_root_directory,
            scan_directory
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
