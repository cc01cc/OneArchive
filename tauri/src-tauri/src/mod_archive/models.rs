//! 归档模块数据模型定义
//! 定义归档功能相关的数据结构
/// 目录统计信息结构体
#[derive(Debug, Clone)]
pub struct DirectoryStatistics {
    pub total_size: u64,
    pub directory_count: u32,
    pub file_count: u32,
    pub max_depth: u32,
}

impl Default for DirectoryStatistics {
    fn default() -> Self {
        Self {
            total_size: 0,
            directory_count: 0,
            file_count: 0,
            max_depth: 0,
        }
    }
}

/// 扫描进度结构
#[derive(Debug, Clone, serde::Serialize)]
pub struct ScanProgress {
    pub processed: u64,
    pub total: u64,
    pub message: String,
    pub progress: f64,
}