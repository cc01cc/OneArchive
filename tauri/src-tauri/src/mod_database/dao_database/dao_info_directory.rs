//! 目录相关数据库操作 DAO

use log::debug;
use rusqlite::{Connection, Result as SqliteResult};
use std::sync::Arc;

use crate::mod_database::{
    common::DatabaseCommonOperations, constants::DirectoryStatus, impl_database::Database, constants_sql,
    schema::InfoDirectory,
};

/// 目录数据访问对象
pub struct InfoDirectoryDao {
    conn: Arc<Connection>,
}

impl InfoDirectoryDao {
    /// 创建新的 DAO 实例
    pub fn new(conn: Arc<Connection>) -> Self {
        Self { conn }
    }

    /// 将指定根目录下的所有目录标记为待删除状态
    pub fn mark_directories_as_wait_to_delete(&self, root_id: i64) -> SqliteResult<()> {
        self.conn.execute(
            constants_sql::info_directory::UPDATE_WAIT_TO_DELETE,
            rusqlite::params![DirectoryStatus::WaitToDelete.as_str(), root_id],
        )?;
        Ok(())
    }

    /// 插入目录信息
    pub fn insert_directory(&self, directory: &InfoDirectory) -> SqliteResult<i64> {
        // 规范化目录路径，确保不以 '/' 结尾
        let normalized_directory_path =
            Database::normalize_directory_path(&directory.directory_path)
                .map(|path| path.replace('\\', "/"));

        self.conn.execute(
            constants_sql::info_directory::INSERT,
            rusqlite::params![
                directory.root_id,
                directory.directory_name,
                directory.directory_mtime,
                normalized_directory_path,
                directory.status.as_str()
            ],
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    /// 更新目录信息
    pub fn update_directory(&self, directory: &InfoDirectory) -> SqliteResult<()> {
        // 规范化目录路径，确保不以 '/' 结尾，并统一使用正斜杠作为路径分隔符
        let normalized_directory_path =
            Database::normalize_directory_path(&directory.directory_path)
                .map(|path| path.replace('\\', "/"));
        debug!("更新现有目录：{}", normalized_directory_path.as_deref().unwrap_or(""));

        self.conn.execute(
            constants_sql::info_directory::UPDATE,
            rusqlite::params![
                directory.directory_name,
                directory.directory_mtime,
                normalized_directory_path,
                directory.status.as_str(),
                directory.id
            ],
        )?;
        Ok(())
    }

    /// 根据路径查找目录
    pub fn find_directory_by_path(
        &self, root_id: i64, path: &str,
    ) -> SqliteResult<Option<InfoDirectory>> {
        // 规范化查询路径，确保使用统一的正斜杠分隔符
        let normalized_path = Database::normalize_directory_path(&Some(path.to_string()));

        self.conn.query_single(
            constants_sql::info_directory::SELECT_BY_PATH,
            rusqlite::params![root_id, normalized_path],
        )
    }

    /// 根据 ID 查找目录
    pub fn find_directory_by_id(&self, id: i64) -> SqliteResult<Option<InfoDirectory>> {
        self.conn.query_single(constants_sql::info_directory::SELECT_BY_ID, rusqlite::params![id])
    }

    /// 根据状态和根目录 ID 查找目录
    pub fn find_by_status_and_root_id(
        &self, root_id: i64, status: Option<DirectoryStatus>,
    ) -> SqliteResult<Vec<InfoDirectory>> {
        let status_str = status.as_ref().map(|s| s.as_str());
        self.conn.query_multiple(
            constants_sql::info_directory::SELECT_BY_STATUS_AND_ROOT_ID,
            rusqlite::params![root_id, status_str],
        )
    }

    /// 查找某个目录的直接父目录
    pub fn find_parent_directory(
        &self, root_id: i64, directory_path: &str,
    ) -> SqliteResult<Option<InfoDirectory>> {
        // 如果目录路径为空或根目录，则没有父目录
        if directory_path.is_empty() {
            return Ok(None);
        }

        // 规范化目录路径
        let normalized_path = Database::normalize_directory_path(&Some(directory_path.to_string()))
            .unwrap_or_default();

        // 查找父目录路径
        let parent_path = if let Some(last_slash_pos) = normalized_path.rfind('/') {
            &normalized_path[..last_slash_pos]
        } else {
            "" // 父目录是根目录
        };

        self.find_directory_by_path(root_id, parent_path)
    }

    /// 查找某个目录下的所有直接子目录
    pub fn find_child_directories(
        &self, root_id: i64, parent_path: &str,
    ) -> SqliteResult<Vec<InfoDirectory>> {
        // 规范化父目录路径
        let normalized_parent_path =
            Database::normalize_directory_path(&Some(parent_path.to_string())).unwrap_or_default();

        let try_from_row = |row: &rusqlite::Row| InfoDirectory::try_from(row);
        let mut stmt = self.conn.prepare(constants_sql::info_directory::SELECT_CHILD_DIRECTORIES)?;
        let rows =
            stmt.query_and_then(rusqlite::params![root_id, normalized_parent_path], try_from_row)?;

        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }
}
