//! ArchiveIn 功能集成测试

use log::info;
use one_archive_lib::mod_archive::impls_archive::ArchiveServices;
use one_archive_lib::mod_archive::traits::DirectoryScanOperations;
use std::fs;
use std::io::{BufWriter, Write};
use std::path::Path;
use tempfile::TempDir;

use one_archive_lib::mod_archive::impls_archive::archive_in_impl::ArchiveInServices;
use one_archive_lib::mod_archive::traits_archive::ArchiveContext;
use one_archive_lib::mod_archive::traits_archive::ArchiveOperations;
use one_archive_lib::mod_database::constants::{
    AssetStatus, DirectoryStatus, FileStatus, MapFileAssetStatus, RootStatus,
};
use one_archive_lib::mod_database::database::Database;
use one_archive_lib::mod_database::traits::{
    ArchiveAssetOperations, DirectoryOperations, FileOperations, InitializationOperations,
    MapFileAssetOperations, RootOperations, ViewOperations,
};

// 常量定义
const LARGE_FILE_SIZE: i64 = 2 * 1024 * 1024; // 2MB
const CHUNK_SIZE: i64 = 512 * 1024; // 512KB
const ASSET_SIZE: i64 = 524288; // 512KB in bytes

/// 测试设置的通用结构
struct TestEnvironment {
    temp_dir: TempDir,
    test_source_dir: std::path::PathBuf,
    test_archive_dir: std::path::PathBuf,
    test_restore_dir: std::path::PathBuf,
    database: Database,
}

impl TestEnvironment {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // 创建临时目录用于测试
        let temp_dir = TempDir::new()?;
        let test_source_dir = temp_dir.path().join("test-source");
        let test_archive_dir = temp_dir.path().join("test-archive");
        let test_restore_dir = temp_dir.path().join("test-restore");

        fs::create_dir_all(&test_source_dir)?;
        fs::create_dir_all(&test_archive_dir)?;
        fs::create_dir_all(&test_restore_dir)?;

        // 初始化数据库
        let db_file = temp_dir.path().join("test_archive.sqlite");
        let db_url = format!("file:{}", db_file.display());
        let database = Database::new(&db_url)?;
        database.initialize_tables(&database.conn)?;

        Ok(TestEnvironment {
            temp_dir,
            test_source_dir,
            test_archive_dir,
            test_restore_dir,
            database,
        })
    }

    fn create_archive_context(&self) -> ArchiveContext {
        ArchiveContext::new(
            "archive".to_string(),
            self.test_archive_dir.to_string_lossy().to_string(),
            CHUNK_SIZE,
        )
    }
}

/// 创建普通测试文件
fn create_test_files(test_source_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    fs::write(
        test_source_dir.join("test1.txt"),
        "This is test file 1 content",
    )?;
    fs::write(
        test_source_dir.join("test2.txt"),
        "This is test file 2 content",
    )?;
    Ok(())
}

/// 创建大测试文件 (2MB) - 内容相同的数据块
fn create_large_test_file_with_same_chunk(
    file_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let file = fs::File::create(file_path)?;
    let mut writer = BufWriter::new(file);

    // 创建一个大小超过存档限制的大文件 (2MB)
    let mut buffer = [0u8; 1024]; // 1KB buffer

    // 使用固定模式填充缓冲区以确保可重复性
    for i in 0..1024 {
        buffer[i] = (i % 256) as u8;
    }

    // 写入 2048 次，总共 2MB
    for _ in 0..2048 {
        writer.write_all(&buffer)?;
    }

    writer.flush()?;
    Ok(())
}

/// 创建内容不同的大测试文件 (2MB)
fn create_large_test_file_with_different_chunks(
    file_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let file = fs::File::create(file_path)?;
    let mut writer = BufWriter::new(file);

    // 创建 4 个不同的 512KB 数据块
    for chunk in 0..4 {
        let mut buffer = [0u8; 1024]; // 1KB buffer

        // 使用不同的模式填充每个块的缓冲区以确保内容不同
        for i in 0..1024 {
            buffer[i] = ((i + chunk * 1000) % 256) as u8;
        }

        // 写入 512 次，总共 512KB
        for _ in 0..512 {
            writer.write_all(&buffer)?;
        }
    }

    writer.flush()?;
    Ok(())
}

/// 验证数据库记录
fn assert_database_records(test_source_dir: &Path, database: &Database) -> Result<(), Box<dyn std::error::Error>> {
    // 验证资产表
    let assets = database.find_archive_assets_by_status(None)?;
    assert!(!assets.is_empty(), "应该创建了存档资产");

    // 验证至少有一个资产
    let first_asset = &assets[0];
    assert!(first_asset.asset_name.len() > 0, "资产应该有名称");
    assert!(first_asset.asset_size > 0, "资产大小应该大于 0");

    // 验证资产状态
    assert_eq!(
        first_asset.status,
        AssetStatus::Health,
        "资产状态应为 Health，实际为 {:?}",
        first_asset.status
    );

    // 验证文件与资源映射表
    let file_assets = database.find_map_file_asset_by_status(None)?;
    assert!(!file_assets.is_empty(), "应该创建了文件到资产的映射");

    // 验证至少有一个映射关系
    let first_map = &file_assets[0];
    assert!(first_map.file_id > 0, "映射应该包含有效的文件 ID");
    assert!(first_map.asset_id > 0, "映射应该包含有效的资产 ID");
    assert!(first_map.volume_order > 0, "映射应该包含有效的卷序号");

    // 验证映射状态
    assert_eq!(
        first_map.status,
        MapFileAssetStatus::Health,
        "文件资产映射状态应为 Health，实际为 {:?}",
        first_map.status
    );

    // 验证根目录状态
    let canonical_source_dir = test_source_dir.canonicalize()?;
    let root_path = canonical_source_dir.to_string_lossy().to_string();
    let root_info = database.find_root_info_by_path(&root_path)?;
    assert!(root_info.is_some(), "应该找到根目录记录");
    let root_info = root_info.unwrap();
    assert_eq!(
        root_info.status,
        RootStatus::Health,
        "根目录状态应为 Health"
    );

    // 验证文件状态
    let files = database.find_files_by_status_and_root_id(root_info.id.unwrap(),None)?;
    assert!(!files.is_empty(), "应该存在文件记录");
    for file in &files {
        assert_eq!(
            file.status,
            FileStatus::Health,
            "文件状态应为 Health，实际为 {:?}，文件名: {}",
            file.status,
            file.file_name
        );
    }

    // 验证目录状态
    let directories = database.find_directories_by_status_and_root_id(root_info.id.unwrap(), None)?;
    assert!(!directories.is_empty(), "应该存在目录记录");
    for directory in &directories {
        assert_eq!(
            directory.status,
            DirectoryStatus::Health,
            "目录状态应为 Health，实际为 {:?}，目录名: {}",
            directory.status,
            directory.directory_name
        );
    }

    Ok(())
}
/// 验证存档文件
fn assert_archive_files(archive_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    assert!(archive_dir.exists(), "存档目录应该存在");

    let entries: Vec<_> = fs::read_dir(archive_dir)?
        .filter_map(|entry| entry.ok())
        .collect();

    // 至少应该创建一个存档文件
    assert!(!entries.is_empty(), "应该创建存档文件");

    // 验证存档文件大小合理性
    for entry in entries {
        let path = entry.path();
        if path.is_file() {
            let size = fs::metadata(&path)?.len() as i64;
            info!("{} 存档文件大小: {}", path.display(), size);
            assert!(size > 0, "存档文件大小应该大于 0");
            // 1024 * 5 模拟头文件的开销
            assert!(
                size <= CHUNK_SIZE + 1024 * 5,
                "存档文件大小应该在合理范围内：{}",
                size
            );
        }
    }

    Ok(())
}

/// 验证分卷功能 - 内容相同的分卷
fn assert_volume_processing_same_content(
    test_source_dir: &Path,
    database: &Database,
) -> Result<(), Box<dyn std::error::Error>> {
    assert_volume_processing_common(
        test_source_dir,
        database,
        "large_file.dat",
        1, // 由于内容相同，只存储一份
        1, // 一个唯一资产
    )
}

/// 验证分卷功能 - 内容不同的情况
fn assert_volume_processing_different_content(
    test_source_dir: &Path,
    database: &Database,
) -> Result<(), Box<dyn std::error::Error>> {
    assert_volume_processing_common(
        test_source_dir,
        database,
        "large_file_different.dat",
        4, // 4个不同分卷
        4, // 四个唯一资产
    )
}

/// 通用的分卷验证函数
fn assert_volume_processing_common(
    test_source_dir: &Path,
    database: &Database,
    file_name: &str,
    expected_volume_count: usize,
    expected_unique_assets: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let test_source_dir_path = test_source_dir.canonicalize()?;
    let root_info =
        database.find_root_info_by_path(&test_source_dir_path.to_string_lossy().to_string())?;

    assert!(root_info.is_some(), "应该存在根目录记录");
    let root_id = root_info.unwrap().id.unwrap();

    // 查找大文件
    let files = database.find_files_by_status_and_root_id(root_id, None)?;
    let large_file = files
        .iter()
        .find(|f| f.file_name == file_name)
        .expect(&format!("应该找到大文件: {}", file_name));

    // 验证大文件大小是 2MB
    assert_eq!(
        large_file.file_size, LARGE_FILE_SIZE,
        "大文件大小应该是 2MB"
    );

    // 验证大文件状态
    assert_eq!(
        large_file.status,
        FileStatus::Health,
        "大文件状态应为 Health，实际为 {:?}",
        large_file.status
    );

    // 根据文件 ID 查找文件与存档资源映射关系
    let file_assets = database.find_map_file_asset_by_file_id(large_file.id.unwrap())?;

    // 验证映射关系存在
    assert!(!file_assets.is_empty(), "大文件应该有对应的存档资源映射");

    // 验证分卷数量
    assert_eq!(
        file_assets.len(),
        expected_volume_count,
        "文件应该被分成 {} 个分卷",
        expected_volume_count
    );

    // 验证分卷顺序
    let mut sorted_file_assets = file_assets.clone();
    sorted_file_assets.sort_by_key(|m| m.volume_order);

    for (i, map) in sorted_file_assets.iter().enumerate() {
        assert_eq!(
            map.volume_order,
            (i + 1) as i32,
            "分卷顺序应该从 1 开始连续递增"
        );

        // 验证映射状态
        assert_eq!(
            map.status,
            MapFileAssetStatus::Health,
            "文件资产映射状态应为 Health，实际为 {:?}",
            map.status
        );
    }

    // 验证唯一资产数量
    let unique_asset_ids: std::collections::HashSet<i64> =
        file_assets.iter().map(|m| m.asset_id).collect();

    assert_eq!(
        unique_asset_ids.len(),
        expected_unique_assets,
        "应该有 {} 个唯一的资产",
        expected_unique_assets
    );

    // 验证资产
    for map in &file_assets {
        let asset = database.find_archive_asset_by_id(map.asset_id)?;
        assert!(asset.is_some(), "每个映射应该关联到有效的资产");
        let asset = asset.unwrap();

        // 验证资产大小
        assert_eq!(asset.asset_size, ASSET_SIZE, "资产大小应该是 512KB");

        // 验证资产状态
        assert_eq!(
            asset.status,
            AssetStatus::Health,
            "资产状态应为 Health，实际为 {:?}",
            asset.status
        );
    }

    Ok(())
}

fn setup_logging() {
    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
        .try_init();
}

/// 测试基本的归档功能
#[test]
fn test_basic_archive_in() -> Result<(), Box<dyn std::error::Error>> {
    setup_logging();
    let env = TestEnvironment::new()?;

    // 创建普通测试文件
    create_test_files(&env.test_source_dir)?;

    // 扫描目录并保存到数据库
    let archive_service = ArchiveServices::new();
    archive_service.scan_and_save_directory_with_events(
        &env.test_source_dir,
        &env.database,
        None::<fn(_)>,
    )?;

    // 执行存档操作
    let mut context = env.create_archive_context();

    let archive_service = ArchiveInServices::new();
    archive_service.archive_file_in_db(
        env.test_source_dir.to_str().unwrap(),
        &mut context,
        &env.database,
    )?;

    // 验证数据库记录
    assert_database_records(&env.test_source_dir, &env.database)?;

    // 验证存档文件
    assert_archive_files(&env.test_archive_dir)?;

    Ok(())
}

/// 测试相同内容的大文件分卷归档
#[test]
fn test_archive_in_with_same_content_chunks() -> Result<(), Box<dyn std::error::Error>> {
    setup_logging();
    let env = TestEnvironment::new()?;

    // 创建内容重复的大文件用于测试分卷功能 (大小超过存档限制)
    create_large_test_file_with_same_chunk(&env.test_source_dir.join("large_file.dat"))?;

    // 扫描目录并保存到数据库
    let archive_service = ArchiveServices::new();
    archive_service.scan_and_save_directory_with_events(
        &env.test_source_dir,
        &env.database,
        None::<fn(_)>,
    )?;

    // 执行存档操作
    let mut context = env.create_archive_context(); // 512KB 限制，强制分卷

    let archive_service = ArchiveInServices::new();
    archive_service.archive_file_in_db(
        env.test_source_dir.to_str().unwrap(),
        &mut context,
        &env.database,
    )?;

    // 验证分卷功能 - 内容相同的情况
    assert_volume_processing_same_content(&env.test_source_dir, &env.database)?;

    Ok(())
}

/// 测试不同内容的大文件分卷归档
#[test]
fn test_archive_in_with_different_content_chunks() -> Result<(), Box<dyn std::error::Error>> {
    setup_logging();
    let env = TestEnvironment::new()?;

    // 创建内容不同的大文件用于测试分卷功能 (2MB)
    create_large_test_file_with_different_chunks(
        &env.test_source_dir.join("large_file_different.dat"),
    )?;

    // 扫描目录并保存到数据库
    let archive_service = ArchiveServices::new();
    archive_service.scan_and_save_directory_with_events(
        &env.test_source_dir,
        &env.database,
        None::<fn(_)>,
    )?;

    // 执行存档操作
    let mut context = env.create_archive_context(); // 512KB 限制，强制分卷

    let archive_service = ArchiveInServices::new();
    archive_service.archive_file_in_db(
        env.test_source_dir.to_str().unwrap(),
        &mut context,
        &env.database,
    )?;

    // 验证分卷功能 - 内容不同的情况
    assert_volume_processing_different_content(&env.test_source_dir, &env.database)?;

    Ok(())
}