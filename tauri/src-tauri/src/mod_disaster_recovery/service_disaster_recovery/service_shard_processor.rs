//! 分片处理器
//! 合并了分片生成和保存功能的组件

use anyhow::Result;
use reed_solomon_erasure::galois_8::ReedSolomon;
use std::fs::File;
use std::io::Write;

use crate::mod_database::dao_database::dao_map_archive_shard::MapArchiveDataShardDao;
use crate::mod_database::dao_database::dao_recovery_data_shard::RecoveryDataShardDao;
use crate::mod_database::dao_database::dao_recovery_parity_shard::RecoveryParityShardDao;
use crate::mod_database::impl_database::Database;
use crate::mod_disaster_recovery::core_disaster_recovery::model_disaster_recovery::RecoveryConfig;
use crate::mod_disaster_recovery::core_disaster_recovery::core_shard_alignment::align_shards;
use crate::utils::calculate_sha256;

/// 分片处理器 trait
/// 负责生成和保存分片数据
pub trait ShardProcessor {
    /// 生成并保存灾备分片文件
    fn gen_and_save_shards(
        &self,
        database: &Database,
        group_id: i64,
        archive_ids: &[i64],
        archive_metas: &[(Option<String>, u64, String)],
        original_archive_paths: &[std::path::PathBuf],
        shard_size: u64,
    ) -> Result<()>;
}

/// 默认分片处理器实现
pub struct DefaultShardProcessor {
    reed_solomon: ReedSolomon,
    config: RecoveryConfig,
}

impl DefaultShardProcessor {
    pub fn new(reed_solomon: ReedSolomon, config: RecoveryConfig) -> Self {
        Self { reed_solomon, config }
    }

    /// 处理分片的情况
    fn handle_shards(
        &self,
        database: &Database,
        group_id: i64,
        archive_ids: &[i64],
        archive_metas: &[(Option<String>, u64, String)],
        aligned_data: &[Vec<u8>],
    ) -> Result<()> {
        let recovery_data_shard_dao = RecoveryDataShardDao::new(database.conn.clone());
        let recovery_parity_shard_dao = RecoveryParityShardDao::new(database.conn.clone());
        let map_archive_data_shard_dao = MapArchiveDataShardDao::new(database.conn.clone());

        // 计算对齐数据的哈希值
        let aligned_hashes: Vec<String> = aligned_data.iter().map(|data| calculate_sha256(data)).collect();

        // 生成校验数据
        let shard_size = aligned_data[0].len();
        let mut parity_shards = vec![vec![0u8; shard_size]; self.config.parity_shards];

        // 合并数据分片和校验分片
        let mut shards: Vec<Vec<u8>> = aligned_data.to_vec();
        shards.append(&mut parity_shards);
        self.reed_solomon.encode(&mut shards)?;

        // 从合并后的 shards 中提取校验分片
        let parity_shards: Vec<Vec<u8>> =
            shards.into_iter().skip(self.config.data_shards).collect();

        // 存储数据分片（如果配置要求）并即时保存到数据库
        if self.config.store_data_shards {
            for (i, (shard, (_, _, _))) in
                aligned_data.iter().zip(archive_metas.iter()).enumerate()
            {
                let storage_path =
                    format!("{}/{}_data_{}.dat", self.config.storage_dir, self.config.prefix, i);

                // 写入数据分片文件
                let mut file = File::create(&storage_path)?;
                file.write_all(shard)?;

                // 保存数据分片到数据库，使用对齐数据的哈希
                let data_shard_id = recovery_data_shard_dao.insert_recovery_data_shard(
                    group_id,
                    i as i64,
                    Some(&storage_path),
                    &aligned_hashes[i],
                )?;

                // 创建归档文件与数据分片的映射关系（仅对有效的归档 ID）
                if i < archive_ids.len() {
                    // 从 archive_metas 获取哈希值
                    let original_hash = &archive_metas[i].2; // 第三个元素是哈希值
                    map_archive_data_shard_dao.insert_archive_shard_map(data_shard_id, archive_ids[i], i as u64 + 1, original_hash)?;
                }
            }
        } else {
            // 不存储数据分片，但仍然创建映射关系
            for (index, (_, _, _)) in archive_metas.iter().enumerate() {
                // 保存数据分片到数据库，使用对齐数据的哈希
                let data_shard_id = recovery_data_shard_dao.insert_recovery_data_shard(
                    group_id,
                    index as i64,
                    None,  // 不存储数据分片，路径为 None
                    &aligned_hashes[index],
                )?;

                // 创建归档文件与数据分片的映射关系（仅对有效的归档 ID）
                if index < archive_ids.len() {
                    // 从 archive_metas 获取哈希值
                    let original_hash = &archive_metas[index].2; // 第三个元素是哈希值
                    map_archive_data_shard_dao.insert_archive_shard_map(data_shard_id, archive_ids[index], index as u64 + 1, original_hash)?;
                }
            }
        }

        // 存储校验分片并即时保存到数据库
        for (i, shard) in parity_shards.iter().enumerate() {
            let storage_path =
                format!("{}/{}_parity_{}.dat", self.config.storage_dir, self.config.prefix, i);

            // 写入校验文件
            let mut file = File::create(&storage_path)?;
            file.write_all(shard)?;

            // 计算校验文件哈希
            let sha256 = calculate_sha256(shard);

            // 保存校验分片到数据库
            recovery_parity_shard_dao.insert_recovery_parity_shard(
                group_id,
                i as i64,
                &storage_path,
                &sha256,
            )?;
        }

        Ok(())
    }
}

impl ShardProcessor for DefaultShardProcessor {
    fn gen_and_save_shards(
        &self,
        database: &Database,
        group_id: i64,
        archive_ids: &[i64],
        archive_metas: &[(Option<String>, u64, String)],
        original_archive_paths: &[std::path::PathBuf],
        shard_size: u64,
    ) -> Result<()> {
        log::debug!("archive_metas: {:?}", archive_metas);

        let aligned_data =
            align_shards(original_archive_paths, shard_size as usize)?;

        // 处理非空分片（所有文件都为空的情况已在上层方法中处理）
        self.handle_shards(database, group_id, archive_ids, archive_metas, &aligned_data)?;

        Ok(())
    }
}