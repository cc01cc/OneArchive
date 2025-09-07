//! 数据库表结构定义
use serde::{Deserialize, Serialize};

use super::constants::{DirectoryStatus, RootStatus, FileStatus, ArchiveStatus, AssetStatus, MapFileAssetStatus};
use rusqlite::{Row, Result as SqliteResult};
use std::convert::TryFrom;

/// 根目录信息表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfoRoot {
    /// 目录唯一标识
    pub id: Option<i64>,
    /// 目录名称
    pub root_name: String,
    /// 根目录路径
    pub root_path: String,
    /// 状态：HEALTH/RECYCLED
    pub status: RootStatus,
    /// 创建时间戳
    pub created_at: Option<i64>,
    /// 更新时间戳
    pub updated_at: Option<i64>,
}

impl InfoRoot {
    /// 创建一个新的 InfoRoot 实例
    pub fn new(root_path: String, root_name: String, status: RootStatus) -> Self {
        Self {
            id: None,
            root_name,
            root_path,
            status,
            created_at: None,
            updated_at: None,
        }
    }
}

impl TryFrom<&Row<'_>> for InfoRoot {
    type Error = rusqlite::Error;
    fn try_from(row: &Row) -> SqliteResult<Self> {
        Ok(InfoRoot {
                id: row.get("id")?,
                root_name: row.get("root_name")?,
                root_path: row.get("root_path")?,
                status: RootStatus::from_str(&row.get::<_, String>("status")?),
                created_at: row.get("created_at")?,
                updated_at: row.get("updated_at")?,
        })
    }
}

/// 目录索引表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfoDirectory {
    /// 目录唯一标识
    pub id: Option<i64>,
    /// 目录根目录ID
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
    pub fn new(root_id: i64, directory_name: String, directory_mtime: i64, directory_path: Option<String>, status: DirectoryStatus) -> Self {
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
    /// 所属目录ID
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
        directory_id: i64,
        file_name: String,
        file_size: i64,
        file_mtime: i64,
        file_hash: Option<String>,
        status: FileStatus
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

/// 存档元数据表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveMetadata {
    /// 存档唯一标识
    pub id: Option<i64>,
    /// 存档名称
    pub archive_name: String,
    /// 存档大小限制
    pub archive_limit_size: i64,
    /// 存档哈希值
    pub archive_hash: Option<String>,
    /// 是否压缩 (0/1)
    pub is_compressed: i32,
    /// 压缩算法
    pub compressed_algorithm: Option<String>,
    /// 是否加密 (0/1)
    pub is_encrypted: i32,
    /// 加密算法
    pub encryption_algorithm: Option<String>,
    /// 状态：HEALTH/RECYCLED
    pub status: ArchiveStatus,
    /// 创建时间戳
    pub created_at: Option<i64>,
    /// 更新时间戳
    pub updated_at: Option<i64>,
}

impl ArchiveMetadata {
    /// 创建一个新的 ArchiveMetadata 实例
    pub fn new(
        archive_name: String,
        archive_limit_size: i64,
        is_compressed: i32,
        is_encrypted: i32,
        status: ArchiveStatus
    ) -> Self {
        Self {
            id: None,
            archive_name,
            archive_limit_size,
            archive_hash: None,
            is_compressed,
            compressed_algorithm: None,
            is_encrypted,
            encryption_algorithm: None,
            status,
            created_at: None,
            updated_at: None,
        }
    }
}

impl TryFrom<&Row<'_>> for ArchiveMetadata {
    type Error = rusqlite::Error;
    fn try_from(row: &Row) -> SqliteResult<Self> {
        Ok(ArchiveMetadata {
                id: row.get("id")?,
                archive_name: row.get("archive_name")?,
                archive_limit_size: row.get("archive_limit_size")?,
                archive_hash: row.get("archive_hash")?,
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

/// 存档内容表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveAsset {
    /// 资源唯一标识
    pub id: Option<i64>,
    /// 所属存档ID
    pub archive_id: i64,
    /// 资源名称
    pub asset_name: String,
    /// 资源大小
    pub asset_size: i64,
    /// 资源哈希值
    pub asset_hash: String,
    /// 修改时间 (unix 时间戳)
    pub asset_mtime: i64,
    /// 资源相对路径
    pub asset_relative_path: String,
    /// 状态：HEALTH/RECYCLED
    pub status: AssetStatus,
    /// 创建时间戳
    pub created_at: Option<i64>,
    /// 更新时间戳
    pub updated_at: Option<i64>,
}

impl ArchiveAsset {
    /// 创建一个新的 ArchiveAsset 实例
    pub fn new(
        archive_id: i64,
        asset_name: String,
        asset_size: i64,
        asset_hash: String,
        asset_mtime: i64,
        asset_relative_path: String,
        status: AssetStatus
    ) -> Self {
        Self {
            id: None,
            archive_id,
            asset_name,
            asset_size,
            asset_hash,
            asset_mtime,
            asset_relative_path,
            status,
            created_at: None,
            updated_at: None,
        }
    }
}

impl TryFrom<&Row<'_>> for ArchiveAsset {
    type Error = rusqlite::Error;
    fn try_from(row: &Row) -> SqliteResult<Self> {
        Ok(ArchiveAsset {
                id: row.get("id")?,
                archive_id: row.get("archive_id")?,
                asset_name: row.get("asset_name")?,
                asset_size: row.get("asset_size")?,
                asset_hash: row.get("asset_hash")?,
                asset_mtime: row.get("asset_mtime")?,
                asset_relative_path: row.get("asset_relative_path")?,
                status: AssetStatus::from_str(&row.get::<_, String>("status")?),
                created_at: row.get("created_at")?,
                updated_at: row.get("updated_at")?,
        })
    }
}

/// 文件与存档资源映射表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapFileAsset {
    /// 映射唯一标识
    pub id: Option<i64>,
    /// 文件ID
    pub file_id: i64,
    /// 资源ID
    pub asset_id: i64,
    /// 卷序号
    pub volume_order: i32,
    /// 状态：HEALTH/RECYCLED
    pub status: MapFileAssetStatus,
    /// 创建时间戳
    pub created_at: Option<i64>,
    /// 更新时间戳
    pub updated_at: Option<i64>,
}

impl MapFileAsset {
    /// 创建一个新的 MapFileAsset 实例
    pub fn new(file_id: i64, asset_id: i64, volume_order: i32, status: MapFileAssetStatus) -> Self {
        Self {
            id: None,
            file_id,
            asset_id,
            volume_order,
            status,
            created_at: None,
            updated_at: None,
        }
    }
}

impl TryFrom<&Row<'_>> for MapFileAsset {
    type Error = rusqlite::Error;
    fn try_from(row: &Row) -> SqliteResult<Self> {
        Ok(MapFileAsset {
                id: row.get("id")?,
                file_id: row.get("file_id")?,
                asset_id: row.get("asset_id")?,
                volume_order: row.get("volume_order")?,
                status: MapFileAssetStatus::from_str(&row.get::<_, String>("status")?),
                created_at: row.get("created_at")?,
                updated_at: row.get("updated_at")?,
        })
    }
}

/// 文件列表视图
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewFile {
    /// 根目录ID
    pub root_id: Option<i64>,
    /// 根目录路径
    pub root_path: String,
    /// 根目录状态
    pub root_status: String,
    /// 目录ID
    pub directory_id: i64,
    /// 目录相对路径
    pub directory_path: Option<String>,
    /// 目录修改时间
    pub directory_mtime: i64,
    /// 目录状态
    pub directory_status: String,
    /// 文件ID
    pub file_id: i64,
    /// 文件名
    pub file_name: String,
    /// 文件大小
    pub file_size: i64,
    /// 文件修改时间
    pub file_mtime: i64,
    /// 文件哈希
    pub file_hash: Option<String>,
    /// 文件状态
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
/// 资源列表视图
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewAsset {
    /// 存档ID
    pub archive_id: Option<i64>,
    /// 存档名称
    pub archive_name: String,
    /// 存档状态
    pub archive_status: String,
    /// 资源ID
    pub asset_id: i64,
    /// 资源名称
    pub asset_name: String,
    /// 资源大小
    pub asset_size: i64,
    /// 资源哈希
    pub asset_hash: String,
    /// 资源修改时间
    pub asset_mtime: i64,
    /// 资源相对路径
    pub asset_relative_path: String,
    /// 资源状态
    pub asset_status: String,
    /// 文件ID
    pub file_id: Option<i64>,
    /// 卷序号
    pub volume_order: Option<i32>,
}

impl TryFrom<&Row<'_>> for ViewAsset {
    type Error = rusqlite::Error;
    fn try_from(row: &Row) -> SqliteResult<Self> {
        Ok(ViewAsset {
                archive_id: row.get("archive_id")?,
                archive_name: row.get("archive_name")?,
                archive_status: row.get("archive_status")?,
                asset_id: row.get("asset_id")?,
                asset_name: row.get("asset_name")?,
                asset_size: row.get("asset_size")?,
                asset_hash: row.get("asset_hash")?,
                asset_mtime: row.get("asset_mtime")?,
                asset_relative_path: row.get("asset_relative_path")?,
                asset_status: row.get("asset_status")?,
                file_id: row.get("file_id")?,
                volume_order: row.get("volume_order")?,
        })
    }
}