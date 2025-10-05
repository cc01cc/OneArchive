//! 文件与归档数据块映射相关数据库操作 DAO

use rusqlite::{Connection, Result as SqliteResult};
use std::sync::Arc;

use crate::mod_database::{
    common::DatabaseCommonOperations, constants::MapFileChunkStatus, constants_sql, schema::MapFileChunk,
};

/// 文件与归档数据块映射数据访问对象
pub struct MapFileChunkDao {
    conn: Arc<Connection>,
}

impl MapFileChunkDao {
    /// 创建新的 DAO 实例
    pub fn new(conn: Arc<Connection>) -> Self {
        Self { conn }
    }

    /// 插入文件与归档数据块映射
    pub fn insert(&self, map: &MapFileChunk) -> SqliteResult<i64> {
        self.conn.execute(
            constants_sql::map_file_chunk::INSERT,
            rusqlite::params![map.file_id, map.chunk_id, map.volume_order, map.status.as_str()],
        )?;

        Ok(self.conn.last_insert_rowid())
    }
    /// 根据文件 ID、数据块 ID 和卷序号查找映射记录
    pub fn find_map_file_chunk_by_file_id_chunk_id_and_volume_order(
        &self, file_id: i64, chunk_id: i64, volume_order: i32,
    ) -> SqliteResult<Option<MapFileChunk>> {
        self.conn.query_single(
            constants_sql::map_file_chunk::SELECT_BY_FILE_ID_CHUNK_ID_VOLUME_ORDER,
            rusqlite::params![file_id, chunk_id, volume_order],
        )
    }
    /// 更新文件与归档数据块映射
    pub fn update_map_file_chunk(&self, map: &MapFileChunk) -> SqliteResult<()> {
        self.conn.execute(
            constants_sql::map_file_chunk::UPDATE,
            rusqlite::params![
                map.file_id,
                map.chunk_id,
                map.volume_order,
                map.status.as_str(),
                map.id
            ],
        )?;
        Ok(())
    }

    /// 根据 ID 查找文件与归档数据块映射
    pub fn find_map_file_chunk_by_id(&self, id: i64) -> SqliteResult<Option<MapFileChunk>> {
        self.conn.query_single(constants_sql::map_file_chunk::SELECT_BY_ID, rusqlite::params![id])
    }

    /// 根据文件 ID 查找文件与归档数据块映射
    pub fn find_map_file_chunk_by_file_id(&self, file_id: i64) -> SqliteResult<Vec<MapFileChunk>> {
        self.conn
            .query_multiple(constants_sql::map_file_chunk::SELECT_BY_FILE_ID, rusqlite::params![file_id])
    }

    /// 根据文件 ID 查找文件与归档数据块映射，并按 volume_order 升序排序
    pub fn find_by_file_id_ordered(
        &self, file_id: i64,
    ) -> SqliteResult<Vec<MapFileChunk>> {
        self.conn.query_multiple(
            constants_sql::map_file_chunk::SELECT_BY_FILE_ID_ORDERED,
            rusqlite::params![file_id],
        )
    }

    /// 根据数据块 ID 查找文件与归档数据块映射
    pub fn find_map_file_chunk_by_chunk_id(
        &self, chunk_id: i64,
    ) -> SqliteResult<Vec<MapFileChunk>> {
        self.conn.query_multiple(
            constants_sql::map_file_chunk::SELECT_BY_CHUNK_ID,
            rusqlite::params![chunk_id],
        )
    }

    /// 根据状态查找文件与归档数据块映射
    pub fn find_map_file_chunk_by_status(
        &self, status: Option<MapFileChunkStatus>,
    ) -> SqliteResult<Vec<MapFileChunk>> {
        let status_str = status.as_ref().map(|s| s.as_str());
        self.conn.query_multiple(
            constants_sql::map_file_chunk::SELECT_BY_STATUS,
            rusqlite::params![status_str],
        )
    }
}
