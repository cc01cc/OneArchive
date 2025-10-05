//! 数据库操作 trait 定义
//! 定义各种数据库操作的接口
//! 如果 传入参数超过 2 个, 则使用结构化参数 params

use super::constants::DatabaseTableName;
use rusqlite::{Connection, Result as SqliteResult};

/// 数据库表状态操作 trait
pub trait StatusOperations {
    /// 更新表记录状态
    fn mark_table_status_by_id(
        &self, table_name: DatabaseTableName, id: i64, status: &str,
    ) -> SqliteResult<()>;
}

/// 数据库初始化操作 trait
pub trait InitializationOperations {
    /// 初始化数据库表结构
    ///
    /// # 参数
    /// * `conn` - 数据库连接引用
    fn initialize_tables(&self, conn: &Connection) -> SqliteResult<()>;
}
