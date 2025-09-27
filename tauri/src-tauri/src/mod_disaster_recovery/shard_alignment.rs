//! 灾备系统分片对齐功能模块
//! 提供对 Reed-Solomon 编码所需的数据分片对齐功能

use anyhow::Result;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

/// 数据分片元数据，包含对齐所需的信息
#[derive(Debug, Clone)]
pub struct DataShardMeta {
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

impl DataShardMeta {
    /// 获取分片大小
    pub fn size(&self) -> u64 {
        self.size
    }
    /// 从文件路径创建数据分片元数据
    pub fn from_path(
        path: &Path, original_archive_path: String, original_archive_id: String,
    ) -> Result<Self> {
        let mut content = Vec::new();
        let mut file = File::open(path)?;
        file.read_to_end(&mut content)?;

        // 计算 SHA-256 哈希
        let mut hasher = Sha256::new();
        hasher.update(&content);
        let sha256 = format!("{:x}", hasher.finalize());

        Ok(DataShardMeta {
            storage_path: path.to_string_lossy().into_owned(),
            original_archive_path: PathBuf::from(original_archive_path),
            size: content.len() as u64,
            sha256,
            original_archive_id,
        })
    }
    pub fn align_shards(file_path_list: &[PathBuf], max_size: usize) -> Result<Vec<Vec<u8>>> {
        if file_path_list.is_empty() {
            return Ok(Vec::new());
        }
        //  读取并填充所有文件
        let mut aligned_data = Vec::with_capacity(file_path_list.len());
        for path in file_path_list {
            let mut content = Vec::new();
            let mut file = File::open(path.clone())?;
            file.read_to_end(&mut content)?;

            // 创建缓冲区并用 0 填充
            let mut buffer = vec![0u8; max_size];
            buffer[..content.len()].copy_from_slice(&content);
            aligned_data.push(buffer);
        }

        Ok(aligned_data)
    }

    /// 使用提供的内容对齐分片数据
    ///
    /// # 参数
    /// * `contents` - 分片内容列表
    ///
    /// # 返回值
    /// 返回对齐后的分片数据，所有分片具有相同的长度
    pub fn align_shards_with_content(contents: &[Vec<u8>]) -> Result<Vec<Vec<u8>>> {
        if contents.is_empty() {
            return Ok(Vec::new());
        }

        // 1. 找到最大文件大小
        let max_size = contents.iter().map(|content| content.len()).max().unwrap_or(0);

        // 2. 填充所有内容
        let mut aligned_data = Vec::with_capacity(contents.len());
        for content in contents {
            let mut buffer = vec![0u8; max_size];
            buffer[..content.len()].copy_from_slice(content);
            aligned_data.push(buffer);
        }

        Ok(aligned_data)
    }
}

/// 奇偶校验分片元数据，包含对齐所需的信息
#[derive(Debug, Clone)]
pub struct ParityShardMeta {
    /// 分片文件路径
    pub storage_path: String,
    /// 分片大小
    pub size: u64,
    /// 分片 SHA256 哈希值
    pub sha256: String,
}

impl ParityShardMeta {
    /// 从文件路径创建奇偶校验分片元数据
    pub fn from_path(path: &Path) -> Result<Self> {
        let mut content = Vec::new();
        let mut file = File::open(path)?;
        file.read_to_end(&mut content)?;

        // 计算 SHA-256 哈希
        let mut hasher = Sha256::new();
        hasher.update(&content);
        let sha256 = format!("{:x}", hasher.finalize());

        Ok(ParityShardMeta {
            storage_path: path.to_string_lossy().into_owned(),
            size: content.len() as u64,
            sha256,
        })
    }
}

/// 统一分片元数据枚举
#[derive(Debug, Clone)]
pub enum ShardMeta {
    Data(DataShardMeta),
    Parity(ParityShardMeta),
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
