use serde::{Deserialize, Serialize};

/// 归档文件信息
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ArchiveFile {
    pub id: i64,
    pub archive_name: String,
    pub archive_size: i64,
}

/// 根目录信息
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RootInfo {
    pub id: i64,
    pub root_path: String,
    pub root_name: String,
}

/// 扫描进度信息
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ScanProgress {
    pub processed: u64,
    pub total: u64,
    pub message: String,
    pub progress: f64,
}

/// 归档进度信息
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ArchiveProgress {
    pub total_files: usize,
    pub processed_files: usize,
    pub processed_bytes: u64,
    pub total_bytes: u64,
    pub current_file: Option<String>,
    pub completed: bool,
    pub error: Option<String>,
}

/// 解档任务参数
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ExtractTask {
    pub root_id: i64,
    pub target_path: String,
    pub overwrite: bool,
}

/// 解档进度信息
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ExtractProgress {
    pub total_files: usize,
    pub processed_files: usize,
    pub current_file: Option<String>,
    pub completed: bool,
    pub error: Option<String>,
}

/// 归档参数
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ArchiveParams {
    pub root_dir: String,
    pub archive_dir: String,
    pub archive_prefix: String,
    pub db_path: String,
    pub archive_limit_size: i64,
}

/// 扫描参数
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ScanParams {
    pub root_path: String,
    pub db_path: String,
}
