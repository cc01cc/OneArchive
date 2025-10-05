use serde::{Deserialize, Serialize};

use crate::{
    mod_database::{
        schema::ArchiveMetadata,
        schema_recovery::{MapArchiveDataShard, RecoveryDataShard, RecoveryParityShard},
    },
};
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RecoveryGroupInfo {
    pub group_id: i64,
    pub data_shards_stored: bool,
    pub num_data_shards: u64,
    pub num_parity_shards: u64,
    pub shard_size: u64,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RecoveryGroupInfoWithShardId {
    pub group_info: RecoveryGroupInfo,
    pub archive_ids: Vec<i64>,
    pub data_shard_ids: Vec<i64>,
    pub parity_shard_ids: Vec<i64>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RecoveryGroupInfoWithShardDetail {
    pub group_info: RecoveryGroupInfo,
    pub data_shards: Vec<RecoveryDataShard>,
    pub parity_shards: Vec<RecoveryParityShard>,
    pub archives: Vec<ArchiveMetadata>,
    pub map_archive_data_shards: Vec<MapArchiveDataShard>,
}

/// 定义分片的类型和来源
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum ShardType {
    // /// 原始归档文件
    // OriginalArchive,
    /// 从原始归档生成的数据分片
    DataShard,
    /// 校验分片
    ParityShard,
}

impl ShardType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::DataShard => "Data Shard",
            Self::ParityShard => "Parity Shard",
            // Self::OriginalArchive => "Original Archive",
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RecoveryConfig {
    pub data_shards: usize,      // 数据 shard 数量
    pub parity_shards: usize,    // 校验 shard 数量
    pub storage_dir: String,     // 存储目录
    pub prefix: String,          // 文件名前缀
    pub store_data_shards: bool, // 是否存储数据分片
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TauriRecoveryConfig {
    pub data_shards: usize,
    pub parity_shards: usize,
    pub storage_dir: String,
    pub prefix: String,
    pub store_data_shards: bool,
}

#[derive(Serialize, Deserialize)]
pub struct GenerateRecoveryResult {
    pub group_id: String,
    pub data_shards_count: usize,
    pub parity_shards_count: usize,
    pub message: String,
}
