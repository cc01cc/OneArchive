use log::info;
use std::fs;
use std::io::Write;
use std::path::Path;
use tempfile::TempDir;

use one_archive_lib::archive::services::{ScanProgress, scan_and_save_directory_with_events};
use one_archive_lib::database::Database;

// 进度回调函数
fn progress_callback(progress: ScanProgress) {
    println!("Progress: {:.2}% - {}", progress.progress, progress.message);
}

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
    Database::initialize_tables(&database.conn)?;

    // 执行扫描和保存操作
    let result = scan_and_save_directory_with_events(root_path, &database, Some(progress_callback));

    // 验证结果
    // 修改这里，让错误信息能够正确显示
    match result {
        Ok(_) => {
            // 继续执行后续验证
        }
        Err(e) => {
            eprintln!("扫描目录失败: {}", e);
            return Err(e);
        }
    }
    assert!(result.is_ok(), "扫描目录应该成功");

    // 验证数据库中的记录
    let roots = database.find_all_root_info()?;
    assert_eq!(roots.len(), 1, "应该只有一个根目录");

    let root = &roots[0];
    assert_eq!(root.root_path, root_path.canonicalize()?.to_string_lossy());

    // 验证目录数量
    let directories = database.find_directories_by_status_and_root_id(root.id.unwrap(), None)?;
    assert_eq!(
        directories.len(),
        4,
        "应该有4个目录 (根目录, dir1, dir2, subdir)"
    );

    // 验证文件数量
    let files = database.find_files_by_status_and_root_id(root.id.unwrap(), None)?;
    assert_eq!(files.len(), 3, "应该有3个文件");

    // 验证特定目录存在
    let dir1_record = database.find_directory_by_path(root.id.unwrap(), "dir1")?;
    assert!(dir1_record.is_some(), "dir1应该存在于数据库中");

    let all_directories =
        database.find_directories_by_status_and_root_id(root.id.unwrap(), None)?;
    info!("数据库中的所有目录:");
    for dir in &all_directories {
        info!("  ID: {:?}, 路径: {:?}", dir.id, dir.directory_path);
    }
    // 尝试不同的路径格式查找subdir
    let subdir_record_backslash =
        database.find_directory_by_path(root.id.unwrap(), "dir1\\subdir")?;
    let subdir_record_slash = database.find_directory_by_path(root.id.unwrap(), "dir1/subdir")?;

    let subdir_record = subdir_record_backslash.or(subdir_record_slash);
    assert!(subdir_record.is_some(), "dir1/subdir应该存在于数据库中");

    // 验证特定文件存在
    if let Some(dir1_info) = dir1_record {
        let files_in_dir1 = database.find_view_files_by_directory_id(dir1_info.id.unwrap())?;
        assert_eq!(files_in_dir1.len(), 1, "dir1中应该有1个文件");
        assert_eq!(files_in_dir1[0].file_name, "file2.txt");
    }

    // 验证根目录中的文件
    let root_dir = database.find_directory_by_path(root.id.unwrap(), "")?;
    assert!(root_dir.is_some(), "根目录应该存在于数据库中");

    if let Some(root_dir_info) = root_dir {
        let files_in_root = database.find_view_files_by_directory_id(root_dir_info.id.unwrap())?;
        assert_eq!(files_in_root.len(), 1, "根目录中应该有1个文件");
        assert_eq!(files_in_root[0].file_name, "file1.txt");
    }

    println!("测试通过!");
    Ok(())
}
