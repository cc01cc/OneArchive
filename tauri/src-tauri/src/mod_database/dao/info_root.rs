//! 根目录相关数据库操作 DAO

use rusqlite::{Connection, Result as SqliteResult};
use std::sync::Arc;

use crate::mod_database::{common::DatabaseCommonOperations, queries, schema::InfoRoot};

/// 根目录数据访问对象
pub struct InfoRootDao {
    conn: Arc<Connection>,
}

impl InfoRootDao {
    /// 创建新的 DAO 实例
    pub fn new(conn: Arc<Connection>) -> Self {
        Self { conn }
    }

    /// 添加根目录信息
    pub fn add_root_directory(
        &self,
        root_path: &str,
        root_name: &str,
        status: &str,
    ) -> SqliteResult<i64> {
        self.conn.execute(
            queries::info_root::INSERT_OR_REPLACE,
            rusqlite::params![root_path, root_name, status],
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    /// 根据路径查找根目录信息
    pub fn find_root_info_by_path(&self, root_path: &str) -> SqliteResult<Option<InfoRoot>> {
        self.conn.query_single(
            queries::info_root::SELECT_BY_PATH,
            rusqlite::params![root_path],
        )
    }

    /// 更新根目录状态
    pub fn update_root_status(&self, root_id: i64, status: &str) -> SqliteResult<()> {
        self.conn.execute(
            queries::info_root::UPDATE_STATUS,
            rusqlite::params![status, root_id],
        )?;
        Ok(())
    }

    /// 查找所有根目录信息
    pub fn find_all_root_info(&self) -> SqliteResult<Vec<InfoRoot>> {
        self.conn
            .query_multiple(queries::info_root::SELECT_ALL, rusqlite::params![])
    }
}
