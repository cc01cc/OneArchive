//! 存档操作 trait 定义
//! 定义存档操作的接口

use anyhow::Result as AnyResult;
use std::fmt;
use std::fs::File;
use std::path::{Path, PathBuf};
use tar::Builder;

use crate::mod_database::schema::ViewFile;
use crate::mod_database::traits::{
    ArchiveAssetOperations, ArchiveMetadataOperations, DirectoryOperations, FileOperations, MapFileAssetOperations, RootOperations, StatusOperations, ViewOperations
};

/// 存档上下文，用于管理当前存档过程中的状态
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
        // 如果没有当前归档ID，或者当前归档大小加上tar文件头大小超过了限制，则需要创建新归档
        self.current_archive_id.is_none()
            || self.current_archive_size + 512 >= self.archive_limit_size
    }
}

/// 存档操作 trait
pub trait ArchiveOperations {
    /// 将指定根目录下的所有文件添加到存档中
    /// 对应 Java 中 ArchiveIn.java 的 archiveFileInDb 方法
    ///
    /// # 参数
    /// * `root_dir` - 根目录路径
    /// * `context` - 归档上下文
    /// * `database` - 数据库实例
    ///
    /// # 返回值
    /// 返回操作结果
    fn archive_file_in_db<D>(
        &self,
        root_dir: &str,
        context: &mut ArchiveContext,
        database: &D,
    ) -> AnyResult<()>
    where
        D: ViewOperations
            + FileOperations
            + DirectoryOperations
            + MapFileAssetOperations
            + ArchiveAssetOperations
            + ArchiveMetadataOperations
            + RootOperations
            + StatusOperations;
}
