//! 数据保存器
//! 负责数据处理和保存恢复的文件

use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

use crate::mod_database::dao_database::dao_archive_metadata::ArchiveMetadataDao;
use crate::mod_database::impl_database::Database;

/// 数据保存器 trait
pub trait DataSaver {
    /// 去除对齐填充，还原为原始文件内容
    fn restore_original_data(&self, recovered_data: Vec<u8>, original_size: i64) -> Vec<u8>;

    /// 将恢复的数据写入文件
    fn save_restored_file(
        &self,
        restored_data: &[u8],
        archive_id: i64,
        archive_name: &str,
        work_dir: &Path,
    ) -> Result<PathBuf>;

    /// 保存恢复的数据（完整流程）
    fn save_recovered_data(
        &self,
        recovered_data: Vec<u8>,
        archive_id: i64,
        database: &Database,
        work_dir: &Path,
    ) -> Result<PathBuf>;
}

/// 默认数据保存器实现
pub struct DefaultDataSaver;

impl DefaultDataSaver {
    pub fn new() -> Self {
        Self
    }
}

impl DataSaver for DefaultDataSaver {
    fn restore_original_data(&self, recovered_data: Vec<u8>, original_size: i64) -> Vec<u8> {
        if (original_size as usize) < recovered_data.len() {
            recovered_data[..original_size as usize].to_vec()
        } else {
            recovered_data
        }
    }

    fn save_restored_file(
        &self,
        restored_data: &[u8],
        archive_id: i64,
        archive_name: &str,
        work_dir: &Path,
    ) -> Result<PathBuf> {
        let recovered_file_path = work_dir.join(format!("recovered_{}_{}", archive_id, archive_name));
        fs::write(&recovered_file_path, restored_data)?;
        Ok(recovered_file_path)
    }

    fn save_recovered_data(
        &self,
        recovered_data: Vec<u8>,
        archive_id: i64,
        database: &Database,
        work_dir: &Path,
    ) -> Result<PathBuf> {
        // 获取原始归档文件大小
        let archive_dao = ArchiveMetadataDao::new(database.conn.clone());
        let metadata = archive_dao.find_by_id(archive_id)?
            .ok_or_else(|| anyhow::anyhow!("未找到归档文件元数据：{}", archive_id))?;

        // 去除对齐填充，还原为原始文件内容
        let original_size = metadata.archive_size.unwrap_or(recovered_data.len() as i64);
        let restored_data = self.restore_original_data(recovered_data, original_size);

        // 保存恢复的文件
        let archive_name = metadata.archive_uri.split('/').last().unwrap_or("unknown");
        self.save_restored_file(&restored_data, archive_id, archive_name, work_dir)
    }
}