use std::{fs::File, path::Path};

use anyhow::Result as AnyResult;
use serde::Serialize;
use sha2::{Digest, Sha256};

/// 计算数据的 SHA256 哈希值
pub fn calculate_sha256(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let hash = hasher.finalize();
    format!("{:x}", hash)
}

/// 计算文件哈希值
pub fn calculate_file_hash(file_path: &Path) -> AnyResult<String> {
    let mut file = File::open(file_path)?;
    let mut hasher = Sha256::new();
    std::io::copy(&mut file, &mut hasher)?;
    let hash = hasher.finalize();
    Ok(format!("{:x}", hash))
}

/// 通用事件类型枚举
#[derive(Debug, Clone, Serialize)]
pub enum EventType {
    Start,
    Progress,
    Complete,
    Error,
    Warning,
    Info,
}

/// 通用进度事件结构
#[derive(Debug, Clone, Serialize)]
pub struct ProgressEvent<T> {
    /// 事件类型
    pub event_type: EventType,
    /// 操作模块 (如 "scan", "archive", "extract")
    pub module: String,
    /// 进度数据
    pub data: T,
    /// 时间戳
    pub timestamp: u64,
    /// 可选的附加消息
    pub message: Option<String>,
}

/// 通用进度回调 trait
pub trait ProgressCallback<T> {
    fn call(&self, event: &ProgressEvent<T>);
}

/// 为 Option<F> 实现通用进度回调 trait，其中 F 是函数
impl<F, T> ProgressCallback<T> for Option<F>
where
    F: Fn(ProgressEvent<T>),
    T: Clone,
{
    fn call(&self, event: &ProgressEvent<T>) {
        if let Some(callback) = self {
            callback(event.clone());
        }
    }
}

/// 创建进度事件的辅助函数
pub fn create_progress_event<T>(
    event_type: EventType,
    module: &str,
    data: T,
    message: Option<String>,
) -> ProgressEvent<T> 
where 
    T: Clone,
{
    use std::time::{SystemTime, UNIX_EPOCH};
    
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;
    
    ProgressEvent {
        event_type,
        module: module.to_string(),
        data,
        timestamp,
        message,
    }
}

/// 通用进度更新函数
pub fn update_progress<F, T>(
    progress_callback: &Option<F>,
    event: &ProgressEvent<T>,
) where
    Option<F>: ProgressCallback<T>,
    T: Clone,
{
    progress_callback.call(event);
}

