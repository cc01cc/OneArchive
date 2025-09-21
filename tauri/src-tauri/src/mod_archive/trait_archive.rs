//! 归档操作 trait 定义
//! 定义归档操作的接口

use anyhow::Result as AnyResult;
use std::fmt;
use std::fs::File;
use std::path::{Path, PathBuf};
use tar::Builder;

use crate::mod_database::trait_database::{
    ArchiveChunkOperations, ArchiveMetadataOperations, DirectoryOperations, FileOperations,
    MapFileChunkOperations, RootOperations, StatusOperations, ViewOperations,
};

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

/// 归档上下文，用于管理当前归档过程中的状态
pub struct ArchiveContext {
    pub archive_file_prefix: String,
    pub archive_directory: String,
    pub archive_limit_size: i64,
    pub archive_counter: i64,
    pub current_archive_file_path: Option<PathBuf>,
    pub current_archive_size: i64,
    pub current_archive_id: Option<i64>,
    pub current_archive_tar_builder: Option<Builder<File>>,
}

impl fmt::Debug for ArchiveContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ArchiveContext")
            .field("archive_file_prefix", &self.archive_file_prefix)
            .field("archive_directory", &self.archive_directory)
            .field("archive_limit_size", &self.archive_limit_size)
            .field("archive_counter", &self.archive_counter)
            .field("current_archive_file", &self.current_archive_file_path)
            .field("current_archive_size", &self.current_archive_size)
            .field("current_archive_id", &self.current_archive_id)
            .field("current_archive_tar_builder", &"Option<Builder<File>>")
            .finish()
    }
}

impl ArchiveContext {
    pub fn new(
        archive_file_prefix: String,
        archive_directory: String,
        archive_limit_size: i64,
    ) -> Self {
        Self {
            archive_file_prefix,
            archive_directory,
            archive_limit_size,
            archive_counter: 1,
            current_archive_file_path: None,
            current_archive_size: 0,
            current_archive_id: None,
            current_archive_tar_builder: None,
        }
    }

    pub fn reset_current(&mut self) {
        self.current_archive_file_path = None;
        self.current_archive_size = 0;
        self.current_archive_id = None;
        self.current_archive_tar_builder = None;
    }

    pub fn need_new_archive(&self) -> bool {
        // 如果没有当前归档 ID，或者当前归档大小加上 tar 文件头大小超过了限制，则需要创建新归档
        self.current_archive_id.is_none()
            || self.current_archive_size + 512 >= self.archive_limit_size
    }
}

/// 归档操作 trait
pub trait ArchiveOperations {
    /// 将指定根目录下的所有文件添加到归档中
    /// 对应 Java 中 ArchiveIn.java 的 archiveFileInDb 方法
    ///
    /// # 参数
    /// * `root_dir` - 根目录路径
    /// * `context` - 归档上下文
    /// * `database` - 数据库实例
    /// * `progress_callback` - 进度回调函数（可选）
    ///
    /// # 返回值
    /// 返回操作结果
    fn archive_file_in_db<D, F>(
        &self,
        root_dir: &str,
        context: &mut ArchiveContext,
        database: &D,
        progress_callback: Option<F>,
    ) -> AnyResult<()>
    where
        D: ViewOperations
            + FileOperations
            + DirectoryOperations
            + MapFileChunkOperations
            + ArchiveChunkOperations
            + ArchiveMetadataOperations
            + RootOperations
            + StatusOperations,
        F: Fn(ArchiveProgress);
}
