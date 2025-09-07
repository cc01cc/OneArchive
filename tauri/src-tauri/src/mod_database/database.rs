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
}