use chrono::Utc;
use log::{debug, info};

use one_archive_lib::mod_database::dao_database::dao_archive_metadata::ArchiveMetadataDao;
use one_archive_lib::mod_database::impl_database::Database;
use one_archive_lib::mod_database::schema::CreateArchiveMetadataParams;
use one_archive_lib::mod_disaster_recovery::service_disaster_recovery::service_encoder::GroupEncoder;
use one_archive_lib::mod_disaster_recovery::core_disaster_recovery::model_disaster_recovery::RecoveryConfig;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{self, Write};
use std::path::Path;
mod common;
use common::*;
mod disaster_recovery_common;
use disaster_recovery_common::*;

/// 测试灾备恢复编码器（数据库集成版本）
#[test]
fn test_disaster_recovery_encoder_with_db() -> Result<(), Box<dyn std::error::Error>> {
    init_test_env_in_file();
    info!("启动灾备恢复编码器测试（数据库集成版本）...");

    // 1. 设置测试环境
    let env = TestEnvironment::new()?;
    let database = &env.database;

    // 2. 创建测试归档文件并插入数据库
    let archive_ids = create_test_archives_and_insert_to_db(&env.source_dir, &database)?;
    info!("创建测试归档文件，ID：{:?}", archive_ids);

    // 3. 初始化编码器
    let config = RecoveryConfig {
        data_shards: 3,
        parity_shards: 2,
        storage_dir: env.archive_dir.to_string_lossy().into_owned(),
        prefix: "test".to_string(),
        store_data_shards: false,
    };
    let encoder = GroupEncoder::new(config);

    // 4. 创建灾备组并保存到数据库
    info!("创建灾备组并保存到数据库...");
    let group_id = encoder.create_recovery_group_and_save_with_archives(&database, archive_ids)?;
    info!("灾备组创建成功，ID：{}", group_id);

    // 5. 验证结果
    debug!("验证灾备组和分片信息...");
    assert_recovery_group_and_shards(&database, group_id, 3, 2)?;

    info!("灾备恢复编码器测试（数据库集成版本）通过！");
    Ok(())
}

/// 测试空文件处理（数据库集成版本）
#[test]
fn test_empty_file_recovery_with_db() -> Result<(), Box<dyn std::error::Error>> {
    init_test_env_in_file();

    let env = TestEnvironment::new()?;
    let database = &env.database;

    // 创建空文件并插入数据库
    let empty_file = env.source_dir.join("empty_archive.tar");
    fs::File::create(&empty_file)?;

    let data = b"";
    let mut hasher = Sha256::new();
    hasher.update(data);
    let sha256 = format!("{:x}", hasher.finalize());

    let archive_dao = ArchiveMetadataDao::new(database.conn.clone());
    let params = CreateArchiveMetadataParams {
        archive_uri: empty_file.to_string_lossy().into_owned(),
        archive_name: "empty_archive.tar".to_string(),
        archive_limit_size: 10 * 1024 * 1024,
        is_compressed: 0,
        is_encrypted: 0,
        compression_algorithm: None,
        encryption_algorithm: None,
        status: one_archive_lib::mod_database::constants::ArchiveStatus::Health,
    };

    let archive_id = archive_dao.create(params).unwrap();

    // 更新记录，添加哈希值和大小
    if let Some(mut metadata) = archive_dao.find_by_id(archive_id).unwrap() {
        metadata.archive_hash = Some(sha256);
        metadata.archive_size = Some(data.len() as i64);
        archive_dao.update(archive_id, metadata).unwrap();
    }

    let config = RecoveryConfig {
        data_shards: 1,
        parity_shards: 1,
        storage_dir: env.archive_dir.to_string_lossy().into_owned(),
        prefix: "empty_test".to_string(),
        store_data_shards: false,
    };
    let encoder = GroupEncoder::new(config);

    // 应该返回错误，因为所有文件都为空
    let result = encoder.create_recovery_group_and_save_with_archives(&database, vec![archive_id]);
    assert!(result.is_err(), "所有归档文件都为空时应该返回错误");

    Ok(())
}

/// 测试不同大小文件处理（数据库集成版本）
#[test]
fn test_mixed_size_recovery_with_db() -> Result<(), Box<dyn std::error::Error>> {
    init_test_env_in_file();

    let env = TestEnvironment::new()?;
    let database = &env.database;

    // 创建不同大小的文件并插入数据库
    let sizes = [512, 1024, 2048, 4096];
    let mut archive_ids = Vec::new();
    let archive_dao = ArchiveMetadataDao::new(database.conn.clone());

    for (i, &size) in sizes.iter().enumerate() {
        let path = env.source_dir.join(format!("file_{}.tar", i));
        let data = vec![1u8; size];
        let mut file = fs::File::create(&path)?;
        file.write_all(&data)?;

        // 计算哈希
        let mut hasher = Sha256::new();
        hasher.update(&data);
        let sha256 = format!("{:x}", hasher.finalize());

        let params = CreateArchiveMetadataParams {
            archive_uri: path.to_string_lossy().into_owned(),
            archive_name: format!("file_{}.tar", i),
            archive_limit_size: 10 * 1024 * 1024,
            is_compressed: 0,
            is_encrypted: 0,
            compression_algorithm: None,
            encryption_algorithm: None,
            status: one_archive_lib::mod_database::constants::ArchiveStatus::Health,
        };

        let archive_id = archive_dao.create(params).unwrap();

        // 更新记录，添加哈希值和大小
        if let Some(mut metadata) = archive_dao.find_by_id(archive_id).unwrap() {
            metadata.archive_hash = Some(sha256);
            metadata.archive_size = Some(data.len() as i64);
            archive_dao.update(archive_id, metadata).unwrap();
        }

        archive_ids.push(archive_id);
    }

    let config = RecoveryConfig {
        data_shards: 4,
        parity_shards: 2,
        storage_dir: env.archive_dir.to_string_lossy().into_owned(),
        prefix: "mixed_test".to_string(),
        store_data_shards: false,
    };
    let encoder = GroupEncoder::new(config);

    let group_id = encoder.create_recovery_group_and_save_with_archives(&database, archive_ids)?;
    assert_recovery_group_and_shards(&database, group_id, 4, 2)?;

    Ok(())
}

/// 测试存储数据分片功能（数据库集成版本）
#[test]
fn test_store_data_shards_with_db() -> Result<(), Box<dyn std::error::Error>> {
    init_test_env_in_file();

    let env = TestEnvironment::new()?;
    let database = &env.database;

    // 创建测试文件并插入数据库
    let sizes = [512, 1024, 2048];
    let mut archive_ids = Vec::new();
    let archive_dao = ArchiveMetadataDao::new(database.conn.clone());

    for (i, &size) in sizes.iter().enumerate() {
        let path = env.source_dir.join(format!("file_{}.tar", i));
        let data = vec![1u8; size];
        let mut file = fs::File::create(&path)?;
        file.write_all(&data)?;

        let mut hasher = Sha256::new();
        hasher.update(&data);
        let sha256 = format!("{:x}", hasher.finalize());

        let params = CreateArchiveMetadataParams {
            archive_uri: path.to_string_lossy().into_owned(),
            archive_name: format!("file_{}.tar", i),
            archive_limit_size: 10 * 1024 * 1024,
            is_compressed: 0,
            is_encrypted: 0,
            compression_algorithm: None,
            encryption_algorithm: None,
            status: one_archive_lib::mod_database::constants::ArchiveStatus::Health,
        };

        let archive_id = archive_dao.create(params).unwrap();

        // 更新记录，添加哈希值和大小
        if let Some(mut metadata) = archive_dao.find_by_id(archive_id).unwrap() {
            metadata.archive_hash = Some(sha256);
            metadata.archive_size = Some(data.len() as i64);
            archive_dao.update(archive_id, metadata).unwrap();
        }

        archive_ids.push(archive_id);
    }

    let config = RecoveryConfig {
        data_shards: 3,
        parity_shards: 2,
        storage_dir: env.archive_dir.to_string_lossy().into_owned(),
        prefix: "store_test".to_string(),
        store_data_shards: true, // 启用数据分片存储
    };
    let encoder = GroupEncoder::new(config);

    let group_id = encoder.create_recovery_group_and_save_with_archives(&database, archive_ids)?;

    // 验证数据分片已存储
    let recovery_group_dao =
        one_archive_lib::mod_database::dao_database::dao_recovery_group::RecoveryGroupDao::new(
            database.conn.clone(),
        );
    let group_info = recovery_group_dao
        .find_by_id(group_id, None)
        .expect("查询灾备组失败")
        .expect("未找到指定 ID 的灾备组");
    assert!(group_info.data_shards_stored, "数据分片应该被存储");

    let data_shard_dao =
        one_archive_lib::mod_database::dao_database::dao_recovery_data_shard::RecoveryDataShardDao::new(
            database.conn.clone(),
        );
    let data_shards = data_shard_dao.find_multi_shards_by_group_id(group_id).unwrap();

    for data_shard in &data_shards {
        assert!(data_shard.shard_path.is_some(), "数据分片的存储路径不应该为空");
        if let Some(ref storage_path) = data_shard.shard_path {
            assert!(Path::new(storage_path).exists(), "数据分片文件应该存在：{}", storage_path);
        }
    }

    Ok(())
}