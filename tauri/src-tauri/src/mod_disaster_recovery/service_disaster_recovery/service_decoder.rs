//! 灾备恢复解码器
//!
//! # 灾备恢复逻辑说明
//!
//! ## 问题背景
//! 用户在创建灾备组的时候可以手动选择是否保存 dataShards。如果没有保存 data shard，
//! 则恢复的时候需要从 archive 恢复，而不是直接读取 data shard，会始终 data shard
//! 为零导致分片数量不足。
//!
//! ## 恢复流程
//!
//! 校验 map archive data shard 和 archive metadata 中的 archive hash 是否一致
//! 如果不一致，说明 archive 文件已经被更改，不再可用，后续也不可用于灾备组恢复
//! 如果一致，说明文件理论上没有变化，那么再读取实际文件内容计算 hash 值，是否和 数据表中的 hash 一致
//! 1. 从完好的 archive (现有校验 archive 是否符合 original_archive_hash) 生成新的 data shard
//!    (和当初创建 data shard 相同的逻辑)
//! 2. 校验新生成的 data shard hash 和 data shard 表中保存的是否一致
//! 3. 如果一致的话，就可以作为分片使用
//!
//! ## 核心逻辑
//!
//! - 当 `data_shards_stored` 为 `false` 时，系统不会从文件系统读取 data shard，
//!   而是从原始归档文件生成数据分片
//! - 通过 `align_shard` 方法从原始归档文件生成对齐的数据分片
//! - 验证生成的数据分片哈希值与数据库中保存的哈希值是否一致
//! - 一致则使用生成的数据分片参与 Reed-Solomon 纠删码恢复
use anyhow::{Context, Result};
use reed_solomon_erasure::galois_8::ReedSolomon;
use std::path::{Path, PathBuf};

use crate::mod_database::dao_database::dao_archive_metadata::ArchiveMetadataDao;
use crate::mod_database::impl_database::Database;

use crate::mod_disaster_recovery::core_disaster_recovery::model_disaster_recovery::RecoveryConfig;
use crate::mod_disaster_recovery::service_disaster_recovery::service_data_saver::{DataSaver, DefaultDataSaver};
use crate::mod_disaster_recovery::service_disaster_recovery::service_file_integrity_validator::FileIntegrityValidator;
use crate::mod_disaster_recovery::service_disaster_recovery::service_recovery_executor::{
    DefaultRecoveryExecutor, RecoveryExecutor,
};
use crate::mod_disaster_recovery::service_disaster_recovery::service_recovery_planner::{
    DefaultRecoveryPlanner, RecoveryPlanner,
};
use crate::mod_disaster_recovery::service_disaster_recovery::service_shard_loader::{DefaultShardLoader, ShardLoader};

/// 灾备恢复解码器
/// 协调灾备恢复过程的主要入口组件
pub struct GroupDecoder {
    reed_solomon: ReedSolomon,
    config: RecoveryConfig,
    shard_loader: Box<dyn ShardLoader>,
    recovery_executor: Box<dyn RecoveryExecutor>,
    recovery_planner: Box<dyn RecoveryPlanner>,
    data_saver: Box<dyn DataSaver>,
}

impl GroupDecoder {
    /// 创建新的解码器实例
    pub fn new(config: RecoveryConfig) -> Self {
        let reed_solomon = ReedSolomon::new(config.data_shards, config.parity_shards)
            .context("创建 Reed-Solomon 编解码器失败")
            .unwrap();
        Self {
            reed_solomon: reed_solomon.clone(),
            config,
            shard_loader: Box::new(DefaultShardLoader::new()),
            recovery_executor: Box::new(DefaultRecoveryExecutor::new(reed_solomon)),
            recovery_planner: Box::new(DefaultRecoveryPlanner::new()),
            data_saver: Box::new(DefaultDataSaver::new()),
        }
    }

    /// 恢复损坏的归档文件
    /// 返回恢复后的文件路径，如果不需要恢复或恢复失败则返回 None
    pub fn recover_archive(
        &self, database: &Database, archive_id: i64, work_dir: &Path,
    ) -> Result<Option<PathBuf>> {
        // 校验 archive, 如果状态正常，则直接返回
        let archive_is_healthy =
            FileIntegrityValidator::verify_archive_sha256(archive_id, database)?;

        if archive_is_healthy {
            let archive_dao = ArchiveMetadataDao::new(database.conn.clone());
            if let Some(metadata) = archive_dao.find_by_id(archive_id)? {
                log::info!("归档文件 {} 状态正常，无需恢复", archive_id);
                return Ok(Some(PathBuf::from(metadata.archive_uri)));
            }
            return Ok(None);
        }

        log::info!("开始恢复损坏的归档文件 {}", archive_id);

        // 准备恢复数据
        let mut recovery_data = self.recovery_planner.prepare_recovery(
            archive_id,
            database,
            self.shard_loader.as_ref(),
            &self.reed_solomon,
        )?;

        if !recovery_data.can_recover {
            log::warn!("归档文件 {} 不满足恢复条件", archive_id);
            return Err(anyhow::anyhow!("归档文件 {} 不满足恢复条件", archive_id));
        }

        log::info!("归档文件 {} 满足恢复条件，开始执行恢复", archive_id);

        // 执行恢复
        let recovered_data = self.recovery_executor.perform_recovery(
            &mut recovery_data.shards,
            archive_id,
            recovery_data.group_id,
            database,
        )?;

        log::info!("归档文件 {} 恢复完成，开始保存恢复的数据", archive_id);

        // 保存恢复的数据
        let recovered_path =
            self.data_saver.save_recovered_data(recovered_data, archive_id, database, work_dir)?;

        log::info!("归档文件 {} 恢复成功，保存到：{}", archive_id, recovered_path.display());

        Ok(Some(recovered_path))
    }
}
