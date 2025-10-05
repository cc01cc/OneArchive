use rusqlite::{Connection, Result as SqliteResult};
use std::sync::Arc;

use crate::mod_database::{
    common::DatabaseCommonOperations, constants::MapArchiveDataShardStatus, constants_sql,
    schema_recovery::MapArchiveDataShard,
};

/// 归档文件与数据分片映射数据访问对象
pub struct MapArchiveDataShardDao {
    conn: Arc<Connection>,
}

impl MapArchiveDataShardDao {
    /// 创建新的 DAO 实例
    pub fn new(conn: Arc<Connection>) -> Self {
        Self { conn }
    }

    /// 创建归档文件与数据分片映射记录
    pub fn insert_archive_shard_map(&self, shard_id: i64, archive_id: i64, archive_order: u64, original_archive_hash: &str) -> SqliteResult<i64> {
        self.conn.insert_and_get_id(
            constants_sql::archive_shard_map::INSERT,
            rusqlite::params![shard_id, archive_id, archive_order, original_archive_hash],
        )
    }

    /// 根据分片 ID 查找映射记录
    pub fn find_by_shard_id(
        &self, shard_id: i64,
    ) -> SqliteResult<Option<MapArchiveDataShard>> {
        self.conn.query_single(
            constants_sql::archive_shard_map::SELECT_BY_SHARD_ID,
            rusqlite::params![shard_id],
        )
    }

    /// 根据归档文件 ID 查找映射记录
    pub fn find_by_archive_id(
        &self, archive_id: i64, status: Option<&MapArchiveDataShardStatus>,
    ) -> SqliteResult<Vec<MapArchiveDataShard>> {
        self.conn.query_multiple(
            constants_sql::archive_shard_map::SELECT_BY_ARCHIVE_ID,
            rusqlite::params![archive_id, status],
        )
    }

    // 此方法已废弃，因为 map_archive_data_shard 表不再包含 group_id 字段
    // pub fn find_by_archive_id_and_group_id(
    //     &self, archive_id: i64, group_id: i64, status: Option<&MapArchiveDataShardStatus>,
    // ) -> SqliteResult<Option<MapArchiveDataShard>> {
    //     self.conn.query_single(
    //         queries::archive_shard_map::SELECT_BY_ARCHIVE_ID_AND_GROUP_ID,
    //         rusqlite::params![archive_id, group_id, status],
    //     )
    // }

    pub fn find_by_archive_id_and_shard_id(
        &self, archive_id: i64, shard_id: i64, status: Option<&MapArchiveDataShardStatus>,
    ) -> SqliteResult<Option<MapArchiveDataShard>> {
        self.conn.query_single(
            constants_sql::archive_shard_map::SELECT_BY_ARCHIVE_ID_AND_SHARD_ID,
            rusqlite::params![archive_id, shard_id, status],
        )
    }

    /// 更新映射记录状态
    pub fn update_archive_shard_map_status(&self, shard_id: i64, status: &str) -> SqliteResult<()> {
        self.conn.execute(
            constants_sql::archive_shard_map::UPDATE_STATUS,
            rusqlite::params![status, shard_id],
        )?;
        Ok(())
    }

    /// 删除映射记录
    pub fn delete_archive_shard_map(&self, shard_id: i64) -> SqliteResult<()> {
        self.conn
            .execute(constants_sql::archive_shard_map::DELETE_BY_SHARD_ID, rusqlite::params![shard_id])?;
        Ok(())
    }
}
