//! 灾备系统分片对齐功能模块
//! 提供对 Reed-Solomon 编码所需的数据分片对齐功能

use anyhow::Result;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

/// 数据分片元数据，包含对齐所需的信息
#[derive(Debug, Clone)]
pub struct DataShardMetadata {
    /// 分片文件路径（存储路径）
    pub storage_path: String,
    /// 原始档案路径
    pub original_archive_path: PathBuf,
    /// 分片大小
    pub size: u64,
    /// 分片 SHA256 哈希值
    pub sha256: String,
    /// 原始档案 ID 或路径
    pub original_archive_id: String,
}

impl DataShardMetadata {
    /// 获取分片大小
    pub fn size(&self) -> u64 {
        self.size
    }
}

///  读取并填充所有文件
pub fn align_shards(file_path_list: &[PathBuf], target_size: usize) -> Result<Vec<Vec<u8>>> {
    if file_path_list.is_empty() {
        return Ok(Vec::new());
    }

    let mut aligned_data = Vec::with_capacity(file_path_list.len());
    for path in file_path_list {
        let buffer = align_shard(&path, target_size)?;
        aligned_data.push(buffer);
    }

    Ok(aligned_data)
}

/// 读取并填充单个文件
pub fn align_shard(file_path: &Path, target_size: usize) -> Result<Vec<u8>> {
    let mut content = Vec::new();
    let mut file = File::open(file_path)?;
    file.read_to_end(&mut content)?;

    // 创建缓冲区并用 0 填充
    let mut buffer = vec![0u8; target_size];
    buffer[..content.len()].copy_from_slice(&content);
    return Ok(buffer);
}

pub fn align_shard_and_write(
    source_path: &Path, target_path: &Path, max_size: usize,
) -> Result<bool> {
    let content = align_shard(source_path, max_size)?;
    let mut file = File::create(target_path)?;
    file.write_all(&content)?;
    Ok(true)
}

pub fn align_shard_and_write_after_verify(
    source_path: &Path, target_path: &Path, target_size: usize, target_sha256: &str,
) -> Result<bool> {
    let content = align_shard(source_path, target_size)?;
    let mut hasher = Sha256::new();
    hasher.update(&content);
    let sha256 = format!("{:x}", hasher.finalize());
    if sha256 != target_sha256 {
        log::info!("SHA256 校验失败，请检查文件 sha256 \n{}", sha256);
        return Ok(false);
    }
    let mut file = File::create(target_path)?;
    file.write_all(&content)?;
    Ok(true)
}

/// 将填充过的文件还原为原始文件
///
/// # 参数
/// * `file_path` - 填充过的文件路径
/// * `target_size` - 原始文件大小
///
/// # 返回值
/// 返回去除填充后的原始文件内容
pub fn restore_shard(file_path: &Path, target_size: usize) -> Result<Vec<u8>> {
    let mut content = Vec::new();
    let mut file = File::open(file_path)?;
    file.read_to_end(&mut content)?;

    // 确保目标大小不超过文件大小
    if target_size > content.len() {
        return Ok(content);
    }

    // 截取原始数据部分
    Ok(content[..target_size].to_vec())
}

pub fn align_shard_with_content(content: &[u8], max_size: usize) -> Result<Vec<u8>> {
    // 创建缓冲区并用 0 填充
    let mut buffer = vec![0u8; max_size];
    buffer[..content.len()].copy_from_slice(content);
    return Ok(buffer);
}

/// 从文件路径创建数据分片元数据
pub fn create_data_shard_metadata_from_path(
    file_path: &Path, original_archive_path: String, original_archive_id: String,
) -> Result<DataShardMetadata> {
    let mut content = Vec::new();
    let mut file = File::open(file_path)?;
    file.read_to_end(&mut content)?;

    // 计算 SHA-256 哈希
    let mut hasher = Sha256::new();
    hasher.update(&content);
    let sha256 = format!("{:x}", hasher.finalize());

    Ok(DataShardMetadata {
        storage_path: file_path.to_string_lossy().into_owned(),
        original_archive_path: PathBuf::from(original_archive_path),
        size: content.len() as u64,
        sha256,
        original_archive_id,
    })
}

/// 奇偶校验分片元数据，包含对齐所需的信息
#[derive(Debug, Clone)]
pub struct ParityShardMetadata {
    /// 分片文件路径
    pub storage_path: String,
    /// 分片大小
    pub size: u64,
    /// 分片 SHA256 哈希值
    pub sha256: String,
}

impl ParityShardMetadata {
    /// 从文件路径创建奇偶校验分片元数据
    pub fn from_path(path: &Path) -> Result<Self> {
        let mut content = Vec::new();
        let mut file = File::open(path)?;
        file.read_to_end(&mut content)?;

        // 计算 SHA-256 哈希
        let mut hasher = Sha256::new();
        hasher.update(&content);
        let sha256 = format!("{:x}", hasher.finalize());

        Ok(ParityShardMetadata {
            storage_path: path.to_string_lossy().into_owned(),
            size: content.len() as u64,
            sha256,
        })
    }
}

/// 统一分片元数据枚举
#[derive(Debug, Clone)]
pub enum ShardMeta {
    Data(DataShardMetadata),
    Parity(ParityShardMetadata),
}

impl ShardMeta {
    /// 获取分片文件路径
    pub fn file_path(&self) -> &str {
        match self {
            ShardMeta::Data(meta) => &meta.storage_path,
            ShardMeta::Parity(meta) => &meta.storage_path,
        }
    }

    /// 获取分片大小
    pub fn size(&self) -> u64 {
        match self {
            ShardMeta::Data(meta) => meta.size,
            ShardMeta::Parity(meta) => meta.size,
        }
    }
}
