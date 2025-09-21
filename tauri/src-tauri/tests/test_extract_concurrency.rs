// //! ArchiveExtract 并发测试
// //!
// //! 该测试模块专门验证解档功能在并发环境下的表现，包括：
// //! - 多线程并发解档
// //! - 高并发场景下的稳定性
// //! - 线程安全性

// use log::info;
// use one_archive_lib::mod_scan::impl_scan::ScanServices;
// use one_archive_lib::mod_scan::model_scan::ScanProgress;
// use one_archive_lib::mod_scan::trait_scan::DirectoryScanOperations;
// use std::fs;
// use std::path::Path;
// use tempfile::TempDir;

// use one_archive_lib::mod_archive::impl_archive::ArchiveInServices;
// use one_archive_lib::mod_archive::trait_archive::{ArchiveContext, ArchiveOperations};
// use one_archive_lib::mod_database::database::Database;
// use one_archive_lib::mod_database::trait_database::InitializationOperations;
// use one_archive_lib::mod_extract::impl_extract::ExtractService;
// use one_archive_lib::mod_extract::model_extract::{ExtractProgress, ExtractTask};
// use one_archive_lib::mod_extract::trait_extract::ExtractOperations;

// /// 测试环境结构体
// struct TestEnvironment {
//     temp_dir: TempDir,
//     database: Database,
//     source_dir: std::path::PathBuf,
//     archive_dir: std::path::PathBuf,
//     extract_dir: std::path::PathBuf,
// }

// /// 创建适用于并发测试的环境
// fn create_concurrent_test_environment() -> Result<TestEnvironment, Box<dyn std::error::Error + Send>> {
//     let temp_dir = TempDir::new().map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)?;

//     let db_file = temp_dir.path().join("test_archive.sqlite");
//     let db_url = format!("file:{}", db_file.display());
//     let database = Database::new(&db_url).map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)?;
//     database.initialize_tables(&database.conn).map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)?;

//     let source_dir = temp_dir.path().join("test-source");
//     let archive_dir = temp_dir.path().join("test-archive");
//     let extract_dir = temp_dir.path().join("test-extract");

//     fs::create_dir_all(&source_dir).map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)?;
//     fs::create_dir_all(&archive_dir).map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)?;
//     fs::create_dir_all(&extract_dir).map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)?;

//     Ok(TestEnvironment {
//         temp_dir,
//         database,
//         source_dir,
//         archive_dir,
//         extract_dir,
//     })
// }

// /// 创建基本测试文件（test1.txt, test2.txt）
// /// 为并发测试特别设计的版本，返回支持 Send 的错误类型
// fn create_basic_test_files(source_dir: &Path) -> Result<(), Box<dyn std::error::Error + Send>> {
//     fs::write(source_dir.join("test1.txt"), "This is test file 1 content").map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)?;
//     fs::write(source_dir.join("test2.txt"), "This is test file 2 content").map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)?;
//     Ok(())
// }

// /// 设置测试用的归档数据
// fn setup_test_archive_data(
//     source_dir: &Path,
//     archive_dir: &Path,
//     database: &Database,
// ) -> Result<(), Box<dyn std::error::Error + Send>> {
//     let scan_service = ScanServices::new();

//     scan_service.scan_and_save_directory_with_events(
//         source_dir,
//         database,
//         None::<fn(ScanProgress)>,
//     ).map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)?;

//     let archive_in_service = ArchiveInServices::new();
//     let mut context = ArchiveContext::new(
//         "test_archive".to_string(),
//         archive_dir.to_string_lossy().to_string(),
//         1024 * 1024, // 1MB 限制
//     );

//     archive_in_service.archive_file_in_db(source_dir.to_str().unwrap(), &mut context, database)
//         .map_err(|e| Box::new(anyhow::anyhow!(e)) as Box<dyn std::error::Error + Send>)?;
//     Ok(())
// }

// /// 验证基本解档文件
// fn assert_basic_extracted_files(extract_dir: &Path) -> Result<(), Box<dyn std::error::Error + Send>> {
//     let test1_path = extract_dir.join("test1.txt");
//     let test2_path = extract_dir.join("test2.txt");

//     assert!(test1_path.exists(), "test1.txt 应该存在");
//     assert!(test2_path.exists(), "test2.txt 应该存在");

//     let content1 = fs::read_to_string(&test1_path).map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)?;
//     let content2 = fs::read_to_string(&test2_path).map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)?;

//     assert_eq!(
//         content1, "This is test file 1 content",
//         "test1.txt 内容应该正确"
//     );
//     assert_eq!(
//         content2, "This is test file 2 content",
//         "test2.txt 内容应该正确"
//     );
//     Ok(())
// }

// /// 测试并发解档操作
// ///
// /// 验证多个解档操作能否同时正确执行，检查是否存在线程安全问题
// #[test]
// fn test_concurrent_extract_operations() -> Result<(), Box<dyn std::error::Error>> {
//     let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
//         .try_init();

//     // 使用线程池并发执行解档操作
//     let mut handles = vec![];
    
//     // 线程1：创建独立的测试环境并执行解档
//     let handle1 = std::thread::spawn(|| -> Result<(), Box<dyn std::error::Error + Send>> {
//         let env = create_concurrent_test_environment()?;
//         create_basic_test_files(&env.source_dir)?;
//         setup_test_archive_data(&env.source_dir, &env.archive_dir, &env.database)?;
        
//         let mut extract_service = ExtractService::new();
//         let task = ExtractTask {
//             root_id: 1,
//             target_path: env.extract_dir.to_string_lossy().to_string(),
//             overwrite: true,
//         };
        
//         let progress = extract_service.extract_archive(&task, &env.database, None::<fn(_)>)
//             .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)?;
        
//         assert!(progress.completed, "解档应该完成");
//         assert_eq!(progress.total_files, 2, "应该有 2 个文件需要解档");
//         assert_eq!(progress.processed_files, 2, "应该处理了 2 个文件");
//         assert!(progress.error.is_none(), "解档过程中不应该有错误");
        
//         assert_basic_extracted_files(&env.extract_dir)?;
//         Ok(())
//     });
    
//     handles.push(handle1);
    
//     // 线程2：创建独立的测试环境并执行解档
//     let handle2 = std::thread::spawn(|| -> Result<(), Box<dyn std::error::Error + Send>> {
//         let env = create_concurrent_test_environment()?;
//         create_basic_test_files(&env.source_dir)?;
//         setup_test_archive_data(&env.source_dir, &env.archive_dir, &env.database)?;
        
//         let mut extract_service = ExtractService::new();
//         let task = ExtractTask {
//             root_id: 1,
//             target_path: env.extract_dir.to_string_lossy().to_string(),
//             overwrite: true,
//         };
        
//         let progress = extract_service.extract_archive(&task, &env.database, None::<fn(_)>)
//             .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)?;
        
//         assert!(progress.completed, "解档应该完成");
//         assert_eq!(progress.total_files, 2, "应该有 2 个文件需要解档");
//         assert_eq!(progress.processed_files, 2, "应该处理了 2 个文件");
//         assert!(progress.error.is_none(), "解档过程中不应该有错误");
        
//         assert_basic_extracted_files(&env.extract_dir)?;
//         Ok(())
//     });
    
//     handles.push(handle2);
    
//     // 等待所有线程完成并检查结果
//     for (index, handle) in handles.into_iter().enumerate() {
//         let result = handle.join().unwrap();
//         assert!(result.is_ok(), "线程 {} 应该成功完成: {:?}", index, result.err());
//     }
    
//     Ok(())
// }

// /// 测试高并发解档操作
// ///
// /// 验证系统在高并发情况下的稳定性和性能
// #[test]
// fn test_high_concurrency_extract_operations() -> Result<(), Box<dyn std::error::Error>> {
//     let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
//         .try_init();

//     const CONCURRENT_OPERATIONS: usize = 5;
//     let mut handles = vec![];
    
//     for i in 0..CONCURRENT_OPERATIONS {
//         let handle = std::thread::spawn(move || -> Result<(), Box<dyn std::error::Error + Send>> {
//             let env = create_concurrent_test_environment()?;
//             create_basic_test_files(&env.source_dir)?;
//             setup_test_archive_data(&env.source_dir, &env.archive_dir, &env.database)?;
            
//             let mut extract_service = ExtractService::new();
//             let task = ExtractTask {
//                 root_id: 1,
//                 target_path: env.extract_dir.to_string_lossy().to_string(),
//                 overwrite: true,
//             };
            
//             let progress = extract_service.extract_archive(&task, &env.database, None::<fn(_)>)
//                 .map_err(|e| Box::new(anyhow::anyhow!(e)) as Box<dyn std::error::Error + Send>)?;
            
//             assert!(progress.completed, "解档 {} 应该完成", i);
//             assert_eq!(progress.total_files, 2, "解档 {} 应该有 2 个文件需要解档", i);
//             assert_eq!(progress.processed_files, 2, "解档 {} 应该处理了 2 个文件", i);
//             assert!(progress.error.is_none(), "解档 {} 过程中不应该有错误", i);
            
//             assert_basic_extracted_files(&env.extract_dir)?;
//             Ok(())
//         });
        
//         handles.push(handle);
//     }
    
//     // 收集所有结果
//     let mut success_count = 0;
//     for (index, handle) in handles.into_iter().enumerate() {
//         let result = handle.join().unwrap();
//         if result.is_ok() {
//             success_count += 1;
//         } else {
//             panic!("线程 {} 失败: {:?}", index, result.err().unwrap());
//         }
//     }
    
//     assert_eq!(success_count, CONCURRENT_OPERATIONS, "所有并发操作都应该成功");
//     Ok(())
// }