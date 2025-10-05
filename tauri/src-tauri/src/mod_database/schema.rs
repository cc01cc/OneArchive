//! 数据库表结构定义
use chrono::Utc;
use one_archive_macros::FromSqliteRow;
use serde::{Deserialize, Serialize};

use super::constants::{
    ArchiveStatus, ChunkStatus, DirectoryStatus, FileStatus, MapFileChunkStatus, RootStatus,
};
use rusqlite::{Result as SqliteResult, Row};
use std::convert::TryFrom;

/// 根目录信息表
#[derive(Debug, Clone, Serialize, Deserialize, FromSqliteRow)]
pub struct InfoRoot {
    /// 目录唯一标识
    pub id: Option<i64>,
    /// 目录名称
    pub root_name: String,
    /// 根目录路径
    pub root_path: String,
    /// 状态：HEALTH/RECYCLED
    #[sqlite(from_str)]
    pub status: RootStatus,
    /// 创建时间戳
    pub created_at: Option<i64>,
    /// 更新时间戳
    pub updated_at: Option<i64>,
}

impl InfoRoot {
    /// 创建一个新的 InfoRoot 实例
    pub fn new(root_path: String, root_name: String, status: RootStatus) -> Self {
        Self { id: None, root_name, root_path, status, created_at: None, updated_at: None }
    }
}


/// 目录索引表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfoDirectory {
    /// 目录唯一标识
    pub id: Option<i64>,
    /// 目录根目录 ID
    pub root_id: i64,
    /// 目录名
    pub directory_name: String,
    /// 修改时间 (unix 时间戳)
    pub directory_mtime: i64,
    /// 相对路径
    pub directory_path: Option<String>,
    /// 状态：HEALTH/RECYCLED
    pub status: DirectoryStatus,
    /// 创建时间戳
    pub created_at: Option<i64>,
    /// 更新时间戳
    pub updated_at: Option<i64>,
}

impl InfoDirectory {
    /// 创建一个新的 InfoDirectory 实例
    pub fn new(
        root_id: i64, directory_name: String, directory_mtime: i64, directory_path: Option<String>,
        status: DirectoryStatus,
    ) -> Self {
        Self {
            id: None,
            root_id,
            directory_name,
            directory_mtime,
            directory_path,
            status,
            created_at: None,
            updated_at: None,
        }
    }
}
impl TryFrom<&Row<'_>> for InfoDirectory {
    type Error = rusqlite::Error;
    fn try_from(row: &Row) -> SqliteResult<Self> {
        Ok(InfoDirectory {
            id: row.get("id")?,
            root_id: row.get("root_id")?,
            directory_name: row.get("directory_name")?,
            directory_mtime: row.get("directory_mtime")?,
            directory_path: row.get("directory_path")?,
            status: DirectoryStatus::from_str(&row.get::<_, String>("status")?),
            created_at: row.get("created_at")?,
            updated_at: row.get("updated_at")?,
        })
    }
}
/// 文件索引表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfoFile {
    /// 文件唯一标识
    pub id: Option<i64>,
    /// 所属目录 ID
    pub directory_id: i64,
    /// 文件名
    pub file_name: String,
    /// 文件大小
    pub file_size: i64,
    /// 修改时间 (unix 时间戳)
    pub file_mtime: i64,
    /// 文件哈希值
    pub file_hash: Option<String>,
    /// 状态：HEALTH/RECYCLED
    pub status: FileStatus,
    /// 创建时间戳
    pub created_at: Option<i64>,
    /// 更新时间戳
    pub updated_at: Option<i64>,
}

impl InfoFile {
    /// 创建一个新的 InfoFile 实例
    pub fn new(
        directory_id: i64, file_name: String, file_size: i64, file_mtime: i64,
        file_hash: Option<String>, status: FileStatus,
    ) -> Self {
        Self {
            id: None,
            directory_id,
            file_name,
            file_size,
            file_mtime,
            file_hash,
            status,
            created_at: None,
            updated_at: None,
        }
    }
}

impl TryFrom<&Row<'_>> for InfoFile {
    type Error = rusqlite::Error;
    fn try_from(row: &Row) -> SqliteResult<Self> {
        Ok(InfoFile {
            id: row.get("id")?,
            directory_id: row.get("directory_id")?,
            file_name: row.get("file_name")?,
            file_size: row.get("file_size")?,
            file_mtime: row.get("file_mtime")?,
            file_hash: row.get("file_hash")?,
            status: FileStatus::from_str(&row.get::<_, String>("status")?),
            created_at: row.get("created_at")?,
            updated_at: row.get("updated_at")?,
        })
    }
}

/// 归档元数据表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveMetadata {
    pub id: Option<i64>,
    pub archive_name: String,
    pub archive_uri: String,
    pub archive_limit_size: i64,
    pub archive_hash: Option<String>,
    pub archive_size: Option<i64>,
    pub is_compressed: i32,
    pub compressed_algorithm: Option<String>,
    pub is_encrypted: i32,
    pub encryption_algorithm: Option<String>,
    /// 状态：HEALTH/RECYCLED
    pub status: ArchiveStatus,
    pub created_at: i64,
    pub updated_at: i64,
}

impl TryFrom<&Row<'_>> for ArchiveMetadata {
    type Error = rusqlite::Error;
    fn try_from(row: &Row) -> SqliteResult<Self> {
        Ok(ArchiveMetadata {
            id: row.get("id")?,
            archive_name: row.get("archive_name")?,
            archive_uri: row.get("archive_uri")?,
            archive_limit_size: row.get("archive_limit_size")?,
            archive_hash: row.get("archive_hash")?,
            archive_size: row.get("archive_size")?,
            is_compressed: row.get("is_compressed")?,
            compressed_algorithm: row.get("compressed_algorithm")?,
            is_encrypted: row.get("is_encrypted")?,
            encryption_algorithm: row.get("encryption_algorithm")?,
            status: ArchiveStatus::from_str(&row.get::<_, String>("status")?),
            created_at: row.get("created_at")?,
            updated_at: row.get("updated_at")?,
        })
    }
}

impl ArchiveMetadata {
    /// 创建一个新的 ArchiveMetadata 实例
    pub fn new(
        archive_uri: String, archive_name: String, archive_limit_size: i64, is_compressed: i32,
        is_encrypted: i32, status: ArchiveStatus,
    ) -> Self {
        Self {
            id: None,
            archive_name,
            archive_uri,
            archive_limit_size,
            archive_hash: None,
            archive_size: None,
            is_compressed,
            compressed_algorithm: None,
            is_encrypted,
            encryption_algorithm: None,
            status,
            created_at: Utc::now().timestamp(),
            updated_at: Utc::now().timestamp(),
        }
    }
}

/// 归档内容表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveChunk {
    pub id: Option<i64>,
    pub archive_id: i64,
    pub chunk_name: String,
    pub chunk_size: i64,
    pub chunk_hash: String,
    /// 修改时间 (unix 时间戳)
    pub chunk_mtime: i64,
    pub chunk_relative_path: String,
    pub status: ChunkStatus,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
}

impl ArchiveChunk {
    /// 创建一个新的 ArchiveChunk 实例
    pub fn new(
        archive_id: i64, chunk_name: String, chunk_size: i64, chunk_hash: String, chunk_mtime: i64,
        chunk_relative_path: String, status: ChunkStatus,
    ) -> Self {
        Self {
            id: None,
            archive_id,
            chunk_name,
            chunk_size,
            chunk_hash,
            chunk_mtime,
            chunk_relative_path,
            status,
            created_at: None,
            updated_at: None,
        }
    }
}

impl TryFrom<&Row<'_>> for ArchiveChunk {
    type Error = rusqlite::Error;
    fn try_from(row: &Row) -> SqliteResult<Self> {
        Ok(ArchiveChunk {
            id: row.get("id")?,
            archive_id: row.get("archive_id")?,
            chunk_name: row.get("chunk_name")?,
            chunk_size: row.get("chunk_size")?,
            chunk_hash: row.get("chunk_hash")?,
            chunk_mtime: row.get("chunk_mtime")?,
            chunk_relative_path: row.get("chunk_relative_path")?,
            status: ChunkStatus::from_str(&row.get::<_, String>("status")?),
            created_at: row.get("created_at")?,
            updated_at: row.get("updated_at")?,
        })
    }
}

/// 文件与归档数据块映射表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapFileChunk {
    pub id: Option<i64>,
    pub file_id: i64,
    pub chunk_id: i64,
    pub volume_order: i32,
    pub status: MapFileChunkStatus,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
}

impl MapFileChunk {
    pub fn new(file_id: i64, chunk_id: i64, volume_order: i32, status: MapFileChunkStatus) -> Self {
        Self {
            id: None,
            file_id,
            chunk_id,
            volume_order,
            status,
            created_at: None,
            updated_at: None,
        }
    }
}

impl TryFrom<&Row<'_>> for MapFileChunk {
    type Error = rusqlite::Error;
    fn try_from(row: &Row) -> SqliteResult<Self> {
        Ok(MapFileChunk {
            id: row.get("id")?,
            file_id: row.get("file_id")?,
            chunk_id: row.get("chunk_id")?,
            volume_order: row.get("volume_order")?,
            status: MapFileChunkStatus::from_str(&row.get::<_, String>("status")?),
            created_at: row.get("created_at")?,
            updated_at: row.get("updated_at")?,
        })
    }
}

/// 文件列表视图
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewFile {
    pub root_id: Option<i64>,
    pub root_path: String,
    pub root_status: String,
    pub directory_id: i64,
    pub directory_path: Option<String>,
    pub directory_mtime: i64,
    pub directory_status: String,
    pub file_id: i64,
    pub file_name: String,
    pub file_size: i64,
    pub file_mtime: i64,
    pub file_hash: Option<String>,
    pub file_status: String,
}

impl TryFrom<&Row<'_>> for ViewFile {
    type Error = rusqlite::Error;

    fn try_from(row: &Row) -> SqliteResult<Self> {
        Ok(ViewFile {
            root_id: row.get("root_id")?,
            root_path: row.get("root_path")?,
            root_status: row.get("root_status")?,
            directory_id: row.get("directory_id")?,
            directory_path: row.get("directory_path")?,
            directory_mtime: row.get("directory_mtime")?,
            directory_status: row.get("directory_status")?,
            file_id: row.get("file_id")?,
            file_name: row.get("file_name")?,
            file_size: row.get("file_size")?,
            file_mtime: row.get("file_mtime")?,
            file_hash: row.get("file_hash")?,
            file_status: row.get("file_status")?,
        })
    }
}
/// 数据块列表视图
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewChunk {
    pub archive_id: Option<i64>,
    pub archive_uri: String,
    pub archive_name: String,
    pub archive_status: String,
    pub chunk_id: i64,
    pub chunk_name: String,
    pub chunk_size: i64,
    pub chunk_hash: String,
    pub chunk_mtime: i64,
    pub chunk_relative_path: String,
    pub chunk_status: String,
    pub file_id: Option<i64>,
    /// 卷序号
    pub volume_order: Option<i32>,
}

impl TryFrom<&Row<'_>> for ViewChunk {
    type Error = rusqlite::Error;
    fn try_from(row: &Row) -> SqliteResult<Self> {
        Ok(ViewChunk {
            archive_id: row.get("archive_id")?,
            archive_uri: row.get("archive_uri")?,
            archive_name: row.get("archive_name")?,
            archive_status: row.get("archive_status")?,
            chunk_id: row.get("chunk_id")?,
            chunk_name: row.get("chunk_name")?,
            chunk_size: row.get("chunk_size")?,
            chunk_hash: row.get("chunk_hash")?,
            chunk_mtime: row.get("chunk_mtime")?,
            chunk_relative_path: row.get("chunk_relative_path")?,
            chunk_status: row.get("chunk_status")?,
            file_id: row.get("file_id")?,
            volume_order: row.get("volume_order")?,
        })
    }
}

/// 创建归档元数据的参数
#[derive(Debug, Clone)]
pub struct CreateArchiveMetadataParams {
    pub archive_uri: String,
    pub archive_name: String,
    pub archive_limit_size: u64,
    pub is_compressed: i8,
    pub is_encrypted: i8,
    pub compression_algorithm: Option<String>,
    pub encryption_algorithm: Option<String>,
    pub status: ArchiveStatus,
}
