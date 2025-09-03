// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
pub mod database;
pub mod archive;



use database::Database;
use std::sync::Mutex;
use tauri::{State, Emitter, AppHandle, Manager};
use std::collections::HashMap;
use std::path::Path;

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
) -> Result<i64, String> {
    let connections = db_state.connections.lock().unwrap();
    let db = connections.get(connection_name).ok_or("Database connection not found")?;
    db.add_root_directory(root_path, root_name).map_err(|e| e.to_string())
}

#[tauri::command]
fn scan_directory(
    db_state: State<DatabaseState>,
    connection_name: &str,
    path: &str,
) -> Result<(), String> {
    let connections = db_state.connections.lock().unwrap();
    let db = connections.get(connection_name).ok_or("Database connection not found")?;
    let path = Path::new(path);
    
    archive::services::scan_and_save_directory(path, db)
        .map_err(|e| format!("扫描目录失败: {}", e))
}

#[tauri::command]
async fn scan_directory_with_progress(
    app_handle: tauri::AppHandle,
    db_state: State<'_, DatabaseState>,
    connection_name: &str,
    path: &str,
) -> Result<(), String> {
    let connections = db_state.connections.lock().unwrap();
    let db = connections.get(connection_name).ok_or("Database connection not found")?;
    let path = Path::new(path);
    
    let callback = move |progress: archive::services::ScanProgress| {
        let _ = app_handle.emit("scan-progress", progress);
    };
    
    // 在实际应用中，这里可能需要使用 tokio::task::spawn_blocking 来避免阻塞主线程
    archive::services::scan_and_save_directory_with_events(path, db, Some(callback))
        .map_err(|e| format!("扫描目录失败: {}", e))
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
            scan_directory,
            scan_directory_with_progress
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}