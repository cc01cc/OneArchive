//! ArchiveExtract 功能集成测试
//!
//! 该测试模块验证了解档功能的各个方面，包括：
//! - 基本解档功能
//! - 进度回调
//! - 覆盖和非覆盖模式
//! - 错误处理
//! - 特殊文件名处理
//! - 分卷数据块处理
//! - 损坏数据块处理

use log::info;
use one_archive_lib::mod_scan::impl_scan::ScanServices;
use one_archive_lib::mod_scan::model_scan::ScanProgress;
use one_archive_lib::mod_scan::trait_scan::DirectoryScanOperations;
use std::fs;
use std::path::Path;

use one_archive_lib::mod_archive::impl_archive::ArchiveServices;
use one_archive_lib::mod_archive::trait_archive::{ArchiveContext, ArchiveOperations};
use one_archive_lib::mod_database::database::Database;
use one_archive_lib::mod_extract::impl_extract::ExtractService;
use one_archive_lib::mod_extract::model_extract::{ExtractProgress, ExtractTask};
use one_archive_lib::mod_extract::trait_extract::ExtractOperations;

mod common;
use common::*;

/// 设置测试用的归档数据
fn setup_test_archive_data(env: &TestEnvironment) -> Result<(), Box<dyn std::error::Error>> {
    let scan_service = ScanServices::new();

    scan_service.scan_and_save_directory_with_events(
        &env.source_dir,
        &env.database,
        None::<fn(ScanProgress)>,
    )?;

    let archive_in_service = ArchiveServices::default();
    let mut context = ArchiveContext::new(
        env.database.clone(),
        "test_archive".to_string(),
        env.archive_dir.to_string_lossy().to_string(),
        1024 * 1024, // 1MB 限制
    );

    archive_in_service.archive(env.source_dir.to_str().unwrap(), &mut context, None::<fn(_)>)?;
    Ok(())
}

/// 验证基本解档文件
fn assert_basic_extracted_files(extract_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let test1_path = extract_dir.join("test1.txt");
    let test2_path = extract_dir.join("test2.txt");

    assert!(test1_path.exists(), "test1.txt 应该存在");
    assert!(test2_path.exists(), "test2.txt 应该存在");

    let content1 = fs::read_to_string(&test1_path)?;
    let content2 = fs::read_to_string(&test2_path)?;

    assert_eq!(content1, "This is test file 1 content", "test1.txt 内容应该正确");
    assert_eq!(content2, "This is test file 2 content", "test2.txt 内容应该正确");
    Ok(())
}

/// 验证完整解档文件结构（包括子目录）
fn assert_extracted_files(extract_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    assert_basic_extracted_files(extract_dir)?;

    let subdir_path = extract_dir.join("subdir");
    assert!(subdir_path.exists(), "子目录应该存在");
    assert!(subdir_path.is_dir(), "subdir 应该是目录");

    let subfile_path = subdir_path.join("subfile.txt");
    assert!(subfile_path.exists(), "subfile.txt 应该存在");

    let subfile_content = fs::read_to_string(&subfile_path)?;
    assert_eq!(subfile_content, "This is a file in subdirectory", "subfile.txt 内容应该正确");
    Ok(())
}

/// 执行解档操作的辅助函数
fn perform_extraction(
    env: &TestEnvironment, root_id: i64, overwrite: bool,
) -> Result<ExtractProgress, Box<dyn std::error::Error>> {
    let mut extract_service = ExtractService::new();
    let task = ExtractTask {
        root_id,
        target_path: env.extract_dir.to_string_lossy().to_string(),
        overwrite,
    };

    Ok(extract_service.extract_archive(&task, &env.database, None::<fn(_)>).map_err(|e| e)?)
}

/// 测试基本解档功能
#[test]
fn test_extract_archive_integration() -> Result<(), Box<dyn std::error::Error>> {
    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
        .try_init();

    let env = TestEnvironment::new()?;
    create_test_files_with_subdir(&env.source_dir)?;
    setup_test_archive_data(&env)?;
    info!("测试数据库已归档数据初始化完成，开始测试解档...");

    let progress = perform_extraction(&env, 1, true)?;

    assert!(progress.completed, "解档应该完成");
    assert_eq!(progress.total_files, 3, "应该有 3 个文件需要解档");
    assert_eq!(progress.processed_files, 3, "应该处理了 3 个文件");
    assert!(progress.error.is_none(), "解档过程中不应该有错误");

    assert_extracted_files(&env.extract_dir)?;
    Ok(())
}

/// 测试使用进度回调的解档操作
#[test]
fn test_extract_with_progress_callback() -> Result<(), Box<dyn std::error::Error>> {
    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
        .try_init();

    let env = TestEnvironment::new()?;
    create_basic_test_files(&env.source_dir)?;
    setup_test_archive_data(&env)?;

    let progress_updates = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let progress_updates_clone = progress_updates.clone();

    let progress_callback = move |progress: ExtractProgress| {
        progress_updates_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        println!("进度更新：{}/{} files processed", progress.processed_files, progress.total_files);
    };

    let mut extract_service = ExtractService::new();
    let task = ExtractTask {
        root_id: 1,
        target_path: env.extract_dir.to_string_lossy().to_string(),
        overwrite: true,
    };

    let progress =
        extract_service.extract_archive(&task, &env.database, Some(progress_callback))?;

    assert!(progress.completed, "解档应该完成");
    assert_eq!(progress.total_files, 2, "应该有 2 个文件需要解档");
    assert_eq!(progress.processed_files, 2, "应该处理了 2 个文件");
    assert!(progress.error.is_none(), "解档过程中不应该有错误");

    let updates_count = progress_updates.load(std::sync::atomic::Ordering::SeqCst);
    assert!(updates_count > 0, "进度回调应该被调用至少一次");

    assert_basic_extracted_files(&env.extract_dir)?;
    Ok(())
}

/// 测试启用覆盖选项的解档
#[test]
fn test_extract_with_overwrite_option() -> Result<(), Box<dyn std::error::Error>> {
    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
        .try_init();

    let env = TestEnvironment::new()?;
    create_basic_test_files(&env.source_dir)?;
    setup_test_archive_data(&env)?;

    // 创建与归档中同名的文件
    fs::write(env.extract_dir.join("test1.txt"), "existing content")?;
    fs::write(env.extract_dir.join("test2.txt"), "existing content")?;

    let progress = perform_extraction(&env, 1, true)?;

    assert!(progress.completed, "解档应该完成");
    assert_eq!(progress.total_files, 2, "应该有 2 个文件需要解档");
    assert_eq!(progress.processed_files, 2, "应该处理了 2 个文件");

    assert_basic_extracted_files(&env.extract_dir)?;
    Ok(())
}

/// 测试禁用覆盖选项的解档
#[test]
fn test_extract_without_overwrite_option() -> Result<(), Box<dyn std::error::Error>> {
    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
        .try_init();

    let env = TestEnvironment::new()?;
    create_basic_test_files(&env.source_dir)?;
    setup_test_archive_data(&env)?;

    // 创建与归档中同名的文件
    let existing_content = "existing content";
    fs::write(env.extract_dir.join("test1.txt"), existing_content)?;
    fs::write(env.extract_dir.join("test2.txt"), existing_content)?;

    let progress = perform_extraction(&env, 1, false)?;

    assert!(progress.completed, "解档应该完成");
    assert_eq!(progress.total_files, 2, "应该有 2 个文件需要解档");
    assert_eq!(progress.processed_files, 2, "应该处理了 2 个文件");

    let content1 = fs::read_to_string(env.extract_dir.join("test1.txt"))?;
    let content2 = fs::read_to_string(env.extract_dir.join("test2.txt"))?;

    assert_eq!(content1, existing_content, "test1.txt 应该保持原有内容");
    assert_eq!(content2, existing_content, "test2.txt 应该保持原有内容");

    Ok(())
}

/// 测试不存在的归档
#[test]
fn test_extract_nonexistent_archive() -> Result<(), Box<dyn std::error::Error>> {
    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
        .try_init();

    let env = TestEnvironment::new()?;

    let result = perform_extraction(&env, 999, true);
    assert!(result.is_err(), "应该返回错误，因为归档不存在");

    Ok(())
}

/// 测试分卷数据块解档
#[test]
fn test_extract_with_volume_chunks() -> Result<(), Box<dyn std::error::Error>> {
    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
        .try_init();

    let env = TestEnvironment::new()?;

    let large_content = "A".repeat(2 * 1024 * 1024); // 2MB 文件
    fs::write(env.source_dir.join("large_file.txt"), large_content)?;

    setup_test_archive_data(&env)?;

    let progress = perform_extraction(&env, 1, true)?;

    assert!(progress.completed, "解档应该完成");
    assert_eq!(progress.total_files, 1, "应该有 1 个文件需要解档");
    assert_eq!(progress.processed_files, 1, "应该处理了 1 个文件");
    assert!(progress.error.is_none(), "解档过程中不应该有错误");

    let extracted_file = env.extract_dir.join("large_file.txt");
    assert!(extracted_file.exists(), "large_file.txt 应该存在");

    Ok(())
}

/// 测试特殊文件名处理
#[test]
fn test_extract_with_special_filenames() -> Result<(), Box<dyn std::error::Error>> {
    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
        .try_init();

    let env = TestEnvironment::new()?;

    fs::write(env.source_dir.join("文件.txt"), "Chinese filename content")?;
    fs::write(env.source_dir.join("файл.txt"), "Cyrillic filename content")?;
    fs::write(env.source_dir.join("file with spaces.txt"), "File with spaces content")?;

    setup_test_archive_data(&env)?;

    let progress = perform_extraction(&env, 1, true)?;

    assert!(progress.completed, "解档应该完成");
    assert_eq!(progress.total_files, 3, "应该有 3 个文件需要解档");
    assert_eq!(progress.processed_files, 3, "应该处理了 3 个文件");
    assert!(progress.error.is_none(), "解档过程中不应该有错误");

    assert!(env.extract_dir.join("文件.txt").exists());
    assert!(env.extract_dir.join("файл.txt").exists());
    assert!(env.extract_dir.join("file with spaces.txt").exists());

    Ok(())
}

/// 测试损坏数据块的处理
#[test]
fn test_extract_with_corrupted_chunks() -> Result<(), Box<dyn std::error::Error>> {
    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
        .try_init();

    let env = TestEnvironment::new()?;
    create_basic_test_files(&env.source_dir)?;
    setup_test_archive_data(&env)?;

    // 模拟数据块文件损坏
    fs::remove_dir_all(&env.archive_dir)?;

    let progress = perform_extraction(&env, 1, true)?;

    assert!(progress.completed, "解档应该完成");
    assert_eq!(progress.total_files, 2, "应该有 2 个文件需要解档");
    assert_eq!(progress.processed_files, 2, "应该处理了 2 个文件");
    assert!(progress.error.is_some(), "解档过程中应该有错误");

    Ok(())
}

/// 测试空文件的解档
#[test]
fn test_extract_empty_files() -> Result<(), Box<dyn std::error::Error>> {
    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
        .try_init();

    let env = TestEnvironment::new()?;

    // 创建空文件
    fs::File::create(env.source_dir.join("empty.txt"))?;

    setup_test_archive_data(&env)?;

    let progress = perform_extraction(&env, 1, true)?;

    assert!(progress.completed, "解档应该完成");
    assert_eq!(progress.total_files, 1, "应该有 1 个文件需要解档");
    assert_eq!(progress.processed_files, 1, "应该处理了 1 个文件");
    assert!(progress.error.is_none(), "解档过程中不应该有错误");

    // 验证空文件存在
    let empty_file = env.extract_dir.join("empty.txt");
    assert!(empty_file.exists(), "空文件应该存在");
    assert_eq!(fs::read_to_string(&empty_file)?, "", "空文件应该确实为空");

    Ok(())
}
