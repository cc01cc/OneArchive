pub mod mod_archive;
pub mod mod_database;
pub mod mod_extract;
pub mod mod_scan;

use mod_scan::impl_scan::ScanServices;
use mod_scan::trait_scan::DirectoryScanOperations;
use mod_database::database::Database;
use mod_database::trait_database::{InitializationOperations};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter};

use crate::mod_scan::model_scan::ScanProgress;

// Using Mutex to wrap database connection for use in Tauri state
// Using HashMap to store multiple database connections, key is connection name
struct DatabaseState {
    connections: Mutex<HashMap<String, Database>>,
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// 删除 scan_root 方法，只保留带进度回调的版本
#[tauri::command]
async fn scan_root_with_progress(
    app_handle: AppHandle,
    root_path: String,
    db_path: String,
) -> Result<(), String> {
    // 创建数据库连接
    let database = Database::new(&db_path).map_err(|e| e.to_string())?;

    // 初始化数据库表
    database
        .initialize_tables(&database.conn)
        .map_err(|e| e.to_string())?;

    // 获取要扫描的根目录路径
    let root_path_obj = Path::new(&root_path);

    // 创建归档服务实例
    let scan_service = ScanServices::new();

    // 创建进度回调函数
    let callback = move |progress: ScanProgress| {
        let _ = app_handle.emit("scan-progress", progress);
    };

    // 调用带进度回调的扫描方法
    scan_service
        .scan_and_save_directory_with_events(
            root_path_obj,
            &database,
            Some(callback),
        )
        .map_err(|e| format!("扫描目录失败：{}", e))?;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(DatabaseState {
            connections: Mutex::new(HashMap::new()),
        })
        .invoke_handler(tauri::generate_handler![
            greet, 
            scan_root_with_progress
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

