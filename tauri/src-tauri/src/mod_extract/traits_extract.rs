//! 解档操作 trait 定义

use anyhow::Result as AnyResult;
use crate::mod_extract::models::{ExtractTask, ExtractProgress};
use crate::mod_database::traits::{
    ArchiveAssetOperations, ArchiveMetadataOperations, DirectoryOperations, MapFileAssetOperations, ViewOperations
};

/// 解档操作 trait
pub trait ExtractOperations {
    /// 解档指定存档到目标路径
    ///
    /// # 参数
    /// * `task` - 解档任务参数
    /// * `database` - 数据库实例
    ///
    /// # 返回值
    /// 返回操作结果
    fn extract_archive<D>(
        &self,
        task: &ExtractTask,
        database: &D,
    ) -> AnyResult<ExtractProgress>
    where
        D: ArchiveAssetOperations +DirectoryOperations+ MapFileAssetOperations + ArchiveMetadataOperations + ViewOperations;
}