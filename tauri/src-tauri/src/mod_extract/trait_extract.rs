//! 解档操作 trait 定义

use anyhow::Result as AnyResult;
use crate::mod_extract::model_extract::{ExtractTask, ExtractProgress};
use crate::mod_database::trait_database::{
    ArchiveChunkOperations, ArchiveMetadataOperations, DirectoryOperations, MapFileChunkOperations, ViewOperations
};

/// 解档操作 trait
pub trait ExtractOperations {
    /// 解档指定归档到目标路径
    ///
    /// # 参数
    /// * `task` - 解档任务参数
    /// * `database` - 数据库实例
    /// * `progress_callback` - 进度回调函数（可选）
    ///
    /// # 返回值
    /// 返回操作结果
    fn extract_archive<D, F>(
        &mut self,
        task: &ExtractTask,
        database: &D,
        progress_callback: Option<F>,
    ) -> AnyResult<ExtractProgress>
    where
        D: ArchiveChunkOperations +DirectoryOperations+ MapFileChunkOperations + ArchiveMetadataOperations + ViewOperations,
        F: Fn(ExtractProgress);
}