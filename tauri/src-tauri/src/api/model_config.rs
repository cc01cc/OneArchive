use serde::{Deserialize, Serialize};

/// 工作区信息
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Workspace {
    pub name: String,
    pub path: String,
}

/// 最后使用设置
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LastSettings {
    pub last_workspace: String,
    pub last_root: Option<i64>,
    pub last_db_path: Option<String>,
    pub window_width: i32,
    pub window_height: i32,
    pub window_x: i32,
    pub window_y: i32,
}

/// 应用配置
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AppConfig {
    pub workspace: Vec<Workspace>,
    pub last_settings: LastSettings,
}

/// 窗口位置参数
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WindowPosition {
    pub x: i32,
    pub y: i32,
}
