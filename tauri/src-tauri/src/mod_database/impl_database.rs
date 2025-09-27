//! 数据库仓库实现
//! 实现数据库操作的具体逻辑

use super::constants::{DatabaseTableName, DirectoryStatus, FileStatus};
use super::database::Database;
use super::schema::{InfoDirectory, InfoFile, InfoRoot, ViewFile};
use super::trait_database::{
    ArchiveChunkOperations, DirectoryOperations, FileOperations, InitializationOperations,
    MapFileChunkOperations, RootOperations, StatusOperations, ViewOperations,
};
use crate::mod_database::common::DatabaseCommonOperations;
use crate::mod_database::constants::{ArchiveStatus, ChunkStatus, MapFileChunkStatus};
use crate::mod_database::schema::{
    ArchiveChunk, ArchiveMetadata, CreateArchiveMetadataParams, MapFileChunk, ViewChunk,
};
use crate::mod_database::{impl_initialize, queries};
use log::warn;
use rusqlite::{Connection, Result as SqliteResult, named_params, params};

// 实现根目录操作 trait
impl RootOperations for Database{
    /// 添加根目录信息
    ///
    /// # 参数
    /// * [root_path](file://w:\zeogit\OneArchive-backend\tauri\src\types\recovery.ts#L18-L18) - 根目录路径
    /// * [root_name](file://w:\zeogit\OneArchive-backend\tauri\src\types\recovery.ts#L19-L19) - 根目录名称
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
    /// * [root_path](file://w:\zeogit\OneArchive-backend\tauri\src\types\recovery.ts#L18-L18) - 根目录路径
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
    ///
    /// # 返回值
    /// 返回插入记录的 ID
    fn insert_directory(&self, directory: &InfoDirectory) -> SqliteResult<i64> {
        // 规范化目录路径，确保不以 '/' 结尾
        let normalized_directory = InfoDirectory {
            directory_path: Self::normalize_directory_path(&directory.directory_path)
                .map(|path| path.replace('\\', "/")),
            ..directory.clone()
        };

        self.conn.execute(
        "INSERT INTO info_directory (root_id, directory_name, directory_mtime, directory_path, status) 
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            normalized_directory.root_id,
            normalized_directory.directory_name,
            normalized_directory.directory_mtime,
            normalized_directory.directory_path,
            normalized_directory.status.as_str()
        ],
    )?;

        Ok(self.conn.last_insert_rowid())
    }

    /// 更新目录信息
    ///
    /// # 参数
    ///
    /// # 返回值
    /// 返回操作结果
    fn update_directory(&self, directory: &InfoDirectory) -> SqliteResult<()> {
        // 规范化目录路径，确保不以 '/' 结尾，并统一使用正斜杠作为路径分隔符
        let normalized_directory = InfoDirectory {
            directory_path: Self::normalize_directory_path(&directory.directory_path)
                .map(|path| path.replace('\\', "/")),
            ..directory.clone()
        };

        self.conn.execute(
        "UPDATE info_directory SET directory_name = ?1, directory_mtime = ?2, status = ?3, updated_at = strftime('%s', 'now') 
         WHERE id = ?4",
        params![
            normalized_directory.directory_name,
            normalized_directory.directory_mtime,
            normalized_directory.status.as_str(),
            normalized_directory.id
        ],
    )?;
        Ok(())
    }

    /// 根据路径查找目录 ID
    ///
    /// # 参数
    /// * `root_id` - 根目录 ID
    /// * [path](\one-archive-api\src\main\java\com\cc01cc\onearchive\api\service\ArchiveService.java#L32-L33) - 目录路径
    ///
    /// # 返回值
    /// 返回目录 ID
    fn find_directory_by_path(
        &self,
        root_id: i64,
        path: &str,
    ) -> SqliteResult<Option<InfoDirectory>> {
        // 规范化查询路径，确保使用统一的正斜杠分隔符
        let normalized_path = Self::normalize_directory_path(&Some(path.to_string()));

        let mut stmt = self
            .conn
            .prepare("SELECT id, root_id, directory_name, directory_mtime, directory_path, status, created_at, updated_at 
              FROM info_directory WHERE root_id = ?1 AND directory_path = ?2")?;

        let mut rows = stmt.query(params![root_id, normalized_path])?;

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
    /// 根据 ID 查找目录
    ///
    /// # 参数
    ///
    /// # 返回值
    /// 返回目录信息
    fn find_directory_by_id(&self, id: i64) -> SqliteResult<Option<InfoDirectory>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, root_id, directory_name, directory_mtime, directory_path, status, created_at, updated_at 
              FROM info_directory WHERE id = ?1")?;

        let mut rows = stmt.query(params![id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(row.try_into()?))
        } else {
            Ok(None)
        }
    }
    /// 查找某个目录的直接父目录
    ///
    /// # 参数
    /// * `root_id` - 根目录 ID
    /// * `directory_path` - 目录路径
    ///
    /// # 返回值
    /// 返回父目录信息
    fn find_parent_directory(
        &self,
        root_id: i64,
        directory_path: &str,
    ) -> SqliteResult<Option<InfoDirectory>> {
        // 如果目录路径为空或根目录，则没有父目录
        if directory_path.is_empty() {
            return Ok(None);
        }

        // 规范化目录路径
        let normalized_path =
            Self::normalize_directory_path(&Some(directory_path.to_string())).unwrap_or_default();

        // 查找父目录路径
        let parent_path = if let Some(last_slash_pos) = normalized_path.rfind('/') {
            &normalized_path[..last_slash_pos]
        } else {
            "" // 父目录是根目录
        };

        self.find_directory_by_path(root_id, parent_path)
    }

    /// 查找某个目录下的所有直接子目录
    ///
    /// # 参数
    /// * `root_id` - 根目录 ID
    /// * `parent_path` - 父目录路径
    ///
    /// # 返回值
    /// 返回直接子目录列表
    fn find_child_directories(
        &self,
        root_id: i64,
        parent_path: &str,
    ) -> SqliteResult<Vec<InfoDirectory>> {
        // 规范化父目录路径
        let normalized_parent_path =
            Self::normalize_directory_path(&Some(parent_path.to_string())).unwrap_or_default();

        let sql = if normalized_parent_path.is_empty() {
            // 查找根目录下的直接子目录（路径中不包含 '/' 的目录）
            "SELECT id, root_id, directory_name, directory_mtime, directory_path, status, created_at, updated_at
             FROM info_directory
             WHERE root_id = ?1 AND directory_path NOT NULL AND directory_path != '' 
             AND directory_path NOT LIKE '%/%'"
        } else {
            // 查找指定目录下的直接子目录（路径以 parent_path 开头，且后面只有一级目录）
            "SELECT id, root_id, directory_name, directory_mtime, directory_path, status, created_at, updated_at
             FROM info_directory
             WHERE root_id = ?1 AND directory_path LIKE ?2 || '/%' 
             AND directory_path NOT LIKE ?3 || '/%/%'"
        };

        let mut stmt = self.conn.prepare(sql)?;
        let mut rows = if normalized_parent_path.is_empty() {
            stmt.query(params![root_id])?
        } else {
            stmt.query(params![
                root_id,
                normalized_parent_path,
                normalized_parent_path
            ])?
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

// 实现归档数据块操作 trait
impl ArchiveChunkOperations for Database {
    /// 插入归档数据块
    ///
    /// # 参数
    /// * `chunk` - 归档数据块
    ///
    /// # 返回值
    /// 返回插入记录的 ID
    fn insert_archive_chunk(&self, chunk: &ArchiveChunk) -> SqliteResult<i64> {
        let normalized_path =
            Database::normalize_directory_path(&Some(chunk.chunk_relative_path.to_string()));
        self.conn.execute(
            "INSERT INTO archive_chunk (archive_id, chunk_name, chunk_size, chunk_hash, chunk_mtime, chunk_relative_path, status)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                chunk.archive_id,
                chunk.chunk_name,
                chunk.chunk_size,
                chunk.chunk_hash,
                chunk.chunk_mtime,
                normalized_path,
                chunk.status.as_str()
            ],
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    /// 更新归档数据块
    ///
    /// # 参数
    /// * `chunk` - 归档数据块
    ///
    /// # 返回值
    /// 返回操作结果
    fn update_archive_chunk(&self, chunk: &ArchiveChunk) -> SqliteResult<()> {
        self.conn.execute(
            "UPDATE archive_chunk SET archive_id = ?1, chunk_name = ?2, chunk_size = ?3, chunk_hash = ?4, chunk_mtime = ?5, chunk_relative_path = ?6, status = ?7, updated_at = strftime('%s', 'now')
             WHERE id = ?8",
            params![
                chunk.archive_id,
                chunk.chunk_name,
                chunk.chunk_size,
                chunk.chunk_hash,
                chunk.chunk_mtime,
                chunk.chunk_relative_path,
                chunk.status.as_str(),
                chunk.id
            ],
        )?;
        Ok(())
    }

    /// 根据 ID 查找归档数据块
    ///
    /// # 参数
    /// * [id](\one-archive-core\src\main\java\com\cc01cc\onearchive\core\entity\MapFileChunk.java#L24-L24) - 数据块 ID
    ///
    /// # 返回值
    /// 返回归档数据块
    fn find_archive_chunk_by_id(&self, id: i64) -> SqliteResult<Option<ArchiveChunk>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, archive_id, chunk_name, chunk_size, chunk_hash, chunk_mtime, chunk_relative_path, status, created_at, updated_at 
             FROM archive_chunk WHERE id = ?1"
        )?;

        let mut rows = stmt.query(params![id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(row.try_into()?))
        } else {
            Ok(None)
        }
    }

    /// 根据归档 ID 查找归档数据块列表
    ///
    /// # 参数
    /// * `archive_id` - 归档 ID
    ///
    /// # 返回值
    /// 返回归档数据块列表
    fn find_archive_chunks_by_archive_id(
        &self,
        archive_id: i64,
    ) -> SqliteResult<Vec<ArchiveChunk>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, archive_id, chunk_name, chunk_size, chunk_hash, chunk_mtime, chunk_relative_path, status, created_at, updated_at 
             FROM archive_chunk WHERE archive_id = ?1"
        )?;

        let mut rows = stmt.query(params![archive_id])?;

        let mut chunks = Vec::new();
        while let Some(row) = rows.next()? {
            let chunk = row.try_into()?;
            chunks.push(chunk);
        }
        Ok(chunks)
    }

    /// 根据状态查找归档数据块
    ///
    /// # 参数
    ///
    /// # 返回值
    /// 返回归档数据块列表
    fn find_archive_chunks_by_status(
        &self,
        status: Option<ChunkStatus>,
    ) -> SqliteResult<Vec<ArchiveChunk>> {
        let sql = if status.is_some() {
            "SELECT id, archive_id, chunk_name, chunk_size, chunk_hash, chunk_mtime, chunk_relative_path, status, created_at, updated_at 
             FROM archive_chunk WHERE status = ?1"
        } else {
            "SELECT id, archive_id, chunk_name, chunk_size, chunk_hash, chunk_mtime, chunk_relative_path, status, created_at, updated_at 
             FROM archive_chunk"
        };

        let mut stmt = self.conn.prepare(sql)?;
        let mut rows = if let Some(ref status) = status {
            stmt.query(params![status.as_str()])?
        } else {
            stmt.query([])?
        };

        let mut chunks = Vec::new();
        while let Some(row) = rows.next()? {
            let chunk = row.try_into()?;
            chunks.push(chunk);
        }
        Ok(chunks)
    }

    fn find_archive_chunk_by_chunk_hash(
        &self,
        chunk_hash: &str,
    ) -> SqliteResult<Option<crate::mod_database::schema::ArchiveChunk>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, archive_id, chunk_name, chunk_size, chunk_hash, chunk_mtime, chunk_relative_path, status, created_at, updated_at 
             FROM archive_chunk WHERE chunk_hash = ?1"
        )?;
        let mut rows = stmt.query(params![chunk_hash])?;
        // 先获取第一行（如果存在）
        if let Some(row) = rows.next()? {
            let first_row = row.try_into()?;

            // 检查是否还有更多行（表示有重复）
            if rows.next()?.is_some() {
                warn!("Duplicate chunk hash found!");
            }
            Ok(Some(first_row))
        } else {
            Ok(None)
        }
    }
}

// 实现文件与归档数据块映射操作 trait
impl MapFileChunkOperations for Database {
    /// 插入文件与归档数据块映射
    ///
    /// # 参数
    /// * [map](\one-archive-core\src\main\java\com\cc01cc\onearchive\core\mapper\InfoRootRowMapper.java#L26-L36) - 映射信息
    ///
    /// # 返回值
    /// 返回插入记录的 ID
    fn insert_map_file_chunk(&self, map: &MapFileChunk) -> SqliteResult<i64> {
        self.conn.execute(
            "INSERT INTO map_file_chunk (file_id, chunk_id, volume_order, status)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                map.file_id,
                map.chunk_id,
                map.volume_order,
                map.status.as_str()
            ],
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    /// 更新文件与归档数据块映射
    ///
    /// # 参数
    /// * [map](\one-archive-core\src\main\java\com\cc01cc\onearchive\core\mapper\InfoRootRowMapper.java#L26-L36) - 映射信息
    ///
    /// # 返回值
    /// 返回操作结果
    fn update_map_file_chunk(&self, map: &MapFileChunk) -> SqliteResult<()> {
        self.conn.execute(
            "UPDATE map_file_chunk SET file_id = ?1, chunk_id = ?2, volume_order = ?3, status = ?4, updated_at = strftime('%s', 'now')
             WHERE id = ?5",
            params![
                map.file_id,
                map.chunk_id,
                map.volume_order,
                map.status.as_str(),
                map.id
            ],
        )?;
        Ok(())
    }

    /// 根据 ID 查找文件与归档数据块映射
    ///
    /// # 参数
    /// * [id](\one-archive-core\src\main\java\com\cc01cc\onearchive\core\entity\MapFileChunk.java#L24-L24) - 映射 ID
    ///
    /// # 返回值
    /// 返回映射信息
    fn find_map_file_chunk_by_id(&self, id: i64) -> SqliteResult<Option<MapFileChunk>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, file_id, chunk_id, volume_order, status, created_at, updated_at 
             FROM map_file_chunk WHERE id = ?1",
        )?;

        let mut rows = stmt.query(params![id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(row.try_into()?))
        } else {
            Ok(None)
        }
    }

    /// 根据文件 ID 查找文件与归档数据块映射
    ///
    /// # 参数
    /// * `file_id` - 文件 ID
    ///
    /// # 返回值
    /// 返回映射信息列表
    fn find_map_file_chunk_by_file_id(&self, file_id: i64) -> SqliteResult<Vec<MapFileChunk>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, file_id, chunk_id, volume_order, status, created_at, updated_at 
             FROM map_file_chunk WHERE file_id = ?1",
        )?;

        let mut rows = stmt.query(params![file_id])?;

        let mut maps = Vec::new();
        while let Some(row) = rows.next()? {
            let map = row.try_into()?;
            maps.push(map);
        }
        Ok(maps)
    }
    /// 根据文件 ID 查找文件与归档数据块映射，并按 volume_order 升序排序
    ///
    /// # 参数
    /// * `file_id` - 文件 ID
    ///
    /// # 返回值
    /// 返回按 volume_order 升序排序的映射信息列表
    fn find_map_file_chunk_by_file_id_ordered(
        &self,
        file_id: i64,
    ) -> SqliteResult<Vec<MapFileChunk>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, file_id, chunk_id, volume_order, status, created_at, updated_at 
             FROM map_file_chunk WHERE file_id = ?1 ORDER BY volume_order ASC",
        )?;

        let mut rows = stmt.query(params![file_id])?;

        let mut maps = Vec::new();
        while let Some(row) = rows.next()? {
            let map = row.try_into()?;
            maps.push(map);
        }
        Ok(maps)
    }

    /// 根据数据块 ID 查找文件与归档数据块映射
    ///
    /// # 参数
    /// * `chunk_id` - 数据块 ID
    ///
    /// # 返回值
    /// 返回映射信息列表
    fn find_map_file_chunk_by_chunk_id(&self, chunk_id: i64) -> SqliteResult<Vec<MapFileChunk>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, file_id, chunk_id, volume_order, status, created_at, updated_at 
             FROM map_file_chunk WHERE chunk_id = ?1",
        )?;

        let mut rows = stmt.query(params![chunk_id])?;

        let mut maps = Vec::new();
        while let Some(row) = rows.next()? {
            let map = row.try_into()?;
            maps.push(map);
        }
        Ok(maps)
    }

    /// 根据状态查找文件与归档数据块映射
    ///
    /// # 参数
    ///
    /// # 返回值
    /// 返回映射信息列表
    fn find_map_file_chunk_by_status(
        &self,
        status: Option<MapFileChunkStatus>,
    ) -> SqliteResult<Vec<MapFileChunk>> {
        let sql = if status.is_some() {
            "SELECT id, file_id, chunk_id, volume_order, status, created_at, updated_at 
             FROM map_file_chunk WHERE status = ?1"
        } else {
            "SELECT id, file_id, chunk_id, volume_order, status, created_at, updated_at 
             FROM map_file_chunk"
        };

        let mut stmt = self.conn.prepare(sql)?;
        let mut rows = if let Some(ref status) = status {
            stmt.query(params![status.as_str()])?
        } else {
            stmt.query([])?
        };

        let mut maps = Vec::new();
        while let Some(row) = rows.next()? {
            let map = row.try_into()?;
            maps.push(map);
        }
        Ok(maps)
    }
}

// 实现视图操作 trait
impl ViewOperations for Database {
    /// 根据根目录 ID 查找所有视图文件
    ///
    /// 注意：此方法已被弃用，请使用 find_view_files_by_root_id_and_status 方法替代
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

    /// 根据 root_id 和 status 获取视图文件
    fn find_view_files_by_root_id_and_status(
        &self,
        root_id: i64,
        status: Option<FileStatus>,
    ) -> SqliteResult<Vec<ViewFile>> {
        let sql = if status.is_some() {
            format!(
                "SELECT root_id, root_path, root_status, directory_id, directory_path, directory_mtime, directory_status,
                        file_id, file_name, file_size, file_mtime, file_hash, file_status
                 FROM view_file 
                 WHERE root_id = ?1 AND file_status = ?2"
            )
        } else {
            format!(
                "SELECT root_id, root_path, root_status, directory_id, directory_path, directory_mtime, directory_status,
                        file_id, file_name, file_size, file_mtime, file_hash, file_status
                 FROM view_file 
                 WHERE root_id = ?1"
            )
        };

        let mut stmt = self.conn.prepare(&sql)?;
        let mut rows = if let Some(ref status) = status {
            stmt.query(params![root_id, status.as_str()])?
        } else {
            stmt.query(params![root_id])?
        };

        let mut view_files = Vec::new();
        while let Some(row) = rows.next()? {
            let view_file: ViewFile = row.try_into()?;
            view_files.push(view_file);
        }
        Ok(view_files)
    }

    fn find_view_files_by_hash_and_status(
        &self,
        hash: &str,
        status: Option<FileStatus>,
    ) -> SqliteResult<Vec<ViewFile>> {
        let sql = if status.is_some() {
            format!(
                "SELECT root_id, root_path, root_status, directory_id, directory_path, directory_mtime, directory_status,
                        file_id, file_name, file_size, file_mtime, file_hash, file_status
                 FROM view_file 
                 WHERE file_hash = ?1 AND file_status = ?2"
            )
        } else {
            format!(
                "SELECT root_id, root_path, root_status, directory_id, directory_path, directory_mtime, directory_status,
                        file_id, file_name, file_size, file_mtime, file_hash, file_status
                 FROM view_file 
                 WHERE file_hash = ?1"
            )
        };

        let mut stmt = self.conn.prepare(&sql)?;
        let mut rows = if let Some(ref status) = status {
            stmt.query(params![hash, status.as_str()])?
        } else {
            stmt.query(params![hash])?
        };

        let mut view_files = Vec::new();
        while let Some(row) = rows.next()? {
            let view_file: ViewFile = row.try_into()?;
            view_files.push(view_file);
        }
        Ok(view_files)
    }

    /// 根据归档 ID 查找视图数据块
    fn find_view_chunks_by_archive_id(&self, archive_id: i64) -> SqliteResult<Vec<ViewChunk>> {
        let sql = "SELECT archive_id, archive_name, archive_status, chunk_id, chunk_name, chunk_size, chunk_hash, chunk_mtime, chunk_relative_path, chunk_status, file_id, volume_order
                 FROM view_chunk 
                 WHERE archive_id = ?1";

        let mut stmt = self.conn.prepare(sql)?;
        let mut rows = stmt.query(params![archive_id])?;

        let mut view_chunks = Vec::new();
        while let Some(row) = rows.next()? {
            let view_chunk: ViewChunk = row.try_into()?;
            view_chunks.push(view_chunk);
        }
        Ok(view_chunks)
    }

    fn find_view_chunks_by_file_id(&self, file_id: i64) -> SqliteResult<Vec<ViewChunk>> {
        let sql = "SELECT archive_id, archive_uri, archive_name, archive_status, chunk_id, chunk_name, chunk_size, chunk_hash, chunk_mtime, chunk_relative_path, chunk_status, file_id, volume_order
                 FROM view_chunk 
                 WHERE file_id = ?1";

        let mut stmt = self.conn.prepare(sql)?;
        let mut rows = stmt.query(params![file_id])?;
        let mut view_chunks = Vec::new();
        while let Some(row) = rows.next()? {
            let view_chunk: ViewChunk = row.try_into()?;
            view_chunks.push(view_chunk);
        }
        Ok(view_chunks)
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

        // Test archive_chunk table
        let mut stmt = db
            .conn
            .prepare("SELECT COUNT(*) FROM archive_chunk")
            .unwrap();
        let count: i64 = stmt.query_row([], |row| row.get(0)).unwrap();
        assert_eq!(count, 0);

        // Test map_file_chunk table
        let mut stmt = db
            .conn
            .prepare("SELECT COUNT(*) FROM map_file_chunk")
            .unwrap();
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
