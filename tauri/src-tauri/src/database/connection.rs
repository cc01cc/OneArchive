//! 数据库连接管理模块
use log::info;
use rusqlite::{Connection, OpenFlags, Result as SqliteResult, params};
use std::path::Path;
// 导入数据库结构体
use crate::database::schema::{InfoDirectory, InfoFile, InfoRoot, ViewFile};
// 导入状态常量
use crate::database::constants::{DatabaseTableName, DirectoryStatus, FileStatus};

/// 数据库连接管理器
pub struct Database {
    /// SQLite连接
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

    /// 初始化数据库表结构
    ///
    /// # 参数
    /// * `conn` - 数据库连接引用
    pub fn initialize_tables(conn: &Connection) -> SqliteResult<()> {
        // 创建根目录索引表 info_root
        conn.execute(
            "CREATE TABLE IF NOT EXISTS info_root (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            root_name TEXT,
            root_path TEXT NOT NULL UNIQUE,
            status TEXT NOT NULL DEFAULT 'HEALTH',
            created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
            updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
        )",
            [],
        )?;
        info!("已创建 info_root 表");

        // 创建目录索引表 info_directory
        conn.execute(
            "CREATE TABLE IF NOT EXISTS info_directory (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            root_id INTEGER NOT NULL,
            directory_name TEXT NOT NULL DEFAULT '',
            directory_mtime INTEGER NOT NULL DEFAULT 0,
            directory_path TEXT,
            status TEXT NOT NULL DEFAULT 'HEALTH',
            created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
            updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
            FOREIGN KEY (root_id) REFERENCES info_root(id) ON DELETE CASCADE,
            UNIQUE(root_id, directory_path)
        )",
            [],
        )?;
        info!("已创建 info_directory 表");

        // 创建文件索引表 info_file
        conn.execute(
            "CREATE TABLE IF NOT EXISTS info_file (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            directory_id INTEGER NOT NULL,
            file_name TEXT NOT NULL,
            file_size INTEGER NOT NULL DEFAULT 0,
            file_mtime INTEGER NOT NULL DEFAULT 0,
            file_hash TEXT,
            status TEXT NOT NULL DEFAULT 'HEALTH',
            created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
            updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
            FOREIGN KEY (directory_id) REFERENCES info_directory(id) ON DELETE CASCADE,
            UNIQUE(directory_id, file_name)
        )",
            [],
        )?;
        info!("已创建 info_file 表");

        // 创建存档元数据表 archive_metadata
        conn.execute(
            "CREATE TABLE IF NOT EXISTS archive_metadata (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            archive_name TEXT NOT NULL UNIQUE,
            archive_limit_size INTEGER NOT NULL,
            archive_hash TEXT,
            is_compressed INTEGER NOT NULL DEFAULT 0,
            compressed_algorithm TEXT,
            is_encrypted INTEGER NOT NULL DEFAULT 0,
            encryption_algorithm TEXT,
            status TEXT NOT NULL DEFAULT 'HEALTH',
            created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
            updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
        )",
            [],
        )?;
        info!("已创建 archive_metadata 表");

        // 创建存档内容表 archive_asset
        conn.execute(
            "CREATE TABLE IF NOT EXISTS archive_asset (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            archive_id INTEGER NOT NULL,
            asset_name TEXT NOT NULL UNIQUE,
            asset_size INTEGER NOT NULL DEFAULT 0,
            asset_hash TEXT NOT NULL UNIQUE,
            asset_mtime INTEGER NOT NULL DEFAULT 0,
            asset_relative_path TEXT NOT NULL DEFAULT './',
            status TEXT NOT NULL DEFAULT 'HEALTH',
            created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
            updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
            FOREIGN KEY (archive_id) REFERENCES archive_metadata(id) ON DELETE CASCADE
        )",
            [],
        )?;
        info!("已创建 archive_asset 表");

        // 创建文件与存档资源表 map_file_asset
        conn.execute(
            "CREATE TABLE IF NOT EXISTS map_file_asset (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            file_id INTEGER NOT NULL,
            asset_id INTEGER NOT NULL,
            volume_order INTEGER NOT NULL DEFAULT 1,
            status TEXT NOT NULL DEFAULT 'HEALTH',
            created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
            updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
            FOREIGN KEY (file_id) REFERENCES info_file(id) ON DELETE CASCADE,
            FOREIGN KEY (asset_id) REFERENCES archive_asset(id) ON DELETE CASCADE,
            UNIQUE(file_id, asset_id)
        )",
            [],
        )?;
        info!("已创建 map_file_asset 表");

        // 创建文件列表视图 view_file
        conn.execute(
            "CREATE VIEW IF NOT EXISTS view_file AS
                SELECT
                    r.id as root_id,
                    r.root_path,
                    r.status as root_status,
                    d.id as directory_id,
                    d.directory_path,
                    d.directory_mtime,
                    d.status as directory_status,
                    f.id as file_id,
                    f.file_name,
                    f.file_size,
                    f.file_mtime,
                    f.file_hash,
                    f.status as file_status
                FROM info_root r
                JOIN info_directory d ON r.id = d.root_id
                JOIN info_file f ON d.id = f.directory_id",
            [],
        )?;
        info!("已创建 view_file 视图");

        // 创建资源列表视图 view_asset
        conn.execute(
            "CREATE VIEW IF NOT EXISTS view_asset AS
                SELECT
                    am.id as archive_id,
                    am.archive_name as archive_name,
                    am.status as archive_status,
                    aa.id as asset_id,
                    aa.asset_name,
                    aa.asset_size,
                    aa.asset_hash,
                    aa.asset_mtime,
                    aa.asset_relative_path as asset_relative_path,
                    aa.status as asset_status,
                    fva.file_id,
                    fva.volume_order
                FROM archive_metadata am
                JOIN archive_asset aa ON am.id = aa.archive_id
                LEFT JOIN map_file_asset fva ON aa.id = fva.asset_id",
            [],
        )?;
        info!("已创建 view_asset 视图");

        Ok(())
    }

    /// 添加一个新的根目录记录
    ///
    /// # 参数
    /// * `root_path` - 根目录路径
    /// * `root_name` - 根目录名称
    ///
    /// # 返回值
    /// 返回插入记录的ID
    pub fn add_root_directory(
        &self,
        root_path: &str,
        root_name: &str,
    ) -> Result<i64, rusqlite::Error> {
        self.conn.execute(
            "INSERT OR REPLACE INTO info_root (root_path, root_name) VALUES (?1, ?2)",
            params![root_path, root_name],
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
    pub fn find_root_info_by_path(
        &self,
        root_path: &str,
    ) -> Result<Option<InfoRoot>, rusqlite::Error> {
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
    /// * `root_id` - 根目录ID
    /// * [status](file://w:\zeogit\OneArchive-backend\backend\one-archive-api\src\main\java\com\cc01cc\onearchive\api\service\ArchiveService.java#L35-L35) - 新状态
    ///
    /// # 返回值
    /// 返回操作结果
    pub fn update_root_status(&self, root_id: i64, status: &str) -> Result<(), rusqlite::Error> {
        self.conn.execute(
            "UPDATE info_root SET status = ?1, updated_at = strftime('%s', 'now') WHERE id = ?2",
            params![status, root_id],
        )?;
        Ok(())
    }

    /// 将指定根目录下的所有目录标记为待删除状态
    ///
    /// # 参数
    /// * `root_id` - 根目录ID
    /// * [status](file://w:\zeogit\OneArchive-backend\backend\one-archive-api\src\main\java\com\cc01cc\onearchive\api\service\ArchiveService.java#L35-L35) - 新状态
    ///
    /// # 返回值
    /// 返回操作结果
    pub fn mark_directories_as_wait_to_delete(&self, root_id: i64) -> Result<(), rusqlite::Error> {
        self.conn.execute(
            "UPDATE info_directory SET status = ?1, updated_at = strftime('%s', 'now') WHERE root_id = ?2",
            params![DirectoryStatus::WAIT_TO_DELETE.as_str(), root_id],
        )?;
        Ok(())
    }

    /// 将指定根目录下的所有文件标记为待删除状态
    ///
    /// # 参数
    /// * `root_id` - 根目录ID
    ///
    /// # 返回值
    /// 返回操作结果
    pub fn mark_files_as_wait_to_delete(&self, root_id: i64) -> Result<(), rusqlite::Error> {
        self.conn.execute(
            "UPDATE info_file SET status = ?1, updated_at = strftime('%s', 'now') 
             WHERE directory_id IN (SELECT id FROM info_directory WHERE root_id = ?2)",
            params![FileStatus::WAIT_TO_DELETE.as_str(), root_id],
        )?;
        Ok(())
    }
    pub fn mark_table_status(
        &self,
        table_name: DatabaseTableName,
        id: i64,
        status: &str,
    ) -> Result<(), rusqlite::Error> {
        self.conn.execute(
            &format!(
                "UPDATE {} SET status = ?1, updated_at = strftime('%s', 'now') WHERE id = ?2",
                table_name.as_str()
            ),
            params![status, id],
        )?;
        Ok(())
    }

    /// 插入目录信息
    ///
    /// # 参数
    /// * [directory](file://w:\zeogit\OneArchive-backend\backend\one-archive-core\src\main\java\com\cc01cc\onearchive\core\config\ArchiveConfig.java#L34-L34) - 目录信息
    ///
    /// # 返回值
    /// 返回插入记录的ID
    pub fn insert_directory(&self, directory: &InfoDirectory) -> Result<i64, rusqlite::Error> {
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
    /// * [directory](file://w:\zeogit\OneArchive-backend\backend\one-archive-core\src\main\java\com\cc01cc\onearchive\core\config\ArchiveConfig.java#L34-L34) - 目录信息
    ///
    /// # 返回值
    /// 返回操作结果
    pub fn update_directory(&self, directory: &InfoDirectory) -> Result<(), rusqlite::Error> {
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

    /// 根据路径查找目录ID
    ///
    /// # 参数
    /// * `root_id` - 根目录ID
    /// * [path](file://w:\zeogit\OneArchive-backend\backend\one-archive-api\src\main\java\com\cc01cc\onearchive\api\service\ArchiveService.java#L32-L33) - 目录路径
    ///
    /// # 返回值
    /// 返回目录ID
    pub fn find_directory_by_path(
        &self,
        root_id: i64,
        path: &str,
    ) -> Result<Option<InfoDirectory>, rusqlite::Error> {
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

    pub fn find_file_by_name(
        &self,
        directory_id: i64,
        name: &str,
    ) -> Result<Option<InfoFile>, rusqlite::Error> {
        let mut stmt = self
            .conn
            .prepare("SELECT id FROM info_file WHERE directory_id = ?1 AND file_name = ?2")?;

        let mut rows = stmt.query(params![directory_id, name])?;

        if let Some(row) = rows.next()? {
            Ok(Some(row.try_into()?))
        } else {
            Ok(None)
        }
    }

    pub fn find_directories_by_status_and_root_id(
        &self,
        root_id: i64,
        status: Option<DirectoryStatus>,
    ) -> Result<Vec<InfoDirectory>, rusqlite::Error> {
        let sql = if let Some(status) = status {
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
        let mut rows = if let Some(status) = status {
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

    pub fn find_files_by_status_and_root_id(
        &self,
        root_id: i64,
        status: Option<FileStatus>,
    ) -> Result<Vec<InfoFile>, rusqlite::Error> {
        let sql = if let Some(status) = status {
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
        let mut rows = if let Some(status) = status {
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
    /// 返回插入记录的ID
    pub fn insert_file(&self, file: &InfoFile) -> Result<i64, rusqlite::Error> {
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
    pub fn update_file(&self, file: &InfoFile) -> Result<(), rusqlite::Error> {
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

    /// 根据根目录ID查找所有视图文件
    pub fn find_view_files_by_root_id(
        &self,
        root_id: i64,
    ) -> Result<Vec<ViewFile>, rusqlite::Error> {
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

    pub fn find_view_files_by_directory_id(
        &self,
        directory_id: i64,
    ) -> Result<Vec<ViewFile>, rusqlite::Error> {
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

    pub fn find_all_root_info(&self) -> Result<Vec<InfoRoot>, rusqlite::Error> {
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
        Database::initialize_tables(&db.conn).expect("Failed to initialize tables");

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
