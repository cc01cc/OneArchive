use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;

// =============================================================================
// 应用级配置结构（存储在 .onearchive/config.json）
// =============================================================================

// 最后使用设置结构
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LastSettings {
    pub last_workspace: String,
    pub last_root: Option<i64>,       // 记住上次选择的根目录 ID
    pub last_db_path: Option<String>, // 记住上次使用的数据库路径
    pub window_width: u32,
    pub window_height: u32,
    pub window_x: i32,
    pub window_y: i32,
}

// 工作区信息结构（应用级别的引用）
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Workspace {
    pub name: String,
    pub path: String,
}

// 应用配置结构（统一配置）
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppConfig {
    pub workspace: Vec<Workspace>,
    pub last_settings: LastSettings,
}

// =============================================================================
// 工作区级配置结构（存储在工作区目录下的 config.json）
// =============================================================================

// 工作区详细配置结构
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WorkspaceConfig {
    pub db_path: Option<String>,
    pub auto_create_db: Option<bool>,
    pub recursive_scan: Option<bool>,
    pub follow_symlinks: Option<bool>,
}

// =============================================================================
// 配置管理函数
// =============================================================================

/// 获取默认配置
fn get_default_config() -> Result<AppConfig, String> {
    // 获取应用目录
    let exe_path = env::current_exe().map_err(|e| e.to_string())?;
    let app_dir = exe_path.parent().ok_or("无法获取应用目录".to_string())?;

    // 构造.onearchive 目录路径
    let onearchive_dir = app_dir.join(".onearchive");

    // 默认工作区路径
    let default_workspace_path = onearchive_dir.join("workspace").join("default");
    let default_workspace = default_workspace_path.to_string_lossy().to_string();

    Ok(AppConfig {
        workspace: vec![],
        last_settings: LastSettings {
            last_workspace: default_workspace,
            last_root: None,
            last_db_path: None,
            window_width: 1000,
            window_height: 680,
            window_x: 100,
            window_y: 100,
        },
    })
}

/// 合并配置，确保所有必填字段都有默认值
/// 注意：由于结构体字段现在都是必需的，这个函数主要用于未来扩展
fn merge_with_defaults(config: AppConfig) -> Result<AppConfig, String> {
    // 目前所有必填字段都应该在反序列化时就有值
    // 如果将来有可选字段需要默认值，可以在这里添加逻辑
    Ok(config)
}

/// 加载应用配置
pub fn load_app_config() -> Result<AppConfig, String> {
    // 获取当前可执行文件的目录
    let exe_path = env::current_exe().map_err(|e| e.to_string())?;
    let app_dir = exe_path.parent().ok_or("无法获取应用目录".to_string())?;

    // 构造.onearchive 目录路径
    let onearchive_dir = app_dir.join(".onearchive");

    // 构造配置文件路径
    let config_path = onearchive_dir.join("config.json");
    log::info!("配置文件路径：{}", config_path.display());

    // 检查配置文件是否存在
    if !config_path.exists() {
        log::info!("配置文件不存在，创建默认配置...");
        // 如果配置文件不存在，创建默认配置并保存
        let default_config = get_default_config()?;
        save_app_config(&default_config)?;
        return Ok(default_config);
    }

    // 读取配置文件
    let config_content = fs::read_to_string(&config_path).map_err(|e| e.to_string())?;

    // 如果配置文件为空或只包含 {}，使用默认配置
    if config_content.trim().is_empty() || config_content.trim() == "{}" {
        log::info!("配置文件为空，使用默认配置...");
        let default_config = get_default_config()?;
        save_app_config(&default_config)?;
        return Ok(default_config);
    }

    // 尝试解析配置，如果失败则使用默认配置
    let config: AppConfig = match serde_json::from_str(&config_content) {
        Ok(config) => config,
        Err(e) => {
            log::warn!("配置文件解析失败：{}，使用默认配置", e);
            let default_config = get_default_config()?;
            save_app_config(&default_config)?;
            return Ok(default_config);
        }
    };

    // 合并默认值，确保所有必填字段都有值
    let merged_config = merge_with_defaults(config)?;

    // 由于所有必填字段现在都是必需的且总是有默认值，这里不再需要检查变化
    // 如果将来添加新的可选字段，可以在这里添加保存逻辑

    log::info!("加载应用配置成功：{:?}", merged_config);
    Ok(merged_config)
}

/// 保存应用配置
pub fn save_app_config(config: &AppConfig) -> Result<(), String> {
    // 获取当前可执行文件的目录
    let exe_path = env::current_exe().map_err(|e| e.to_string())?;
    let app_dir = exe_path.parent().ok_or("无法获取应用目录".to_string())?;

    // 构造.onearchive 目录路径
    let onearchive_dir = app_dir.join(".onearchive");

    // 创建目录（如果不存在）
    fs::create_dir_all(&onearchive_dir).map_err(|e| e.to_string())?;

    // 构造配置文件路径
    let config_path = onearchive_dir.join("config.json");

    // 序列化并写入配置文件
    let config_json = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    fs::write(&config_path, config_json).map_err(|e| e.to_string())?;

    Ok(())
}

/// 加载工作区详细配置
pub fn load_workspace_config(workspace_path: &str) -> Result<WorkspaceConfig, String> {
    let workspace_dir = Path::new(workspace_path);
    let config_path = workspace_dir.join("config.json");

    // 检查配置文件是否存在
    if !config_path.exists() {
        let db_path = workspace_dir.join("default.sqlite");
        let config = WorkspaceConfig {
            db_path: Some(db_path.to_string_lossy().to_string()),
            auto_create_db: Some(true),
            recursive_scan: Some(true),
            follow_symlinks: Some(false),
        };

        // 同时将默认配置写入配置文件
        // TODO 是否需要写入默认配置，这个需要更进一步考虑
        let config_json = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;
        fs::write(&config_path, config_json).map_err(|e| e.to_string())?;

        return Ok(config);
    }

    // 读取配置文件
    let config_content = fs::read_to_string(&config_path).map_err(|e| e.to_string())?;

    // 解析 JSON
    let config: WorkspaceConfig =
        serde_json::from_str(&config_content).map_err(|e| e.to_string())?;

    Ok(config)
}

// =============================================================================
// 应用初始化函数
// =============================================================================

/// 初始化配置文件
pub fn initialize_config_file(
    onearchive_dir: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = onearchive_dir.join("config.json");

    // 如果配置文件已存在，则不覆盖
    if config_path.exists() {
        return Ok(());
    }

    // 创建包含默认值的配置文件
    let default_config = get_default_config()?;

    let config_json = serde_json::to_string_pretty(&default_config)?;
    fs::write(&config_path, config_json)?;

    Ok(())
}

/// 初始化.onearchive 目录
pub fn initialize_onearchive_dir() -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    // 获取当前可执行文件的目录
    let exe_path = env::current_exe()?;
    let app_dir = exe_path.parent().ok_or("无法获取应用目录")?;

    // 构造.onearchive 目录路径
    let onearchive_dir = app_dir.join(".onearchive");

    // 创建目录（如果不存在，即不覆盖）
    fs::create_dir_all(&onearchive_dir)?;

    Ok(onearchive_dir)
}
