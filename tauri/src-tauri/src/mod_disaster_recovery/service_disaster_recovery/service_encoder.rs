//! 灾备系统实现 - 整合编码器与数据库操作

use anyhow::Result;
use log::debug;
use reed_solomon_erasure::galois_8::ReedSolomon;

use crate::mod_database::impl_database::Database;
use crate::mod_disaster_recovery::core_disaster_recovery::model_disaster_recovery::RecoveryConfig;
use crate::mod_disaster_recovery::service_disaster_recovery::service_group_creator::{GroupCreator, DefaultGroupCreator};
use crate::mod_disaster_recovery::service_disaster_recovery::service_shard_loader::{ArchiveReader, DefaultShardLoader};
use crate::mod_disaster_recovery::service_disaster_recovery::service_shard_processor::{ShardProcessor, DefaultShardProcessor};

/// 灾备组编码器
/// 协调灾备编码过程的入口组件
pub struct GroupEncoder {
    reed_solomon: ReedSolomon,
    config: RecoveryConfig,
    archive_reader: Box<dyn ArchiveReader>,
    group_creator: Box<dyn GroupCreator>,
    shard_processor: Box<dyn ShardProcessor>,
}

impl GroupEncoder {
    /// 创建新的编码器实例
    pub fn new(config: RecoveryConfig) -> Self {
        let reed_solomon = ReedSolomon::new(config.data_shards, config.parity_shards).unwrap();
        Self {
            reed_solomon: reed_solomon.clone(),
            config: config.clone(),
            archive_reader: Box::new(DefaultShardLoader::new()),
            group_creator: Box::new(DefaultGroupCreator::new(config.clone())),
            shard_processor: Box::new(DefaultShardProcessor::new(reed_solomon, config)),
        }
    }

    /// 创建灾备组并将其信息保存到数据库（使用归档文件 ID）
    /// TODO 后续支持分块功能
    pub fn create_recovery_group_and_save_with_archives(
        &self, database: &Database, archive_ids: Vec<i64>,
    ) -> Result<i64> {
        // 从数据库获取归档文件路径
        let archive_paths = self.archive_reader.get_archive_paths_from_ids(database, &archive_ids)?;

        // 读取文件元数据
        let (archive_metas, original_archive_paths) = self.archive_reader.read_archive_files(&archive_paths)?;

        // TODO 这里重新从本地读取并计算了 archive 的 sha256, 可以借机和数据库中记录的值进行对比

        // 检查是否所有文件都为空，如果是则无需创建灾备组
        if archive_metas.iter().all(|(_, size, _)| *size == 0) {
            debug!("所有归档文件都为空，无需创建灾备组");
            return Err(anyhow::anyhow!("所有归档文件都为空"));
        }

        // 计算 shard_size，使用最大的文件大小作为 shard_size
        let shard_size = archive_metas.iter().map(|(_, size, _)| *size).max().unwrap_or(0);

        // 创建灾备组记录
        let group_id = self.group_creator.create_recovery_group_record(database, shard_size)?;

        // 生成灾备分片并即时保存到数据库
        self.shard_processor.gen_and_save_shards(
            database,
            group_id,
            &archive_ids,
            &archive_metas,
            &original_archive_paths,
            shard_size,
        )?;

        Ok(group_id)
    }

}
