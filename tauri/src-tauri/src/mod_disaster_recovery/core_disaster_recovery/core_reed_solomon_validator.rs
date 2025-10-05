//! Reed-Solomon 验证器模块
//! 提供基于 Reed-Solomon 纠删码的验证功能

use std::path::PathBuf;

use anyhow::Result;
use reed_solomon_erasure::galois_8::ReedSolomon;

use crate::{
    mod_database::schema::ArchiveMetadata,
    mod_database::schema_recovery::{RecoveryDataShard, RecoveryParityShard},
    mod_disaster_recovery::core_disaster_recovery::{
        model_disaster_recovery::RecoveryGroupInfo,
        core_shard_alignment::align_shard,
    },
};

/// Reed-Solomon 验证器
pub struct ReedSolomonValidator;

impl ReedSolomonValidator {
    /// 使用 Reed-Solomon 验证恢复组
    pub fn verify_recovery_group_with_reed_solomon(
        group_info: &RecoveryGroupInfo,
        data_shards: &[RecoveryDataShard],
        parity_shards: &[RecoveryParityShard],
        archives: &[ArchiveMetadata],
    ) -> Result<bool> {
        let num_data_shards = group_info.num_data_shards as usize;
        let num_parity_shards = group_info.num_parity_shards as usize;
        let shard_size = group_info.shard_size as usize;

        let reed_solomon = ReedSolomon::new(num_data_shards, num_parity_shards)?;
        let mut shards: Vec<Vec<u8>> = Vec::new();

        if group_info.data_shards_stored {
            for data_shard in data_shards {
                let data_shard_path = PathBuf::from(data_shard.shard_path.as_ref().unwrap());
                let data_shard = std::fs::read(data_shard_path)?;
                shards.push(data_shard);
            }
        } else {
            for archive in archives {
                let archive_path = PathBuf::from(archive.archive_uri.clone());
                let archive = align_shard(&archive_path, shard_size)?;
                shards.push(archive);
            }
        }

        for parity_shard in parity_shards {
            let parity_shard_path = PathBuf::from(parity_shard.shard_path.clone());
            let parity_shard = std::fs::read(parity_shard_path)?;
            shards.push(parity_shard);
        }

        Ok(reed_solomon.verify(&shards)?)
    }
}