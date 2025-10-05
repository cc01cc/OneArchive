use crate::mod_archive::impl_archive::ArchiveServices;
use crate::mod_archive::trait_archive::{ArchiveContext, ArchiveOperations};
use crate::mod_archive::utils_archive::ArchiveProgress;
use crate::mod_database::impl_database::Database;
use crate::mod_database::trait_database::InitializationOperations;
use crate::mod_disaster_recovery::core_disaster_recovery::model_disaster_recovery::{
    GenerateRecoveryResult, RecoveryConfig, TauriRecoveryConfig,
};
use crate::mod_disaster_recovery::service_disaster_recovery::service_encoder::GroupEncoder;
use crate::mod_extract::impl_extract::ExtractService;
use crate::mod_extract::model_extract::{ExtractProgress, ExtractTask};
use crate::mod_extract::trait_extract::ExtractOperations;
use crate::mod_scan::impl_scan::ScanServices;
use crate::mod_scan::model_scan::ScanProgress;
use crate::mod_scan::trait_scan::DirectoryScanOperations;
use crate::utils::ProgressEvent;
use std::path::Path;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};

#[tauri::command]
pub fn generate_recovery_files(
    config: TauriRecoveryConfig, archive_groups: Vec<Vec<i64>>, db_path: String,
) -> Result<Vec<GenerateRecoveryResult>, String> {
    // 创建数据库连接
    let database = Database::new(&db_path).map_err(|e| e.to_string())?;

    // 转换配置格式
    let recovery_config = RecoveryConfig {
        data_shards: config.data_shards,
        parity_shards: config.parity_shards,
        storage_dir: config.storage_dir,
        prefix: config.prefix,
        store_data_shards: config.store_data_shards,
    };

    // 创建编码器
    let encoder = GroupEncoder::new(recovery_config);

    let mut results = Vec::new();

    // 为每个分组生成灾备文件
    for (index, archive_ids) in archive_groups.iter().enumerate() {
        if archive_ids.is_empty() {
            return Err("归档文件组不能为空".to_string());
        }

        // 确保每个组的文件数量与 data_shards 一致
        if archive_ids.len() != config.data_shards {
            return Err(format!(
                "第{}组归档文件数量 ({}) 与数据分片数 ({}) 不匹配",
                index + 1,
                archive_ids.len(),
                config.data_shards
            ));
        }

        // 生成灾备组
        let group_id = encoder
            .create_recovery_group_and_save_with_archives(&database, archive_ids.clone())
            .map_err(|e| format!("生成第{}组灾备文件失败：{}", index + 1, e))?;

        results.push(GenerateRecoveryResult {
            group_id: group_id.to_string(),
            data_shards_count: config.data_shards,
            parity_shards_count: config.parity_shards,
            message: format!("第{}组灾备文件生成成功", index + 1),
        });
    }

    Ok(results)
}

#[tauri::command]
pub async fn scan_root_with_progress(
    app_handle: AppHandle, root_path: String, db_path: String,
) -> Result<(), String> {
    // 创建数据库连接
    let database = Database::new(&db_path).map_err(|e| e.to_string())?;

    // 初始化数据库表
    database.initialize_tables(&database.conn).map_err(|e| e.to_string())?;

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
pub async fn extract_archive(
    app_handle: AppHandle, root_id: i64, target_path: String, db_path: String,
) -> Result<ExtractProgress, String> {
    // 创建数据库连接
    let database = Database::new(&db_path).map_err(|e| e.to_string())?;

    // 初始化数据库表
    database.initialize_tables(&database.conn).map_err(|e| e.to_string())?;

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

#[tauri::command]
pub async fn archive_files(
    app_handle: AppHandle, root_dir: String, archive_dir: String, archive_prefix: String,
    db_path: String, archive_limit_size: i64,
) -> Result<String, String> {
    // 创建数据库连接
    let database = Database::new(&db_path).map_err(|e| e.to_string())?;

    // 初始化数据库表
    database.initialize_tables(&database.conn).map_err(|e| e.to_string())?;

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
    let mut context =
        ArchiveContext::new(Arc::new(database), archive_prefix, archive_dir, archive_limit_size);

    // 创建归档服务实例
    let archive_service = ArchiveServices;

    // 创建归档进度回调函数
    let archive_callback = move |progress: ProgressEvent<ArchiveProgress>| {
        let _ = app_handle.emit("archive-progress", progress);
    };

    // 执行归档操作
    archive_service
        .archive(&root_dir, &mut context, Some(archive_callback))
        .map_err(|e| format!("归档失败：{}", e))?;

    Ok("归档完成".to_string())
}
