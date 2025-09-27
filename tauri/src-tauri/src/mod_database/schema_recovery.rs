//! 灾备相关数据结构定义

use rusqlite::{Result as SqliteResult, Row};
use serde::{Deserialize, Serialize};

/// 灾备组结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryGroup {
    pub id: i64,
    pub group_id: String,
    pub num_data_shards: i64,
    pub num_parity_shards: i64,
    pub data_shards_stored: bool,
    pub status: String,
    pub created_at: i64,
    pub updated_at: i64,
}

impl TryFrom<&Row<'_>> for RecoveryGroup {
    type Error = rusqlite::Error;

    fn try_from(row: &Row) -> SqliteResult<Self> {
        Ok(RecoveryGroup {
            id: row.get("id")?,
            group_id: row.get("group_id")?,
            num_data_shards: row.get("num_data_shards")?,
            num_parity_shards: row.get("num_parity_shards")?,
            data_shards_stored: row.get("data_shards_stored")?,
            status: row.get("status")?,
            created_at: row.get("created_at")?,
            updated_at: row.get("updated_at")?,
        })
    }
}

/// 数据分片结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryDataShard {
    pub id: i64,
    pub group_id: i64,
    pub shard_index: i64,
    pub shard_path: Option<String>, // 数据分片可能没有存储路径
    pub shard_hash: String,
    pub status: String, // HEALTH, CORRUPTED, MISSING
    pub last_verified: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}

impl TryFrom<&Row<'_>> for RecoveryDataShard {
    type Error = rusqlite::Error;

    fn try_from(row: &Row) -> SqliteResult<Self> {
        Ok(RecoveryDataShard {
            id: row.get("id")?,
            group_id: row.get("group_id")?,
            shard_index: row.get("shard_index")?,
            shard_path: row.get("shard_path")?,
            shard_hash: row.get("shard_hash")?,
            status: row.get("status")?,
            last_verified: row.get("last_verified")?,
            created_at: row.get("created_at")?,
            updated_at: row.get("updated_at")?,
        })
    }
}

/// 校验分片结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryParityShard {
    pub id: i64,
    pub group_id: i64,
    pub shard_index: i64,
    pub shard_path: String, // 校验分片总是有存储路径
    pub shard_hash: String,
    pub status: String, // HEALTH, CORRUPTED, MISSING
    pub last_verified: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}

impl TryFrom<&Row<'_>> for RecoveryParityShard {
    type Error = rusqlite::Error;

    fn try_from(row: &Row) -> SqliteResult<Self> {
        Ok(RecoveryParityShard {
            id: row.get("id")?,
            group_id: row.get("group_id")?,
            shard_index: row.get("shard_index")?,
            shard_path: row.get("shard_path")?,
            shard_hash: row.get("shard_hash")?,
            status: row.get("status")?,
            last_verified: row.get("last_verified")?,
            created_at: row.get("created_at")?,
            updated_at: row.get("updated_at")?,
        })
    }
}

/// 灾备组与归档文件映射结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryGroupArchive {
    pub id: i64,
    pub group_id: i64,
    pub archive_id: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

impl TryFrom<&Row<'_>> for RecoveryGroupArchive {
    type Error = rusqlite::Error;

    fn try_from(row: &Row) -> SqliteResult<Self> {
        Ok(RecoveryGroupArchive {
            id: row.get("id")?,
            group_id: row.get("group_id")?,
            archive_id: row.get("archive_id")?,
            created_at: row.get("created_at")?,
            updated_at: row.get("updated_at")?,
        })
    }
}

/// 创建灾备组参数结构
#[derive(Debug, Clone)]
pub struct CreateRecoveryGroupParams {
    pub group_id: String,
    pub num_data_shards: i64,
    pub num_parity_shards: i64,
    pub data_shards_stored: bool,
    pub status: String,
}

/// 归档文件与数据分片映射结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveShardMap {
    pub id: i64,
    pub shard_id: i64,
    pub archive_id: i64,
    pub status: String,
    pub created_at: i64,
    pub updated_at: i64,
}

impl TryFrom<&Row<'_>> for ArchiveShardMap {
    type Error = rusqlite::Error;

    fn try_from(row: &Row) -> SqliteResult<Self> {
        Ok(ArchiveShardMap {
            id: row.get("id")?,
            shard_id: row.get("shard_id")?,
            archive_id: row.get("archive_id")?,
            status: row.get("status")?,
            created_at: row.get("created_at")?,
            updated_at: row.get("updated_at")?,
        })
    }
}
