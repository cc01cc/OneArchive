//! 灾备恢复解码器
use anyhow::{Context, Result};
use reed_solomon_erasure::galois_8::ReedSolomon;
use sha2::{Digest, Sha256};
use std::fs;

use crate::mod_database::dao::archive_metadata::ArchiveMetadataDao;
use crate::mod_database::dao::map_archive_shard::ArchiveShardMapDao;
use crate::mod_database::dao::recovery_data_shard::RecoveryDataShardDao;
use crate::mod_database::dao::recovery_group::RecoveryGroupDao;
use crate::mod_database::dao::recovery_parity_shard::RecoveryParityShardDao;
use crate::mod_database::database::Database;
use crate::mod_database::schema_recovery::{RecoveryDataShard, RecoveryParityShard};
use crate::mod_disaster_recovery::model::RecoveryConfig;

/// 灾备恢复解码器
pub struct GroupDecoder {
    reed_solomon: ReedSolomon,
    config: RecoveryConfig,
}

impl GroupDecoder {
    /// 创建新的解码器实例
    pub fn new(config: RecoveryConfig) -> Self {
        let reed_solomon = ReedSolomon::new(config.data_shards, config.parity_shards)
            .context("创建 Reed-Solomon 编解码器失败")
            .unwrap();
        Self { reed_solomon, config }
    }

    /// 从数据库加载灾备组信息并恢复丢失的文件
    pub fn recover_archive_from_database(
        &self,
        database: &Database,
        group_id: &str,
        lost_shard_index: usize,
        lost_shard_type: &str, // "data" 或 "parity"
    ) -> Result<Vec<u8>> {
        // 初始化 DAO
        let recovery_group_dao = RecoveryGroupDao::new(database.conn.clone());
        let recovery_data_shard_dao = RecoveryDataShardDao::new(database.conn.clone());
        let recovery_parity_shard_dao = RecoveryParityShardDao::new(database.conn.clone());
        let archive_metadata_dao = ArchiveMetadataDao::new(database.conn.clone());
        let archive_shard_map_dao = ArchiveShardMapDao::new(database.conn.clone());

        // 1. 从数据库加载灾备组信息
        let db_group = recovery_group_dao
            .find_recovery_group_by_group_id(group_id)?
            .ok_or_else(|| anyhow::anyhow!("未找到灾备组：{}", group_id))?;

        // 2. 加载所有分片信息
        let db_data_shards =
            recovery_data_shard_dao.find_recovery_data_shards_by_group_id(db_group.id)?;
        let db_parity_shards =
            recovery_parity_shard_dao.find_recovery_parity_shards_by_group_id(db_group.id)?;

        // 3. 验证丢失的分片是否属于该灾备组
        if lost_shard_type == "data" {
            db_data_shards
                .iter()
                .find(|s| s.shard_index == lost_shard_index as i64)
                .ok_or_else(|| anyhow::anyhow!("丢失的数据分片不在该灾备组中"))?;
        } else {
            db_parity_shards
                .iter()
                .find(|s| s.shard_index == lost_shard_index as i64)
                .ok_or_else(|| anyhow::anyhow!("丢失的校验分片不在该灾备组中"))?;
        };

        // 4. 收集可用的分片（排除丢失的分片）
        let mut available_data_shards: Vec<RecoveryDataShard> = Vec::new();
        let mut available_parity_shards: Vec<RecoveryParityShard> = Vec::new();

        // 处理数据分片
        for db_shard in &db_data_shards {
            // 跳过丢失的分片
            if lost_shard_type == "data" && db_shard.shard_index == lost_shard_index as i64 {
                continue;
            }
            available_data_shards.push(db_shard.clone());
        }

        // 处理校验分片
        for db_shard in &db_parity_shards {
            // 跳过丢失的分片
            if lost_shard_type == "parity" && db_shard.shard_index == lost_shard_index as i64 {
                continue;
            }
            available_parity_shards.push(db_shard.clone());
        }

        // 5. 确保有足够的分片进行恢复
        if available_data_shards.len() + available_parity_shards.len() < self.config.data_shards {
            return Err(anyhow::anyhow!(
                "可用分片不足：需要至少 {} 个分片，当前只有 {} 个",
                self.config.data_shards,
                available_data_shards.len() + available_parity_shards.len()
            ));
        }

        // 6. 准备分片数据
        let mut shards = vec![None; self.config.data_shards + self.config.parity_shards];

        // 填充可用的数据分片
        for db_shard in &available_data_shards {
            let content = if let Some(ref path) = db_shard.shard_path {
                // 如果有分片路径，直接读取分片文件
                fs::read(path).context(format!("无法读取数据分片文件：{}", path))?
            } else {
                // 如果没有分片路径，需要从原始归档文件获取
                let archive_shard_map = archive_shard_map_dao
                    .find_archive_shard_map_by_shard_id(db_shard.id)?
                    .ok_or_else(|| {
                        anyhow::anyhow!(
                            "未找到数据分片 {} 对应的归档文件映射",
                            db_shard.shard_index
                        )
                    })?;

                let archive_metadata = archive_metadata_dao
                    .find_by_id(archive_shard_map.archive_id)?
                    .ok_or_else(|| {
                        anyhow::anyhow!(
                            "未找到数据分片 {} 对应的归档文件元数据",
                            db_shard.shard_index
                        )
                    })?;

                fs::read(&archive_metadata.archive_uri)
                    .context(format!("无法读取原始归档文件：{}", archive_metadata.archive_uri))?
            };

            // 验证内容哈希
            let mut hasher = Sha256::new();
            hasher.update(&content);
            let hash = format!("{:x}", hasher.finalize());

            if hash != db_shard.shard_hash {
                return Err(anyhow::anyhow!(
                    "数据分片哈希不匹配：索引 {} (预期：{}, 实际：{})",
                    db_shard.shard_index,
                    db_shard.shard_hash,
                    hash
                ));
            }

            shards[db_shard.shard_index as usize] = Some(content);
        }

        // 填充可用的校验分片
        for db_shard in &available_parity_shards {
            let content = fs::read(&db_shard.shard_path)
                .context(format!("无法读取校验分片文件：{}", db_shard.shard_path))?;

            // 验证内容哈希
            let mut hasher = Sha256::new();
            hasher.update(&content);
            let hash = format!("{:x}", hasher.finalize());

            if hash != db_shard.shard_hash {
                return Err(anyhow::anyhow!(
                    "校验分片哈希不匹配：索引 {} (预期：{}, 实际：{})",
                    db_shard.shard_index,
                    db_shard.shard_hash,
                    hash
                ));
            }

            // 校验分片的索引需要偏移 data_shards 的数量
            let parity_index = self.config.data_shards + db_shard.shard_index as usize;
            shards[parity_index] = Some(content);
        }

        // 7. 执行恢复
        self.reed_solomon.reconstruct(&mut shards).context("恢复操作失败")?;

        // 8. 提取恢复的数据
        let lost_index = if lost_shard_type == "data" {
            lost_shard_index
        } else {
            // parity shard index needs to be offset by data_shards count
            self.config.data_shards + lost_shard_index
        };

        let recovered_data =
            shards[lost_index].as_ref().ok_or_else(|| anyhow::anyhow!("恢复数据缺失"))?.clone();

        Ok(recovered_data)
    }
}
