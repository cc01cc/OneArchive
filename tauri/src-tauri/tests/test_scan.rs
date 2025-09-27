//! ArchiveScan 功能集成测试
//!
//! 该测试模块验证了扫描功能的各个方面，包括：
//! - 目录统计功能
//! - 首次扫描功能
//! - 增量扫描功能
//! - 多根目录处理

use log::{debug, info};
use one_archive_lib::mod_scan::impl_scan::ScanServices;
use one_archive_lib::mod_scan::model_scan::ScanProgress;
use one_archive_lib::mod_scan::trait_scan::{
    DirectoryScanOperations, DirectoryStatisticsOperations,
};
use std::fs;

use one_archive_lib::mod_database::constants::{DirectoryStatus, FileStatus, RootStatus};
use one_archive_lib::mod_database::dao::info_directory::InfoDirectoryDao;
use one_archive_lib::mod_database::dao::info_file::InfoFileDao;
use one_archive_lib::mod_database::dao::info_root::InfoRootDao;
use one_archive_lib::mod_database::dao::view_file::ViewFileDao;
use one_archive_lib::mod_database::database::Database;
use one_archive_lib::mod_database::schema::InfoFile;

mod common;
use common::*;

/// 进度回调函数
fn progress_callback(progress: ScanProgress) {
    println!("Progress: {:.2}% - {}", progress.progress, progress.message);
}

/// 验证目录统计结果
fn assert_directory_statistics(
    stats: &one_archive_lib::mod_scan::model_scan::DirectoryStatistics, expected_size: u64,
    expected_dirs: usize, expected_files: usize, expected_depth: usize, test_name: &str,
) {
    assert_eq!(stats.total_size, expected_size, "{}: 总大小应该等于所有文件大小之和", test_name);
    assert_eq!(stats.directory_count, expected_dirs as u32, "{}: 目录数量应该正确", test_name);
    assert_eq!(stats.file_count, expected_files as u32, "{}: 文件数量应该正确", test_name);
    assert_eq!(stats.max_depth, expected_depth as u32, "{}: 最大深度应该正确", test_name);
}

/// 验证扫描结果
fn assert_scan_results(
    database: &Database, root_path: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let root_dao = InfoRootDao::new(database.conn.clone());
    let roots = root_dao.find_all_root_info()?;
    assert_eq!(roots.len(), 1, "应该只有一个根目录");

    let root = &roots[0];
    // 处理 Windows 长路径前缀问题，统一移除可能存在的 \\?\ 前缀
    let stored_path = root.root_path.trim_start_matches(r"\\?\");
    let canonical_path = root_path.canonicalize()?;
    let actual_path = canonical_path.to_string_lossy();
    assert_eq!(stored_path, actual_path.trim_start_matches(r"\\?\"));

    // 验证根目录状态（首次扫描应为 WaitToArchive）
    assert_eq!(root.status, RootStatus::WaitToArchive, "根目录状态应该是 WaitToArchive");

    let directory_dao = InfoDirectoryDao::new(database.conn.clone());
    // 验证目录数量
    let directories =
        directory_dao.find_by_status_and_root_id(root.id.unwrap(), None)?;
    assert_eq!(directories.len(), 4, "应该有 4 个目录 (dir1, dir2, subdir, 根目录)");

    // 验证每个目录的状态（首次扫描应为 WaitToArchive）
    for dir in &directories {
        assert_eq!(dir.status, DirectoryStatus::WaitToArchive, "所有目录状态应该是 WaitToArchive");
    }

    let file_dao = InfoFileDao::new(database.conn.clone());
    // 验证文件数量
    let files = file_dao.find_files_by_status_and_root_id(root.id.unwrap(), None)?;
    assert_eq!(files.len(), 3, "应该有 3 个文件");

    // 验证每个文件的状态（首次扫描应为 WaitToArchive）
    for file in &files {
        assert_eq!(
            file.status,
            FileStatus::WaitToArchive,
            "所有文件状态应该是 WaitToArchive，但文件 {} 状态为 {:?}",
            file.file_name,
            file.status
        );
    }

    // 验证特定目录存在
    let dir1_record = directory_dao.find_directory_by_path(root.id.unwrap(), "dir1")?;
    assert!(dir1_record.is_some(), "dir1 应该存在于数据库中");
    if let Some(ref dir) = dir1_record {
        assert_eq!(dir.status, DirectoryStatus::WaitToArchive, "dir1 状态应该是 WaitToArchive");
    }

    // 尝试不同的路径格式查找 subdir
    let subdir_record_backslash =
        directory_dao.find_directory_by_path(root.id.unwrap(), "dir1\\subdir")?;
    let subdir_record_slash =
        directory_dao.find_directory_by_path(root.id.unwrap(), "dir1/subdir")?;

    let subdir_record = subdir_record_backslash.or(subdir_record_slash);
    assert!(subdir_record.is_some(), "dir1/subdir 应该存在于数据库中");
    if let Some(ref dir) = subdir_record {
        assert_eq!(dir.status, DirectoryStatus::WaitToArchive, "subdir 状态应该是 WaitToArchive");
    }

    let view_file_dao = ViewFileDao::new(database.conn.clone());
    // 验证特定文件存在
    if let Some(dir1_info) = dir1_record {
        let files_in_dir1 = view_file_dao.find_view_files_by_directory_id(dir1_info.id.unwrap())?;
        assert_eq!(files_in_dir1.len(), 1, "dir1 中应该有 1 个文件");
        assert_eq!(files_in_dir1[0].file_name, "file2.txt");
        assert_eq!(
            files_in_dir1[0].file_status,
            FileStatus::WaitToArchive.as_str(),
            "file2.txt 状态应该是 WaitToArchive"
        );
    }

    // 验证根目录中的文件
    let root_dir = directory_dao.find_directory_by_path(root.id.unwrap(), "")?;
    assert!(root_dir.is_some(), "根目录应该存在于数据库中");

    if let Some(root_dir_info) = root_dir {
        let files_in_root =
            view_file_dao.find_view_files_by_directory_id(root_dir_info.id.unwrap())?;
        assert_eq!(files_in_root.len(), 1, "根目录中应该有 1 个文件");
        assert_eq!(files_in_root[0].file_name, "file1.txt");
        assert_eq!(
            files_in_root[0].file_status,
            FileStatus::WaitToArchive.as_str(),
            "file1.txt 状态应该是 WaitToArchive"
        );
    }

    Ok(())
}

/// 测试目录统计功能
#[test]
fn test_get_directory_statistics() -> Result<(), Box<dyn std::error::Error>> {
    let _ = simple_logger::SimpleLogger::new().with_level(log::LevelFilter::Debug).init();

    let env = TestEnvironment::new()?;
    create_basic_test_structure(&env.source_dir)?;

    let scan_service = ScanServices::new();
    let stats = scan_service.get_directory_statistics(&env.source_dir)?;

    assert_directory_statistics(&stats, 13 + 15 + 23, 4, 3, 3, "基本目录统计");

    println!("目录统计信息测试通过！");
    Ok(())
}

/// 测试空目录统计功能
#[test]
fn test_get_directory_statistics_empty_dir() -> Result<(), Box<dyn std::error::Error>> {
    let _ = simple_logger::SimpleLogger::new().with_level(log::LevelFilter::Debug).init();

    let env = TestEnvironment::new()?;

    let scan_service = ScanServices::new();
    let stats = scan_service.get_directory_statistics(&env.source_dir)?;

    assert_directory_statistics(&stats, 0, 1, 0, 0, "空目录统计");

    println!("空目录统计信息测试通过！");
    Ok(())
}

/// 测试嵌套目录结构统计功能
#[test]
fn test_get_directory_statistics_nested_structure() -> Result<(), Box<dyn std::error::Error>> {
    let _ = simple_logger::SimpleLogger::new().with_level(log::LevelFilter::Debug).init();

    let env = TestEnvironment::new()?;
    create_nested_test_structure(&env.source_dir)?;

    let scan_service = ScanServices::new();
    let stats = scan_service.get_directory_statistics(&env.source_dir)?;

    assert_directory_statistics(&stats, 4 + 6 + 6 + 6, 4, 4, 4, "嵌套目录统计");

    println!("嵌套目录统计信息测试通过！");
    Ok(())
}

/// 测试首次扫描功能
#[test]
fn test_scan_and_save_directory_with_events() -> Result<(), Box<dyn std::error::Error>> {
    let _ = simple_logger::SimpleLogger::new().with_level(log::LevelFilter::Debug).init();

    let env = TestEnvironment::new()?;
    create_basic_test_structure(&env.source_dir)?;

    let scan_service = ScanServices::new();
    let result = scan_service.scan_and_save_directory_with_events(
        &env.source_dir,
        &env.database,
        Some(progress_callback),
    );

    assert!(result.is_ok(), "扫描目录应该成功：{:?}", result.err());
    assert_scan_results(&env.database, &env.source_dir)?;

    println!("首次扫描测试通过！");
    Ok(())
}

/// 测试增量扫描功能
#[test]
fn test_incremental_scan_and_save_directory_with_events() -> Result<(), Box<dyn std::error::Error>>
{
    let _ = simple_logger::SimpleLogger::new().with_level(log::LevelFilter::Debug).init();

    let env = TestEnvironment::new()?;
    create_basic_test_structure(&env.source_dir)?;

    let scan_service = ScanServices::new();

    // 首次扫描
    let result1 = scan_service.scan_and_save_directory_with_events(
        &env.source_dir,
        &env.database,
        Some(progress_callback),
    );
    assert!(result1.is_ok(), "首次扫描应该成功");

    let root_dao = InfoRootDao::new(env.database.conn.clone());
    let roots = root_dao.find_all_root_info()?;
    let root_id = roots[0].id.unwrap();

    // 修改文件内容以测试文件更新
    let file2_path = env.source_dir.join("dir1").join("file2.txt");
    fs::write(&file2_path, b"This is file 2 updated")?;

    // 添加新文件和目录
    let new_dir = env.source_dir.join("new_dir");
    fs::create_dir_all(&new_dir)?;
    let new_file_path = new_dir.join("new_file.txt");
    fs::write(&new_file_path, b"This is a new file")?;

    // 删除一个文件
    let file3_path = env.source_dir.join("dir1").join("subdir").join("file3.txt");
    fs::remove_file(&file3_path)?;

    debug!("开始第二次扫描");
    // 第二次扫描（增量扫描）
    let result2 = scan_service.scan_and_save_directory_with_events(
        &env.source_dir,
        &env.database,
        Some(progress_callback),
    );
    if let Err(e) = &result2 {
        eprintln!("第二次扫描失败，详细错误信息：{:?}", e);
        eprintln!("错误链详情：{:#}", e);
    }
    assert!(result2.is_ok(), "第二次扫描应该成功");

    // 验证根目录状态
    let roots = root_dao.find_all_root_info()?;
    assert_eq!(roots.len(), 1, "应该只有一个根目录");
    assert_eq!(roots[0].status, RootStatus::WaitToArchive, "根目录状态应该是 WaitToArchive");

    let directory_dao = InfoDirectoryDao::new(env.database.conn.clone());
    // 验证目录数量和状态
    let directories = directory_dao.find_by_status_and_root_id(root_id, None)?;
    assert_eq!(directories.len(), 5, "应该有 5 个目录 (dir1, dir2, subdir, new_dir, 根目录)");

    // 验证目录状态
    for dir in &directories {
        assert_eq!(dir.status, DirectoryStatus::WaitToArchive, "所有目录状态应该是 WaitToArchive");
    }

    let file_dao = InfoFileDao::new(env.database.conn.clone());
    // 验证文件数量和状态（获取所有非 WaitToDelete 状态的文件）
    let all_files = file_dao.find_files_by_status_and_root_id(root_id, None)?;
    let active_files: Vec<&InfoFile> =
        all_files.iter().filter(|f| f.status != FileStatus::WaitToDelete).collect();
    assert_eq!(
        active_files.len(),
        3,
        "应该有 3 个有效文件 (file1.txt, file2.txt[更新], new_file.txt)"
    );

    // 查找特定文件并验证其状态
    let file1 = all_files.iter().find(|f| f.file_name == "file1.txt").unwrap();
    let file2 = all_files.iter().find(|f| f.file_name == "file2.txt").unwrap();
    let file3 = all_files.iter().find(|f| f.file_name == "file3.txt").unwrap();
    let new_file = all_files.iter().find(|f| f.file_name == "new_file.txt").unwrap();

    // file1.txt 应该保持 WaitToArchive 状态（根据新的设计，所有扫描的文件都标记为 WaitToArchive）
    assert_eq!(file1.status, FileStatus::WaitToArchive, "file1.txt 应该是 WaitToArchive 状态");

    // file2.txt 应该是 WaitToArchive 状态（内容更新）
    assert_eq!(
        file2.status,
        FileStatus::WaitToArchive,
        "file2.txt 应该是 WaitToArchive 状态（内容已更新）"
    );

    // new_file.txt 应该是 WaitToArchive 状态（新文件）
    assert_eq!(
        new_file.status,
        FileStatus::WaitToArchive,
        "new_file.txt 应该是 WaitToArchive 状态（新文件）"
    );

    // file3.txt 应该是 WaitToDelete 状态（已被删除）
    assert_eq!(
        file3.status,
        FileStatus::WaitToDelete,
        "file3.txt 应该是 WaitToDelete 状态（已被删除）"
    );

    // 验证特定目录存在
    let dir1_record = directory_dao.find_directory_by_path(root_id, "dir1")?;
    assert!(dir1_record.is_some(), "dir1 应该存在于数据库中");

    let new_dir_record = directory_dao.find_directory_by_path(root_id, "new_dir")?;
    assert!(new_dir_record.is_some(), "new_dir 应该存在于数据库中");

    let view_file_dao = ViewFileDao::new(env.database.conn.clone());
    // 验证新目录中的文件
    if let Some(new_dir_info) = new_dir_record {
        let files_in_new_dir =
            view_file_dao.find_view_files_by_directory_id(new_dir_info.id.unwrap())?;
        assert_eq!(files_in_new_dir.len(), 1, "new_dir 中应该有 1 个文件");
        assert_eq!(files_in_new_dir[0].file_name, "new_file.txt");
        assert_eq!(
            files_in_new_dir[0].file_status,
            FileStatus::WaitToArchive.as_str(),
            "new_file.txt 状态应该是 WaitToArchive"
        );
    }

    println!("增量扫描测试通过！");
    Ok(())
}
/// 测试多个独立根目录处理
#[test]
fn test_multiple_independent_roots() -> Result<(), Box<dyn std::error::Error>> {
    let _ = simple_logger::SimpleLogger::new().with_level(log::LevelFilter::Debug).init();

    let env1 = TestEnvironment::new()?;
    let env2 = TestEnvironment::new()?;

    create_multiple_root_test_structures(&env1.source_dir, &env2.source_dir)?;

    let scan_service = ScanServices::new();

    // 扫描第一个根目录
    let result1 = scan_service.scan_and_save_directory_with_events(
        &env1.source_dir,
        &env1.database,
        Some(progress_callback),
    );
    assert!(result1.is_ok(), "第一个根目录扫描应该成功");

    // 扫描第二个根目录
    let result2 = scan_service.scan_and_save_directory_with_events(
        &env2.source_dir,
        &env1.database, // 使用相同的数据库来测试隔离性
        Some(progress_callback),
    );
    assert!(result2.is_ok(), "第二个根目录扫描应该成功");

    // 验证数据库中的记录
    let root_dao = InfoRootDao::new(env1.database.conn.clone());
    let roots = root_dao.find_all_root_info()?;
    assert_eq!(roots.len(), 2, "应该有两个根目录");

    // 验证两个根目录状态都为 WaitToArchive
    for root in &roots {
        assert_eq!(root.status, RootStatus::WaitToArchive, "所有根目录状态应该是 WaitToArchive");
    }

    let directory_dao = InfoDirectoryDao::new(env1.database.conn.clone());
    let file_dao = InfoFileDao::new(env1.database.conn.clone());
    // 验证每个根目录的目录和文件记录
    for root in &roots {
        let directories =
            directory_dao.find_by_status_and_root_id(root.id.unwrap(), None)?;
        // 每个根目录应该有 2 个目录（自身根目录和一个子目录）
        assert_eq!(directories.len(), 2, "每个根目录应该有 2 个目录记录");

        // 验证目录状态
        for dir in &directories {
            assert_eq!(
                dir.status,
                DirectoryStatus::WaitToArchive,
                "所有目录状态应该是 WaitToArchive"
            );
        }

        let files = file_dao.find_files_by_status_and_root_id(root.id.unwrap(), None)?;
        // 每个根目录应该有 1 个文件
        assert_eq!(files.len(), 1, "每个根目录应该有 1 个文件记录");

        // 验证文件状态
        for file in &files {
            assert_eq!(file.status, FileStatus::WaitToArchive, "所有文件状态应该是 WaitToArchive");
        }
    }

    println!("多根目录测试通过！");
    Ok(())
}
