//! 数据库仓库实现
//! 实现数据库操作的具体逻辑

use crate::mod_database::constants::{ArchiveStatus, AssetStatus, MapFileAssetStatus};
use crate::mod_database::impl_initialize;
use crate::mod_database::schema::{ArchiveAsset, ArchiveMetadata, MapFileAsset, ViewAsset};

use super::constants::{DatabaseTableName, DirectoryStatus, FileStatus};
use super::database::Database;
use super::schema::{InfoDirectory, InfoFile, InfoRoot, ViewFile};
use super::trait_database::{
    ArchiveAssetOperations, ArchiveMetadataOperations, DirectoryOperations, FileOperations,
    InitializationOperations, MapFileAssetOperations, RootOperations, StatusOperations,
    ViewOperations,
};
use log::warn;
use rusqlite::{Connection, Result as SqliteResult, params};

// 实现根目录操作 trait
impl RootOperations for Database {
    /// 添加根目录信息
    ///
    /// # 参数
    /// * `root_path` - 根目录路径
    /// * `root_name` - 根目录名称
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
    /// * [directory](\one-archive-core\src\main\java\com\cc01cc\onearchive\core\config\ArchiveConfig.java#L34-L34) - 目录信息
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
    /// * [directory](\one-archive-core\src\main\java\com\cc01cc\onearchive\core\config\ArchiveConfig.java#L34-L34) - 目录信息
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
    /// * [path](\one-archive-api\src\main\java\com\cc01cc\onearchive\api\service\ArchiveService.java#L32-L33) - 目录路径
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

// 实现存档元数据操作 trait
impl ArchiveMetadataOperations for Database {
    /// 插入存档元数据
    ///
    /// # 参数
    /// * [archive](\one-archive-core\src\main\java\com\cc01cc\onearchive\core\config\ArchiveConfig.java#L26-L26) - 存档元数据
    ///
    /// # 返回值
    /// 返回插入记录的 ID
    fn insert_archive_metadata(
        &self,
        archive: &ArchiveMetadata,
    ) -> SqliteResult<i64> {
        self.conn.execute(
            "INSERT INTO archive_metadata (archive_uri, archive_name, archive_limit_size, archive_hash, is_compressed, compressed_algorithm, is_encrypted, encryption_algorithm, status)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8,?9)",
            params![
                archive.archive_uri,
                archive.archive_name,
                archive.archive_limit_size,
                archive.archive_hash,
                archive.is_compressed,
                archive.compressed_algorithm,
                archive.is_encrypted,
                archive.encryption_algorithm,
                archive.status.as_str()
            ],
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    /// 更新存档元数据
    ///
    /// # 参数
    /// * [archive](\one-archive-core\src\main\java\com\cc01cc\onearchive\core\config\ArchiveConfig.java#L26-L26) - 存档元数据
    ///
    /// # 返回值
    /// 返回操作结果
    fn update_archive_metadata(
        &self,
        archive: &ArchiveMetadata,
    ) -> SqliteResult<()> {
        self.conn.execute(
            "UPDATE archive_metadata SET archive_name = ?1, archive_limit_size = ?2, archive_hash = ?3, is_compressed = ?4, compressed_algorithm = ?5, is_encrypted = ?6, encryption_algorithm = ?7, status = ?8, updated_at = strftime('%s', 'now')
             WHERE id = ?9",
            params![
                archive.archive_name,
                archive.archive_limit_size,
                archive.archive_hash.as_ref(),
                archive.is_compressed,
                archive.compressed_algorithm.as_ref(),
                archive.is_encrypted,
                archive.encryption_algorithm.as_ref(),
                archive.status.as_str(),
                archive.id
            ],
        )?;
        Ok(())
    }

    /// 根据 ID 查找存档元数据
    ///
    /// # 参数
    /// * [id](\one-archive-core\src\main\java\com\cc01cc\onearchive\core\entity\MapFileAsset.java#L24-L24) - 存档 ID
    ///
    /// # 返回值
    /// 返回存档元数据
    fn find_archive_metadata_by_id(
        &self,
        id: i64,
    ) -> SqliteResult<Option<ArchiveMetadata>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, archive_name, archive_limit_size, archive_hash, is_compressed, compressed_algorithm, is_encrypted, encryption_algorithm, status, created_at, updated_at 
             FROM archive_metadata WHERE id = ?1"
        )?;

        let mut rows = stmt.query(params![id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(row.try_into()?))
        } else {
            Ok(None)
        }
    }

    /// 根据名称查找存档元数据
    ///
    /// # 参数
    /// * [name](\one-archive-api\src\main\java\com\cc01cc\onearchive\api\service\ArchiveService.java#L29-L30) - 存档名称
    ///
    /// # 返回值
    /// 返回存档元数据
    fn find_archive_metadata_by_name(
        &self,
        name: &str,
    ) -> SqliteResult<Option<ArchiveMetadata>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, archive_name, archive_limit_size, archive_hash, is_compressed, compressed_algorithm, is_encrypted, encryption_algorithm, status, created_at, updated_at 
             FROM archive_metadata WHERE archive_name = ?1"
        )?;

        let mut rows = stmt.query(params![name])?;

        if let Some(row) = rows.next()? {
            Ok(Some(row.try_into()?))
        } else {
            Ok(None)
        }
    }

    /// 根据状态查找存档元数据
    ///
    /// # 参数
    ///
    /// # 返回值
    /// 返回存档元数据列表
    fn find_archive_metadata_by_status(
        &self,
        status: Option<ArchiveStatus>,
    ) -> SqliteResult<Vec<ArchiveMetadata>> {
        let sql = if status.is_some() {
            "SELECT id, archive_name, archive_limit_size, archive_hash, is_compressed, compressed_algorithm, is_encrypted, encryption_algorithm, status, created_at, updated_at 
             FROM archive_metadata WHERE status = ?1"
        } else {
            "SELECT id, archive_name, archive_limit_size, archive_hash, is_compressed, compressed_algorithm, is_encrypted, encryption_algorithm, status, created_at, updated_at 
             FROM archive_metadata"
        };

        let mut stmt = self.conn.prepare(sql)?;
        let mut rows = if let Some(ref status) = status {
            stmt.query(params![status.as_str()])?
        } else {
            stmt.query([])?
        };

        let mut archives = Vec::new();
        while let Some(row) = rows.next()? {
            let archive = row.try_into()?;
            archives.push(archive);
        }
        Ok(archives)
    }
}

// 实现存档资源操作 trait
impl ArchiveAssetOperations for Database {
    /// 插入存档资源
    ///
    /// # 参数
    /// * `asset` - 存档资源
    ///
    /// # 返回值
    /// 返回插入记录的 ID
    fn insert_archive_asset(
        &self,
        asset: &ArchiveAsset,
    ) -> SqliteResult<i64> {
        self.conn.execute(
            "INSERT INTO archive_asset (archive_id, asset_name, asset_size, asset_hash, asset_mtime, asset_relative_path, status)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                asset.archive_id,
                asset.asset_name,
                asset.asset_size,
                asset.asset_hash,
                asset.asset_mtime,
                asset.asset_relative_path,
                asset.status.as_str()
            ],
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    /// 更新存档资源
    ///
    /// # 参数
    /// * `asset` - 存档资源
    ///
    /// # 返回值
    /// 返回操作结果
    fn update_archive_asset(&self, asset: &ArchiveAsset) -> SqliteResult<()> {
        self.conn.execute(
            "UPDATE archive_asset SET archive_id = ?1, asset_name = ?2, asset_size = ?3, asset_hash = ?4, asset_mtime = ?5, asset_relative_path = ?6, status = ?7, updated_at = strftime('%s', 'now')
             WHERE id = ?8",
            params![
                asset.archive_id,
                asset.asset_name,
                asset.asset_size,
                asset.asset_hash,
                asset.asset_mtime,
                asset.asset_relative_path,
                asset.status.as_str(),
                asset.id
            ],
        )?;
        Ok(())
    }

    /// 根据 ID 查找存档资源
    ///
    /// # 参数
    /// * [id](\one-archive-core\src\main\java\com\cc01cc\onearchive\core\entity\MapFileAsset.java#L24-L24) - 资源 ID
    ///
    /// # 返回值
    /// 返回存档资源
    fn find_archive_asset_by_id(
        &self,
        id: i64,
    ) -> SqliteResult<Option<ArchiveAsset>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, archive_id, asset_name, asset_size, asset_hash, asset_mtime, asset_relative_path, status, created_at, updated_at 
             FROM archive_asset WHERE id = ?1"
        )?;

        let mut rows = stmt.query(params![id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(row.try_into()?))
        } else {
            Ok(None)
        }
    }

    /// 根据存档 ID 查找存档资源列表
    ///
    /// # 参数
    /// * `archive_id` - 存档 ID
    ///
    /// # 返回值
    /// 返回存档资源列表
    fn find_archive_assets_by_archive_id(
        &self,
        archive_id: i64,
    ) -> SqliteResult<Vec<ArchiveAsset>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, archive_id, asset_name, asset_size, asset_hash, asset_mtime, asset_relative_path, status, created_at, updated_at 
             FROM archive_asset WHERE archive_id = ?1"
        )?;

        let mut rows = stmt.query(params![archive_id])?;

        let mut assets = Vec::new();
        while let Some(row) = rows.next()? {
            let asset = row.try_into()?;
            assets.push(asset);
        }
        Ok(assets)
    }

    /// 根据状态查找存档资源
    ///
    /// # 参数
    ///
    /// # 返回值
    /// 返回存档资源列表
    fn find_archive_assets_by_status(
        &self,
        status: Option<AssetStatus>,
    ) -> SqliteResult<Vec<ArchiveAsset>> {
        let sql = if status.is_some() {
            "SELECT id, archive_id, asset_name, asset_size, asset_hash, asset_mtime, asset_relative_path, status, created_at, updated_at 
             FROM archive_asset WHERE status = ?1"
        } else {
            "SELECT id, archive_id, asset_name, asset_size, asset_hash, asset_mtime, asset_relative_path, status, created_at, updated_at 
             FROM archive_asset"
        };

        let mut stmt = self.conn.prepare(sql)?;
        let mut rows = if let Some(ref status) = status {
            stmt.query(params![status.as_str()])?
        } else {
            stmt.query([])?
        };

        let mut assets = Vec::new();
        while let Some(row) = rows.next()? {
            let asset = row.try_into()?;
            assets.push(asset);
        }
        Ok(assets)
    }

    fn find_archive_asset_by_asset_hash(
        &self,
        asset_hash: &str,
    ) -> SqliteResult<Option<crate::mod_database::schema::ArchiveAsset>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, archive_id, asset_name, asset_size, asset_hash, asset_mtime, asset_relative_path, status, created_at, updated_at 
             FROM archive_asset WHERE asset_hash = ?1"
        )?;
        let mut rows = stmt.query(params![asset_hash])?;
        // 先获取第一行（如果存在）
        if let Some(row) = rows.next()? {
            let first_row = row.try_into()?;

            // 检查是否还有更多行（表示有重复）
            if rows.next()?.is_some() {
                warn!("Duplicate asset hash found!");
            }
            Ok(Some(first_row))
        } else {
            Ok(None)
        }
    }
}

// 实现文件与存档资源映射操作 trait
impl MapFileAssetOperations for Database {
    /// 插入文件与存档资源映射
    ///
    /// # 参数
    /// * [map](\one-archive-core\src\main\java\com\cc01cc\onearchive\core\mapper\InfoRootRowMapper.java#L26-L36) - 映射信息
    ///
    /// # 返回值
    /// 返回插入记录的 ID
    fn insert_map_file_asset(&self, map: &MapFileAsset) -> SqliteResult<i64> {
        self.conn.execute(
            "INSERT INTO map_file_asset (file_id, asset_id, volume_order, status)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                map.file_id,
                map.asset_id,
                map.volume_order,
                map.status.as_str()
            ],
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    /// 更新文件与存档资源映射
    ///
    /// # 参数
    /// * [map](\one-archive-core\src\main\java\com\cc01cc\onearchive\core\mapper\InfoRootRowMapper.java#L26-L36) - 映射信息
    ///
    /// # 返回值
    /// 返回操作结果
    fn update_map_file_asset(&self, map: &MapFileAsset) -> SqliteResult<()> {
        self.conn.execute(
            "UPDATE map_file_asset SET file_id = ?1, asset_id = ?2, volume_order = ?3, status = ?4, updated_at = strftime('%s', 'now')
             WHERE id = ?5",
            params![
                map.file_id,
                map.asset_id,
                map.volume_order,
                map.status.as_str(),
                map.id
            ],
        )?;
        Ok(())
    }

    /// 根据 ID 查找文件与存档资源映射
    ///
    /// # 参数
    /// * [id](\one-archive-core\src\main\java\com\cc01cc\onearchive\core\entity\MapFileAsset.java#L24-L24) - 映射 ID
    ///
    /// # 返回值
    /// 返回映射信息
    fn find_map_file_asset_by_id(
        &self,
        id: i64,
    ) -> SqliteResult<Option<MapFileAsset>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, file_id, asset_id, volume_order, status, created_at, updated_at 
             FROM map_file_asset WHERE id = ?1",
        )?;

        let mut rows = stmt.query(params![id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(row.try_into()?))
        } else {
            Ok(None)
        }
    }

    /// 根据文件 ID 查找文件与存档资源映射
    ///
    /// # 参数
    /// * `file_id` - 文件 ID
    ///
    /// # 返回值
    /// 返回映射信息列表
    fn find_map_file_asset_by_file_id(
        &self,
        file_id: i64,
    ) -> SqliteResult<Vec<MapFileAsset>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, file_id, asset_id, volume_order, status, created_at, updated_at 
             FROM map_file_asset WHERE file_id = ?1",
        )?;

        let mut rows = stmt.query(params![file_id])?;

        let mut maps = Vec::new();
        while let Some(row) = rows.next()? {
            let map = row.try_into()?;
            maps.push(map);
        }
        Ok(maps)
    }
    /// 根据文件 ID 查找文件与存档资源映射，并按 volume_order 升序排序
    ///
    /// # 参数
    /// * `file_id` - 文件 ID
    ///
    /// # 返回值
    /// 返回按 volume_order 升序排序的映射信息列表
    fn find_map_file_asset_by_file_id_ordered(
        &self,
        file_id: i64,
    ) -> SqliteResult<Vec<MapFileAsset>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, file_id, asset_id, volume_order, status, created_at, updated_at 
             FROM map_file_asset WHERE file_id = ?1 ORDER BY volume_order ASC",
        )?;

        let mut rows = stmt.query(params![file_id])?;

        let mut maps = Vec::new();
        while let Some(row) = rows.next()? {
            let map = row.try_into()?;
            maps.push(map);
        }
        Ok(maps)
    }

    /// 根据资源 ID 查找文件与存档资源映射
    ///
    /// # 参数
    /// * `asset_id` - 资源 ID
    ///
    /// # 返回值
    /// 返回映射信息列表
    fn find_map_file_asset_by_asset_id(
        &self,
        asset_id: i64,
    ) -> SqliteResult<Vec<MapFileAsset>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, file_id, asset_id, volume_order, status, created_at, updated_at 
             FROM map_file_asset WHERE asset_id = ?1",
        )?;

        let mut rows = stmt.query(params![asset_id])?;

        let mut maps = Vec::new();
        while let Some(row) = rows.next()? {
            let map = row.try_into()?;
            maps.push(map);
        }
        Ok(maps)
    }

    /// 根据状态查找文件与存档资源映射
    ///
    /// # 参数
    ///
    /// # 返回值
    /// 返回映射信息列表
    fn find_map_file_asset_by_status(
        &self,
        status: Option<MapFileAssetStatus>,
    ) -> SqliteResult<Vec<MapFileAsset>> {
        let sql = if status.is_some() {
            "SELECT id, file_id, asset_id, volume_order, status, created_at, updated_at 
             FROM map_file_asset WHERE status = ?1"
        } else {
            "SELECT id, file_id, asset_id, volume_order, status, created_at, updated_at 
             FROM map_file_asset"
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

    /// 根据存档 ID 查找视图资源
    fn find_view_assets_by_archive_id(
        &self,
        archive_id: i64,
    ) -> SqliteResult<Vec<ViewAsset>> {
        let sql = "SELECT archive_id, archive_name, archive_status, asset_id, asset_name, asset_size, asset_hash, asset_mtime, asset_relative_path, asset_status, file_id, volume_order
                 FROM view_asset 
                 WHERE archive_id = ?1";

        let mut stmt = self.conn.prepare(sql)?;
        let mut rows = stmt.query(params![archive_id])?;

        let mut view_assets = Vec::new();
        while let Some(row) = rows.next()? {
            let view_asset: ViewAsset = row.try_into()?;
            view_assets.push(view_asset);
        }
        Ok(view_assets)
    }

    fn find_view_assets_by_file_id(
        &self,
        file_id: i64,
    ) -> SqliteResult<Vec<ViewAsset>> {
        let sql = "SELECT archive_id, archive_uri, archive_name, archive_status, asset_id, asset_name, asset_size, asset_hash, asset_mtime, asset_relative_path, asset_status, file_id, volume_order
                 FROM view_asset 
                 WHERE file_id = ?1";

        let mut stmt = self.conn.prepare(sql)?;
        let mut rows = stmt.query(params![file_id])?;
        let mut view_assets = Vec::new();
        while let Some(row) = rows.next()? {
            let view_asset: ViewAsset = row.try_into()?;
            view_assets.push(view_asset);
        }
        Ok(view_assets)
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
