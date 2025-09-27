//! 灾备系统实现 - 整合编码器与数据库操作

use anyhow::Result;
use log::debug;
use reed_solomon_erasure::galois_8::ReedSolomon;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use uuid::Uuid;

use crate::mod_database::dao::archive_metadata::ArchiveMetadataDao;
use crate::mod_database::dao::recovery_archive::RecoveryArchiveDao;
use crate::mod_database::dao::recovery_data_shard::RecoveryDataShardDao;
use crate::mod_database::dao::recovery_group::RecoveryGroupDao;
use crate::mod_database::dao::recovery_parity_shard::RecoveryParityShardDao;
use crate::mod_database::database::Database;
use crate::mod_database::schema_recovery::{
    CreateRecoveryGroupParams, RecoveryDataShard, RecoveryParityShard,
};
use crate::mod_disaster_recovery::model::RecoveryConfig;
use crate::mod_disaster_recovery::shard_alignment;

/// 灾备组编码器
pub struct GroupEncoder {
    reed_solomon: ReedSolomon,
    config: RecoveryConfig,
}

impl GroupEncoder {
    /// 创建新的编码器实例
    pub fn new(config: RecoveryConfig) -> Self {
        let reed_solomon = ReedSolomon::new(config.data_shards, config.parity_shards).unwrap();
        Self { reed_solomon, config }
    }

    /// 创建灾备组并将其信息保存到数据库（使用归档文件 ID）
    pub fn create_recovery_group_and_save_with_archives(
        &self, database: &Database, archive_ids: Vec<i64>,
    ) -> Result<i64> {
        // 从数据库获取归档文件路径
        let archive_paths = self.get_archive_paths_from_ids(database, &archive_ids)?;

        // 生成灾备组
        let recovery_group = self.gen_shard(archive_paths.iter().map(|s| s.as_ref()).collect())?;

        // 保存到数据库，包括归档文件关联关系
        let group_id = self.save_recovery_group_to_database_with_archives(
            database,
            &recovery_group,
            &archive_ids,
        )?;

        Ok(group_id)
    }

    /// 根据归档文件 ID 获取归档文件路径
    fn get_archive_paths_from_ids(
        &self, database: &Database, archive_ids: &[i64],
    ) -> Result<Vec<String>> {
        let archive_metadata_dao = ArchiveMetadataDao::new(database.conn.clone());
        let mut archive_paths = Vec::new();

        for &archive_id in archive_ids {
            if let Some(archive_metadata) = archive_metadata_dao.find_by_id(archive_id)? {
                archive_paths.push(archive_metadata.archive_uri);
            } else {
                return Err(anyhow::anyhow!("找不到 ID 为 {} 的归档文件", archive_id));
            }
        }

        Ok(archive_paths)
    }

    /// 生成灾备分片文件
    pub fn gen_shard(
        &self, archive_paths: Vec<&Path>,
    ) -> Result<(Vec<RecoveryDataShard>, Vec<RecoveryParityShard>)> {
        // 1. 读取文件并计算哈希
        let mut archive_metas = Vec::new();
        let mut original_archive_paths = Vec::new(); // 存储原始档案路径

        for (index, path) in archive_paths.iter().enumerate() {
            let mut content = Vec::new();
            let mut file = File::open(path)?;
            file.read_to_end(&mut content)?;

            // 使用 sha2 库计算 SHA-256
            let mut hasher = Sha256::new();
            hasher.update(&content);
            let sha256 = format!("{:x}", hasher.finalize());

            // 直接生成 storage_path 路径
            let storage_path = if self.config.store_data_shards {
                Some(format!(
                    "{}/{}_data_{}.dat",
                    self.config.storage_dir, self.config.prefix, index
                ))
            } else {
                // 即使不存储也生成临时路径，用于处理对齐等操作
                Some(format!(
                    "{}/{}_temp_data_{}.dat",
                    self.config.storage_dir, self.config.prefix, index
                ))
            };

            // 保存原始档案路径
            original_archive_paths.push(path.to_path_buf());

            archive_metas.push((storage_path, content.len() as u64, sha256));
        }

        debug!("archive_metas: {:?}", archive_metas);

        let aligned_data = shard_alignment::DataShardMeta::align_shards(
            &original_archive_paths,
            archive_metas.iter().map(|(_, size, _)| *size as usize).max().unwrap_or(0),
        )?;

        // 检查是否所有文件都为空
        let shard_size = aligned_data[0].len();
        if shard_size == 0 {
            // 处理所有文件都为空的情况
            let mut parity_shards = vec![Vec::<u8>::new(); self.config.parity_shards];

            // 为每个校验分片创建元数据
            let mut parity_metas = Vec::new();
            for i in 0..self.config.parity_shards {
                // 直接生成 parity 文件的 storage_path
                let storage_path =
                    format!("{}/{}_parity_{}.dat", self.config.storage_dir, self.config.prefix, i);

                // 创建空的校验文件
                let mut file = File::create(&storage_path)?;
                file.write_all(&[])?; // 写入空内容

                // 计算校验文件哈希（空文件的哈希）
                let mut hasher = Sha256::new();
                hasher.update(&[]);
                let sha256 = format!("{:x}", hasher.finalize());

                parity_metas.push(RecoveryParityShard {
                    id: 0,       // 占位符，实际 ID 将在保存到数据库时分配
                    group_id: 0, // 占位符
                    shard_index: i as i64,
                    shard_path: storage_path,
                    shard_hash: sha256,
                    status: "HEALTH".to_string(),
                    last_verified: None,
                    created_at: 0, // 占位符
                    updated_at: 0, // 占位符
                });
            }

            let mut data_metas = Vec::new();
            // 存储数据分片（如果配置要求）
            let data_metas = if self.config.store_data_shards {
                for (i, shard) in aligned_data.iter().enumerate() {
                    let storage_path = format!(
                        "{}/{}_data_{}.dat",
                        self.config.storage_dir, self.config.prefix, i
                    );

                    // 写入数据分片文件
                    let mut file = File::create(&storage_path)?;
                    file.write_all(shard)?;

                    // 计算数据分片哈希
                    let mut hasher = Sha256::new();
                    hasher.update(shard);
                    let sha256 = format!("{:x}", hasher.finalize());

                    data_metas.push(RecoveryDataShard {
                        id: 0,       // 占位符，实际 ID 将在保存到数据库时分配
                        group_id: 0, // 占位符
                        shard_index: i as i64,
                        shard_path: Some(storage_path),
                        shard_hash: sha256,
                        status: "HEALTH".to_string(),
                        last_verified: None,
                        created_at: 0, // 占位符
                        updated_at: 0, // 占位符
                    });
                }
                data_metas
            } else {
                // 不存储数据分片时，删除临时文件并使用原始文件信息
                for (index, (storage_path, _, sha256)) in archive_metas.iter().enumerate() {
                    if let Some(temp_path) = storage_path {
                        // 删除临时文件
                        let _ = std::fs::remove_file(temp_path);
                    }

                    data_metas.push(RecoveryDataShard {
                        id: 0,       // 占位符
                        group_id: 0, // 占位符
                        shard_index: index as i64,
                        shard_path: storage_path.clone(),
                        shard_hash: sha256.clone(),
                        status: "HEALTH".to_string(),
                        last_verified: None,
                        created_at: 0, // 占位符
                        updated_at: 0, // 占位符
                    });
                }
                data_metas
            };

            return Ok((data_metas, parity_metas));
        }

        // 3. 生成校验数据
        let parity_shards = vec![vec![0u8; shard_size]; self.config.parity_shards];

        // 合并数据分片和校验分片
        let mut shards: Vec<Vec<u8>> =
            aligned_data.clone().into_iter().chain(parity_shards).collect();
        self.reed_solomon.encode(&mut shards)?;

        // 从合并后的 shards 中提取校验分片
        let parity_shards: Vec<Vec<u8>> =
            shards.into_iter().skip(self.config.data_shards).collect();

        // 4. 创建分片文件并生成元数据
        let mut data_metas = Vec::new();
        let mut parity_metas = Vec::new();

        // 存储数据分片（如果配置要求）
        if self.config.store_data_shards {
            for (i, shard) in aligned_data.iter().enumerate() {
                // 直接使用已生成的 storage_path
                let storage_path =
                    format!("{}/{}_data_{}.dat", self.config.storage_dir, self.config.prefix, i);

                // 写入数据分片文件
                let mut file = File::create(&storage_path)?;
                file.write_all(shard)?;

                // 计算数据分片哈希
                let mut hasher = Sha256::new();
                hasher.update(shard);
                let sha256 = format!("{:x}", hasher.finalize());

                data_metas.push(RecoveryDataShard {
                    id: 0,       // 占位符
                    group_id: 0, // 占位符
                    shard_index: i as i64,
                    shard_path: Some(storage_path),
                    shard_hash: sha256,
                    status: "HEALTH".to_string(),
                    last_verified: None,
                    created_at: 0, // 占位符
                    updated_at: 0, // 占位符
                });
            }
        } else {
            // 不存储数据分片，删除临时文件并使用原始文件信息
            for (index, (storage_path, _, sha256)) in archive_metas.iter().enumerate() {
                if let Some(temp_path) = storage_path {
                    // 删除临时文件
                    let _ = std::fs::remove_file(temp_path);
                }

                data_metas.push(RecoveryDataShard {
                    id: 0,       // 占位符
                    group_id: 0, // 占位符
                    shard_index: index as i64,
                    shard_path: storage_path.clone(),
                    shard_hash: sha256.clone(),
                    status: "HEALTH".to_string(),
                    last_verified: None,
                    created_at: 0, // 占位符
                    updated_at: 0, // 占位符
                });
            }
        }

        // 存储校验分片
        for (i, shard) in parity_shards.iter().enumerate() {
            // 直接生成 parity 文件的 storage_path
            let storage_path =
                format!("{}/{}_parity_{}.dat", self.config.storage_dir, self.config.prefix, i);

            // 写入校验文件
            let mut file = File::create(&storage_path)?;
            file.write_all(shard)?;

            // 计算校验文件哈希
            let mut hasher = Sha256::new();
            hasher.update(shard);
            let sha256 = format!("{:x}", hasher.finalize());

            parity_metas.push(RecoveryParityShard {
                id: 0,       // 占位符
                group_id: 0, // 占位符
                shard_index: i as i64,
                shard_path: storage_path,
                shard_hash: sha256,
                status: "HEALTH".to_string(),
                last_verified: None,
                created_at: 0, // 占位符
                updated_at: 0, // 占位符
            });
        }

        Ok((data_metas, parity_metas))
    }

    /// 使用提供的内容对齐归档文件大小
    fn align_archives_with_content(&self, contents: &[Vec<u8>]) -> Result<Vec<Vec<u8>>> {
        // 1. 找到最大文件大小
        let max_size = contents.iter().map(|content| content.len()).max().unwrap_or(0);

        // 2. 填充所有内容
        let mut aligned_data = Vec::with_capacity(contents.len());
        for content in contents {
            let mut buffer = vec![0u8; max_size];

            // 复制实际内容
            buffer[..content.len()].copy_from_slice(content);

            aligned_data.push(buffer);
        }

        Ok(aligned_data)
    }

    /// 将灾备组信息保存到数据库（包括归档文件关联关系）
    fn save_recovery_group_to_database_with_archives(
        &self, database: &Database,
        recovery_shards: &(Vec<RecoveryDataShard>, Vec<RecoveryParityShard>), archive_ids: &[i64],
    ) -> Result<i64> {
        // 初始化 DAO
        let recovery_group_dao = RecoveryGroupDao::new(database.conn.clone());
        let recovery_data_shard_dao = RecoveryDataShardDao::new(database.conn.clone());
        let recovery_parity_shard_dao = RecoveryParityShardDao::new(database.conn.clone());
        let recovery_archive_dao = RecoveryArchiveDao::new(database.conn.clone());

        let (data_shards, parity_shards) = recovery_shards;

        // 创建灾备组记录
        let params = CreateRecoveryGroupParams {
            group_id: Uuid::new_v4().to_string(),
            num_data_shards: data_shards.len() as i64,
            num_parity_shards: parity_shards.len() as i64,
            data_shards_stored: self.config.store_data_shards,
            status: "active".to_string(), // 默认状态
        };

        let group_id = recovery_group_dao.insert_recovery_group(params)?;

        // 保存数据分片信息
        for shard in data_shards {
            recovery_data_shard_dao.insert_recovery_data_shard(
                group_id,
                shard.shard_index,
                shard.shard_path.as_deref(),
                &shard.shard_hash,
            )?;
        }

        // 保存校验分片信息
        for shard in parity_shards {
            recovery_parity_shard_dao.insert_recovery_parity_shard(
                group_id,
                shard.shard_index,
                &shard.shard_path,
                &shard.shard_hash,
            )?;
        }

        // 保存灾备组与归档文件的关联关系
        self.save_recovery_group_archive_relations(recovery_archive_dao, group_id, archive_ids)?;

        Ok(group_id)
    }

    /// 保存灾备组与归档文件的关联关系
    fn save_recovery_group_archive_relations(
        &self, recovery_archive_dao: RecoveryArchiveDao, group_id: i64, archive_ids: &[i64],
    ) -> Result<()> {
        // 保存数据分片与归档文件的关联关系
        for &archive_id in archive_ids {
            recovery_archive_dao.insert_recovery_group_archive(group_id, archive_id)?;
        }

        Ok(())
    }
}
