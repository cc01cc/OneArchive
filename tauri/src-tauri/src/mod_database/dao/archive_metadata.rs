use std::sync::Arc;

use crate::mod_database::common::DatabaseCommonOperations;
use crate::mod_database::constants::{ArchiveStatus, ChunkStatus, MapFileChunkStatus};
use crate::mod_database::database::Database;
use crate::mod_database::schema::{
    ArchiveChunk, ArchiveMetadata, CreateArchiveMetadataParams, MapFileChunk, ViewChunk,
};
use crate::mod_database::{impl_initialize, queries};
use log::warn;
use rusqlite::{Connection, Result as SqliteResult, named_params, params};

/// 归档元数据数据访问对象
pub struct ArchiveMetadataDao {
    conn: Arc<Connection>,
}

impl ArchiveMetadataDao {
    /// 创建新的 DAO 实例
    pub fn new(conn: Arc<Connection>) -> Self {
        Self { conn }
    }

    /// 创建归档元数据记录
    pub fn create(
        &self,
        params: CreateArchiveMetadataParams,
    ) -> SqliteResult<i64> {
        let normalized_path = Database::normalize_directory_path(&Some(params.archive_uri));

        self.conn.insert_and_get_id_named(
            queries::archive_metadata::INSERT,
            named_params! {
                ":archive_name": params.archive_name,
                ":archive_uri": normalized_path,
                ":archive_hash": None::<String>,
                ":archive_size": None::<u64>,
                ":archive_limit_size": params.archive_limit_size,
                ":is_compressed": params.is_compressed,
                ":compressed_algorithm": params.compression_algorithm,
                ":is_encrypted": params.is_encrypted,
                ":encryption_algorithm": params.encryption_algorithm,
                ":status": params.status
            },
        )
    }

    /// 根据 ID 查找归档元数据
    pub fn find_by_id(&self, id: i64) -> SqliteResult<Option<ArchiveMetadata>> {
        self.conn
            .query_single(queries::archive_metadata::SELECT_BY_ID, params![id])
    }

    /// 根据 URI 查找归档元数据
    pub fn find_by_uri(&self, uri: &str) -> SqliteResult<Option<ArchiveMetadata>> {
        let normalized_uri = Database::normalize_directory_path(&Some(uri.to_string()));
        self.conn.query_single(
            queries::archive_metadata::SELECT_BY_URI,
            params![normalized_uri],
        )
    }

    /// 更新归档元数据状态
    pub fn update_status(&self, id: i64, status: &str) -> SqliteResult<()> {
        self.conn.execute(
            queries::archive_metadata::UPDATE_STATUS,
            params![status, id],
        )?;
        Ok(())
    }

    /// 删除归档元数据
    pub fn delete(&self, id: i64) -> SqliteResult<()> {
        self.conn
            .execute(queries::archive_metadata::DELETE_BY_ID, params![id])?;
        Ok(())
    }

    /// 根据状态查找归档元数据
    pub fn find_by_status_and_root_id(
        &self,
        root_id: i64,
        status: Option<&str>,
    ) -> SqliteResult<Vec<ArchiveMetadata>> {
        let result = self.conn.query_multiple(
            queries::archive_metadata::SELECT_BY_STATUS_AND_ROOT_ID,
            params![root_id, status],
        )?;
        Ok(result)
    }
}
