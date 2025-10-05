//! 灾备相关数据结构定义

use chrono::Utc;
use one_archive_macros::FromSqliteRow;
use rusqlite::{Result as SqliteResult, Row};
use serde::{Deserialize, Serialize};

use crate::mod_database::constants::{
    MapArchiveDataShardStatus, RecoveryDataShardStatus,
    RecoveryGroupStatus, RecoveryParityShardStatus,
};
/// 灾备组结构
#[derive(Debug, Clone, Serialize, Deserialize, FromSqliteRow)]
pub struct RecoveryGroup {
    pub id: Option<i64>,
    pub group_id: String,
    pub num_data_shards: u64,
    pub num_parity_shards: u64,
    pub shard_size: u64,
    pub data_shards_stored: bool,
    pub last_verified: Option<i64>,
    #[sqlite(from_str)]
    pub status: RecoveryGroupStatus,
    pub created_at: i64,
    pub updated_at: i64,
}

/// 创建灾备组参数结构
#[derive(Debug, Clone)]
pub struct CreateRecoveryGroupParams {
    pub group_id: String,
    pub num_data_shards: u64,
    pub num_parity_shards: u64,
    pub shard_size: u64,
    pub data_shards_stored: bool,
    pub status: RecoveryGroupStatus,
    pub last_verified: Option<i64>,
}

impl RecoveryGroup {
    pub fn new(params: CreateRecoveryGroupParams) -> Self {
        Self {
            id: None,
            group_id: params.group_id,
            num_data_shards: params.num_data_shards,
            num_parity_shards: params.num_parity_shards,
            shard_size: params.shard_size,
            data_shards_stored: params.data_shards_stored,
            last_verified: params.last_verified,
            status: params.status,
            created_at: Utc::now().timestamp(),
            updated_at: Utc::now().timestamp(),
        }
    }
}


/// 数据分片结构
#[derive(Debug, Clone, Serialize, Deserialize, FromSqliteRow)]
pub struct RecoveryDataShard {
    pub id: i64,
    pub group_id: i64,
    pub shard_index: i64,
    pub shard_path: Option<String>,
    pub shard_hash: String,
    #[sqlite(from_str)]
    pub status: RecoveryDataShardStatus,
    pub last_verified: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}

/// 校验分片结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryParityShard {
    pub id: i64,
    pub group_id: i64,
    pub shard_index: i64,
    pub shard_path: String, // 校验分片总是有存储路径
    pub shard_hash: String,
    pub status: RecoveryParityShardStatus,
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
            status: RecoveryParityShardStatus::from_str(&row.get::<_, String>("status")?),
            last_verified: row.get("last_verified")?,
            created_at: row.get("created_at")?,
            updated_at: row.get("updated_at")?,
        })
    }
}

/// 归档文件与数据分片映射结构
#[derive(Debug, Clone, Serialize, Deserialize, FromSqliteRow)]
pub struct MapArchiveDataShard {
    pub id: i64,
    pub shard_id: i64,
    pub archive_id: i64,
    pub archive_order: u64,
    pub original_archive_hash: String,
    #[sqlite(from_str)]
    pub status: MapArchiveDataShardStatus,
    pub created_at: i64,
    pub updated_at: i64,
}
