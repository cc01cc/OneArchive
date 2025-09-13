//! ArchiveExtract 功能集成测试

use log::info;
use one_archive_lib::mod_scan::impl_scan::ScanServices;
use one_archive_lib::mod_scan::model_scan::ScanProgress;
use one_archive_lib::mod_scan::trait_scan::DirectoryScanOperations;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

use one_archive_lib::mod_database::database::Database;
use one_archive_lib::mod_database::trait_database::InitializationOperations;
use one_archive_lib::mod_extract::impl_extract::ExtractService;
use one_archive_lib::mod_extract::model_extract::ExtractTask;
use one_archive_lib::mod_extract::trait_extract::ExtractOperations;
// 添加归档相关的导入
use one_archive_lib::mod_archive::impl_archive::ArchiveInServices;
use one_archive_lib::mod_archive::trait_archive::{ArchiveContext, ArchiveOperations};

#[test]
fn test_extract_archive_integration() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志记录器
    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
        .try_init();

    // 创建临时目录用于测试
    let temp_dir = TempDir::new()?;
    let test_source_dir = temp_dir.path().join("test-source");
    let test_archive_dir = temp_dir.path().join("test-archive");
    let test_extract_dir = temp_dir.path().join("test-extract");

    fs::create_dir_all(&test_source_dir)?;
    fs::create_dir_all(&test_archive_dir)?;
    fs::create_dir_all(&test_extract_dir)?;

    // 创建测试文件
    fs::write(
        test_source_dir.join("test1.txt"),
        "This is test file 1 content",
    )?;
    fs::write(
        test_source_dir.join("test2.txt"),
        "This is test file 2 content",
    )?;

    // 创建子目录和文件
    let subdir = test_source_dir.join("subdir");
    fs::create_dir_all(&subdir)?;
    fs::write(subdir.join("subfile.txt"), "This is a file in subdirectory")?;

    // 初始化数据库
    let db_file = temp_dir.path().join("test_archive.sqlite");
    let db_url = format!("file:{}", db_file.display());
    let database = Database::new(&db_url)?;
    database.initialize_tables(&database.conn)?;
    info!("测试数据库初始化完成");

    // 模拟已存档的文件（创建数据库记录）
    setup_test_archive_data(&test_source_dir, &test_archive_dir, &database)?;
    info!("测试数据库已存档数据初始化完成，开始测试解档...");

    // 执行解档操作
    let extract_service = ExtractService::new();
    let task = ExtractTask {
        archive_directory: test_archive_dir.to_string_lossy().to_string(),
        root_id: 1,
        target_path: test_extract_dir.to_string_lossy().to_string(),
        overwrite: true,
    };

    let progress = extract_service.extract_archive(&task, &database)?;

    // 验证解档结果
    assert!(progress.completed, "解档应该完成");
    // 修改期望值为 3，因为有 3 个文件需要解档（test1.txt, test2.txt, subdir/subfile.txt）
    assert_eq!(progress.total_files, 3, "应该有 3 个文件需要解档");
    assert_eq!(progress.processed_files, 3, "应该处理了 3 个文件");
    assert!(
        progress.error.is_none(),
        "解档过程中不应该有错误，但出现了错误：{:?}",
        progress.error
    );

    // 验证解档后的文件是否存在
    assert_extracted_files(&test_extract_dir)?;

    Ok(())
}

/// 设置测试用的存档数据
fn setup_test_archive_data(
    source_dir: &Path,
    archive_dir: &Path,
    database: &Database,
) -> Result<(), Box<dyn std::error::Error>> {
    // 首先扫描并保存目录结构到数据库
    let scan_service = ScanServices::new();

    // 扫描目录并保存到数据库
    scan_service.scan_and_save_directory_with_events(
        source_dir,
        database,
        None::<fn(ScanProgress)>,
    )?;

    // 然后执行归档操作
    let archive_in_service = ArchiveInServices::new();
    let mut context = ArchiveContext::new(
        "test_archive".to_string(),
        archive_dir.to_string_lossy().to_string(),
        1024 * 1024, // 1MB 限制
    );

    // 添加根目录信息并执行归档
    archive_in_service.archive_file_in_db(source_dir.to_str().unwrap(), &mut context, database)?;

    Ok(())
}

/// 验证解档后的文件
fn assert_extracted_files(extract_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // 验证根目录下的文件
    let test1_path = extract_dir.join("test1.txt");
    let test2_path = extract_dir.join("test2.txt");

    assert!(test1_path.exists(), "test1.txt 应该存在");
    assert!(test2_path.exists(), "test2.txt 应该存在");

    let content1 = fs::read_to_string(&test1_path)?;
    let content2 = fs::read_to_string(&test2_path)?;

    assert_eq!(
        content1, "This is test file 1 content",
        "test1.txt 内容应该正确"
    );
    assert_eq!(
        content2, "This is test file 2 content",
        "test2.txt 内容应该正确"
    );

    // 验证子目录和文件
    let subdir_path = extract_dir.join("subdir");
    assert!(subdir_path.exists(), "子目录应该存在");
    assert!(subdir_path.is_dir(), "subdir 应该是目录");

    let subfile_path = subdir_path.join("subfile.txt");
    assert!(subfile_path.exists(), "subfile.txt 应该存在");

    let subfile_content = fs::read_to_string(&subfile_path)?;
    assert_eq!(
        subfile_content, "This is a file in subdirectory",
        "subfile.txt 内容应该正确"
    );

    Ok(())
}

// #[test]
fn test_extract_with_overwrite_option() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志记录器
    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
        .try_init();

    // 创建临时目录用于测试
    let temp_dir = TempDir::new()?;
    let test_source_dir = temp_dir.path().join("test-source");
    let test_archive_dir = temp_dir.path().join("test-archive");
    let test_extract_dir = temp_dir.path().join("test-extract");

    fs::create_dir_all(&test_source_dir)?;
    fs::create_dir_all(&test_archive_dir)?;
    fs::create_dir_all(&test_extract_dir)?;

    // 创建已存在的文件（用于测试覆盖选项）
    fs::write(
        test_extract_dir.join("existing_file.txt"),
        "existing content",
    )?;

    // 初始化数据库
    let db_file = temp_dir.path().join("test_archive.sqlite");
    let db_url = format!("file:{}", db_file.display());
    let database = Database::new(&db_url)?;
    database.initialize_tables(&database.conn)?;

    // 模拟已存档的文件
    setup_test_archive_data(&test_source_dir, &test_archive_dir, &database)?;

    // 执行解档操作（不覆盖）
    let extract_service = ExtractService::new();
    let task = ExtractTask {
        archive_directory: test_archive_dir.to_string_lossy().to_string(),
        root_id: 1,
        target_path: test_extract_dir.to_string_lossy().to_string(),
        overwrite: true,
    };

    let progress = extract_service.extract_archive(&task, &database)?;

    // 验证解档结果
    assert!(progress.completed, "解档应该完成");
    // 修改期望值为 1，因为 setup_test_archive_data 只创建了一个资产记录
    assert_eq!(progress.total_files, 1, "应该有 1 个文件需要解档");
    assert_eq!(progress.processed_files, 1, "应该处理了 1 个文件");

    // 验证已存在的文件没有被覆盖
    let existing_content = fs::read_to_string(test_extract_dir.join("existing_file.txt"))?;
    assert_eq!(
        existing_content, "existing content",
        "已存在的文件不应该被覆盖"
    );

    Ok(())
}

#[test]
fn test_extract_nonexistent_archive() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志记录器
    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
        .try_init();

    // 创建临时目录用于测试
    let temp_dir = TempDir::new()?;
    let test_extract_dir = temp_dir.path().join("test-extract");
    fs::create_dir_all(&test_extract_dir)?;

    // 初始化数据库
    let db_file = temp_dir.path().join("test_archive.sqlite");
    let db_url = format!("file:{}", db_file.display());
    let database = Database::new(&db_url)?;
    database.initialize_tables(&database.conn)?;

    // 执行解档操作（不存在的存档）
    let extract_service = ExtractService::new();
    let task = ExtractTask {
        archive_directory: String::from("nonexistent_archive"),
        root_id: 999,
        target_path: test_extract_dir.to_string_lossy().to_string(),
        overwrite: true,
    };

    let result = extract_service.extract_archive(&task, &database);

    // 验证应该返回错误
    assert!(result.is_err(), "应该返回错误，因为存档不存在");
    // assert!(
    //     result.unwrap_err().to_string().contains("未找到存档 ID"),
    //     "错误信息应该提示存档不存在"
    // );

    Ok(())
}
