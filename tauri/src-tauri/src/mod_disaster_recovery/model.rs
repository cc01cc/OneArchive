use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{mod_database::{schema::ArchiveMetadata, schema_recovery::{RecoveryDataShard, RecoveryParityShard}}, mod_disaster_recovery::shard_alignment::{DataShardMeta, ParityShardMeta}};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RecoveryGroup {
    pub group_id: Uuid,
    pub data_shards: Vec<RecoveryDataShard>,
    pub parity_shards: Vec<RecoveryParityShard>,
    pub original_data: Vec<ArchiveMetadata>,
}

/// 定义分片的类型和来源
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum ShardType {
    /// 原始归档文件
    OriginalArchive,
    /// 从原始归档生成的数据分片
    DataShard,
    /// 校验分片
    ParityShard,
}

#[derive(Serialize, Deserialize)]
pub struct RecoveryConfig {
    pub data_shards: usize,      // 数据 shard 数量
    pub parity_shards: usize,    // 校验 shard 数量
    pub storage_dir: String,     // 存储目录
    pub prefix: String,          // 文件名前缀
    pub store_data_shards: bool, // 是否存储数据分片
}

#[derive(Serialize, Deserialize)]
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
