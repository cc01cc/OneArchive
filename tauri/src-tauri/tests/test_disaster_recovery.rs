use log::{debug, info};

use one_archive_lib::mod_disaster_recovery::encoder::GroupEncoder;
use one_archive_lib::mod_disaster_recovery::model::RecoveryConfig;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{self, Write};
use std::path::Path;
mod common;
use common::*;
use std::sync::Once;
static INIT: Once = Once::new();

fn init_test_env_in_file() {
    INIT.call_once(|| {
        let _ = color_eyre::install();
        let _ =
            env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
                .try_init();
    });
}

/// 创建测试用的归档文件
fn create_test_archives(dir: &Path) -> io::Result<Vec<String>> {
    let mut archives = Vec::new();

    for i in 0..3 {
        let path = dir.join(format!("archive_{}.tar", i));
        let mut file = fs::File::create(&path)?;

        // 写入不同大小的测试数据
        let data = vec![i as u8; 1024 * (i + 1)]; // 1KB, 2KB, 3KB
        file.write_all(&data)?;

        archives.push(path.to_string_lossy().into_owned());
    }

    Ok(archives)
}

/// 验证灾备文件
fn assert_recovery_files(
    data_shards: &Vec<one_archive_lib::mod_database::schema_recovery::RecoveryDataShard>,
    parity_shards: &Vec<one_archive_lib::mod_database::schema_recovery::RecoveryParityShard>,
    expected_archives: usize,
) -> io::Result<()> {
    // 验证元数据
    assert_eq!(data_shards.len(), expected_archives, "应该有 {} 个原始归档文件", expected_archives);
    assert_eq!(parity_shards.len(), 2, "应该有 2 个灾备文件");

    // 验证所有灾备文件大小一致
    if parity_shards.len() > 0 && Path::new(&parity_shards[0].shard_path).exists() {
        let parity_size = fs::metadata(&parity_shards[0].shard_path)?.len() as usize;
        for parity in parity_shards {
            let size = fs::metadata(&parity.shard_path)?.len() as usize;
            assert_eq!(size, parity_size, "所有灾备文件大小应该一致");
        }

        // 只验证实际存储的数据分片文件大小
        for data_shard in data_shards {
            // 检查数据分片是否实际存储（路径存在且文件存在）
            if let Some(ref path) = data_shard.shard_path {
                if Path::new(path).exists() {
                    let size = fs::metadata(path)?.len() as usize;
                    assert!(size <= parity_size, "原始文件大小应该不超过灾备文件大小");
                }
            }
        }
    }

    // 验证哈希值（只验证实际存在的文件）
    for data_shard in data_shards {
        // 对于数据分片，如果 storage_path 存在且文件存在，则验证文件
        if let Some(ref storage_path) = data_shard.shard_path {
            if Path::new(storage_path).exists() {
                let mut file = fs::File::open(storage_path)?;
                let mut hasher = Sha256::new();
                io::copy(&mut file, &mut hasher)?;
                let computed_hash = format!("{:x}", hasher.finalize());

                assert_eq!(
                    computed_hash, data_shard.shard_hash,
                    "数据分片文件哈希值不匹配：{}",
                    storage_path
                );
            }
        }
        // 如果 storage_path 不存在或文件不存在，则跳过验证
    }

    // 验证校验分片哈希值（只验证实际存在的文件）
    for parity_shard in parity_shards {
        if Path::new(&parity_shard.shard_path).exists() {
            let mut file = fs::File::open(&parity_shard.shard_path)?;
            let mut hasher = Sha256::new();
            io::copy(&mut file, &mut hasher)?;
            let computed_hash = format!("{:x}", hasher.finalize());

            assert_eq!(
                computed_hash, parity_shard.shard_hash,
                "校验分片文件哈希值不匹配：{}",
                parity_shard.shard_path
            );
        }
    }

    Ok(())
}

/// 测试灾备恢复编码器
#[test]
fn test_disaster_recovery_encoder() -> Result<(), Box<dyn std::error::Error>> {
    init_test_env_in_file();
    info!("启动灾备恢复编码器测试...");

    // 1. 设置测试环境
    let env = TestEnvironment::new()?;
    let archives = create_test_archives(&env.source_dir)?;
    let max_size = 3 * 1024; // 最大文件大小 (3KB)

    info!("创建测试归档文件：{:?}", archives);

    // 2. 初始化编码器
    let config = RecoveryConfig {
        data_shards: 3,
        parity_shards: 2,
        storage_dir: env.archive_dir.to_string_lossy().into_owned(),
        prefix: "test".to_string(),
        store_data_shards: false,
    };
    let encoder = GroupEncoder::new(config);

    // 3. 生成灾备文件
    info!("生成灾备恢复文件...");
    let archive_paths: Vec<&Path> = archives.iter().map(Path::new).collect();
    let (data_shards, parity_shards) = encoder.gen_shard(archive_paths)?;

    // 4. 验证结果
    debug!("验证灾备文件...data_shards: {:#?}, parity_shards: {:#?}", data_shards, parity_shards);
    assert_recovery_files(&data_shards, &parity_shards, 3)?;

    info!("灾备恢复编码器测试通过！");
    Ok(())
}

/// 测试空文件处理
#[test]
fn test_empty_file_recovery() -> Result<(), Box<dyn std::error::Error>> {
    init_test_env_in_file();

    let env = TestEnvironment::new()?;

    // 创建空文件
    let empty_file = env.source_dir.join("empty_archive.tar");
    fs::File::create(&empty_file)?;

    let config = RecoveryConfig {
        data_shards: 1,
        parity_shards: 1,
        storage_dir: env.archive_dir.to_string_lossy().into_owned(),
        prefix: "empty_test".to_string(),
        store_data_shards: false,
    };
    let encoder = GroupEncoder::new(config);

    let archive_paths: Vec<&Path> = vec![empty_file.as_path()];
    let (data_shards, parity_shards) = encoder.gen_shard(archive_paths)?;

    assert_eq!(data_shards.len(), 1);
    assert_eq!(parity_shards.len(), 1);

    // 检查文件大小（只检查实际存在的文件）
    if let Some(ref path) = data_shards[0].shard_path {
        if Path::new(path).exists() {
            assert_eq!(fs::metadata(path)?.len(), 0);
        }
    }

    // 始终检查 parity shard，因为它们总是被存储
    assert_eq!(fs::metadata(&parity_shards[0].shard_path)?.len(), 0);

    Ok(())
}

/// 测试不同大小文件处理
#[test]
fn test_mixed_size_recovery() -> Result<(), Box<dyn std::error::Error>> {
    init_test_env_in_file();

    let env = TestEnvironment::new()?;

    // 创建不同大小的文件
    let sizes = [512, 1024, 2048, 4096];
    let mut archives = Vec::new();

    for (i, &size) in sizes.iter().enumerate() {
        let path = env.source_dir.join(format!("file_{}.tar", i));
        let mut file = fs::File::create(&path)?;
        file.write_all(&vec![1u8; size])?;
        archives.push(path.to_string_lossy().into_owned());
    }

    let config = RecoveryConfig {
        data_shards: 4,
        parity_shards: 2,
        storage_dir: env.archive_dir.to_string_lossy().into_owned(),
        prefix: "mixed_test".to_string(),
        store_data_shards: false,
    };
    let encoder = GroupEncoder::new(config);

    let archive_paths: Vec<&Path> = archives.iter().map(Path::new).collect();
    let (data_shards, parity_shards) = encoder.gen_shard(archive_paths)?;
    assert_recovery_files(&data_shards, &parity_shards, 4)?;

    Ok(())
}

/// 测试存储数据分片功能
#[test]
fn test_store_data_shards() -> Result<(), Box<dyn std::error::Error>> {
    init_test_env_in_file();

    let env = TestEnvironment::new()?;

    // 创建测试文件
    let sizes = [512, 1024, 2048];
    let mut archives = Vec::new();

    for (i, &size) in sizes.iter().enumerate() {
        let path = env.source_dir.join(format!("file_{}.tar", i));
        let mut file = fs::File::create(&path)?;
        file.write_all(&vec![1u8; size])?;
        archives.push(path.to_string_lossy().into_owned());
    }

    let config = RecoveryConfig {
        data_shards: 3,
        parity_shards: 2,
        storage_dir: env.archive_dir.to_string_lossy().into_owned(),
        prefix: "store_test".to_string(),
        store_data_shards: true, // 启用数据分片存储
    };
    let encoder = GroupEncoder::new(config);

    let archive_paths: Vec<&Path> = archives.iter().map(Path::new).collect();
    let (data_shards, parity_shards) = encoder.gen_shard(archive_paths)?;

    // 验证数据分片也已存储
    for data_shard in &data_shards {
        // 检查数据分片的存储路径是否存在
        if let Some(ref storage_path) = data_shard.shard_path {
            assert!(Path::new(storage_path).exists(), "数据分片文件应该存在：{}", storage_path);
        } else {
            panic!("数据分片的存储路径不应该为空");
        }
    }

    // 验证校验分片已存储
    for parity_shard in &parity_shards {
        assert!(
            Path::new(&parity_shard.shard_path).exists(),
            "校验分片文件应该存在：{}",
            parity_shard.shard_path
        );
    }

    Ok(())
}
