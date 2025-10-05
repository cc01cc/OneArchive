//! 校验分片相关数据库操作 DAO

use rusqlite::{Connection, Result as SqliteResult};
use std::sync::Arc;

use crate::mod_database::{
    common::DatabaseCommonOperations, constants_sql, schema_recovery::RecoveryParityShard,
};

/// 校验分片数据访问对象
pub struct RecoveryParityShardDao {
    conn: Arc<Connection>,
}

impl RecoveryParityShardDao {
    /// 创建新的 DAO 实例
    pub fn new(conn: Arc<Connection>) -> Self {
        Self { conn }
    }

    /// 创建校验分片记录
    pub fn insert_recovery_parity_shard(
        &self,
        group_id: i64,
        shard_index: i64,
        shard_path: &str,
        shard_hash: &str,
    ) -> SqliteResult<i64> {
        self.conn.insert_and_get_id(
            constants_sql::recovery_parity_shard::INSERT,
            rusqlite::params![group_id, shard_index, shard_path, shard_hash],
        )
    }

    /// 根据组 ID 查找所有校验分片
    pub fn find_multi_shards_by_group_id(
        &self,
        group_id: i64,
    ) -> SqliteResult<Vec<RecoveryParityShard>> {
        self.conn.query_multiple(
            constants_sql::recovery_parity_shard::SELECT_BY_GROUP_ID,
            rusqlite::params![group_id],
        )
    }

    /// 根据 ID 查找校验分片
    pub fn find_shard_by_id(
        &self,
        id: i64,
    ) -> SqliteResult<Option<RecoveryParityShard>> {
        self.conn.query_single(
            constants_sql::recovery_parity_shard::SELECT_BY_ID,
            rusqlite::params![id],
        )
    }

    /// 更新校验分片状态
    pub fn update_recovery_parity_shard_status(&self, id: i64, status: &str) -> SqliteResult<()> {
        self.conn.execute(
            constants_sql::recovery_parity_shard::UPDATE_STATUS,
            rusqlite::params![status, id],
        )?;
        Ok(())
    }

    /// 删除校验分片
    pub fn delete_recovery_parity_shard(&self, id: i64) -> SqliteResult<()> {
        self.conn.execute(
            constants_sql::recovery_parity_shard::DELETE_BY_ID,
            rusqlite::params![id],
        )?;
        Ok(())
    }
}
