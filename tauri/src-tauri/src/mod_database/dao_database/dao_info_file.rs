//! 文件相关数据库操作 DAO

use rusqlite::{Connection, Result as SqliteResult};
use std::sync::Arc;

use crate::mod_database::{
    common::DatabaseCommonOperations, constants::FileStatus, constants_sql, schema::InfoFile,
};

/// 文件数据访问对象
pub struct InfoFileDao {
    conn: Arc<Connection>,
}

impl InfoFileDao {
    /// 创建新的 DAO 实例
    pub fn new(conn: Arc<Connection>) -> Self {
        Self { conn }
    }

    /// 将指定根目录下的所有文件标记为待删除状态
    pub fn mark_files_as_wait_to_delete(&self, root_id: i64) -> SqliteResult<()> {
        self.conn.execute(
            constants_sql::info_file::UPDATE_WAIT_TO_DELETE,
            rusqlite::params![FileStatus::WaitToDelete.as_str(), root_id],
        )?;
        Ok(())
    }

    /// 根据名称查找文件
    pub fn find_file_by_name(
        &self,
        directory_id: i64,
        name: &str,
    ) -> SqliteResult<Option<InfoFile>> {
        self.conn.query_single(
            constants_sql::info_file::SELECT_BY_NAME,
            rusqlite::params![directory_id, name],
        )
    }

    /// 根据状态和根目录 ID 查找文件
    pub fn find_files_by_status_and_root_id(
        &self,
        root_id: i64,
        status: Option<FileStatus>,
    ) -> SqliteResult<Vec<InfoFile>> {
        let status_str = status.as_ref().map(|s| s.as_str());
        self.conn.query_multiple(
            constants_sql::info_file::SELECT_BY_STATUS_AND_ROOT_ID,
            rusqlite::params![root_id, status_str],
        )
    }

    /// 插入文件信息
    pub fn insert_file(&self, file: &InfoFile) -> SqliteResult<i64> {
        self.conn.execute(
            constants_sql::info_file::INSERT,
            rusqlite::params![
                file.directory_id,
                file.file_name,
                file.file_size,
                file.file_mtime,
                file.file_hash,
                file.status.as_str()
            ],
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    /// 更新文件信息
    pub fn update_file(&self, file: &InfoFile) -> SqliteResult<()> {
        self.conn.execute(
            constants_sql::info_file::UPDATE,
            rusqlite::params![
                file.file_name,
                file.file_size,
                file.file_mtime,
                file.file_hash.as_ref(),
                file.status.as_str(),
                file.id
            ],
        )?;
        Ok(())
    }
}
