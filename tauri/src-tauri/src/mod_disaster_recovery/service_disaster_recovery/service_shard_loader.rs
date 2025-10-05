//! 分片加载器与归档文件读取器
//!
//! 本模块负责以下功能：
//! - 加载已存储的分片数据
//! - 从归档文件生成分片数据
//! - 读取原始归档文件并计算哈希
//! - 根据归档文件ID获取文件路径

use anyhow::Result;
use reed_solomon_erasure::galois_8::ReedSolomon;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::mod_database::dao_database::dao_archive_metadata::ArchiveMetadataDao;
use crate::mod_database::dao_database::dao_recovery_data_shard::RecoveryDataShardDao;
use crate::mod_database::dao_database::dao_recovery_parity_shard::RecoveryParityShardDao;
use crate::mod_database::impl_database::Database;
use crate::mod_database::schema_recovery::{MapArchiveDataShard, RecoveryParityShard};
use crate::mod_disaster_recovery::core_disaster_recovery::model_disaster_recovery::RecoveryGroupInfoWithShardDetail;
use crate::mod_disaster_recovery::core_disaster_recovery::core_shard_alignment::align_shard;
use crate::mod_disaster_recovery::service_disaster_recovery::service_file_integrity_validator::FileIntegrityValidator;
use crate::utils::calculate_sha256;

/// 归档文件读取器 trait
/// 负责读取原始归档文件并计算哈希
pub trait ArchiveReader {
    /// 根据归档文件 ID 获取归档文件路径
    fn get_archive_paths_from_ids(
        &self,
        database: &Database,
        archive_ids: &[i64],
    ) -> Result<Vec<PathBuf>>;

    /// 读取归档文件并计算哈希
    fn read_archive_files(
        &self,
        archive_paths: &[PathBuf],
    ) -> Result<(Vec<(Option<String>, u64, String)>, Vec<PathBuf>)>;
}

/// 分片加载器 trait
/// 负责加载和生成分片数据
pub trait ShardLoader {
    /// 加载已存储的分片数据
    fn load_stored_shards(
        &self,
        group_info: &RecoveryGroupInfoWithShardDetail,
        database: &Database,
        reed_solomon: &ReedSolomon,
    ) -> Result<(Vec<Option<Vec<u8>>>, u64)>;

    /// 从归档文件生成分片数据
    fn generate_shards_from_archives(
        &self,
        group_info: &RecoveryGroupInfoWithShardDetail,
        database: &Database,
        reed_solomon: &ReedSolomon,
    ) -> Result<(Vec<Option<Vec<u8>>>, u64)>;
}

/// 默认分片加载器实现
/// 同时实现分片加载和归档文件读取功能
pub struct DefaultShardLoader;

impl DefaultShardLoader {
    pub fn new() -> Self {
        Self
    }
}

impl ArchiveReader for DefaultShardLoader {
    fn get_archive_paths_from_ids(
        &self,
        database: &Database,
        archive_ids: &[i64],
    ) -> Result<Vec<PathBuf>> {
        let archive_metadata_dao = ArchiveMetadataDao::new(database.conn.clone());
        let mut archive_paths = Vec::new();

        for &archive_id in archive_ids {
            if let Some(archive_metadata) = archive_metadata_dao.find_by_id(archive_id)? {
                archive_paths.push(PathBuf::from(archive_metadata.archive_uri));
            } else {
                return Err(anyhow::anyhow!("找不到 ID 为 {} 的归档文件", archive_id));
            }
        }

        Ok(archive_paths)
    }

    fn read_archive_files(
        &self,
        archive_paths: &[PathBuf],
    ) -> Result<(Vec<(Option<String>, u64, String)>, Vec<PathBuf>)> {
        let mut archive_metas = Vec::new();
        let mut original_archive_paths = Vec::new();

        for path in archive_paths {
            let content = std::fs::read(path)?;
            let sha256 = calculate_sha256(&content);

            original_archive_paths.push(path.clone());
            archive_metas.push((None, content.len() as u64, sha256));
        }
        Ok((archive_metas, original_archive_paths))
    }
}

impl DefaultShardLoader {
    /// 预构建数据分片和归档文件的查找映射
    fn build_lookup_maps(
        &self,
        group_info: &RecoveryGroupInfoWithShardDetail,
    ) -> (HashMap<i64, (usize, String)>, HashMap<i64, String>) {
        let mut data_shard_map = HashMap::new();
        let mut archive_map = HashMap::new();

        // 构建数据分片映射：shard_id -> (shard_index, hash)
        for data_shard in &group_info.data_shards {
            data_shard_map.insert(
                data_shard.id,
                (data_shard.shard_index as usize, data_shard.shard_hash.clone()),
            );
        }

        // 构建归档文件映射：archive_id -> archive_uri
        for archive in &group_info.archives {
            if let Some(archive_id) = archive.id {
                archive_map.insert(archive_id, archive.archive_uri.clone());
            }
        }

        (data_shard_map, archive_map)
    }

    /// 验证并收集可用的归档文件分片信息
    fn collect_available_archive_shards(
        &self,
        map_archive_data_shards: &[MapArchiveDataShard],
        data_shard_map: &HashMap<i64, (usize, String)>,
        archive_map: &HashMap<i64, String>,
        database: &Database,
    ) -> Result<Vec<(i64, i64, usize, String, PathBuf)>> {
        let mut available_archive_shards = Vec::new();

        for map in map_archive_data_shards {
            let archive_id = map.archive_id;
            let shard_id = map.shard_id;

            // 从预构建的映射中获取信息，避免重复查找
            let (shard_index, data_shard_hash) = data_shard_map
                .get(&shard_id)
                .ok_or_else(|| anyhow::anyhow!("未找到数据分片：{}", shard_id))?;

            let archive_uri = archive_map
                .get(&archive_id)
                .ok_or_else(|| anyhow::anyhow!("未找到归档文件：{}", archive_id))?;

            // 验证归档文件完整性
            if FileIntegrityValidator::verify_original_archive_integrity(archive_id, shard_id, database)? {
                log::debug!("归档文件 {} 完整性验证成功", archive_id);
                available_archive_shards.push((
                    archive_id,
                    shard_id,
                    *shard_index,
                    data_shard_hash.clone(),
                    PathBuf::from(archive_uri),
                ));
            } else {
                log::debug!("归档文件 {} 完整性验证失败", archive_id);
            }
        }

        Ok(available_archive_shards)
    }

    /// 处理校验分片并组装最终的分片数组
    fn assemble_shards_with_parity(
        &self,
        generated_shards: Vec<Option<Vec<u8>>>,
        parity_shards: &[RecoveryParityShard],
        num_data_shards: u64,
        database: &Database,
    ) -> Result<(Vec<Option<Vec<u8>>>, usize)> {
        let total_shards = (num_data_shards + parity_shards.len() as u64) as usize;
        let mut assembled_shards: Vec<Option<Vec<u8>>> = vec![None; total_shards];

        // 填充生成的数据分片
        for (index, shard) in generated_shards.into_iter().enumerate() {
            assembled_shards[index] = shard;
        }

        // 处理校验分片
        let mut available_parity_count = 0;
        for parity_shard in parity_shards {
            if FileIntegrityValidator::verify_shard_sha256_with_id(
                &crate::mod_disaster_recovery::core_disaster_recovery::model_disaster_recovery::ShardType::ParityShard,
                parity_shard.id,
                database,
            )? {
                let shard_data = fs::read(&parity_shard.shard_path)?;
                let parity_index = num_data_shards as usize + parity_shard.shard_index as usize;
                assembled_shards[parity_index] = Some(shard_data);
                available_parity_count += 1;
            }
        }

        Ok((assembled_shards, available_parity_count))
    }

    fn generate_data_shards_from_map(
        &self,
        map_available_archive_shards: Vec<(i64, i64, usize, String, PathBuf)>,
        total_shard_count: usize,
        target_size: usize,
    ) -> Result<(Vec<Option<Vec<u8>>>, usize)> {
        let mut available_data_shard_count = 0;
        let mut shards: Vec<Option<Vec<u8>>> = vec![None; total_shard_count];

        for (_archive_id, _shard_id, shard_index, data_shard_hash, archive_path) in
            map_available_archive_shards
        {
            let archive_path_str = archive_path.to_str().unwrap();
            log::debug!("开始从归档文件 {} 生成分片 {}", archive_path_str, shard_index);
            let shard_data = align_shard(&archive_path, target_size)?;

            if calculate_sha256(&shard_data) == data_shard_hash {
                log::debug!("归档文件 {} DATA SHARD 恢复成功", archive_path_str);
                shards[shard_index] = Some(shard_data);
                available_data_shard_count += 1;
            } else {
                log::debug!("归档文件 {} DATA SHARD 恢复失败", archive_path_str);
                shards[shard_index] = None;
            }
        }

        Ok((shards, available_data_shard_count))
    }
}

impl ShardLoader for DefaultShardLoader {
    fn load_stored_shards(
        &self,
        group_info: &RecoveryGroupInfoWithShardDetail,
        database: &Database,
        reed_solomon: &ReedSolomon,
    ) -> Result<(Vec<Option<Vec<u8>>>, u64)> {
        let mut available_count = 0;
        let mut available_data_shard_ids: Vec<i64> = Vec::new();
        let mut available_parity_shard_ids: Vec<i64> = Vec::new();

        // 检查 data shard 是否有损坏
        let data_shard_dao = RecoveryDataShardDao::new(database.conn.clone());
        for data_shard in &group_info.data_shards {
            if let Some(ref _shard_path) = data_shard.shard_path {
                if FileIntegrityValidator::verify_shard_sha256_with_id(
                    &crate::mod_disaster_recovery::core_disaster_recovery::model_disaster_recovery::ShardType::DataShard,
                    data_shard.id,
                    database,
                )? {
                    available_count += 1;
                    available_data_shard_ids.push(data_shard.id);
                }
            }
        }

        // 检查 parity shard 是否有损坏
        let parity_shard_dao = RecoveryParityShardDao::new(database.conn.clone());
        for parity_shard in &group_info.parity_shards {
            if FileIntegrityValidator::verify_shard_sha256_with_id(
                &crate::mod_disaster_recovery::core_disaster_recovery::model_disaster_recovery::ShardType::ParityShard,
                parity_shard.id,
                database,
            )? {
                available_count += 1;
                available_parity_shard_ids.push(parity_shard.id);
            }
        }

        // 准备分片数据
        let mut shards: Vec<Option<Vec<u8>>> = vec![None; reed_solomon.total_shard_count()];

        // 填充可用的数据分片
        for &data_shard_id in &available_data_shard_ids {
            let data_shard = data_shard_dao
                .find_by_id(data_shard_id)?
                .ok_or_else(|| anyhow::anyhow!("未找到数据分片：{}", data_shard_id))?;

            let shard_path = data_shard
                .shard_path
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("数据分片 {} 路径为空", data_shard_id))?;

            let data = fs::read(shard_path)?;
            shards[data_shard.shard_index as usize] = Some(data);
        }

        // 填充可用的校验分片
        for &parity_shard_id in &available_parity_shard_ids {
            let parity_shard = parity_shard_dao
                .find_shard_by_id(parity_shard_id)?
                .ok_or_else(|| anyhow::anyhow!("未找到校验分片：{}", parity_shard_id))?;

            let data = fs::read(&parity_shard.shard_path)?;
            shards[group_info.group_info.num_data_shards as usize + parity_shard.shard_index as usize] = Some(data);
        }

        log::debug!(
            "存储的数据分片 - 可用：{}，所需：{}",
            available_data_shard_ids.len(),
            group_info.group_info.num_data_shards
        );

        Ok((shards, available_count))
    }

    fn generate_shards_from_archives(
        &self,
        group_info: &RecoveryGroupInfoWithShardDetail,
        database: &Database,
        reed_solomon: &ReedSolomon,
    ) -> Result<(Vec<Option<Vec<u8>>>, u64)> {
        // 预构建查找映射，避免重复查询
        let (data_shard_map, archive_map) = self.build_lookup_maps(group_info);

        // 验证并收集可用的归档文件分片信息
        let available_archive_shards = self.collect_available_archive_shards(
            &group_info.map_archive_data_shards,
            &data_shard_map,
            &archive_map,
            database,
        )?;

        // 从归档文件生成数据分片
        let (generated_shards, generated_available_count) = self.generate_data_shards_from_map(
            available_archive_shards,
            reed_solomon.total_shard_count(),
            group_info.group_info.shard_size as usize,
        )?;

        // 处理校验分片并组装最终的分片数组
        let (assembled_shards, available_parity_count) = self.assemble_shards_with_parity(
            generated_shards,
            &group_info.parity_shards,
            group_info.group_info.num_data_shards,
            database,
        )?;

        // 计算总的可用分片数
        let available_count = generated_available_count as u64 + available_parity_count as u64;

        log::debug!(
            "从归档生成的数据分片 - 成功生成：{}，校验分片可用：{}",
            generated_available_count,
            available_parity_count
        );

        Ok((assembled_shards, available_count))
    }
}