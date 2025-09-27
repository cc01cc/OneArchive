use crate::mod_database::database::Database;
use crate::mod_database::trait_database::InitializationOperations;
use serde_json::{Map, Value};
use std::env;
use std::fs;
use std::path::Path;

// 应用设置结构
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct AppSettings {
    pub last_workspace: Option<String>,
    pub last_root: Option<i64>,       // 记住上次选择的根目录ID
    pub last_db_path: Option<String>, // 记住上次使用的数据库路径
    pub window_width: Option<u32>,
    pub window_height: Option<u32>,
    pub window_x: Option<i32>,
    pub window_y: Option<i32>,
    // 可以添加更多设置项
}

// 工作区信息结构
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct Workspace {
    pub name: String,
    pub path: String,
}

// 工作区详细配置结构
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct WorkspaceConfig {
    pub db_path: Option<String>,
    pub auto_create_db: Option<bool>,
    pub recursive_scan: Option<bool>,
    pub follow_symlinks: Option<bool>,
}

// 创建工作区的参数结构
#[derive(serde::Deserialize)]
pub struct CreateWorkspaceParams {
    pub name: String,
    pub path: String,
}

// 导入工作区的参数结构
#[derive(serde::Deserialize)]
pub struct ImportWorkspaceParams {
    pub path: String,
}

// 移除工作区的参数结构
#[derive(serde::Deserialize)]
pub struct RemoveWorkspaceParams {
    pub path: String,
}

// 获取配置中的 workspace 值
#[tauri::command]
pub fn get_config_workspace() -> Result<Vec<Workspace>, String> {
    // 获取当前可执行文件的目录
    let exe_path = env::current_exe().map_err(|e| e.to_string())?;
    let app_dir = exe_path.parent().ok_or("无法获取应用目录".to_string())?;

    // 构造.onearchive 目录路径
    let onearchive_dir = app_dir.join(".onearchive");

    // 构造配置文件路径
    let config_path = onearchive_dir.join("config.json");

    // 检查配置文件是否存在
    if !config_path.exists() {
        // 如果配置文件不存在，创建默认工作区
        create_default_workspace(&onearchive_dir)?;
    }

    // 读取并解析工作区配置
    let workspaces = read_workspaces_from_config(&config_path)?;

    // 如果没有工作区，创建默认工作区
    if workspaces.is_empty() {
        create_default_workspace(&onearchive_dir)?;
        // 重新读取配置
        return read_workspaces_from_config(&config_path);
    }

    Ok(workspaces)
}

// 加载应用设置
#[tauri::command]
pub fn load_app_settings() -> Result<AppSettings, String> {
    // 获取当前可执行文件的目录
    let exe_path = env::current_exe().map_err(|e| e.to_string())?;
    let app_dir = exe_path.parent().ok_or("无法获取应用目录".to_string())?;

    // 构造.onearchive 目录路径
    let onearchive_dir = app_dir.join(".onearchive");

    // 构造应用设置文件路径
    let settings_path = onearchive_dir.join("app_settings.json");

    // 检查设置文件是否存在
    if !settings_path.exists() {
        // 如果设置文件不存在，返回默认设置
        return Ok(AppSettings {
            last_workspace: None,
            last_root: None,
            last_db_path: None,
            window_width: None,
            window_height: None,
            window_x: None,
            window_y: None,
        });
    }

    // 读取设置文件
    let settings_content = fs::read_to_string(&settings_path).map_err(|e| e.to_string())?;

    // 解析 JSON
    let settings: AppSettings =
        serde_json::from_str(&settings_content).map_err(|e| e.to_string())?;

    Ok(settings)
}

// 保存应用设置
#[tauri::command]
pub fn save_app_settings(settings: AppSettings) -> Result<(), String> {
    // 获取当前可执行文件的目录
    let exe_path = env::current_exe().map_err(|e| e.to_string())?;
    let app_dir = exe_path.parent().ok_or("无法获取应用目录".to_string())?;

    // 构造.onearchive 目录路径
    let onearchive_dir = app_dir.join(".onearchive");

    // 创建目录（如果不存在）
    fs::create_dir_all(&onearchive_dir).map_err(|e| e.to_string())?;

    // 构造应用设置文件路径
    let settings_path = onearchive_dir.join("app_settings.json");

    // 序列化设置
    let settings_json = serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?;

    // 写入设置文件
    fs::write(&settings_path, settings_json).map_err(|e| e.to_string())?;

    Ok(())
}

// 加载工作区详细配置
#[tauri::command]
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

// 从配置文件中读取工作区列表
fn read_workspaces_from_config(config_path: &Path) -> Result<Vec<Workspace>, String> {
    // 读取配置文件
    let config_content = fs::read_to_string(config_path).map_err(|e| e.to_string())?;

    // 解析 JSON
    let config: Value = serde_json::from_str(&config_content).map_err(|e| e.to_string())?;

    // 获取 workspace 数组
    if let Some(obj) = config.as_object() {
        if let Some(workspace_array) = obj.get("workspace") {
            if let Some(array) = workspace_array.as_array() {
                let mut workspaces: Vec<Workspace> = Vec::new();
                for item in array {
                    if let (Some(name), Some(path)) = (
                        item.get("name").and_then(|v| v.as_str()),
                        item.get("path").and_then(|v| v.as_str()),
                    ) {
                        workspaces.push(Workspace {
                            name: name.to_string(),
                            path: path.to_string(),
                        });
                    }
                }
                return Ok(workspaces);
            }
        }
    }

    Ok(vec![])
}

// 创建默认工作区
fn create_default_workspace(onearchive_dir: &Path) -> Result<(), String> {
    // 构造默认工作区路径
    let default_workspace_path = onearchive_dir.join("workspace-default");

    // 创建目录（如果不存在）
    fs::create_dir_all(&default_workspace_path).map_err(|e| e.to_string())?;

    // 检查目录是否为空，如果不是则清空
    let is_empty = default_workspace_path
        .read_dir()
        .map_err(|e| format!("无法读取目录：{}", e))?
        .next()
        .is_none();

    if !is_empty {
        // 清空目录
        for entry in default_workspace_path
            .read_dir()
            .map_err(|e| e.to_string())?
        {
            let entry = entry.map_err(|e| e.to_string())?;
            if entry.file_type().map_err(|e| e.to_string())?.is_dir() {
                fs::remove_dir_all(entry.path()).map_err(|e| e.to_string())?;
            } else {
                fs::remove_file(entry.path()).map_err(|e| e.to_string())?;
            }
        }
    }

    // 构造配置文件路径
    let config_path = onearchive_dir.join("config.json");

    // 创建默认配置
    let default_config = serde_json::json!({
        "workspace": [{
            "name": "默认工作区",
            "path": default_workspace_path.to_string_lossy()
        }]
    });

    // 写入配置
    let updated_config =
        serde_json::to_string_pretty(&default_config).map_err(|e| e.to_string())?;
    fs::write(&config_path, updated_config).map_err(|e| e.to_string())?;

    // 创建并初始化 SQLite 数据库
    let db_path = default_workspace_path.join("default.sqlite");
    let database = Database::new(db_path.to_str().ok_or("路径转换失败")?)
        .map_err(|e| format!("创建数据库失败：{}", e))?;

    database
        .initialize_tables(&database.conn)
        .map_err(|e| format!("初始化数据库失败：{}", e))?;

    Ok(())
}

// 创建工作区
#[tauri::command]
pub fn create_workspace(params: CreateWorkspaceParams) -> Result<(), String> {
    let path = Path::new(&params.path);

    // 检查路径是否存在
    if !path.exists() {
        return Err("指定的路径不存在".to_string());
    }

    // 检查是否为目录
    if !path.is_dir() {
        return Err("指定的路径不是目录".to_string());
    }

    // 检查目录是否为空
    let is_empty = path
        .read_dir()
        .map_err(|e| format!("无法读取目录：{}", e))?
        .next()
        .is_none();

    if !is_empty {
        return Err("指定的目录不为空".to_string());
    }

    // 获取当前可执行文件的目录
    let exe_path = env::current_exe().map_err(|e| e.to_string())?;
    let app_dir = exe_path.parent().ok_or("无法获取应用目录".to_string())?;

    // 构造.onearchive 目录路径
    let onearchive_dir = app_dir.join(".onearchive");

    // 构造配置文件路径
    let config_path = onearchive_dir.join("config.json");

    // 读取现有配置
    let config_content = fs::read_to_string(&config_path).map_err(|e| e.to_string())?;
    let mut config: Value = serde_json::from_str(&config_content).map_err(|e| e.to_string())?;

    // 获取或创建 workspace 数组
    let workspace_array = config
        .as_object_mut()
        .ok_or("配置文件格式错误")?
        .entry("workspace")
        .or_insert_with(|| Value::Array(vec![]))
        .as_array_mut()
        .ok_or("workspace 字段不是数组类型")?;

    // 添加新的 workspace 项
    let new_workspace = serde_json::json!({
        "name": params.name,
        "path": params.path
    });
    workspace_array.push(new_workspace);

    // 写入更新后的配置
    let updated_config = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;
    fs::write(&config_path, updated_config).map_err(|e| e.to_string())?;

    // 创建并初始化 SQLite 数据库
    let db_path = path.join("default.sqlite");
    let database = Database::new(db_path.to_str().ok_or("路径转换失败")?)
        .map_err(|e| format!("创建数据库失败：{}", e))?;

    database
        .initialize_tables(&database.conn)
        .map_err(|e| format!("初始化数据库失败：{}", e))?;

    // 创建工作区配置文件并写入数据库路径
    let workspace_config_path = path.join("config.json");
    let workspace_config = serde_json::json!({
        "db_path": db_path.to_string_lossy(),
        "auto_create_db": true,
        "recursive_scan": true,
        "follow_symlinks": false
    });
    let workspace_config_str =
        serde_json::to_string_pretty(&workspace_config).map_err(|e| e.to_string())?;
    fs::write(&workspace_config_path, workspace_config_str).map_err(|e| e.to_string())?;

    Ok(())
}

// 导入工作区
#[tauri::command]
pub fn import_workspace(params: ImportWorkspaceParams) -> Result<Workspace, String> {
    let path = Path::new(&params.path);

    // 检查路径是否存在
    if !path.exists() {
        return Err("指定的路径不存在".to_string());
    }

    // 检查是否为目录
    if !path.is_dir() {
        return Err("指定的路径不是目录".to_string());
    }

    // 获取目录名作为工作区名称
    let workspace_name = path
        .file_name()
        .ok_or("无法获取目录名")?
        .to_str()
        .ok_or("目录名包含非有效 UTF-8 字符")?
        .to_string();

    // 获取当前可执行文件的目录
    let exe_path = env::current_exe().map_err(|e| e.to_string())?;
    let app_dir = exe_path.parent().ok_or("无法获取应用目录".to_string())?;

    // 构造.onearchive 目录路径
    let onearchive_dir = app_dir.join(".onearchive");

    // 构造配置文件路径
    let config_path = onearchive_dir.join("config.json");

    // 读取现有配置
    let config_content = fs::read_to_string(&config_path).map_err(|e| e.to_string())?;
    let mut config: Value = serde_json::from_str(&config_content).map_err(|e| e.to_string())?;

    // 获取或创建 workspace 数组
    let workspace_array = config
        .as_object_mut()
        .ok_or("配置文件格式错误")?
        .entry("workspace")
        .or_insert_with(|| Value::Array(vec![]))
        .as_array_mut()
        .ok_or("workspace 字段不是数组类型")?;

    // 检查是否已存在相同路径的工作区
    for workspace in workspace_array.iter() {
        if let Some(existing_path) = workspace.get("path").and_then(|v| v.as_str()) {
            if existing_path == params.path {
                return Err("该工作区已存在".to_string());
            }
        }
    }

    // 添加新的 workspace 项
    let new_workspace = serde_json::json!({
        "name": workspace_name,
        "path": params.path
    });
    workspace_array.push(new_workspace);

    // 写入更新后的配置
    let updated_config = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;
    fs::write(&config_path, updated_config).map_err(|e| e.to_string())?;

    // 返回导入的工作区信息
    Ok(Workspace {
        name: workspace_name,
        path: params.path,
    })
}

// 移除工作区（从配置中移除，但不删除实际目录）
#[tauri::command]
pub fn remove_workspace(params: RemoveWorkspaceParams) -> Result<(), String> {
    // 获取当前可执行文件的目录
    let exe_path = env::current_exe().map_err(|e| e.to_string())?;
    let app_dir = exe_path.parent().ok_or("无法获取应用目录".to_string())?;

    // 构造.onearchive 目录路径
    let onearchive_dir = app_dir.join(".onearchive");

    // 构造配置文件路径
    let config_path = onearchive_dir.join("config.json");

    // 检查配置文件是否存在
    if !config_path.exists() {
        return Err("配置文件不存在".to_string());
    }

    // 读取现有配置
    let config_content = fs::read_to_string(&config_path).map_err(|e| e.to_string())?;
    let mut config: Value = serde_json::from_str(&config_content).map_err(|e| e.to_string())?;

    // 获取 workspace 数组
    let workspace_array = config
        .as_object_mut()
        .ok_or("配置文件格式错误")?
        .entry("workspace")
        .or_insert_with(|| Value::Array(vec![]))
        .as_array_mut()
        .ok_or("workspace 字段不是数组类型")?;

    // 过滤掉要移除的工作区
    workspace_array.retain(|workspace| {
        if let Some(existing_path) = workspace.get("path").and_then(|v| v.as_str()) {
            existing_path != params.path
        } else {
            true // 保留格式不正确的工作区条目
        }
    });

    // 写入更新后的配置
    let updated_config = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;
    fs::write(&config_path, updated_config).map_err(|e| e.to_string())?;

    Ok(())
}
