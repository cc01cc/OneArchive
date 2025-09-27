//! 解档操作 trait 定义

use crate::mod_database::database::Database;
use crate::mod_extract::model_extract::{ExtractProgress, ExtractTask};
use anyhow::Result as AnyResult;

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
    fn extract_archive<F>(
        &mut self, task: &ExtractTask, database: &Database, progress_callback: Option<F>,
    ) -> AnyResult<ExtractProgress>
    where
        F: Fn(ExtractProgress);
}
