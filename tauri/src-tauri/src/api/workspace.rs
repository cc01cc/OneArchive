use crate::api::config::{AppConfig, Workspace, WorkspaceConfig, load_app_config, save_app_config};
use crate::mod_database::impl_database::Database;
use crate::mod_database::trait_database::InitializationOperations;
use serde::Deserialize;
use std::fs;
use std::path::Path;

// =============================================================================
// 工作区操作参数结构
// =============================================================================

// 创建工作区的参数结构
#[derive(Deserialize)]
pub struct CreateWorkspaceParams {
    pub name: String,
    pub path: String,
}

// 导入工作区的参数结构
#[derive(Deserialize)]
pub struct ImportWorkspaceParams {
    pub path: String,
}

// 移除工作区的参数结构
#[derive(Deserialize)]
pub struct RemoveWorkspaceParams {
    pub path: String,
}

// =============================================================================
// 工作区管理函数
// =============================================================================

/// 创建默认工作区
pub fn create_default_workspace(onearchive_dir: &Path) -> Result<(), String> {
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

    // 创建默认配置
    let default_config = AppConfig {
        workspace: vec![Workspace {
            name: "默认工作区".to_string(),
            path: default_workspace_path.to_string_lossy().to_string(),
        }],
        last_settings: super::config::LastSettings {
            last_workspace: default_workspace_path.to_string_lossy().to_string(),
            last_root: None,
            last_db_path: None,
            window_width: 1000,
            window_height: 680,
            window_x: 100,
            window_y: 100,
        },
    };

    // 保存配置
    save_app_config(&default_config)?;

    // 创建并初始化 SQLite 数据库
    let db_path = default_workspace_path.join("default.sqlite");
    let database = Database::new(db_path.to_str().ok_or("路径转换失败")?)
        .map_err(|e| format!("创建数据库失败：{}", e))?;

    database
        .initialize_tables(&database.conn)
        .map_err(|e| format!("初始化数据库失败：{}", e))?;

    Ok(())
}

/// 创建工作区
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

    // 加载现有配置
    let mut config = load_app_config()?;

    // 添加新的 workspace 项
    let new_workspace = Workspace {
        name: params.name.clone(),
        path: params.path.clone(),
    };
    config.workspace.push(new_workspace);

    // 保存配置
    save_app_config(&config)?;

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

/// 导入工作区
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

    // 加载现有配置
    let mut config = load_app_config()?;

    // 检查是否已存在相同路径的工作区
    for workspace in &config.workspace {
        if workspace.path == params.path {
            return Err("该工作区已存在".to_string());
        }
    }

    // 添加新的 workspace 项
    let new_workspace = Workspace {
        name: workspace_name.clone(),
        path: params.path.clone(),
    };
    config.workspace.push(new_workspace);

    // 保存配置
    save_app_config(&config)?;

    // 返回导入的工作区信息
    Ok(Workspace {
        name: workspace_name,
        path: params.path,
    })
}

/// 移除工作区（从配置中移除，但不删除实际目录）
pub fn remove_workspace(params: RemoveWorkspaceParams) -> Result<(), String> {
    // 加载现有配置
    let mut config = load_app_config()?;

    // 过滤掉要移除的工作区
    config.workspace.retain(|workspace| workspace.path != params.path);

    // 保存配置
    save_app_config(&config)?;

    Ok(())
}
