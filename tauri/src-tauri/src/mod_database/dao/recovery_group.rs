//! 灾备组相关数据库操作 DAO

use rusqlite::{Connection, Result as SqliteResult, named_params, params};
use std::sync::Arc;

use crate::mod_database::{
    common::DatabaseCommonOperations,
    queries,
    schema_recovery::{CreateRecoveryGroupParams, RecoveryGroup},
};

/// 灾备组数据访问对象
pub struct RecoveryGroupDao {
    conn: Arc<Connection>,
}

impl RecoveryGroupDao {
    /// 创建新的 DAO 实例
    pub fn new(conn: Arc<Connection>) -> Self {
        Self { conn }
    }

    /// 创建灾备组
    pub fn insert_recovery_group(&self, params: CreateRecoveryGroupParams) -> SqliteResult<i64> {
        self.conn.insert_and_get_id_named(
            queries::recovery_group::INSERT,
            named_params! {
                ":group_id": params.group_id,
                ":num_data_shards": params.num_data_shards,
                ":num_parity_shards": params.num_parity_shards,
                ":data_shards_stored": params.data_shards_stored,
                ":status": params.status
            },
        )
    }

    /// 根据组 ID 查找灾备组
    pub fn find_recovery_group_by_group_id(
        &self,
        group_id: &str,
    ) -> SqliteResult<Option<RecoveryGroup>> {
        self.conn.query_single(
            queries::recovery_group::SELECT_BY_GROUP_ID,
            params![group_id],
        )
    }

    /// 根据 ID 查找灾备组
    pub fn find_recovery_group_by_id(&self, id: i64) -> SqliteResult<Option<RecoveryGroup>> {
        self.conn
            .query_single(queries::recovery_group::SELECT_BY_ID, params![id])
    }

    /// 更新灾备组状态
    pub fn update_recovery_group_status(&self, id: i64, status: &str) -> SqliteResult<()> {
        self.conn
            .execute(queries::recovery_group::UPDATE_STATUS, params![status, id])?;
        Ok(())
    }

    /// 删除灾备组
    pub fn delete_recovery_group(&self, id: i64) -> SqliteResult<()> {
        self.conn
            .execute(queries::recovery_group::DELETE_BY_ID, params![id])?;
        Ok(())
    }
}
