//! 数据库连接和基本操作
//! 提供数据库连接管理和基础操作
//! TODO 数据库连接断开时的异常处理机制

use rusqlite::{Connection, OpenFlags, Result as SqliteResult, params};
use std::{path::Path, sync::Arc};
use super::constants::DatabaseTableName;
use super::trait_database::{InitializationOperations, StatusOperations};
use crate::mod_database::impl_initialize;

/// 数据库连接管理器

#[derive(Clone)]
pub struct Database {
    /// SQLite 连接
    pub conn: Arc<Connection>,
}
impl Database {
    /// 创建新的数据库连接
    ///
    /// # 返回值
    /// 返回数据库连接实例
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self, rusqlite::Error> {
        let conn = Connection::open_with_flags(
            db_path,
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE,
        )?;

        Ok(Database {
            conn: Arc::new(conn),
        })
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

// 实现初始化操作 trait
impl InitializationOperations for Database {
    /// 初始化数据库表结构
    ///
    /// # 参数
    /// * `conn` - 数据库连接引用
    fn initialize_tables(&self, conn: &Connection) -> SqliteResult<()> {
        impl_initialize::initialize_tables(conn)
    }
}

// 实现状态操作 trait
impl StatusOperations for Database {
    /// 更新表记录状态
    fn mark_table_status_by_id(
        &self, table_name: DatabaseTableName, id: i64, status: &str,
    ) -> SqliteResult<()> {
        self.conn.execute(
            &format!(
                "UPDATE {} SET status = ?1, updated_at = strftime('%s', 'now') WHERE id = ?2",
                table_name.as_str()
            ),
            params![status, id],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_initialize_tables() {
        // 初始化日志记录器
        let _ = simple_logger::SimpleLogger::new().init();

        // Create a temporary directory for our test database
        let temp_dir = TempDir::new().expect("Failed to create temporary directory");
        let db_path = temp_dir.path().join("test.db");

        // Create database instance which will call initialize_tables
        let db = Database::new(&db_path).expect("Failed to create database");

        // Test that all tables were created successfully by querying them
        db.initialize_tables(&db.conn).expect("Failed to initialize tables");

        // Test info_root table
        let mut stmt = db.conn.prepare("SELECT COUNT(*) FROM info_root").unwrap();
        let count: i64 = stmt.query_row([], |row| row.get(0)).unwrap();
        assert_eq!(count, 0); // Should be empty but exist

        // Test info_directory table
        let mut stmt = db.conn.prepare("SELECT COUNT(*) FROM info_directory").unwrap();
        let count: i64 = stmt.query_row([], |row| row.get(0)).unwrap();
        assert_eq!(count, 0);

        // Test info_file table
        let mut stmt = db.conn.prepare("SELECT COUNT(*) FROM info_file").unwrap();
        let count: i64 = stmt.query_row([], |row| row.get(0)).unwrap();
        assert_eq!(count, 0);

        // Test archive_metadata table
        let mut stmt = db.conn.prepare("SELECT COUNT(*) FROM archive_metadata").unwrap();
        let count: i64 = stmt.query_row([], |row| row.get(0)).unwrap();
        assert_eq!(count, 0);

        // Test archive_chunk table
        let mut stmt = db.conn.prepare("SELECT COUNT(*) FROM archive_chunk").unwrap();
        let count: i64 = stmt.query_row([], |row| row.get(0)).unwrap();
        assert_eq!(count, 0);

        // Test map_file_chunk table
        let mut stmt = db.conn.prepare("SELECT COUNT(*) FROM map_file_chunk").unwrap();
        let count: i64 = stmt.query_row([], |row| row.get(0)).unwrap();
        assert_eq!(count, 0);

        // Test views were created
        let mut stmt = db.conn.prepare("SELECT COUNT(*) FROM view_file").unwrap();
        let count: i64 = stmt.query_row([], |row| row.get(0)).unwrap();
        assert_eq!(count, 0);

        let mut stmt = db.conn.prepare("SELECT COUNT(*) FROM view_chunk").unwrap();
        let count: i64 = stmt.query_row([], |row| row.get(0)).unwrap();
        assert_eq!(count, 0);
    }
}
