//! 文件完整性验证器模块
//! 提供基于 SHA256 的文件完整性验证功能

use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::{
    mod_database::{
        constants::STATUS_HEALTH,
        dao_database::{
            dao_archive_metadata::ArchiveMetadataDao, dao_map_archive_shard::MapArchiveDataShardDao,
            dao_recovery_data_shard::RecoveryDataShardDao,
            dao_recovery_parity_shard::RecoveryParityShardDao,
        },
        impl_database::Database,
    },
    mod_disaster_recovery::core_disaster_recovery::model_disaster_recovery::{RecoveryGroupInfoWithShardDetail, ShardType},
    utils::verify_file_sha256,
};

/// 文件完整性验证器
pub struct FileIntegrityValidator;

impl FileIntegrityValidator {
    /// 验证恢复组的 SHA256 完整性
    pub fn verify_group_sha256(
        recovery_group_detail: &RecoveryGroupInfoWithShardDetail,
    ) -> Result<bool> {
        // 验证所有原始档案文件
        for archive in recovery_group_detail.archives.clone() {
            let archive_hash = archive.archive_hash.as_ref().ok_or_else(|| {
                anyhow::anyhow!("归档文件 {} 没有哈希值", archive.id.unwrap_or(-1))
            })?;
            if !verify_file_sha256(&Path::new(&archive.archive_uri), archive_hash)? {
                return Ok(false);
            }
        }

        if recovery_group_detail.group_info.data_shards_stored {
            // 验证所有数据分片
            for data_shard in recovery_group_detail.data_shards.clone() {
                if !verify_file_sha256(
                    &Path::new(&data_shard.shard_path.unwrap()),
                    &data_shard.shard_hash,
                )? {
                    return Ok(false);
                }
            }
        }

        // 验证所有校验分片
        for parity_shard in recovery_group_detail.parity_shards.clone() {
            if !verify_file_sha256(&Path::new(&parity_shard.shard_path), &parity_shard.shard_hash)?
            {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// 验证特定归档文件的 SHA256
    pub fn verify_archive_sha256(archive_id: i64, database: &Database) -> Result<bool> {
        let (file_path, expected_hash, status) =
            Self::get_path_and_hash_from_archive_metadata(archive_id, database)?;
        // 校验 archive 数据库状态
        if status != STATUS_HEALTH {
            log::debug!("archive {} status is not health", archive_id);
            return Ok(false);
        }

        // 同时校验文件是否存在，以及文件的哈希值是否与数据库记录一致
        match crate::utils::verify_file_sha256(&file_path, &expected_hash) {
            Ok(result) => Ok(result),
            Err(_) => Ok(false), // 任何错误都视为验证失败
        }
    }

    /// 验证特定分片的 SHA256
    pub fn verify_shard_sha256_with_id(
        shard_type: &ShardType, shard_id: i64, database: &Database,
    ) -> Result<bool> {
        let (file_path, expected_hash, status) =
            Self::get_shard_path_and_hash_from_db(shard_type, shard_id, database)?;
        // 校验 shard 数据库状态
        if status != STATUS_HEALTH {
            log::debug!("shard type {}; {} status is not health", shard_type.as_str(), shard_id);
            return Ok(false);
        }

        // 同时校验文件是否存在，以及文件的哈希值是否与数据库记录一致
        match crate::utils::verify_file_sha256(&file_path, &expected_hash) {
            Ok(result) => Ok(result),
            Err(_) => Ok(false), // 任何错误都视为验证失败
        }
    }

    /// 从数据库获取分片路径和哈希
    fn get_shard_path_and_hash_from_db(
        shard_type: &ShardType, shard_id: i64, database: &Database,
    ) -> Result<(PathBuf, String, String)> {
        let (file_path, expected_hash, status) = match shard_type {
            ShardType::DataShard => {
                let data_shard_dao = RecoveryDataShardDao::new(database.conn.clone());
                let shard = data_shard_dao
                    .find_by_id(shard_id)?
                    .ok_or_else(|| anyhow::anyhow!("未找到数据分片：{}", shard_id))?;
                // shard.shard_path 是 Option<String>，取引用后 clone 出一个 String
                let path = shard
                    .shard_path
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("数据分片 {} 没有路径信息", shard_id))?
                    .clone();
                let expected_hash = shard.shard_hash.clone();
                let status = shard.status.clone();
                (path, expected_hash, status.as_str())
            }

            ShardType::ParityShard => {
                let parity_shard_dao = RecoveryParityShardDao::new(database.conn.clone());
                let shard = parity_shard_dao
                    .find_shard_by_id(shard_id)?
                    .ok_or_else(|| anyhow::anyhow!("未找到校验分片：{}", shard_id))?;
                // parity shard 的 shard_path 是 String（总是有），直接 clone
                let path = shard.shard_path.clone();
                let expected_hash = shard.shard_hash.clone();
                let status = shard.status.clone();
                (path, expected_hash, status.as_str())
            }
        };
        Ok((PathBuf::from(file_path), expected_hash, String::from(status)))
    }

    /// 从 archive metadata 获取归档文件路径、哈希和状态
    fn get_path_and_hash_from_archive_metadata(
        archive_id: i64, database: &Database,
    ) -> Result<(PathBuf, String, String)> {
        let archive_dao = ArchiveMetadataDao::new(database.conn.clone());
        let archive = archive_dao
            .find_by_id(archive_id)?
            .ok_or_else(|| anyhow::anyhow!("未找到归档文件：{}", archive_id))?;
        let path = archive.archive_uri.clone();
        let meta_hash = archive
            .archive_hash
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("归档文件 {} 没有哈希值", archive_id))?
            .clone();
        let status = archive.status.clone();
        Ok((PathBuf::from(path), meta_hash, String::from(status.as_str())))
    }

    /// 从 map_archive_data_shard 获取原始归档文件哈希
    fn get_original_archive_hash_from_map(
        archive_id: i64, shard_id: i64, database: &Database,
    ) -> Result<(String, String)> {
        let map_archive_data_shard_dao = MapArchiveDataShardDao::new(database.conn.clone());
        let map_archive_data_shard = map_archive_data_shard_dao
            .find_by_archive_id_and_shard_id(archive_id, shard_id, None)?
            .ok_or_else(|| {
                anyhow::anyhow!("未找到归档文件 {} 的数据分片 {}", archive_id, shard_id)
            })?;
        let original_archive_hash = map_archive_data_shard.original_archive_hash;
        let status = map_archive_data_shard.status.clone();
        Ok((original_archive_hash, String::from(status.as_str())))
    }

    /// 验证原始归档文件的完整性（比较 metadata 和 map_archive_data_shard 中的哈希）
    pub fn verify_original_archive_integrity(
        archive_id: i64, shard_id: i64, database: &Database,
    ) -> Result<bool> {
        let (path, meta_hash, meta_status) =
            Self::get_path_and_hash_from_archive_metadata(archive_id, database)?;
        let (map_hash, map_status) =
            Self::get_original_archive_hash_from_map(archive_id, shard_id, database)?;

        // 校验 archive 数据库状态
        if meta_status != STATUS_HEALTH || map_status != STATUS_HEALTH {
            log::debug!("archive {} status is not health", archive_id);
            return Ok(false);
        }

        // 同时校验文件是否存在，以及文件的哈希值是否与数据库记录一致
        if !crate::utils::verify_file_sha256(&path, &meta_hash)? {
            return Ok(false);
        }

        if !map_hash.eq(&meta_hash) {
            log::debug!(
                "归档文件 {} 的数据分片 {} 的原始归档文件哈希值与元数据不一致",
                archive_id,
                shard_id
            );
            return Ok(false);
        }

        Ok(true)
    }
}
