//! 分片映射服务
//!
//! 提供归档文件、数据分片和灾备组之间的映射查找功能

use anyhow::{Context, Result};
use rusqlite::Connection;
use std::sync::Arc;

use crate::mod_database::dao_database::dao_map_archive_shard::MapArchiveDataShardDao;
use crate::mod_database::dao_database::dao_recovery_data_shard::RecoveryDataShardDao;

/// 分片映射服务
pub struct ShardMappingService {
    conn: Arc<Connection>,
}

impl ShardMappingService {
    /// 创建新的分片映射服务实例
    pub fn new(conn: Arc<Connection>) -> Self {
        Self { conn }
    }

    /// 根据归档文件 ID 和灾备组 ID 查找数据分片 ID
    ///
    /// 返回该归档文件在指定灾备组中对应的数据分片 ID
    pub fn find_shard_id_by_archive_and_group(
        &self, archive_id: i64, group_id: i64,
    ) -> Result<i64> {
        let map_dao = MapArchiveDataShardDao::new(self.conn.clone());
        let data_shard_dao = RecoveryDataShardDao::new(self.conn.clone());

        // 查找该归档文件的所有映射
        let map_entries =
            map_dao.find_by_archive_id(archive_id, None).context("查询归档文件映射关系失败")?;

        // 找到属于指定灾备组的分片
        for map_entry in map_entries {
            let data_shard = data_shard_dao
                .find_by_id(map_entry.shard_id)?
                .ok_or_else(|| anyhow::anyhow!("未找到数据分片 ID: {}", map_entry.shard_id))?;

            if data_shard.group_id == group_id {
                return Ok(map_entry.shard_id);
            }
        }

        Err(anyhow::anyhow!("未找到归档文件 {} 在灾备组 {} 中的映射关系", archive_id, group_id))
    }

    /// 根据归档文件 ID 和数据分片 ID 查找灾备组 ID
    pub fn find_group_id_by_archive_and_shard(
        &self, archive_id: i64, shard_id: i64,
    ) -> Result<i64> {
        let data_shard_dao = RecoveryDataShardDao::new(self.conn.clone());

        // 验证映射关系存在
        let map_dao = MapArchiveDataShardDao::new(self.conn.clone());
        if map_dao.find_by_archive_id_and_shard_id(archive_id, shard_id, None)?.is_none() {
            return Err(anyhow::anyhow!(
                "未找到归档文件 {} 和分片 {} 的映射关系",
                archive_id,
                shard_id
            ));
        }

        // 获取分片的灾备组 ID
        let data_shard = data_shard_dao
            .find_by_id(shard_id)?
            .ok_or_else(|| anyhow::anyhow!("未找到数据分片 ID: {}", shard_id))?;

        Ok(data_shard.group_id)
    }

    /// 根据灾备组 ID 和数据分片 ID 查找归档文件 ID
    pub fn find_archive_id_by_group_and_shard(&self, group_id: i64, shard_id: i64) -> Result<i64> {
        let data_shard_dao = RecoveryDataShardDao::new(self.conn.clone());
        let map_dao = MapArchiveDataShardDao::new(self.conn.clone());

        // 验证分片属于指定灾备组
        let data_shard = data_shard_dao
            .find_by_id(shard_id)?
            .ok_or_else(|| anyhow::anyhow!("未找到数据分片 ID: {}", shard_id))?;

        if data_shard.group_id != group_id {
            return Err(anyhow::anyhow!("数据分片 {} 不属于灾备组 {}", shard_id, group_id));
        }

        // 获取映射的归档文件 ID
        let map_entry = map_dao
            .find_by_shard_id(shard_id)?
            .ok_or_else(|| anyhow::anyhow!("未找到分片 {} 的映射关系", shard_id))?;

        Ok(map_entry.archive_id)
    }
}
