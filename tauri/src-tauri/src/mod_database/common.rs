//! 通用数据库操作
//! 提供通用的数据库操作方法，减少重复代码

use rusqlite::{Connection, Error as SqliteError, Result as SqliteResult, Row};
use std::convert::TryFrom;

/// 为 Connection 扩展通用方法
pub trait DatabaseCommonOperations {
    /// 通用查询单个记录的方法
    fn query_single<T>(
        &self, sql: &str, params: &[&dyn rusqlite::ToSql],
    ) -> SqliteResult<Option<T>>
    where
        T: for<'r> TryFrom<&'r Row<'r>, Error = SqliteError>;

    /// 通用查询多个记录的方法
    fn query_multiple<T>(&self, sql: &str, params: &[&dyn rusqlite::ToSql]) -> SqliteResult<Vec<T>>
    where
        T: for<'r> TryFrom<&'r Row<'r>, Error = SqliteError>;

    /// 通用插入并返回 ID 的方法
    fn insert_and_get_id(&self, sql: &str, params: &[&dyn rusqlite::ToSql]) -> SqliteResult<i64>;

    /// 通用插入并返回 ID 的方法 (支持命名参数)
    fn insert_and_get_id_named(
        &self, sql: &str, params: &[(&str, &dyn rusqlite::ToSql)],
    ) -> SqliteResult<i64>;
}

impl DatabaseCommonOperations for Connection {
    /// 通用查询单个记录的方法
    fn query_single<T>(&self, sql: &str, params: &[&dyn rusqlite::ToSql]) -> SqliteResult<Option<T>>
    where
        T: for<'r> TryFrom<&'r Row<'r>, Error = SqliteError>,
    {
        self.prepare(sql)?.query_map(params, |row| T::try_from(row))?.next().transpose()
    }

    /// 通用查询多个记录的方法
    fn query_multiple<T>(&self, sql: &str, params: &[&dyn rusqlite::ToSql]) -> SqliteResult<Vec<T>>
    where
        T: for<'r> TryFrom<&'r Row<'r>, Error = SqliteError>,
    {
        self.prepare(sql)?.query_map(params, |row| T::try_from(row))?.collect()
    }

    /// 通用插入并返回 ID 的方法
    fn insert_and_get_id(&self, sql: &str, params: &[&dyn rusqlite::ToSql]) -> SqliteResult<i64> {
        let mut stmt = self.prepare(sql)?;
        let mut rows = stmt.query(params)?;
        if let Some(row) = rows.next()? {
            row.get(0)
        } else {
            Ok(self.last_insert_rowid())
        }
    }

    /// 通用插入并返回 ID 的方法 (支持命名参数)
    fn insert_and_get_id_named(
        &self, sql: &str, params: &[(&str, &dyn rusqlite::ToSql)],
    ) -> SqliteResult<i64> {
        let mut stmt = self.prepare(sql)?;
        let mut rows = stmt.query(params)?;
        if let Some(row) = rows.next()? {
            row.get(0)
        } else {
            Ok(self.last_insert_rowid())
        }
    }
}
