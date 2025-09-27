//! 归档工具模块
//!
//! 包含归档过程中使用的通用工具函数

use anyhow::Result as AnyResult;
use anyhow::anyhow;
use chrono::Utc;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use crate::utils::EventType;
use crate::utils::ProgressEvent;
use crate::utils::create_progress_event;
use crate::utils::update_progress;

/// 获取文件数据块
pub fn get_file_chunk_bytes(file_path: &Path, offset: i64, size: i64) -> AnyResult<Vec<u8>> {
    let mut file = File::open(file_path)?;
    let mut buffer = vec![0u8; size as usize];

    // 跳转到指定偏移量
    file.seek(SeekFrom::Start(offset as u64))?;

    // 读取指定大小的数据
    file.read_exact(&mut buffer)?;

    Ok(buffer)
}

/// 创建新的归档文件路径
pub fn create_new_archive_file_path(
    archive_directory: &str, archive_file_prefix: &str, archive_counter: u32,
) -> PathBuf {
    let archive_dir = Path::new(archive_directory);
    let timestamp = Utc::now().format("%Y%m%d%H%M%S");
    let archive_file_name = format!(
        "{}_{}_{}.tar",
        archive_file_prefix, timestamp, archive_counter
    );
    archive_dir.join(&archive_file_name)
}
/// 归档进度信息
#[derive(Debug, Clone, serde::Serialize)]
pub struct ArchiveProgress {
    /// 总文件数
    pub total_files: usize,
    /// 已处理文件数
    pub processed_files: usize,
    /// 已处理字节数
    pub processed_bytes: u64,
    /// 总字节数
    pub total_bytes: u64,
    /// 当前正在处理的文件
    pub current_file: Option<String>,
    /// 是否完成
    pub completed: bool,
    /// 错误信息（如果有）
    pub error: Option<String>,
}
pub fn update_archive_progress<F>(
    progress_callback: &Option<F>, progress: &ArchiveProgress, message: &str, event_type: EventType,
) where
    F: Fn(ProgressEvent<ArchiveProgress>),
{
    let event = create_progress_event(
        event_type,
        "archive",
        progress.clone(),
        Some(message.to_string()),
    );
    update_progress(progress_callback, &event);
}
