//! 灾备相关数据库操作 trait 定义

use rusqlite::Result as SqliteResult;

use crate::mod_database::schema_recovery::{
    RecoveryDataShard, RecoveryGroup, RecoveryGroupArchive, RecoveryParityShard,
};

// /// 灾备组操作 trait
// pub trait RecoveryGroupOperations {
//     /// 创建灾备组
//     fn insert_recovery_group(
//         &self,
//         group_id: &str,
//         num_data_shards: i64,
//         num_parity_shards: i64,
//         data_shards_stored: bool,
//     ) -> SqliteResult<i64>;

//     /// 根据组 ID 查找灾备组
//     fn find_recovery_group_by_group_id(
//         &self,
//         group_id: &str,
//     ) -> SqliteResult<Option<RecoveryGroup>>;

//     /// 根据 ID 查找灾备组
//     fn find_recovery_group_by_id(&self, id: i64) -> SqliteResult<Option<RecoveryGroup>>;

//     /// 更新灾备组状态
//     fn update_recovery_group_status(&self, id: i64, status: &str) -> SqliteResult<()>;

//     /// 删除灾备组
//     fn delete_recovery_group(&self, id: i64) -> SqliteResult<()>;
// }

/// 数据分片操作 trait
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

/// 灾备组与归档文件映射操作 trait
pub trait RecoveryGroupArchiveOperations {
    /// 创建灾备组与归档文件映射
    fn insert_recovery_group_archive(&self, group_id: i64, archive_id: i64) -> SqliteResult<i64>;

    /// 根据组 ID 查找所有关联的归档文件
    fn find_recovery_group_archives_by_group_id(
        &self,
        group_id: i64,
    ) -> SqliteResult<Vec<RecoveryGroupArchive>>;

    /// 根据归档 ID 查找所有关联的灾备组
    fn find_recovery_groups_by_archive_id(
        &self,
        archive_id: i64,
    ) -> SqliteResult<Vec<RecoveryGroupArchive>>;

    /// 删除灾备组与归档文件的映射关系
    fn delete_recovery_group_archive(&self, group_id: i64, archive_id: i64) -> SqliteResult<()>;
}
