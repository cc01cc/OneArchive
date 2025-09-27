//! 归档数据块相关数据库操作 DAO

use rusqlite::{Connection, Result as SqliteResult};
use std::sync::Arc;

use crate::mod_database::{
    common::DatabaseCommonOperations, constants::ChunkStatus, database::Database, queries,
    schema::ArchiveChunk,
};

/// 归档数据块数据访问对象
pub struct ArchiveChunkDao {
    conn: Arc<Connection>,
}

impl ArchiveChunkDao {
    /// 创建新的 DAO 实例
    pub fn new(conn: Arc<Connection>) -> Self {
        Self { conn }
    }

    /// 插入归档数据块
    pub fn insert_archive_chunk(&self, chunk: &ArchiveChunk) -> SqliteResult<i64> {
        let normalized_path =
            Database::normalize_directory_path(&Some(chunk.chunk_relative_path.to_string()));

        self.conn.execute(
            queries::archive_chunk::INSERT,
            rusqlite::params![
                chunk.archive_id,
                chunk.chunk_name,
                chunk.chunk_size,
                chunk.chunk_hash,
                chunk.chunk_mtime,
                normalized_path,
                chunk.status.as_str()
            ],
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    /// 更新归档数据块
    pub fn update_archive_chunk(&self, chunk: &ArchiveChunk) -> SqliteResult<()> {
        self.conn.execute(
            queries::archive_chunk::UPDATE,
            rusqlite::params![
                chunk.archive_id,
                chunk.chunk_name,
                chunk.chunk_size,
                chunk.chunk_hash,
                chunk.chunk_mtime,
                chunk.chunk_relative_path,
                chunk.status.as_str(),
                chunk.id
            ],
        )?;
        Ok(())
    }

    /// 根据 ID 查找归档数据块
    pub fn find_archive_chunk_by_id(&self, id: i64) -> SqliteResult<Option<ArchiveChunk>> {
        self.conn
            .query_single(queries::archive_chunk::SELECT_BY_ID, rusqlite::params![id])
    }

    /// 根据归档 ID 查找归档数据块列表
    pub fn find_archive_chunks_by_archive_id(
        &self,
        archive_id: i64,
    ) -> SqliteResult<Vec<ArchiveChunk>> {
        self.conn.query_multiple(
            queries::archive_chunk::SELECT_BY_ARCHIVE_ID,
            rusqlite::params![archive_id],
        )
    }

    /// 根据状态查找归档数据块
    pub fn find_archive_chunks_by_status(
        &self,
        status: Option<ChunkStatus>,
    ) -> SqliteResult<Vec<ArchiveChunk>> {
        let status_str = status.as_ref().map(|s| s.as_str());
        self.conn.query_multiple(
            queries::archive_chunk::SELECT_BY_STATUS,
            rusqlite::params![status_str],
        )
    }

    /// 根据 chunk hash 查找归档数据块
    pub fn find_by_chunk_hash(
        &self,
        chunk_hash: &str,
    ) -> SqliteResult<Option<ArchiveChunk>> {
        self.conn.query_single(
            queries::archive_chunk::SELECT_BY_CHUNK_HASH,
            rusqlite::params![chunk_hash],
        )
    }
}
