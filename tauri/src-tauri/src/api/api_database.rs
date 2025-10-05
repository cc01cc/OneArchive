use crate::mod_database::dao_database::dao_archive_metadata::ArchiveMetadataDao;
use crate::mod_database::dao_database::dao_info_directory::InfoDirectoryDao;
use crate::mod_database::dao_database::dao_info_file::InfoFileDao;
use crate::mod_database::dao_database::dao_info_root::InfoRootDao;
use crate::mod_database::dao_database::dao_view_file::ViewFileDao;
use crate::mod_database::impl_database::Database;
use crate::mod_database::schema::{ArchiveMetadata, InfoDirectory, InfoFile, InfoRoot, ViewFile};

#[tauri::command]
pub fn get_archives_by_root_id(db_path: String, root_id: i64) -> Result<Vec<ArchiveMetadata>, String> {
    // 创建数据库连接
    let database = Database::new(&db_path).map_err(|e| e.to_string())?;

    // 查询指定根目录下的所有归档文件
    // 这里我们查找状态为"ARCHIVED"的文件，这些是已经归档的文件
    // FIXME 此处状态存在问题，需要修复
    let archives = ArchiveMetadataDao::new(database.conn.clone())
        .find_by_status_and_root_id(root_id, None)
        .map_err(|e| format!("查询归档文件失败：{}", e))?;

    Ok(archives)
}

#[tauri::command]
pub fn get_files_by_root_id(db_path: String, root_id: i64) -> Result<Vec<InfoFile>, String> {
    // 创建数据库连接
    let database = Database::new(&db_path).map_err(|e| e.to_string())?;

    let file_dao = InfoFileDao::new(database.conn.clone());
    // 查询指定根目录下的所有文件
    let files = file_dao
        .find_files_by_status_and_root_id(root_id, None)
        .map_err(|e| format!("查询文件失败：{}", e))?;

    Ok(files)
}

#[tauri::command]
pub fn get_directory_by_id(db_path: String, directory_id: i64) -> Result<InfoDirectory, String> {
    // 创建数据库连接
    let database = Database::new(&db_path).map_err(|e| e.to_string())?;

    let directory_dao = InfoDirectoryDao::new(database.conn.clone());
    // 查询指定 ID 的目录
    let directory = directory_dao
        .find_directory_by_id(directory_id)
        .map_err(|e| format!("查询目录失败：{}", e))?;

    match directory {
        Some(dir) => Ok(dir),
        None => Err("未找到指定目录".to_string()),
    }
}

#[tauri::command]
pub fn get_child_directories(
    db_path: String, root_id: i64, parent_path: String,
) -> Result<Vec<InfoDirectory>, String> {
    // 创建数据库连接
    let database = Database::new(&db_path).map_err(|e| e.to_string())?;
    let directory_dao = InfoDirectoryDao::new(database.conn.clone());
    // 查询指定目录下的直接子目录
    let directories = directory_dao
        .find_child_directories(root_id, &parent_path)
        .map_err(|e| format!("查询子目录失败：{}", e))?;

    Ok(directories)
}

#[tauri::command]
pub fn get_files_by_directory_id(db_path: String, directory_id: i64) -> Result<Vec<ViewFile>, String> {
    // 创建数据库连接
    let database = Database::new(&db_path).map_err(|e| e.to_string())?;
    let view_file_dao = ViewFileDao::new(database.conn.clone());
    // 查询指定目录下的所有文件
    let files = view_file_dao
        .find_view_files_by_directory_id(directory_id)
        .map_err(|e| format!("查询文件失败：{}", e))?;

    Ok(files)
}

#[tauri::command]
pub fn get_all_roots(db_path: String) -> Result<Vec<InfoRoot>, String> {
    // 创建数据库连接
    let database = Database::new(&db_path).map_err(|e| e.to_string())?;
    let root_dao = InfoRootDao::new(database.conn.clone());

    // 查询所有根目录
    let roots = root_dao.find_all().map_err(|e| format!("查询根目录失败：{}", e))?;

    Ok(roots)
}

#[tauri::command]
pub fn get_directories_by_root_id(db_path: String, root_id: i64) -> Result<Vec<InfoDirectory>, String> {
    // 创建数据库连接
    let database = Database::new(&db_path).map_err(|e| e.to_string())?;
    let directory_dao = InfoDirectoryDao::new(database.conn.clone());

    // 查询指定根目录下的所有目录
    let directories = directory_dao
        .find_by_status_and_root_id(root_id, None)
        .map_err(|e| format!("查询目录失败：{}", e))?;

    Ok(directories)
}