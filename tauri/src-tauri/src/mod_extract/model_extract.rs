//! 解档功能数据模型定义

use serde::{Deserialize, Serialize};

/// 解档任务参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractTask {
    /// Root id
    pub root_id: i64,
    /// 解档目标路径
    pub target_path: String,
    /// 是否覆盖已存在的文件
    pub overwrite: bool,
}

/// 解档进度信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractProgress {
    /// 总文件数
    pub total_files: usize,
    /// 已处理文件数
    pub processed_files: usize,
    /// 当前正在处理的文件
    pub current_file: Option<String>,
    /// 是否完成
    pub completed: bool,
    /// 错误信息（如果有）
    pub error: Option<String>,
}
