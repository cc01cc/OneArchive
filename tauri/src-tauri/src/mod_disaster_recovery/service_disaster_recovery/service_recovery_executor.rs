//! 恢复执行器
//! 负责执行 Reed-Solomon 恢复操作

use anyhow::{Context, Result};
use reed_solomon_erasure::galois_8::ReedSolomon;

use crate::mod_database::dao_database::dao_recovery_data_shard::RecoveryDataShardDao;
use crate::mod_database::impl_database::Database;
use crate::mod_disaster_recovery::service_disaster_recovery::service_shard_mapping_service::ShardMappingService;
use crate::utils::calculate_sha256;

/// 恢复执行器 trait
pub trait RecoveryExecutor {
    /// 执行实际的恢复操作
    fn perform_recovery(
        &self,
        shards: &mut Vec<Option<Vec<u8>>>,
        target_archive_id: i64,
        group_id: i64,
        database: &Database,
    ) -> Result<Vec<u8>>;
}

/// 默认恢复执行器实现
pub struct DefaultRecoveryExecutor {
    reed_solomon: ReedSolomon,
}

impl DefaultRecoveryExecutor {
    pub fn new(reed_solomon: ReedSolomon) -> Self {
        Self { reed_solomon }
    }

    /// 记录分片状态
    fn log_shard_status(&self, shards: &Vec<Option<Vec<u8>>>) {
        for (i, shard) in shards.iter().enumerate() {
            if let Some(data) = shard {
                let hash = calculate_sha256(data);
                log::debug!("重建后分片 {}: 大小 {} 字节，哈希 {}", i, data.len(), hash);
            } else {
                log::debug!("重建后分片 {}: 仍然缺失", i);
            }
        }
    }
}

impl RecoveryExecutor for DefaultRecoveryExecutor {
    fn perform_recovery(
        &self,
        shards: &mut Vec<Option<Vec<u8>>>,
        target_archive_id: i64,
        group_id: i64,
        database: &Database,
    ) -> Result<Vec<u8>> {
        log::debug!("开始执行恢复操作，目标归档文件 ID: {}", target_archive_id);

        // 执行恢复
        match self.reed_solomon.reconstruct(shards) {
            Ok(()) => {
                log::debug!("Reed-Solomon 恢复操作成功");
                // 记录重建后的分片状态
                self.log_shard_status(shards);
            }
            Err(e) => {
                log::error!("Reed-Solomon 恢复操作失败：{:?}", e);
                return Err(e).context("恢复操作失败");
            }
        }

        // 使用分片映射服务找到目标归档文件对应的数据分片 ID
        let mapping_service = ShardMappingService::new(database.conn.clone());
        let shard_id = mapping_service
            .find_shard_id_by_archive_and_group(target_archive_id, group_id)
            .context("查找归档文件对应的数据分片失败")?;

        log::debug!(
            "找到归档文件 {} 在灾备组 {} 中的分片 ID: {}",
            target_archive_id,
            group_id,
            shard_id
        );

        // 通过 shard_id 获取数据分片信息，以获取 shard_index
        let data_shard_dao = RecoveryDataShardDao::new(database.conn.clone());
        let data_shard = data_shard_dao
            .find_by_id(shard_id)?
            .ok_or_else(|| anyhow::anyhow!("未找到数据分片 ID: {}", shard_id))?;

        log::debug!(
            "找到数据分片：index={}, hash={}",
            data_shard.shard_index,
            data_shard.shard_hash
        );

        // 获取恢复的数据
        let shard_index = data_shard.shard_index as usize;
        let recovered_data = shards[shard_index]
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("恢复数据分片缺失：索引 {}", shard_index))?
            .clone();

        let recovered_hash = calculate_sha256(&recovered_data);
        log::debug!(
            "成功获取恢复的数据，大小：{} 字节，哈希：{}",
            recovered_data.len(),
            recovered_hash
        );

        Ok(recovered_data)
    }
}