pub mod config;
pub mod mod_archive;
pub mod mod_database;
pub mod mod_extract;
pub mod mod_scan;

use mod_database::database::Database;
use mod_database::trait_database::InitializationOperations;
use mod_scan::impl_scan::ScanServices;
use mod_scan::trait_scan::DirectoryScanOperations;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter};

use crate::mod_archive::trait_archive::{ArchiveOperations, ArchiveProgress};
use crate::mod_database::schema::{InfoDirectory, InfoFile, InfoRoot, ViewFile};
use crate::mod_database::trait_database::{
    DirectoryOperations, FileOperations, RootOperations, ViewOperations,
};
use crate::mod_extract::trait_extract::ExtractOperations;
use crate::mod_scan::model_scan::ScanProgress;
use std::env;
use std::fs;
use std::io::Write;

use crate::mod_archive::impl_archive::ArchiveServices;
use crate::mod_archive::trait_archive::ArchiveContext;
use crate::mod_extract::impl_extract::ExtractService;
use crate::mod_extract::model_extract::{ExtractProgress, ExtractTask};

// Using Mutex to wrap database connection for use in Tauri state
// Using HashMap to store multiple database connections, key is connection name
struct DatabaseState {
    connections: Mutex<HashMap<String, Database>>,
}
// 工作区信息结构
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Workspace {
    pub name: String,
    pub path: String,
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}
#[tauri::command]
fn get_files_by_root_id(db_path: String, root_id: i64) -> Result<Vec<InfoFile>, String> {
    // 创建数据库连接
    let database = Database::new(&db_path).map_err(|e| e.to_string())?;

    // 查询指定根目录下的所有文件
    let files = database
        .find_files_by_status_and_root_id(root_id, None)
        .map_err(|e| format!("查询文件失败：{}", e))?;

    Ok(files)
}

#[tauri::command]
fn get_directory_by_id(db_path: String, directory_id: i64) -> Result<InfoDirectory, String> {
    // 创建数据库连接
    let database = Database::new(&db_path).map_err(|e| e.to_string())?;

    // 查询指定 ID 的目录
    let directory = database
        .find_directory_by_id(directory_id)
        .map_err(|e| format!("查询目录失败：{}", e))?;

    match directory {
        Some(dir) => Ok(dir),
        None => Err("未找到指定目录".to_string()),
    }
}

#[tauri::command]
fn get_child_directories(
    db_path: String,
    root_id: i64,
    parent_path: String,
) -> Result<Vec<InfoDirectory>, String> {
    // 创建数据库连接
    let database = Database::new(&db_path).map_err(|e| e.to_string())?;

    // 查询指定目录下的直接子目录
    let directories = database
        .find_child_directories(root_id, &parent_path)
        .map_err(|e| format!("查询子目录失败：{}", e))?;

    Ok(directories)
}

#[tauri::command]
fn get_files_by_directory_id(db_path: String, directory_id: i64) -> Result<Vec<ViewFile>, String> {
    // 创建数据库连接
    let database = Database::new(&db_path).map_err(|e| e.to_string())?;

    // 查询指定目录下的所有文件
    let files = database
        .find_view_files_by_directory_id(directory_id)
        .map_err(|e| format!("查询文件失败：{}", e))?;

    Ok(files)
}
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
        .scan_and_save_directory_with_events(root_path_obj, &database, Some(callback))
        .map_err(|e| format!("扫描目录失败：{}", e))?;

    Ok(())
}

#[tauri::command]
fn get_all_roots(db_path: String) -> Result<Vec<InfoRoot>, String> {
    // 创建数据库连接
    let database = Database::new(&db_path).map_err(|e| e.to_string())?;

    // 查询所有根目录
    let roots = database
        .find_all_root_info()
        .map_err(|e| format!("查询根目录失败：{}", e))?;

    Ok(roots)
}

#[tauri::command]
fn get_directories_by_root_id(db_path: String, root_id: i64) -> Result<Vec<InfoDirectory>, String> {
    // 创建数据库连接
    let database = Database::new(&db_path).map_err(|e| e.to_string())?;

    // 查询指定根目录下的所有目录
    let directories = database
        .find_directories_by_status_and_root_id(root_id, None)
        .map_err(|e| format!("查询目录失败：{}", e))?;

    Ok(directories)
}

// 添加解档命令函数
#[tauri::command]
async fn extract_archive(
    app_handle: AppHandle,
    root_id: i64,
    target_path: String,
    db_path: String,
) -> Result<ExtractProgress, String> {
    // 创建数据库连接
    let database = Database::new(&db_path).map_err(|e| e.to_string())?;

    // 初始化数据库表
    database
        .initialize_tables(&database.conn)
        .map_err(|e| e.to_string())?;

    // 创建解档任务
    let task = ExtractTask {
        root_id,
        target_path,
        overwrite: false, // 默认不覆盖
    };

    // 创建解档服务实例
    let mut extract_service = ExtractService::new();

    // 创建进度回调函数
    let callback = move |progress: ExtractProgress| {
        let _ = app_handle.emit("extract-progress", progress);
    };

    // 执行解档操作
    let progress = extract_service
        .extract_archive(&task, &database, Some(callback))
        .map_err(|e| format!("解档失败：{}", e))?;

    Ok(progress)
}
// TODO 创建初始化引导窗口
// 初始化.onearchive 目录
fn initialize_onearchive_dir() -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    // 获取当前可执行文件的目录
    let exe_path = env::current_exe()?;
    let app_dir = exe_path.parent().ok_or("无法获取应用目录")?;

    // 构造.onearchive 目录路径
    let onearchive_dir = app_dir.join(".onearchive");

    // 创建目录（如果不存在，即不覆盖）
    fs::create_dir_all(&onearchive_dir)?;

    Ok(onearchive_dir)
}

// 添加归档命令函数
#[tauri::command]
async fn archive_files(
    app_handle: AppHandle,
    root_dir: String,
    archive_dir: String,
    archive_prefix: String,
    db_path: String,
    archive_limit_size: i64,
) -> Result<String, String> {
    // 创建数据库连接
    let database = Database::new(&db_path).map_err(|e| e.to_string())?;

    // 初始化数据库表
    database
        .initialize_tables(&database.conn)
        .map_err(|e| e.to_string())?;

    // 获取要扫描的根目录路径
    let root_path_obj = Path::new(&root_dir);

    // 创建扫描服务实例
    let scan_service = ScanServices::new();

    // 创建扫描进度回调函数
    let scan_callback = {
        let app_handle = app_handle.clone();
        move |progress: ScanProgress| {
            let _ = app_handle.emit("scan-progress", progress);
        }
    };

    // 先执行扫描操作
    scan_service
        .scan_and_save_directory_with_events(root_path_obj, &database, Some(scan_callback))
        .map_err(|e| format!("扫描目录失败：{}", e))?;

    // 创建归档上下文
    let mut context = ArchiveContext::new(archive_prefix, archive_dir, archive_limit_size);

    // 创建归档服务实例
    let archive_service = ArchiveServices::default();

    // 创建归档进度回调函数
    let archive_callback = move |progress: ArchiveProgress| {
        let _ = app_handle.emit("archive-progress", progress);
    };

    // 执行归档操作
    archive_service
        .archive_file_in_db(&root_dir, &mut context, &database, Some(archive_callback))
        .map_err(|e| format!("归档失败：{}", e))?;

    Ok("归档完成".to_string())
}
// 初始化配置文件
fn initialize_config_file(
    onearchive_dir: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = onearchive_dir.join("config.json");

    // 如果配置文件已存在，则不覆盖
    if config_path.exists() {
        return Ok(());
    }

    // 创建空的配置文件
    let mut file = fs::File::create(&config_path)?;
    file.write_all(b"{}")?;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 在应用启动时初始化.onearchive 目录
    match initialize_onearchive_dir() {
        Ok(onearchive_dir) => {
            // 初始化配置文件
            if let Err(e) = initialize_config_file(&onearchive_dir) {
                eprintln!("初始化配置文件失败：{}", e);
            }
        }
        Err(e) => {
            eprintln!("初始化.onearchive 目录失败：{}", e);
        }
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(DatabaseState {
            connections: Mutex::new(HashMap::new()),
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            scan_root_with_progress,
            archive_files,
            extract_archive,
            get_all_roots,
            get_files_by_root_id,
            get_files_by_directory_id,
            get_directory_by_id,
            get_child_directories,
            get_directories_by_root_id,
            get_files_by_root_id,
            config::get_config_workspace,
            config::create_workspace,
            config::load_workspace_config,
            config::import_workspace,
            config::remove_workspace
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
