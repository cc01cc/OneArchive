//! 恢复规划器
//! 合并了恢复分析和准备功能的组件

use crate::mod_database::dao_database::dao_recovery_group::RecoveryGroupDao;
use crate::mod_disaster_recovery::core_disaster_recovery::model_disaster_recovery::{
    RecoveryGroupInfo, RecoveryGroupInfoWithShardDetail,
};
use crate::mod_disaster_recovery::core_disaster_recovery::core_reed_solomon_validator::ReedSolomonValidator;
use crate::mod_disaster_recovery::service_disaster_recovery::service_file_integrity_validator::FileIntegrityValidator;
use crate::mod_disaster_recovery::service_disaster_recovery::service_shard_loader::ShardLoader;
use crate::{
    mod_database::{
        dao_database::{
            dao_archive_metadata::ArchiveMetadataDao, dao_map_archive_shard::MapArchiveDataShardDao,
            dao_recovery_data_shard::RecoveryDataShardDao,
            dao_recovery_parity_shard::RecoveryParityShardDao,
        },
        impl_database::Database,
    },
};
use anyhow::{Context, Result};
use reed_solomon_erasure::galois_8::ReedSolomon;
use std::collections::HashSet;
use std::result::Result::Ok;

/// 恢复数据结构体
#[derive(Debug)]
pub struct RecoveryData {
    pub group_id: i64,
    pub can_recover: bool,
    pub available_count: u64,
    pub lost_limit: u64,
    pub shards: Vec<Option<Vec<u8>>>,
    pub recover_group_info: RecoveryGroupInfoWithShardDetail,
}

/// 恢复规划器 trait
/// 负责分析和准备恢复所需的数据
pub trait RecoveryPlanner {
    /// 获取恢复组信息
    fn get_recovery_group_by_archive_id(
        &self,
        archive_id: i64,
        database: &Database,
    ) -> Result<RecoveryGroupInfoWithShardDetail>;

    /// 验证恢复组的完整性
    fn verify_group_by_archive_id(&self, archive_id: i64, database: &Database) -> Result<bool>;

    /// 准备恢复所需的数据
    fn prepare_recovery(
        &self,
        archive_id: i64,
        database: &Database,
        shard_loader: &dyn ShardLoader,
        reed_solomon: &ReedSolomon,
    ) -> Result<RecoveryData>;
}

/// 默认恢复规划器实现
pub struct DefaultRecoveryPlanner;

impl DefaultRecoveryPlanner {
    pub fn new() -> Self {
        Self
    }

    /// 判断恢复可行性
    fn determine_recovery_feasibility(
        &self,
        available_count: u64,
        required_shards: u64,
        archive_id: i64,
    ) -> bool {
        let can_recover = available_count >= required_shards;

        log::debug!(
            "归档文件 {} 恢复检查：最终可用分片数量 {} >= 所需分片数量 {} ? {}",
            archive_id,
            available_count,
            required_shards,
            can_recover
        );

        if !can_recover {
            log::debug!(
                "归档文件 {} 不满足恢复条件：最终可用分片数量 {} < 所需分片数量 {}",
                archive_id,
                available_count,
                required_shards
            );
        }

        can_recover
    }
}

impl RecoveryPlanner for DefaultRecoveryPlanner {
    fn get_recovery_group_by_archive_id(
        &self,
        archive_id: i64,
        database: &Database,
    ) -> Result<RecoveryGroupInfoWithShardDetail> {
        let recovery_group_dao = RecoveryGroupDao::new(database.conn.clone());
        let archive_metadata_dao = ArchiveMetadataDao::new(database.conn.clone());
        let recovery_data_shard_dao = RecoveryDataShardDao::new(database.conn.clone());
        let recovery_parity_shard_dao = RecoveryParityShardDao::new(database.conn.clone());
        let map_archive_data_shard_dao = MapArchiveDataShardDao::new(database.conn.clone());

        // 1. 通过 archive_id 找到对应的 data_shard
        let map_entries = map_archive_data_shard_dao
            .find_by_archive_id(archive_id, None)
            .context("查找归档文件映射关系失败")?;

        if map_entries.is_empty() {
            return Err(anyhow::anyhow!("未找到归档文件 {} 的映射关系", archive_id));
        }

        // 取第一个映射关系（理论上一个归档文件只属于一个灾备组）
        let map_entry = &map_entries[0];
        let data_shard_id = map_entry.shard_id;

        // 2. 通过 data_shard_id 找到 group_id
        let data_shard = recovery_data_shard_dao
            .find_by_id(data_shard_id)?
            .ok_or_else(|| anyhow::anyhow!("未找到数据分片 ID: {}", data_shard_id))?;

        let group_id = data_shard.group_id;

        // 3. 获取恢复组信息
        let recovery_group = recovery_group_dao
            .find_by_id(group_id, None)?
            .ok_or_else(|| anyhow::anyhow!("未找到灾备组 ID: {}", group_id))?;

        let group_info = RecoveryGroupInfo {
            group_id: recovery_group.id.unwrap_or(0), // 灾备组 ID 应该是有效的
            data_shards_stored: recovery_group.data_shards_stored,
            num_data_shards: recovery_group.num_data_shards as u64,
            num_parity_shards: recovery_group.num_parity_shards as u64,
            shard_size: recovery_group.shard_size as u64,
        };

        // 4. 获取该组的所有数据分片
        let data_shards = recovery_data_shard_dao.find_multi_shards_by_group_id(group_id)?;

        // 5. 获取该组的所有校验分片
        let parity_shards = recovery_parity_shard_dao.find_multi_shards_by_group_id(group_id)?;

        // 6. 获取该组的所有归档文件
        let mut archive_ids = HashSet::new();
        let mut map_archive_data_shards = Vec::new();

        // 通过数据分片获取映射关系
        for data_shard in &data_shards {
            if let Some(map_entry) = map_archive_data_shard_dao.find_by_shard_id(data_shard.id)? {
                archive_ids.insert(map_entry.archive_id);
                map_archive_data_shards.push(map_entry);
            }
        }

        let mut archives = Vec::new();
        for &archive_id in &archive_ids {
            if let Some(archive) = archive_metadata_dao.find_by_id(archive_id)? {
                archives.push(archive);
            }
        }

        Ok(RecoveryGroupInfoWithShardDetail {
            group_info,
            data_shards,
            parity_shards,
            archives,
            map_archive_data_shards,
        })
    }

    fn verify_group_by_archive_id(&self, archive_id: i64, database: &Database) -> Result<bool> {
        let recovery_group_detail: RecoveryGroupInfoWithShardDetail =
            self.get_recovery_group_by_archive_id(archive_id, database)?;

        if !FileIntegrityValidator::verify_group_sha256(&recovery_group_detail)? {
            return Ok(false);
        }
        if !ReedSolomonValidator::verify_recovery_group_with_reed_solomon(
            &recovery_group_detail.group_info,
            &recovery_group_detail.data_shards,
            &recovery_group_detail.parity_shards,
            &recovery_group_detail.archives,
        )? {
            return Ok(false);
        }
        Ok(true)
    }

    fn prepare_recovery(
        &self,
        archive_id: i64,
        database: &Database,
        shard_loader: &dyn ShardLoader,
        reed_solomon: &ReedSolomon,
    ) -> Result<RecoveryData> {
        let group_info = self.get_recovery_group_by_archive_id(archive_id, database)?;

        // Reed-Solomon 纠删码：需要至少 num_data_shards 个分片才能重建数据
        let required_shards_for_recovery = group_info.group_info.num_data_shards;
        let lost_limit = group_info.group_info.num_parity_shards;

        // 根据是否存储了 data shard 来计算可用分片
        let (shards, available_count) = if group_info.group_info.data_shards_stored {
            shard_loader.load_stored_shards(&group_info, database, reed_solomon)?
        } else {
            shard_loader.generate_shards_from_archives(&group_info, database, reed_solomon)?
        };

        // 现在基于最终的可用分片数量判断是否可以恢复
        let can_recover = self.determine_recovery_feasibility(available_count, required_shards_for_recovery, archive_id);

        Ok(RecoveryData {
            group_id: group_info.group_info.group_id,
            can_recover,
            available_count,
            lost_limit,
            shards,
            recover_group_info: group_info,
        })
    }
}