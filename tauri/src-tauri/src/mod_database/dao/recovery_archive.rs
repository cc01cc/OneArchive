//! 灾备组与归档文件映射相关数据库操作 DAO

use rusqlite::{Connection, Result as SqliteResult};
use std::sync::Arc;

use crate::mod_database::{
    common::DatabaseCommonOperations, queries, schema_recovery::RecoveryGroupArchive,
};

/// 灾备组与归档文件映射数据访问对象
pub struct RecoveryArchiveDao {
    conn: Arc<Connection>,
}

impl RecoveryArchiveDao {
    /// 创建新的 DAO 实例
    pub fn new(conn: Arc<Connection>) -> Self {
        Self { conn }
    }

    /// 创建灾备组与归档文件映射
    pub fn insert_recovery_group_archive(
        &self,
        group_id: i64,
        archive_id: i64,
    ) -> SqliteResult<i64> {
        self.conn.insert_and_get_id(
            queries::recovery_archive::INSERT,
            rusqlite::params![group_id, archive_id],
        )
    }

    /// 根据组 ID 查找所有关联的归档文件
    pub fn find_recovery_group_archives_by_group_id(
        &self,
        group_id: i64,
    ) -> SqliteResult<Vec<RecoveryGroupArchive>> {
        self.conn.query_multiple(
            queries::recovery_archive::SELECT_BY_GROUP_ID,
            rusqlite::params![group_id],
        )
    }

    /// 根据归档 ID 查找所有关联的灾备组
    pub fn find_recovery_groups_by_archive_id(
        &self,
        archive_id: i64,
    ) -> SqliteResult<Vec<RecoveryGroupArchive>> {
        self.conn.query_multiple(
            queries::recovery_archive::SELECT_BY_ARCHIVE_ID,
            rusqlite::params![archive_id],
        )
    }

    /// 删除灾备组与归档文件的映射关系
    pub fn delete_recovery_group_archive(
        &self,
        group_id: i64,
        archive_id: i64,
    ) -> SqliteResult<()> {
        self.conn.execute(
            queries::recovery_archive::DELETE_BY_GROUP_AND_ARCHIVE_ID,
            rusqlite::params![group_id, archive_id],
        )?;
        Ok(())
    }
}
