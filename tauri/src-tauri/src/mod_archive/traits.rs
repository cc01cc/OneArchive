//! 归档操作 trait 定义
//! 定义各种归档操作的接口

use crate::mod_database::traits::DirectoryOperations;
use crate::mod_database::traits::FileOperations;
use crate::mod_database::traits::RootOperations;
use crate::mod_database::traits::StatusOperations;
use anyhow::Result as AnyResult;
use std::path::Path;

/// 扫描进度回调 trait
pub trait ProgressCallback {
    /// 处理扫描进度更新
    fn on_progress(&self, processed: u64, total: u64, message: String, progress: f64);
}

/// 目录扫描 trait
pub trait DirectoryScanOperations {
    /// 扫描目录并更新数据库
    ///
    /// # 参数
    /// * `start_path` - 起始路径
    /// * `database` - 数据库实例
    /// * `progress_callback` - 进度回调函数（可选）
    ///
    /// # 返回值
    /// 返回操作结果
    fn scan_and_save_directory_with_events<D, F>(
        &self,
        start_path: &Path,
        database: &D,
        progress_callback: Option<F>,
    ) -> AnyResult<()>
    where
        D: RootOperations + DirectoryOperations + FileOperations + StatusOperations,
        F: Fn(crate::mod_archive::models::ScanProgress);
}

/// 目录统计 trait
pub trait DirectoryStatisticsOperations {
    /// 获取目录统计信息
    ///
    /// # 参数
    /// * `path` - 目录路径
    ///
    /// # 返回值
    /// 返回目录统计信息
    fn get_directory_statistics(
        &self,
        path: &Path,
    ) -> AnyResult<crate::mod_archive::models::DirectoryStatistics>;
}
