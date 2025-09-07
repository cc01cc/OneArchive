//! 数据库仓库实现
//! 实现数据库操作的具体逻辑

use super::super::constants::{DatabaseTableName, DirectoryStatus, FileStatus};
use super::super::database::Database;
use super::super::schema::{InfoDirectory, InfoFile, InfoRoot, ViewFile};
use super::super::traits::{
    DirectoryOperations, FileOperations, InitializationOperations, RootOperations,
    StatusOperations, ViewOperations,
};
use rusqlite::{Connection, Result as SqliteResult, params};

// 实现根目录操作 trait
impl RootOperations for Database {
    /// 添加根目录信息
    ///
    /// # 参数
    /// * `root_path` - 根目录路径
    /// * `root_name` - 根目录名称
    /// * `status` - 根目录状态
    ///
    /// # 返回值
    /// 返回插入记录的 ID
    fn add_root_directory(
        &self,
        root_path: &str,
        root_name: &str,
        status: &str,
    ) -> SqliteResult<i64> {
        self.conn.execute(
            "INSERT OR REPLACE INTO info_root (root_path, root_name, status) VALUES (?1, ?2, ?3)",
            params![root_path, root_name, status],
        )?;

        self.conn
            .last_insert_rowid()
            .try_into()
            .map_err(|_| rusqlite::Error::ExecuteReturnedResults)
    }

    /// 根据路径查找根目录信息
    ///
    /// # 参数
    /// * `root_path` - 根目录路径
    ///
    /// # 返回值
    /// 返回根目录信息
    fn find_root_info_by_path(&self, root_path: &str) -> SqliteResult<Option<InfoRoot>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, root_name, root_path, status, created_at, updated_at FROM info_root WHERE root_path = ?1"
        )?;

        let mut rows = stmt.query(params![root_path])?;

        if let Some(row) = rows.next()? {
            Ok(Some(row.try_into()?))
        } else {
            Ok(None)
        }
    }

    /// 更新根目录状态
    ///
    /// # 参数
    /// * `root_id` - 根目录 ID
    /// * `status` - 新状态
    ///
    /// # 返回值
    /// 返回操作结果
    fn update_root_status(&self, root_id: i64, status: &str) -> SqliteResult<()> {
        self.conn.execute(
            "UPDATE info_root SET status = ?1, updated_at = strftime('%s', 'now') WHERE id = ?2",
            params![status, root_id],
        )?;
        Ok(())
    }

    /// 查找所有根目录信息
    fn find_all_root_info(&self) -> SqliteResult<Vec<InfoRoot>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, root_name, root_path, status, created_at, updated_at FROM info_root",
        )?;
        let mut rows = stmt.query([])?;

        let mut roots = Vec::new();
        while let Some(row) = rows.next()? {
            let root = row.try_into()?;
            roots.push(root);
        }
        Ok(roots)
    }
}

// 实现目录操作 trait
impl DirectoryOperations for Database {
    /// 将指定根目录下的所有目录标记为待删除状态
    ///
    /// # 参数
    /// * `root_id` - 根目录 ID
    ///
    /// # 返回值
    /// 返回操作结果
    fn mark_directories_as_wait_to_delete(&self, root_id: i64) -> SqliteResult<()> {
        self.conn.execute(
            "UPDATE info_directory SET status = ?1, updated_at = strftime('%s', 'now') WHERE root_id = ?2",
            params![DirectoryStatus::WaitToDelete.as_str(), root_id],
        )?;
        Ok(())
    }

    /// 插入目录信息
    ///
    /// # 参数
    /// * `directory` - 目录信息
    ///
    /// # 返回值
    /// 返回插入记录的 ID
    fn insert_directory(&self, directory: &InfoDirectory) -> SqliteResult<i64> {
        self.conn.execute(
            "INSERT INTO info_directory (root_id, directory_name, directory_mtime, directory_path, status) 
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                directory.root_id,
                directory.directory_name,
                directory.directory_mtime,
                directory.directory_path,
                directory.status.as_str()
            ],
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    /// 更新目录信息
    ///
    /// # 参数
    /// * `directory` - 目录信息
    ///
    /// # 返回值
    /// 返回操作结果
    fn update_directory(&self, directory: &InfoDirectory) -> SqliteResult<()> {
        self.conn.execute(
            "UPDATE info_directory SET directory_name = ?1, directory_mtime = ?2, status = ?3, updated_at = strftime('%s', 'now') 
             WHERE id = ?4",
            params![
                directory.directory_name,
                directory.directory_mtime,
                directory.status.as_str(),
                directory.id
            ],
        )?;
        Ok(())
    }

    /// 根据路径查找目录 ID
    ///
    /// # 参数
    /// * `root_id` - 根目录 ID
    /// * `path` - 目录路径
    ///
    /// # 返回值
    /// 返回目录 ID
    fn find_directory_by_path(
        &self,
        root_id: i64,
        path: &str,
    ) -> SqliteResult<Option<InfoDirectory>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, root_id, directory_name, directory_mtime, directory_path, status, created_at, updated_at 
              FROM info_directory WHERE root_id = ?1 AND directory_path = ?2")?;

        let mut rows = stmt.query(params![root_id, path])?;

        if let Some(row) = rows.next()? {
            Ok(Some(row.try_into()?))
        } else {
            Ok(None)
        }
    }

    /// 根据状态和根目录 ID 查找目录
    fn find_directories_by_status_and_root_id(
        &self,
        root_id: i64,
        status: Option<DirectoryStatus>,
    ) -> SqliteResult<Vec<InfoDirectory>> {
        let sql = if status.is_some() {
            format!(
                "SELECT id, root_id, directory_name, directory_mtime, directory_path, status, created_at, updated_at 
                 FROM info_directory 
                 WHERE root_id = ?1 AND status = ?2"
            )
        } else {
            format!(
                "SELECT id, root_id, directory_name, directory_mtime, directory_path, status, created_at, updated_at 
                 FROM info_directory 
                 WHERE root_id = ?1"
            )
        };

        let mut stmt = self.conn.prepare(&sql)?;
        let mut rows = if let Some(ref status) = status {
            stmt.query(params![root_id, status])?
        } else {
            stmt.query(params![root_id])?
        };

        let mut directories = Vec::new();
        while let Some(row) = rows.next()? {
            let directory = row.try_into()?;
            directories.push(directory);
        }
        Ok(directories)
    }
}

// 实现文件操作 trait
impl FileOperations for Database {
    /// 将指定根目录下的所有文件标记为待删除状态
    ///
    /// # 参数
    /// * `root_id` - 根目录 ID
    ///
    /// # 返回值
    /// 返回操作结果
    fn mark_files_as_wait_to_delete(&self, root_id: i64) -> SqliteResult<()> {
        self.conn.execute(
            "UPDATE info_file SET status = ?1, updated_at = strftime('%s', 'now') 
             WHERE directory_id IN (SELECT id FROM info_directory WHERE root_id = ?2)",
            params![FileStatus::WaitToDelete.as_str(), root_id],
        )?;
        Ok(())
    }

    /// 根据名称查找文件
    fn find_file_by_name(&self, directory_id: i64, name: &str) -> SqliteResult<Option<InfoFile>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, directory_id, file_name, file_size, file_mtime, file_hash, status, created_at, updated_at 
                      FROM info_file WHERE directory_id = ?1 AND file_name = ?2")?;

        let mut rows = stmt.query(params![directory_id, name])?;

        if let Some(row) = rows.next()? {
            Ok(Some(row.try_into()?))
        } else {
            Ok(None)
        }
    }

    /// 根据状态和根目录 ID 查找文件
    fn find_files_by_status_and_root_id(
        &self,
        root_id: i64,
        status: Option<FileStatus>,
    ) -> SqliteResult<Vec<InfoFile>> {
        let sql = if status.is_some() {
            format!(
                "SELECT * FROM info_file
                 WHERE directory_id IN (SELECT id FROM info_directory WHERE root_id = ?1) AND status = ?2"
            )
        } else {
            format!(
                "SELECT * FROM info_file
                 WHERE directory_id IN (SELECT id FROM info_directory WHERE root_id = ?1)"
            )
        };

        let mut stmt = self.conn.prepare(&sql)?;
        let mut rows = if let Some(ref status) = status {
            stmt.query(params![root_id, status])?
        } else {
            stmt.query(params![root_id])?
        };

        let mut directories = Vec::new();
        while let Some(row) = rows.next()? {
            let directory = row.try_into()?;
            directories.push(directory);
        }
        Ok(directories)
    }

    /// 插入文件信息
    ///
    /// # 参数
    /// * `file` - 文件信息
    ///
    /// # 返回值
    /// 返回插入记录的 ID
    fn insert_file(&self, file: &InfoFile) -> SqliteResult<i64> {
        self.conn.execute(
            "INSERT INTO info_file (directory_id, file_name, file_size, file_mtime, file_hash, status) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
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
    ///
    /// # 参数
    /// * `file` - 文件信息
    ///
    /// # 返回值
    /// 返回操作结果
    fn update_file(&self, file: &InfoFile) -> SqliteResult<()> {
        self.conn.execute(
            "UPDATE info_file SET file_name = ?1, file_size = ?2, file_mtime = ?3, file_hash = ?4, status = ?5, updated_at = strftime('%s', 'now') 
             WHERE id = ?6",
            params![
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

// 实现视图操作 trait
impl ViewOperations for Database {
    /// 根据根目录 ID 查找所有视图文件
    fn find_view_files_by_root_id(&self, root_id: i64) -> SqliteResult<Vec<ViewFile>> {
        let sql = format!(
            "SELECT file_id, file_name, file_size, file_mtime, file_hash, file_status 
             FROM view_file 
             WHERE root_id = ?1"
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let mut rows = stmt.query(params![root_id])?;
        let mut view_files = Vec::new();
        while let Some(row) = rows.next()? {
            let view_file: ViewFile = row.try_into()?;
            view_files.push(view_file);
        }
        Ok(view_files)
    }

    /// 根据目录 ID 查找视图文件
    fn find_view_files_by_directory_id(&self, directory_id: i64) -> SqliteResult<Vec<ViewFile>> {
        let sql = format!(
            "SELECT root_id, root_path, root_status, directory_id, directory_path, directory_mtime, directory_status,
                    file_id, file_name, file_size, file_mtime, file_hash, file_status
             FROM view_file 
             WHERE directory_id = ?1"
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let mut rows = stmt.query(params![directory_id])?;
        let mut view_files = Vec::new();
        while let Some(row) = rows.next()? {
            let view_file: ViewFile = row.try_into()?;
            view_files.push(view_file);
        }
        Ok(view_files)
    }
}

// 实现初始化操作 trait
impl InitializationOperations for Database {
    /// 初始化数据库表结构
    ///
    /// # 参数
    /// * `conn` - 数据库连接引用
    fn initialize_tables(&self, conn: &Connection) -> SqliteResult<()> {
        super::initializer::initialize_tables(conn)
    }
}

// 实现状态操作 trait
impl StatusOperations for Database {
    /// 更新表记录状态
    fn mark_table_status(
        &self,
        table_name: DatabaseTableName,
        id: i64,
        status: &str,
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
        db.initialize_tables(&db.conn)
            .expect("Failed to initialize tables");

        // Test info_root table
        let mut stmt = db.conn.prepare("SELECT COUNT(*) FROM info_root").unwrap();
        let count: i64 = stmt.query_row([], |row| row.get(0)).unwrap();
        assert_eq!(count, 0); // Should be empty but exist

        // Test info_directory table
        let mut stmt = db
            .conn
            .prepare("SELECT COUNT(*) FROM info_directory")
            .unwrap();
        let count: i64 = stmt.query_row([], |row| row.get(0)).unwrap();
        assert_eq!(count, 0);

        // Test info_file table
        let mut stmt = db.conn.prepare("SELECT COUNT(*) FROM info_file").unwrap();
        let count: i64 = stmt.query_row([], |row| row.get(0)).unwrap();
        assert_eq!(count, 0);

        // Test archive_metadata table
        let mut stmt = db
            .conn
            .prepare("SELECT COUNT(*) FROM archive_metadata")
            .unwrap();
        let count: i64 = stmt.query_row([], |row| row.get(0)).unwrap();
        assert_eq!(count, 0);

        // Test archive_asset table
        let mut stmt = db
            .conn
            .prepare("SELECT COUNT(*) FROM archive_asset")
            .unwrap();
        let count: i64 = stmt.query_row([], |row| row.get(0)).unwrap();
        assert_eq!(count, 0);

        // Test map_file_asset table
        let mut stmt = db
            .conn
            .prepare("SELECT COUNT(*) FROM map_file_asset")
            .unwrap();
        let count: i64 = stmt.query_row([], |row| row.get(0)).unwrap();
        assert_eq!(count, 0);

        // Test views were created
        let mut stmt = db.conn.prepare("SELECT COUNT(*) FROM view_file").unwrap();
        let count: i64 = stmt.query_row([], |row| row.get(0)).unwrap();
        assert_eq!(count, 0);

        let mut stmt = db.conn.prepare("SELECT COUNT(*) FROM view_asset").unwrap();
        let count: i64 = stmt.query_row([], |row| row.get(0)).unwrap();
        assert_eq!(count, 0);
    }
}
