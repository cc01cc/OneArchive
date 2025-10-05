//! 数据分片相关数据库操作 DAO

use rusqlite::{Connection, Result as SqliteResult};
use std::sync::Arc;

use crate::mod_database::{
    common::DatabaseCommonOperations, constants_sql, schema_recovery::RecoveryDataShard,
};

/// 数据分片数据访问对象
pub struct RecoveryDataShardDao {
    conn: Arc<Connection>,
}

impl RecoveryDataShardDao {
    /// 创建新的 DAO 实例
    pub fn new(conn: Arc<Connection>) -> Self {
        Self { conn }
    }

    /// 创建数据分片记录
    pub fn insert_recovery_data_shard(
        &self,
        group_id: i64,
        shard_index: i64,
        shard_path: Option<&str>,
        shard_hash: &str,
    ) -> SqliteResult<i64> {
        self.conn.insert_and_get_id(
            constants_sql::recovery_data_shard::INSERT,
            rusqlite::params![group_id, shard_index, shard_path, shard_hash],
        )
    }

    /// 根据组 ID 查找所有数据分片
    pub fn find_multi_shards_by_group_id(
        &self,
        group_id: i64,
    ) -> SqliteResult<Vec<RecoveryDataShard>> {
        self.conn.query_multiple(
            constants_sql::recovery_data_shard::SELECT_BY_GROUP_ID,
            rusqlite::params![group_id],
        )
    }

    /// 根据 ID 查找数据分片
    pub fn find_by_id(
        &self,
        id: i64,
    ) -> SqliteResult<Option<RecoveryDataShard>> {
        self.conn.query_single(
            constants_sql::recovery_data_shard::SELECT_BY_ID,
            rusqlite::params![id],
        )
    }

    /// 更新数据分片状态
    pub fn update_recovery_data_shard_status(&self, id: i64, status: &str) -> SqliteResult<()> {
        self.conn.execute(
            constants_sql::recovery_data_shard::UPDATE_STATUS,
            rusqlite::params![status, id],
        )?;
        Ok(())
    }

    /// 删除数据分片
    pub fn delete_recovery_data_shard(&self, id: i64) -> SqliteResult<()> {
        self.conn.execute(
            constants_sql::recovery_data_shard::DELETE_BY_ID,
            rusqlite::params![id],
        )?;
        Ok(())
    }
}
