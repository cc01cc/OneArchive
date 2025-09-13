use log::info;
use one_archive_lib::mod_scan::impl_scan::ScanServices;
use one_archive_lib::mod_scan::model_scan::ScanProgress;
use one_archive_lib::mod_scan::trait_scan::{DirectoryScanOperations, DirectoryStatisticsOperations};
use std::fs;
use std::io::Write;
use tempfile::TempDir;

use one_archive_lib::mod_database::constants::{DirectoryStatus, FileStatus, RootStatus};
use one_archive_lib::mod_database::database::Database;
use one_archive_lib::mod_database::schema::InfoFile;
use one_archive_lib::mod_database::trait_database::{
    DirectoryOperations, FileOperations, InitializationOperations, RootOperations, ViewOperations,
};

// 进度回调函数
fn progress_callback(progress: ScanProgress) {
    println!("Progress: {:.2}% - {}", progress.progress, progress.message);
}

/**
 * 测试目录统计功能
   - 基本目录结构统计
   - 空目录统计
   - 嵌套目录结构统计
 */
#[test]
fn test_get_directory_statistics() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志记录器
    let _ = simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Debug)
        .init();

    // 创建临时目录用于测试
    let temp_dir = TempDir::new()?;
    let root_path = temp_dir.path();

    // 创建测试目录结构
    let dir1 = root_path.join("dir1");
    let dir2 = root_path.join("dir2");
    let subdir = dir1.join("subdir");

    fs::create_dir_all(&dir1)?;
    fs::create_dir_all(&dir2)?;
    fs::create_dir_all(&subdir)?;

    // 创建测试文件
    let file1_path = root_path.join("file1.txt");
    let file2_path = dir1.join("file2.txt");
    let file3_path = subdir.join("file3.txt");

    // 写入文件内容
    let mut file1 = fs::File::create(&file1_path)?;
    file1.write_all(b"Hello, world!")?; // 13 bytes

    let mut file2 = fs::File::create(&file2_path)?;
    file2.write_all(b"This is file 2")?; // 15 bytes

    let mut file3 = fs::File::create(&file3_path)?;
    file3.write_all(b"This is file 3 in subdir")?; // 23 bytes

    // 创建归档服务实例
    let scan_service = ScanServices::new();

    // 获取目录统计信息
    let stats = scan_service.get_directory_statistics(root_path)?;

    // 验证统计结果
    assert_eq!(
        stats.total_size,
        13 + 15 + 23,
        "总大小应该等于所有文件大小之和"
    );
    assert_eq!(
        stats.directory_count, 4,
        "应该有 4 个目录 (dir1, dir2, subdir, 根目录)"
    );
    assert_eq!(stats.file_count, 3, "应该有 3 个文件");
    assert_eq!(
        stats.max_depth, 3,
        "最大深度应该是 3 (根目录->dir1->subdir)"
    );

    println!("目录统计信息测试通过！");
    Ok(())
}

#[test]
fn test_get_directory_statistics_empty_dir() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志记录器
    let _ = simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Debug)
        .init();

    // 创建空的临时目录用于测试
    let temp_dir = TempDir::new()?;
    let root_path = temp_dir.path();

    // 创建归档服务实例
    let scan_service = ScanServices::new();

    // 获取目录统计信息
    let stats = scan_service.get_directory_statistics(root_path)?;

    // 验证统计结果
    assert_eq!(stats.total_size, 0, "空目录总大小应该为 0");
    assert_eq!(stats.directory_count, 1, "应该只有根目录");
    assert_eq!(stats.file_count, 0, "应该没有文件");
    assert_eq!(stats.max_depth, 0, "空目录最大深度应该为 0");

    println!("空目录统计信息测试通过！");
    Ok(())
}

#[test]
fn test_get_directory_statistics_nested_structure() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志记录器
    let _ = simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Debug)
        .init();

    // 创建临时目录用于测试
    let temp_dir = TempDir::new()?;
    let root_path = temp_dir.path();

    // 创建嵌套目录结构
    let level1_dir = root_path.join("level1");
    let level2_dir = level1_dir.join("level2");
    let level3_dir = level2_dir.join("level3");

    fs::create_dir_all(&level3_dir)?;

    // 创建测试文件
    let root_file = root_path.join("root_file.txt");
    let level1_file = level1_dir.join("level1_file.txt");
    let level2_file = level2_dir.join("level2_file.txt");
    let level3_file = level3_dir.join("level3_file.txt");

    fs::File::create(&root_file)?.write_all(b"root")?; // 4 bytes
    fs::File::create(&level1_file)?.write_all(b"level1")?; // 6 bytes
    fs::File::create(&level2_file)?.write_all(b"level2")?; // 6 bytes
    fs::File::create(&level3_file)?.write_all(b"level3")?; // 6 bytes

    // 创建归档服务实例
    let scan_service = ScanServices::new();

    // 获取目录统计信息
    let stats = scan_service.get_directory_statistics(root_path)?;

    // 验证统计结果
    assert_eq!(
        stats.total_size,
        4 + 6 + 6 + 6,
        "总大小应该等于所有文件大小之和"
    );
    assert_eq!(
        stats.directory_count, 4,
        "应该有 4 个目录 (level1, level2, level3, 根目录)"
    );
    assert_eq!(stats.file_count, 4, "应该有 4 个文件");
    assert_eq!(
        stats.max_depth, 4,
        "最大深度应该是 4 (根目录->level1->level2->level3)"
    );

    println!("嵌套目录统计信息测试通过！");
    Ok(())
}
/**
 * 测试首次扫描功能
   - 验证新创建记录的状态为 WaitToArchive
   - 验证数据库记录正确性
 */
#[test]
fn test_scan_and_save_directory_with_events() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志记录器
    let _ = simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Debug)
        .init();

    // 创建临时目录用于测试
    let temp_dir = TempDir::new()?;
    let root_path = temp_dir.path();

    // 创建测试目录结构
    let dir1 = root_path.join("dir1");
    let dir2 = root_path.join("dir2");
    let subdir = dir1.join("subdir");

    fs::create_dir_all(&dir1)?;
    fs::create_dir_all(&dir2)?;
    fs::create_dir_all(&subdir)?;

    // 创建测试文件
    let file1_path = root_path.join("file1.txt");
    let file2_path = dir1.join("file2.txt");
    let file3_path = subdir.join("file3.txt");

    // 写入文件内容
    let mut file1 = fs::File::create(&file1_path)?;
    file1.write_all(b"Hello, world!")?;

    let mut file2 = fs::File::create(&file2_path)?;
    file2.write_all(b"This is file 2")?;

    let mut file3 = fs::File::create(&file3_path)?;
    file3.write_all(b"This is file 3 in subdir")?;

    // 创建另一个临时目录用于存放数据库文件
    let temp_db_dir = TempDir::new()?;

    // 创建临时数据库
    let db_file = temp_db_dir.path().join("test.db");
    let database = Database::new(&db_file)?;
    info!("Created database at {}", db_file.display());

    // 创建数据库
    database.initialize_tables(&database.conn)?;

    // 执行扫描和保存操作
    let scan_service = ScanServices::new();
    let result = scan_service.scan_and_save_directory_with_events(
        root_path,
        &database,
        Some(progress_callback),
    );

    // 验证结果
    // 修改这里，让错误信息能够正确显示
    match &result {
        Ok(_) => {
            // 继续执行后续验证
        }
        Err(e) => {
            eprintln!("扫描目录失败：{}", e);
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("{:?}", e),
            )));
        }
    }
    assert!(result.is_ok(), "扫描目录应该成功");

    // 验证数据库中的记录
    let roots = database.find_all_root_info()?;
    assert_eq!(roots.len(), 1, "应该只有一个根目录");

    let root = &roots[0];
    // 处理 Windows 长路径前缀问题，统一移除可能存在的 \\?\ 前缀
    let stored_path = root.root_path.trim_start_matches(r"\\?\");
    let canonical_path = root_path.canonicalize()?;
    let actual_path = canonical_path.to_string_lossy();
    assert_eq!(stored_path, actual_path.trim_start_matches(r"\\?\"));

    // 验证根目录状态（首次扫描应为 WaitToArchive）
    assert_eq!(
        root.status,
        RootStatus::WaitToArchive,
        "根目录状态应该是 WaitToArchive"
    );

    // 验证目录数量
    let directories = database.find_directories_by_status_and_root_id(root.id.unwrap(), None)?;
    assert_eq!(
        directories.len(),
        4,
        "应该有 4 个目录 (dir1, dir2, subdir, 根目录)"
    );

    // 验证每个目录的状态（首次扫描应为 WaitToArchive）
    for dir in &directories {
        assert_eq!(
            dir.status,
            DirectoryStatus::WaitToArchive,
            "所有目录状态应该是 WaitToArchive"
        );
    }

    // 验证文件数量
    let files = database.find_files_by_status_and_root_id(root.id.unwrap(), None)?;
    assert_eq!(files.len(), 3, "应该有 3 个文件");

    // 验证每个文件的状态（首次扫描应为 WaitToArchive）
    for file in &files {
        assert_eq!(
            file.status,
            FileStatus::WaitToArchive,
            "所有文件状态应该是 WaitToArchive"
        );
    }

    // 验证特定目录存在
    let dir1_record = database.find_directory_by_path(root.id.unwrap(), "dir1")?;
    assert!(dir1_record.is_some(), "dir1 应该存在于数据库中");
    if let Some(ref dir) = dir1_record {
        assert_eq!(
            dir.status,
            DirectoryStatus::WaitToArchive,
            "dir1 状态应该是 WaitToArchive"
        );
    }

    // 尝试不同的路径格式查找 subdir
    let subdir_record_backslash =
        database.find_directory_by_path(root.id.unwrap(), "dir1\\subdir")?;
    let subdir_record_slash = database.find_directory_by_path(root.id.unwrap(), "dir1/subdir")?;

    let subdir_record = subdir_record_backslash.or(subdir_record_slash);
    assert!(subdir_record.is_some(), "dir1/subdir 应该存在于数据库中");
    if let Some(ref dir) = subdir_record {
        assert_eq!(
            dir.status,
            DirectoryStatus::WaitToArchive,
            "subdir 状态应该是 WaitToArchive"
        );
    }

    // 验证特定文件存在
    if let Some(dir1_info) = dir1_record {
        let files_in_dir1 = database.find_view_files_by_directory_id(dir1_info.id.unwrap())?;
        assert_eq!(files_in_dir1.len(), 1, "dir1 中应该有 1 个文件");
        assert_eq!(files_in_dir1[0].file_name, "file2.txt");
        assert_eq!(
            files_in_dir1[0].file_status,
            FileStatus::WaitToArchive.as_str(),
            "file2.txt 状态应该是 WaitToArchive"
        );
    }

    // 验证根目录中的文件
    let root_dir = database.find_directory_by_path(root.id.unwrap(), "")?;
    assert!(root_dir.is_some(), "根目录应该存在于数据库中");

    if let Some(root_dir_info) = root_dir {
        let files_in_root = database.find_view_files_by_directory_id(root_dir_info.id.unwrap())?;
        assert_eq!(files_in_root.len(), 1, "根目录中应该有 1 个文件");
        assert_eq!(files_in_root[0].file_name, "file1.txt");
        assert_eq!(
            files_in_root[0].file_status,
            FileStatus::WaitToArchive.as_str(),
            "file1.txt 状态应该是 WaitToArchive"
        );
    }

    println!("首次扫描测试通过！");
    Ok(())
}

/**
 * 测试增量扫描功能
   - 文件修改检测
   - 新增文件和目录处理
   - 文件删除标记处理
   - 状态正确性验证
 */
#[test]
fn test_incremental_scan_and_save_directory_with_events() -> Result<(), Box<dyn std::error::Error>>
{
    // 初始化日志记录器
    let _ = simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Debug)
        .init();

    // 创建临时目录用于测试
    let temp_dir = TempDir::new()?;
    let root_path = temp_dir.path();

    // 创建测试目录结构
    let dir1 = root_path.join("dir1");
    let dir2 = root_path.join("dir2");
    let subdir = dir1.join("subdir");

    fs::create_dir_all(&dir1)?;
    fs::create_dir_all(&dir2)?;
    fs::create_dir_all(&subdir)?;

    // 创建测试文件
    let file1_path = root_path.join("file1.txt");
    let file2_path = dir1.join("file2.txt");
    let file3_path = subdir.join("file3.txt");

    // 写入文件内容
    let mut file1 = fs::File::create(&file1_path)?;
    file1.write_all(b"Hello, world!")?;

    let mut file2 = fs::File::create(&file2_path)?;
    file2.write_all(b"This is file 2")?;

    let mut file3 = fs::File::create(&file3_path)?;
    file3.write_all(b"This is file 3 in subdir")?;

    // 创建另一个临时目录用于存放数据库文件
    let temp_db_dir = TempDir::new()?;

    // 创建临时数据库
    let db_file = temp_db_dir.path().join("test.db");
    let database = Database::new(&db_file)?;
    info!("Created database at {}", db_file.display());

    // 创建数据库
    database.initialize_tables(&database.conn)?;

    // 创建归档服务实例
    let scan_service = ScanServices::new();

    // 首次扫描
    let result1 = scan_service.scan_and_save_directory_with_events(
        root_path,
        &database,
        Some(progress_callback),
    );
    assert!(result1.is_ok(), "首次扫描应该成功");

    // 验证首次扫描后的状态
    let roots = database.find_all_root_info()?;
    let root_id = roots[0].id.unwrap();

    // 修改文件内容以测试文件更新
    fs::write(&file2_path, b"This is file 2 updated")?;

    // 添加新文件和目录
    let new_dir = root_path.join("new_dir");
    fs::create_dir_all(&new_dir)?;
    let new_file_path = new_dir.join("new_file.txt");
    fs::write(&new_file_path, b"This is a new file")?;

    // 删除一个文件
    fs::remove_file(&file3_path)?;

    // 第二次扫描（增量扫描）
    let result2 = scan_service.scan_and_save_directory_with_events(
        root_path,
        &database,
        Some(progress_callback),
    );
    assert!(result2.is_ok(), "第二次扫描应该成功");

    // 验证根目录状态
    let roots = database.find_all_root_info()?;
    assert_eq!(roots.len(), 1, "应该只有一个根目录");
    assert_eq!(
        roots[0].status,
        RootStatus::WaitToArchive,
        "根目录状态应该是 WaitToArchive"
    );

    // 验证目录数量和状态
    let directories = database.find_directories_by_status_and_root_id(root_id, None)?;
    assert_eq!(
        directories.len(),
        5,
        "应该有 5 个目录 (dir1, dir2, subdir, new_dir, 根目录)"
    );

    // 验证目录状态
    for dir in &directories {
        // 所有目录都应该处于 WaitToArchive 状态（新扫描的）
        assert_eq!(
            dir.status,
            DirectoryStatus::WaitToArchive,
            "所有目录状态应该是 WaitToArchive"
        );
    }

    // 验证文件数量和状态（获取所有非 WaitToDelete 状态的文件）
    let all_files = database.find_files_by_status_and_root_id(root_id, None)?;
    let active_files: Vec<&InfoFile> = all_files
        .iter()
        .filter(|f| f.status != FileStatus::WaitToDelete)
        .collect();
    assert_eq!(
        active_files.len(),
        3,
        "应该有 3 个有效文件 (file1.txt, file2.txt[更新], new_file.txt)"
    );

    // 查找特定文件并验证其状态
    let file1 = all_files
        .iter()
        .find(|f| f.file_name == "file1.txt")
        .unwrap();
    let file2 = all_files
        .iter()
        .find(|f| f.file_name == "file2.txt")
        .unwrap();
    let file3 = all_files
        .iter()
        .find(|f| f.file_name == "file3.txt")
        .unwrap();
    let new_file = all_files
        .iter()
        .find(|f| f.file_name == "new_file.txt")
        .unwrap();

    // file1.txt 应该保持 WaitToArchive 状态（根据新的设计，所有扫描的文件都标记为 WaitToArchive）
    assert_eq!(
        file1.status,
        FileStatus::WaitToArchive,
        "file1.txt 应该是 WaitToArchive 状态"
    );

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
    let dir1_record = database.find_directory_by_path(root_id, "dir1")?;
    assert!(dir1_record.is_some(), "dir1 应该存在于数据库中");

    let new_dir_record = database.find_directory_by_path(root_id, "new_dir")?;
    assert!(new_dir_record.is_some(), "new_dir 应该存在于数据库中");

    // 验证新目录中的文件
    if let Some(new_dir_info) = new_dir_record {
        let files_in_new_dir =
            database.find_view_files_by_directory_id(new_dir_info.id.unwrap())?;
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

/**
 * 测试多个独立根目录处理
   - 验证多个根目录可以正确共存
   - 验证各根目录的数据隔离性
 */
#[test]
fn test_multiple_independent_roots() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志记录器
    let _ = simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Debug)
        .init();

    // 创建两个独立的临时目录用于测试
    let temp_dir1 = TempDir::new()?;
    let root_path1 = temp_dir1.path();

    let temp_dir2 = TempDir::new()?;
    let root_path2 = temp_dir2.path();

    // 在第一个目录中创建测试结构
    let dir1 = root_path1.join("dir1");
    fs::create_dir_all(&dir1)?;
    let file1_path = dir1.join("file1.txt");
    fs::write(&file1_path, b"File in first root")?;

    // 在第二个目录中创建测试结构
    let dir2 = root_path2.join("dir2");
    fs::create_dir_all(&dir2)?;
    let file2_path = dir2.join("file2.txt");
    fs::write(&file2_path, b"File in second root")?;

    // 创建临时数据库
    let temp_db_dir = TempDir::new()?;
    let db_file = temp_db_dir.path().join("test.db");
    let database = Database::new(&db_file)?;
    info!("Created database at {}", db_file.display());

    // 创建数据库
    database.initialize_tables(&database.conn)?;

    // 创建归档服务实例
    let scan_service = ScanServices::new();

    // 扫描第一个根目录
    let result1 = scan_service.scan_and_save_directory_with_events(
        root_path1,
        &database,
        Some(progress_callback),
    );
    assert!(result1.is_ok(), "第一个根目录扫描应该成功");

    // 扫描第二个根目录
    let result2 = scan_service.scan_and_save_directory_with_events(
        root_path2,
        &database,
        Some(progress_callback),
    );
    assert!(result2.is_ok(), "第二个根目录扫描应该成功");

    // 验证数据库中的记录
    let roots = database.find_all_root_info()?;
    assert_eq!(roots.len(), 2, "应该有两个根目录");

    // 验证两个根目录状态都为 WaitToArchive
    for root in &roots {
        assert_eq!(
            root.status,
            RootStatus::WaitToArchive,
            "所有根目录状态应该是 WaitToArchive"
        );
    }

    // 验证每个根目录的目录和文件记录
    for root in &roots {
        let directories =
            database.find_directories_by_status_and_root_id(root.id.unwrap(), None)?;
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

        let files = database.find_files_by_status_and_root_id(root.id.unwrap(), None)?;
        // 每个根目录应该有 1 个文件
        assert_eq!(files.len(), 1, "每个根目录应该有 1 个文件记录");

        // 验证文件状态
        for file in &files {
            assert_eq!(
                file.status,
                FileStatus::WaitToArchive,
                "所有文件状态应该是 WaitToArchive"
            );
        }
    }

    println!("多根目录测试通过！");
    Ok(())
}
