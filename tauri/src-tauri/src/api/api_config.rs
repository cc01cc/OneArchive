// 重新导出配置相关类型
pub use crate::api::config::{
    AppConfig, LastSettings, Workspace, WorkspaceConfig
};

// 重新导出工作区相关类型
pub use crate::api::workspace::{
    CreateWorkspaceParams, ImportWorkspaceParams, RemoveWorkspaceParams
};

// =============================================================================
// Tauri API 接口
// =============================================================================

// 加载应用配置（统一 API）
#[tauri::command]
pub fn load_app_config_command() -> Result<AppConfig, String> {
    let mut config = crate::api::config::load_app_config()?;

    // 如果没有工作区，创建默认工作区
    if config.workspace.is_empty() {
        // 获取当前可执行文件的目录
        let exe_path = std::env::current_exe().map_err(|e| e.to_string())?;
        let app_dir = exe_path.parent().ok_or("无法获取应用目录".to_string())?;

        // 构造.onearchive 目录路径
        let onearchive_dir = app_dir.join(".onearchive");

        // 创建默认工作区
        crate::api::workspace::create_default_workspace(&onearchive_dir)?;

        // 重新加载配置
        config = crate::api::config::load_app_config()?;
    }

    Ok(config)
}

// 保存应用设置
#[tauri::command]
pub fn save_app_settings(settings: LastSettings) -> Result<(), String> {
    let mut config = crate::api::config::load_app_config()?;
    config.last_settings = settings;
    crate::api::config::save_app_config(&config)
}

// 加载工作区详细配置
#[tauri::command]
pub fn load_workspace_config(workspace_path: &str) -> Result<WorkspaceConfig, String> {
    crate::api::config::load_workspace_config(workspace_path)
}

// 创建工作区
#[tauri::command]
pub fn create_workspace(params: CreateWorkspaceParams) -> Result<(), String> {
    crate::api::workspace::create_workspace(params)
}

// 导入工作区
#[tauri::command]
pub fn import_workspace(params: ImportWorkspaceParams) -> Result<Workspace, String> {
    crate::api::workspace::import_workspace(params)
}

// 移除工作区（从配置中移除，但不删除实际目录）
#[tauri::command]
pub fn remove_workspace(params: RemoveWorkspaceParams) -> Result<(), String> {
    crate::api::workspace::remove_workspace(params)
}

// 获取当前窗口位置
#[tauri::command]
pub async fn get_window_position(window: tauri::Window) -> Result<(i32, i32), String> {
    let position = window.outer_position().map_err(|e| format!("获取窗口位置失败：{}", e))?;

    Ok((position.x, position.y))
}

// 设置窗口位置
#[tauri::command]
pub async fn set_window_position(window: tauri::Window, x: i32, y: i32) -> Result<(), String> {
    window
        .set_position(tauri::PhysicalPosition { x, y })
        .map_err(|e| format!("设置窗口位置失败：{}", e))
}
