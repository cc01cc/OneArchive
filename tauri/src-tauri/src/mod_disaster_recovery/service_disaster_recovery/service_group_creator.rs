//! 灾备组创建器
//! 负责创建灾备组记录

use anyhow::Result;
use uuid::Uuid;

use crate::mod_database::constants::RecoveryGroupStatus;
use crate::mod_database::dao_database::dao_recovery_group::RecoveryGroupDao;
use crate::mod_database::impl_database::Database;
use crate::mod_database::schema_recovery::CreateRecoveryGroupParams;
use crate::mod_disaster_recovery::core_disaster_recovery::model_disaster_recovery::RecoveryConfig;

/// 灾备组创建器 trait
pub trait GroupCreator {
    /// 创建灾备组记录
    fn create_recovery_group_record(&self, database: &Database, shard_size: u64) -> Result<i64>;
}

/// 默认灾备组创建器实现
pub struct DefaultGroupCreator {
    config: RecoveryConfig,
}

impl DefaultGroupCreator {
    pub fn new(config: RecoveryConfig) -> Self {
        Self { config }
    }
}

impl GroupCreator for DefaultGroupCreator {
    fn create_recovery_group_record(&self, database: &Database, shard_size: u64) -> Result<i64> {
        let recovery_group_dao = RecoveryGroupDao::new(database.conn.clone());

        let params = CreateRecoveryGroupParams {
            group_id: Uuid::new_v4().to_string(),
            num_data_shards: self.config.data_shards as u64,
            num_parity_shards: self.config.parity_shards as u64,
            shard_size,
            data_shards_stored: self.config.store_data_shards,
            status: RecoveryGroupStatus::Health,
            last_verified: None,
        };

        let group_id = recovery_group_dao.insert_recovery_group(params)?;
        Ok(group_id)
    }
}