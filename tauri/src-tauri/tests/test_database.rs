use tempfile::TempDir;
use one_archive_lib::mod_database::database::Database;
use one_archive_lib::mod_database::trait_database::{RootOperations, InitializationOperations};

#[test]
fn test_database_operations() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    let _ = simple_logger::SimpleLogger::new().with_level(log::LevelFilter::Debug).init();

    // 创建临时目录
    let temp_dir = TempDir::new()?;
    let root_path = temp_dir.path().canonicalize()?;
    let db_path = temp_dir.path().join("test.db");

    // 创建数据库
    let database = Database::new(&db_path)?;
    database.initialize_tables(&database.conn)?;

    // 测试路径
    let normal_path = root_path.to_string_lossy().to_string();
    let long_path_prefix = "\\\\?\\".to_string() + &normal_path;

    println!("Normal path: {}", normal_path);
    println!("Long path with prefix: {}", long_path_prefix);

    // 添加根目录记录（使用普通路径）
    let root_id = database.add_root_directory(&normal_path, "test_root", "UNARCHIVED")?;
    println!("Added root directory with ID: {}", root_id);

    // 使用普通路径查找
    let result1 = database.find_root_info_by_path(&normal_path)?;
    println!("Find by normal path: {:?}", result1.is_some());

    // 使用带前缀的路径查找
    let result2 = database.find_root_info_by_path(&long_path_prefix)?;
    println!("Find by long path prefix: {:?}", result2.is_some());

    // 显示数据库中的所有根目录记录
    let all_roots = database.find_all_root_info()?;
    println!("All roots in database:");
    for root in all_roots {
        println!("  ID: {:?}, Path: {}", root.id, root.root_path);
    }
    
    Ok(())
}