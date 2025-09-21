//! ArchiveIn 功能集成测试
//!
//! 该测试模块验证了归档功能的各个方面，包括：
//! - 基本归档功能
//! - 分卷处理（相同内容和不同内容）
//! - 空文件归档
//! - 数据库记录验证
//! - 归档文件验证

use log::info;
use one_archive_lib::mod_scan::impl_scan::ScanServices;
use one_archive_lib::mod_scan::trait_scan::DirectoryScanOperations;
use std::fs;
use std::io::{BufWriter, Write};
use std::path::Path;

use one_archive_lib::mod_archive::impl_archive::ArchiveServices;
use one_archive_lib::mod_archive::trait_archive::ArchiveContext;
use one_archive_lib::mod_archive::trait_archive::ArchiveOperations;
use one_archive_lib::mod_database::constants::{
    ChunkStatus, DirectoryStatus, FileStatus, MapFileChunkStatus, RootStatus,
};
use one_archive_lib::mod_database::database::Database;
use one_archive_lib::mod_database::trait_database::{
    ArchiveChunkOperations, DirectoryOperations, FileOperations, MapFileChunkOperations,
    RootOperations,
};

// 常量定义
const LARGE_FILE_SIZE: i64 = 2 * 1024 * 1024; // 2MB
const CHUNK_SIZE: i64 = 512 * 1024; // 512KB
const ASSET_SIZE: i64 = 524288; // 512KB in bytes

mod common;
use common::*;

/// 创建普通测试文件
fn create_test_files(source_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    fs::write(source_dir.join("test1.txt"), "This is test file 1 content")?;
    fs::write(source_dir.join("test2.txt"), "This is test file 2 content")?;
    Ok(())
}

/// 创建大测试文件 (2MB) - 内容相同的数据块
fn create_large_test_file_with_same_chunk(
    file_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let file = fs::File::create(file_path)?;
    let mut writer = BufWriter::new(file);

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

/// 执行归档操作的辅助函数
fn perform_archiving(env: &TestEnvironment) -> Result<(), Box<dyn std::error::Error>> {
    let scan_service = ScanServices::new();
    scan_service.scan_and_save_directory_with_events(
        &env.source_dir,
        &env.database,
        None::<fn(_)>,
    )?;

    let mut context = ArchiveContext::new(
        "archive".to_string(),
        env.archive_dir.to_string_lossy().to_string(),
        CHUNK_SIZE,
    );
    let archive_service = ArchiveServices::default();
    archive_service.archive_file_in_db(
        env.source_dir.to_str().unwrap(),
        &mut context,
        &env.database,
        None::<fn(_)>,
    )?;
    Ok(())
}

/// 验证数据库记录
fn assert_database_records(
    source_dir: &Path,
    database: &Database,
) -> Result<(), Box<dyn std::error::Error>> {
    // 验证数据块表
    let chunks = database.find_archive_chunks_by_status(None)?;
    assert!(!chunks.is_empty(), "应该创建了归档数据块");

    let first_chunk = &chunks[0];
    assert!(first_chunk.chunk_name.len() > 0, "数据块应该有名称");

    // 检查是否是空文件测试
    let is_empty_file_test = std::fs::read_dir(source_dir)?
        .filter_map(Result::ok)
        .any(|entry| entry.metadata().map(|m| m.len() == 0).unwrap_or(false));

    if is_empty_file_test {
        // 对于空文件，数据块大小应该为 0
        assert_eq!(first_chunk.chunk_size, 0, "空文件的数据块大小应该为 0");
    } else {
        // 对于非空文件，数据块大小应该大于 0
        assert!(first_chunk.chunk_size > 0, "数据块大小应该大于 0");
    }

    // 验证数据块状态
    assert_eq!(
        first_chunk.status,
        ChunkStatus::Health,
        "数据块状态应为 Health，实际为 {:?}",
        first_chunk.status
    );

    // 验证文件与数据块映射表
    let file_chunks = database.find_map_file_chunk_by_status(None)?;
    assert!(!file_chunks.is_empty(), "应该创建了文件到数据块的映射");

    let first_map = &file_chunks[0];
    assert!(first_map.file_id > 0, "映射应该包含有效的文件 ID");
    assert!(first_map.chunk_id > 0, "映射应该包含有效的数据块 ID");
    assert!(first_map.volume_order > 0, "映射应该包含有效的卷序号");

    // 验证映射状态
    assert_eq!(
        first_map.status,
        MapFileChunkStatus::Health,
        "文件数据块映射状态应为 Health，实际为 {:?}",
        first_map.status
    );

    // 验证根目录状态
    let canonical_source_dir = source_dir.canonicalize()?;
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
    let files = database.find_files_by_status_and_root_id(root_info.id.unwrap(), None)?;
    assert!(!files.is_empty(), "应该存在文件记录");
    for file in &files {
        assert_eq!(
            file.status,
            FileStatus::Health,
            "文件状态应为 Health，实际为 {:?}，文件名：{}",
            file.status,
            file.file_name
        );
    }

    // 验证目录状态
    let directories =
        database.find_directories_by_status_and_root_id(root_info.id.unwrap(), None)?;
    for directory in &directories {
        assert_eq!(
            directory.status,
            DirectoryStatus::Health,
            "目录状态应为 Health，实际为 {:?}，目录名：{}",
            directory.status,
            directory.directory_name
        );
    }

    Ok(())
}

/// 验证归档文件
fn assert_archive_files(archive_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    assert!(archive_dir.exists(), "归档目录应该存在");

    let entries: Vec<_> = fs::read_dir(archive_dir)?
        .filter_map(|entry| entry.ok())
        .collect();

    assert!(!entries.is_empty(), "应该创建归档文件");

    for entry in entries {
        let path = entry.path();
        if path.is_file() {
            let size = fs::metadata(&path)?.len() as i64;
            info!("{} 归档文件大小：{}", path.display(), size);
            assert!(size > 0, "归档文件大小应该大于 0");
            // 1024 * 5 模拟头文件的开销
            assert!(
                size <= CHUNK_SIZE + 1024 * 5,
                "归档文件大小应该在合理范围内：{}",
                size
            );
        }
    }

    Ok(())
}

/// 通用的分卷验证函数
fn assert_volume_processing_common(
    source_dir: &Path,
    database: &Database,
    file_name: &str,
    expected_volume_count: usize,
    expected_unique_chunks: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let source_dir_path = source_dir.canonicalize()?;
    let root_info =
        database.find_root_info_by_path(&source_dir_path.to_string_lossy().to_string())?;

    assert!(root_info.is_some(), "应该存在根目录记录");
    let root_id = root_info.unwrap().id.unwrap();

    // 查找大文件
    let files = database.find_files_by_status_and_root_id(root_id, None)?;
    let large_file = files
        .iter()
        .find(|f| f.file_name == file_name)
        .expect(&format!("应该找到大文件：{}", file_name));

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

    // 根据文件 ID 查找文件与归档数据块映射关系
    let file_chunks = database.find_map_file_chunk_by_file_id(large_file.id.unwrap())?;

    // 验证映射关系存在
    assert!(!file_chunks.is_empty(), "大文件应该有对应的归档数据块映射");

    // 验证分卷数量
    assert_eq!(
        file_chunks.len(),
        expected_volume_count,
        "文件应该被分成 {} 个分卷",
        expected_volume_count
    );

    // 验证分卷顺序
    let mut sorted_file_chunks = file_chunks.clone();
    sorted_file_chunks.sort_by_key(|m| m.volume_order);

    for (i, map) in sorted_file_chunks.iter().enumerate() {
        assert_eq!(
            map.volume_order,
            (i + 1) as i32,
            "分卷顺序应该从 1 开始连续递增"
        );

        // 验证映射状态
        assert_eq!(
            map.status,
            MapFileChunkStatus::Health,
            "文件数据块映射状态应为 Health，实际为 {:?}",
            map.status
        );
    }

    // 验证唯一数据块数量
    let unique_chunk_ids: std::collections::HashSet<i64> =
        file_chunks.iter().map(|m| m.chunk_id).collect();

    assert_eq!(
        unique_chunk_ids.len(),
        expected_unique_chunks,
        "应该有 {} 个唯一的数据块",
        expected_unique_chunks
    );

    // 验证数据块
    for map in &file_chunks {
        let chunk = database.find_archive_chunk_by_id(map.chunk_id)?;
        assert!(chunk.is_some(), "每个映射应该关联到有效的数据块");
        let chunk = chunk.unwrap();

        // 验证数据块大小
        assert_eq!(chunk.chunk_size, ASSET_SIZE, "数据块大小应该是 512KB");

        // 验证数据块状态
        assert_eq!(
            chunk.status,
            ChunkStatus::Health,
            "数据块状态应为 Health，实际为 {:?}",
            chunk.status
        );
    }

    Ok(())
}

/// 验证分卷功能 - 内容相同的分卷
fn assert_volume_processing_same_content(
    source_dir: &Path,
    database: &Database,
) -> Result<(), Box<dyn std::error::Error>> {
    assert_volume_processing_common(
        source_dir,
        database,
        "large_file.dat",
        1, // 由于内容相同，只存储一份
        1, // 一个唯一数据块
    )
}

/// 验证分卷功能 - 内容不同的情况
fn assert_volume_processing_different_content(
    source_dir: &Path,
    database: &Database,
) -> Result<(), Box<dyn std::error::Error>> {
    assert_volume_processing_common(
        source_dir,
        database,
        "large_file_different.dat",
        4, // 4 个不同分卷
        4, // 四个唯一数据块
    )
}

/// 测试基本的归档功能
#[test]
fn test_basic_archive_in() -> Result<(), Box<dyn std::error::Error>> {
    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
        .try_init();

    let env = TestEnvironment::new()?;
    create_test_files(&env.source_dir)?;
    perform_archiving(&env)?;

    assert_database_records(&env.source_dir, &env.database)?;
    assert_archive_files(&env.archive_dir)?;

    Ok(())
}

/// 测试相同内容的大文件分卷归档
#[test]
fn test_archive_in_with_same_content_chunks() -> Result<(), Box<dyn std::error::Error>> {
    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
        .try_init();

    let env = TestEnvironment::new()?;
    create_large_test_file_with_same_chunk(&env.source_dir.join("large_file.dat"))?;
    perform_archiving(&env)?;

    assert_volume_processing_same_content(&env.source_dir, &env.database)?;

    Ok(())
}

/// 测试不同内容的大文件分卷归档
#[test]
fn test_archive_in_with_different_content_chunks() -> Result<(), Box<dyn std::error::Error>> {
    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
        .try_init();

    let env = TestEnvironment::new()?;
    create_large_test_file_with_different_chunks(&env.source_dir.join("large_file_different.dat"))?;
    perform_archiving(&env)?;

    assert_volume_processing_different_content(&env.source_dir, &env.database)?;

    Ok(())
}

/// 测试空文件归档
#[test]
fn test_archive_empty_file() -> Result<(), Box<dyn std::error::Error>> {
    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
        .try_init();

    let env = TestEnvironment::new()?;
    fs::File::create(env.source_dir.join("empty.txt"))?;
    perform_archiving(&env)?;

    assert_database_records(&env.source_dir, &env.database)?;
    assert_archive_files(&env.archive_dir)?;

    Ok(())
}

/// TODO 测试归档目录权限不足的情况
// #[test]
fn test_archive_directory_permission_denied() -> Result<(), Box<dyn std::error::Error>> {
    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
        .try_init();

    let env = TestEnvironment::new()?;
    create_test_files(&env.source_dir)?;

    // 创建一个只读的归档目录
    let readonly_archive_dir = env.temp_dir.path().join("readonly-archive");
    fs::create_dir_all(&readonly_archive_dir)?;
    let mut perms = fs::metadata(&readonly_archive_dir)?.permissions();
    perms.set_readonly(true);
    fs::set_permissions(&readonly_archive_dir, perms)?;

    let scan_service = ScanServices::new();
    scan_service.scan_and_save_directory_with_events(
        &env.source_dir,
        &env.database,
        None::<fn(_)>,
    )?;

    // 执行归档操作，应该失败
    let mut context = ArchiveContext::new(
        "archive".to_string(),
        readonly_archive_dir.to_string_lossy().to_string(),
        CHUNK_SIZE,
    );

    let archive_service = ArchiveServices::default();
    let result = archive_service.archive_file_in_db(
        env.source_dir.to_str().unwrap(),
        &mut context,
        &env.database,
        None::<fn(_)>,
    );

    assert!(result.is_err());
    Ok(())
}
