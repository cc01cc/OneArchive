pub mod api;
pub mod mod_archive;
pub mod mod_database;
pub mod mod_disaster_recovery;
pub mod mod_extract;
pub mod mod_scan;
pub mod utils;

use std::env;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 在应用启动时初始化.onearchive 目录
    match api::config::initialize_onearchive_dir() {
        Ok(onearchive_dir) => {
            // 初始化配置文件
            if let Err(e) = api::config::initialize_config_file(&onearchive_dir) {
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
        .plugin(tauri_plugin_log::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            api::api_operations::scan_root_with_progress,
            api::api_operations::archive_files,
            api::api_operations::extract_archive,
            api::api_database::get_all_roots,
            api::api_database::get_files_by_root_id,
            api::api_database::get_files_by_directory_id,
            api::api_database::get_directory_by_id,
            api::api_database::get_child_directories,
            api::api_database::get_directories_by_root_id,
            api::api_database::get_files_by_root_id,
            api::api_operations::generate_recovery_files,
            api::api_database::get_archives_by_root_id,
            api::api_config::load_app_config_command,
            api::api_config::create_workspace,
            api::api_config::load_workspace_config,
            api::api_config::import_workspace,
            api::api_config::remove_workspace,
            api::api_config::save_app_settings,
            api::api_config::get_window_position,
            api::api_config::set_window_position
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
