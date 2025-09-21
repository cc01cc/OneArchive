//! 通用测试框架
//! 提供统一的测试环境设置和常用测试工具函数

use log::info;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

use one_archive_lib::mod_database::database::Database;
use one_archive_lib::mod_database::trait_database::InitializationOperations;

/// 通用测试环境结构体
pub struct TestEnvironment {
    pub temp_dir: TempDir,
    pub database: Database,
    pub source_dir: std::path::PathBuf,
    pub archive_dir: std::path::PathBuf,
    pub extract_dir: std::path::PathBuf,
}

impl TestEnvironment {
    /// 创建一个新的测试环境
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;

        let db_file = temp_dir.path().join("test_archive.sqlite");
        let db_url = format!("file:{}", db_file.display());
        let database = Database::new(&db_url)?;
        database.initialize_tables(&database.conn)?;

        let source_dir = temp_dir.path().join("test-source");
        let archive_dir = temp_dir.path().join("test-archive");
        let extract_dir = temp_dir.path().join("test-extract");

        fs::create_dir_all(&source_dir)?;
        fs::create_dir_all(&archive_dir)?;
        fs::create_dir_all(&extract_dir)?;

        Ok(TestEnvironment {
            temp_dir,
            database,
            source_dir,
            archive_dir,
            extract_dir,
        })
    }
}

/// 初始化日志记录器
pub fn init_logger() {
    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
        .try_init();
}

/// 创建测试文件
pub fn create_test_file(path: &Path, content: &str) -> Result<(), Box<dyn std::error::Error>> {
    fs::write(path, content)?;
    Ok(())
}

/// 创建基本测试文件（test1.txt, test2.txt）
pub fn create_basic_test_files(dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    create_test_file(&dir.join("test1.txt"), "This is test file 1 content")?;
    create_test_file(&dir.join("test2.txt"), "This is test file 2 content")?;
    Ok(())
}

/// 创建大文件
pub fn create_large_file(path: &Path, size: usize) -> Result<(), Box<dyn std::error::Error>> {
    let content = "A".repeat(size);
    fs::write(path, content)?;
    Ok(())
}

/// 创建空文件
pub fn create_empty_file(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    fs::File::create(path)?;
    Ok(())
}

/// 创建带子目录的测试文件结构
pub fn create_test_files_with_subdir(dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    create_basic_test_files(dir)?;

    let subdir = dir.join("subdir");
    fs::create_dir_all(&subdir)?;
    create_test_file(
        &subdir.join("subfile.txt"),
        "This is a file in subdirectory",
    )?;
    Ok(())
}
/// 创建基本测试目录结构
pub fn create_basic_test_structure(
    root_path: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let dir1 = root_path.join("dir1");
    let dir2 = root_path.join("dir2");
    let subdir = dir1.join("subdir");

    fs::create_dir_all(&dir1)?;
    fs::create_dir_all(&dir2)?;
    fs::create_dir_all(&subdir)?;

    let file1_path = root_path.join("file1.txt");
    let file2_path = dir1.join("file2.txt");
    let file3_path = subdir.join("file3.txt");

    fs::write(&file1_path, b"Hello, world!")?; // 13 bytes
    fs::write(&file2_path, b"This is file 2")?; // 15 bytes
    fs::write(&file3_path, b"This is file 3 in subdir")?; // 23 bytes

    Ok(())
}
/// 创建嵌套目录结构
pub fn create_nested_test_structure(
    root_path: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let level1_dir = root_path.join("level1");
    let level2_dir = level1_dir.join("level2");
    let level3_dir = level2_dir.join("level3");

    fs::create_dir_all(&level3_dir)?;

    let root_file = root_path.join("root_file.txt");
    let level1_file = level1_dir.join("level1_file.txt");
    let level2_file = level2_dir.join("level2_file.txt");
    let level3_file = level3_dir.join("level3_file.txt");

    fs::write(&root_file, b"root")?; // 4 bytes
    fs::write(&level1_file, b"level1")?; // 6 bytes
    fs::write(&level2_file, b"level2")?; // 6 bytes
    fs::write(&level3_file, b"level3")?; // 6 bytes

    Ok(())
}

/// 创建多个根目录测试结构
pub fn create_multiple_root_test_structures(
    root_path1: &Path,
    root_path2: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let dir1 = root_path1.join("dir1");
    fs::create_dir_all(&dir1)?;
    create_test_file(&dir1.join("file1.txt"), "File in first root")?;

    let dir2 = root_path2.join("dir2");
    fs::create_dir_all(&dir2)?;
    create_test_file(&dir2.join("file2.txt"), "File in second root")?;

    Ok(())
}
