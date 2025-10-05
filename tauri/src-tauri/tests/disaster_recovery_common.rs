//! 灾备模块专用测试辅助函数
//! 提供灾备相关测试的通用工具函数

use chrono::Utc;
use log::{debug, info};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::sync::Arc;

use one_archive_lib::mod_database::dao_database::dao_archive_metadata::ArchiveMetadataDao;
use one_archive_lib::mod_database::impl_database::Database;
use one_archive_lib::mod_database::schema::CreateArchiveMetadataParams;

/// 创建测试用的归档文件并插入到数据库
pub fn create_test_archives_and_insert_to_db(dir: &Path, database: &Database) -> io::Result<Vec<i64>> {
    let mut archive_ids = Vec::new();
    let archive_dao = ArchiveMetadataDao::new(database.conn.clone());

    for i in 0..3 {
        let path = dir.join(format!("archive_{}.tar", i));
        let mut file = fs::File::create(&path)?;

        // 写入不同大小的测试数据
        let data = vec![i as u8; 1024 * (i + 1)]; // 1KB, 2KB, 3KB
        file.write_all(&data)?;

        // 计算文件哈希
        let mut hasher = Sha256::new();
        hasher.update(&data);
        let sha256 = format!("{:x}", hasher.finalize());

        // 插入归档元数据到数据库
        let params = CreateArchiveMetadataParams {
            archive_uri: path.to_string_lossy().into_owned(),
            archive_name: format!("archive_{}.tar", i),
            archive_limit_size: 10 * 1024 * 1024, // 10MB
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

    Ok(archive_ids)
}

/// 验证灾备组和分片信息
pub fn assert_recovery_group_and_shards(
    database: &Database, group_id: i64, expected_data_shards: usize, expected_parity_shards: usize,
) -> io::Result<()> {
    // 获取灾备组信息
    let recovery_group_dao =
        one_archive_lib::mod_database::dao_database::dao_recovery_group::RecoveryGroupDao::new(
            database.conn.clone(),
        );
    let group_info = recovery_group_dao
        .find_by_id(group_id, None)
        .expect("查询灾备组失败")
        .expect("未找到指定 ID 的灾备组");

    // 验证灾备组信息
    assert_eq!(group_info.num_data_shards as usize, expected_data_shards);
    assert_eq!(group_info.num_parity_shards as usize, expected_parity_shards);

    // 获取数据分片
    let data_shard_dao =
        one_archive_lib::mod_database::dao_database::dao_recovery_data_shard::RecoveryDataShardDao::new(
            database.conn.clone(),
        );
    let data_shards = data_shard_dao.find_multi_shards_by_group_id(group_id).unwrap();

    // 获取校验分片
    let parity_shard_dao =
        one_archive_lib::mod_database::dao_database::dao_recovery_parity_shard::RecoveryParityShardDao::new(
            database.conn.clone(),
        );
    let parity_shards = parity_shard_dao.find_multi_shards_by_group_id(group_id).unwrap();

    // 验证分片数量
    assert_eq!(data_shards.len(), expected_data_shards);
    assert_eq!(parity_shards.len(), expected_parity_shards);

    // 验证校验分片文件存在性和哈希值
    for parity_shard in &parity_shards {
        assert!(
            Path::new(&parity_shard.shard_path).exists(),
            "校验分片文件应该存在：{}",
            parity_shard.shard_path
        );

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

    // 验证数据分片（如果存储了数据分片）
    if group_info.data_shards_stored {
        for data_shard in &data_shards {
            if let Some(ref storage_path) = data_shard.shard_path {
                assert!(Path::new(storage_path).exists(), "数据分片文件应该存在：{}", storage_path);

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
    }

    Ok(())
}