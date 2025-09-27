use rusqlite::{Connection, Result as SqliteResult};
use std::sync::Arc;

use crate::mod_database::{
    common::DatabaseCommonOperations, queries, schema_recovery::ArchiveShardMap,
};

/// 归档文件与数据分片映射数据访问对象
pub struct ArchiveShardMapDao {
    conn: Arc<Connection>,
}

impl ArchiveShardMapDao {
    /// 创建新的 DAO 实例
    pub fn new(conn: Arc<Connection>) -> Self {
        Self { conn }
    }

    /// 创建归档文件与数据分片映射记录
    pub fn insert_archive_shard_map(&self, shard_id: i64, archive_id: i64) -> SqliteResult<i64> {
        self.conn.insert_and_get_id(
            queries::archive_shard_map::INSERT,
            rusqlite::params![shard_id, archive_id],
        )
    }

    /// 根据分片 ID 查找映射记录
    pub fn find_archive_shard_map_by_shard_id(
        &self,
        shard_id: i64,
    ) -> SqliteResult<Option<ArchiveShardMap>> {
        self.conn.query_single(
            queries::archive_shard_map::SELECT_BY_SHARD_ID,
            rusqlite::params![shard_id],
        )
    }

    /// 根据归档文件 ID 查找映射记录
    pub fn find_archive_shard_maps_by_archive_id(
        &self,
        archive_id: i64,
    ) -> SqliteResult<Vec<ArchiveShardMap>> {
        self.conn.query_multiple(
            queries::archive_shard_map::SELECT_BY_ARCHIVE_ID,
            rusqlite::params![archive_id],
        )
    }

    /// 更新映射记录状态
    pub fn update_archive_shard_map_status(&self, shard_id: i64, status: &str) -> SqliteResult<()> {
        self.conn.execute(
            queries::archive_shard_map::UPDATE_STATUS,
            rusqlite::params![status, shard_id],
        )?;
        Ok(())
    }

    /// 删除映射记录
    pub fn delete_archive_shard_map(&self, shard_id: i64) -> SqliteResult<()> {
        self.conn.execute(
            queries::archive_shard_map::DELETE_BY_SHARD_ID,
            rusqlite::params![shard_id],
        )?;
        Ok(())
    }
}
