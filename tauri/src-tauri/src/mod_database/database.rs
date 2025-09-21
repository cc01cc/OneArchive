//! 数据库连接和基本操作
//! 提供数据库连接管理和基础操作

use rusqlite::{Connection, OpenFlags};
use std::path::Path;

/// 数据库连接管理器
pub struct Database {
    /// SQLite 连接
    pub conn: Connection,
}

impl Database {
    /// 创建新的数据库连接
    ///
    /// # 参数
    /// * `db_path` - 数据库文件路径
    ///
    /// # 返回值
    /// 返回数据库连接实例
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self, rusqlite::Error> {
        let conn = Connection::open_with_flags(
            db_path,
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE,
        )?;

        Ok(Database { conn })
    }

    /// 规范化目录路径
    /// 确保路径不以 '/' 结尾，空路径返回 None
    pub fn normalize_directory_path(path: &Option<String>) -> Option<String> {
        if let Some(path_str) = path {
            // 统一使用正斜杠作为路径分隔符
            let normalized_path = path_str.replace('\\', "/");
       
            // 移除路径末尾的 '/' 字符（如果存在）
            let trimmed_path = normalized_path.trim_end_matches('/');
            // 如果路径全是 '/' 字符，规范化为空字符串（表示根目录级别）
            if trimmed_path.is_empty() {
                Some(String::new()) // 根目录应该返回空字符串而不是 None
            } else {
                Some(trimmed_path.to_string())
            }
        } else {
            None
        }
    }
}
