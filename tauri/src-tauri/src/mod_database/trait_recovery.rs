//! 灾备相关数据库操作 trait 定义

use rusqlite::Result as SqliteResult;

use crate::mod_database::schema_recovery::{
    RecoveryDataShard,  RecoveryParityShard,
};


/// TODO 和 dao 绑定数据分片操作 trait
pub trait RecoveryDataShardOperations {
    /// 创建数据分片记录
    fn insert_recovery_data_shard(
        &self,
        group_id: i64,
        shard_index: i64,
        shard_path: Option<&str>, // 可选的存储路径
        shard_hash: &str,
    ) -> SqliteResult<i64>;

    /// 根据组 ID 查找所有数据分片
    fn find_recovery_data_shards_by_group_id(
        &self,
        group_id: i64,
    ) -> SqliteResult<Vec<RecoveryDataShard>>;

    /// 根据 ID 查找数据分片
    fn find_recovery_data_shard_by_id(&self, id: i64) -> SqliteResult<Option<RecoveryDataShard>>;

    /// 更新数据分片状态
    fn update_recovery_data_shard_status(&self, id: i64, status: &str) -> SqliteResult<()>;

    /// 删除数据分片
    fn delete_recovery_data_shard(&self, id: i64) -> SqliteResult<()>;
}

/// 校验分片操作 trait
pub trait RecoveryParityShardOperations {
    /// 创建校验分片记录
    fn insert_recovery_parity_shard(
        &self,
        group_id: i64,
        shard_index: i64,
        shard_path: &str, // 必需的存储路径
        shard_hash: &str,
    ) -> SqliteResult<i64>;

    /// 根据组 ID 查找所有校验分片
    fn find_recovery_parity_shards_by_group_id(
        &self,
        group_id: i64,
    ) -> SqliteResult<Vec<RecoveryParityShard>>;

    /// 根据 ID 查找校验分片
    fn find_recovery_parity_shard_by_id(
        &self,
        id: i64,
    ) -> SqliteResult<Option<RecoveryParityShard>>;

    /// 更新校验分片状态
    fn update_recovery_parity_shard_status(&self, id: i64, status: &str) -> SqliteResult<()>;

    /// 删除校验分片
    fn delete_recovery_parity_shard(&self, id: i64) -> SqliteResult<()>;
}
