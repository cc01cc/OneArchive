//! 数据块视图相关数据库操作 DAO

use rusqlite::{Connection, Result as SqliteResult};
use std::sync::Arc;

use crate::mod_database::{common::DatabaseCommonOperations, constants_sql, schema::ViewChunk};

/// 数据块视图数据访问对象
pub struct ViewChunkDao {
    conn: Arc<Connection>,
}

impl ViewChunkDao {
    /// 创建新的 DAO 实例
    pub fn new(conn: Arc<Connection>) -> Self {
        Self { conn }
    }

    /// 根据归档 ID 查找视图数据块
    pub fn find_view_chunks_by_archive_id(&self, archive_id: i64) -> SqliteResult<Vec<ViewChunk>> {
        self.conn.query_multiple(
            constants_sql::view_chunk::SELECT_BY_ARCHIVE_ID,
            rusqlite::params![archive_id],
        )
    }

    /// 根据文件 ID 查找视图数据块
    pub fn find_by_file_id(&self, file_id: i64) -> SqliteResult<Vec<ViewChunk>> {
        self.conn.query_multiple(
            constants_sql::view_chunk::SELECT_BY_FILE_ID,
            rusqlite::params![file_id],
        )
    }
}
