//! 文件视图相关数据库操作 DAO

use rusqlite::{Connection, Result as SqliteResult};
use std::sync::Arc;

use crate::mod_database::{
    common::DatabaseCommonOperations, constants::FileStatus, constants_sql, schema::ViewFile,
};

/// 文件视图数据访问对象
pub struct ViewFileDao {
    conn: Arc<Connection>,
}

impl ViewFileDao {
    /// 创建新的 DAO 实例
    pub fn new(conn: Arc<Connection>) -> Self {
        Self { conn }
    }

    /// 根据根目录 ID 查找所有视图文件
    pub fn find_view_files_by_root_id(&self, root_id: i64) -> SqliteResult<Vec<ViewFile>> {
        self.conn.query_multiple(
            constants_sql::view_file::SELECT_BY_ROOT_ID,
            rusqlite::params![root_id],
        )
    }

    /// 根据目录 ID 查找视图文件
    pub fn find_view_files_by_directory_id(
        &self,
        directory_id: i64,
    ) -> SqliteResult<Vec<ViewFile>> {
        self.conn.query_multiple(
            constants_sql::view_file::SELECT_BY_DIRECTORY_ID,
            rusqlite::params![directory_id],
        )
    }

    /// 根据 root_id 和 status 获取视图文件
    pub fn find_by_root_id_and_status(
        &self,
        root_id: i64,
        status: Option<FileStatus>,
    ) -> SqliteResult<Vec<ViewFile>> {
        let status_str = status.as_ref().map(|s| s.as_str());
        self.conn.query_multiple(
            constants_sql::view_file::SELECT_BY_ROOT_ID_AND_STATUS,
            rusqlite::params![root_id, status_str],
        )
    }

    /// 根据 hash 和 status 获取视图文件
    pub fn find_by_hash_and_status(
        &self,
        hash: &str,
        status: Option<FileStatus>,
    ) -> SqliteResult<Vec<ViewFile>> {
        let status_str = status.as_ref().map(|s| s.as_str());
        self.conn.query_multiple(
            constants_sql::view_file::SELECT_BY_HASH_AND_STATUS,
            rusqlite::params![hash, status_str],
        )
    }
}
