//! 数据库操作 trait 定义
//! 定义各种数据库操作的接口

use super::constants::{DatabaseTableName, DirectoryStatus, FileStatus};
use super::schema::{InfoDirectory, InfoFile, InfoRoot, ViewFile};
use rusqlite::{Connection, Result as SqliteResult};

/// 数据库根目录操作 trait
pub trait RootOperations {
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
    ) -> SqliteResult<i64>;

    /// 根据路径查找根目录信息
    ///
    /// # 参数
    /// * `root_path` - 根目录路径
    ///
    /// # 返回值
    /// 返回根目录信息
    fn find_root_info_by_path(&self, root_path: &str) -> SqliteResult<Option<InfoRoot>>;

    /// 更新根目录状态
    ///
    /// # 参数
    /// * `root_id` - 根目录 ID
    ///
    /// # 返回值
    /// 返回操作结果
    fn update_root_status(&self, root_id: i64, status: &str) -> SqliteResult<()>;

    /// 查找所有根目录信息
    fn find_all_root_info(&self) -> SqliteResult<Vec<InfoRoot>>;

}

/// 数据库目录操作 trait
pub trait DirectoryOperations {
    /// 将指定根目录下的所有目录标记为待删除状态
    ///
    /// # 参数
    /// * `root_id` - 根目录 ID
    ///
    /// # 返回值
    /// 返回操作结果
    fn mark_directories_as_wait_to_delete(&self, root_id: i64) -> SqliteResult<()>;

    /// 插入目录信息
    ///
    /// # 参数
    /// * [directory](\one-archive-core\src\main\java\com\cc01cc\onearchive\core\config\ArchiveConfig.java#L34-L34) - 目录信息
    ///
    /// # 返回值
    /// 返回插入记录的 ID
    fn insert_directory(&self, directory: &InfoDirectory) -> SqliteResult<i64>;

    /// 更新目录信息
    ///
    /// # 参数
    /// * [directory](\one-archive-core\src\main\java\com\cc01cc\onearchive\core\config\ArchiveConfig.java#L34-L34) - 目录信息
    ///
    /// # 返回值
    /// 返回操作结果
    fn update_directory(&self, directory: &InfoDirectory) -> SqliteResult<()>;

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
    ) -> SqliteResult<Option<InfoDirectory>>;

    /// 根据状态和根目录 ID 查找目录
    fn find_directories_by_status_and_root_id(
        &self,
        root_id: i64,
        status: Option<DirectoryStatus>,
    ) -> SqliteResult<Vec<InfoDirectory>>;
}

/// 数据库文件操作 trait
pub trait FileOperations {
    /// 将指定根目录下的所有文件标记为待删除状态
    ///
    /// # 参数
    /// * `root_id` - 根目录 ID
    ///
    /// # 返回值
    /// 返回操作结果
    fn mark_files_as_wait_to_delete(&self, root_id: i64) -> SqliteResult<()>;

    /// 根据名称查找文件
    fn find_file_by_name(&self, directory_id: i64, name: &str) -> SqliteResult<Option<InfoFile>>;

    /// 根据状态和根目录 ID 查找文件
    fn find_files_by_status_and_root_id(
        &self,
        root_id: i64,
        status: Option<FileStatus>,
    ) -> SqliteResult<Vec<InfoFile>>;

    /// 插入文件信息
    ///
    /// # 参数
    /// * `file` - 文件信息
    ///
    /// # 返回值
    /// 返回插入记录的 ID
    fn insert_file(&self, file: &InfoFile) -> SqliteResult<i64>;

    /// 更新文件信息
    ///
    /// # 参数
    /// * `file` - 文件信息
    ///
    /// # 返回值
    /// 返回操作结果
    fn update_file(&self, file: &InfoFile) -> SqliteResult<()>;
}

/// 数据库存档元数据操作 trait
pub trait ArchiveMetadataOperations {
    /// 插入存档元数据
    ///
    /// # 参数
    /// * [archive](\one-archive-core\src\main\java\com\cc01cc\onearchive\core\config\ArchiveConfig.java#L26-L26) - 存档元数据
    ///
    /// # 返回值
    /// 返回插入记录的 ID
    fn insert_archive_metadata(
        &self,
        archive: &super::schema::ArchiveMetadata,
    ) -> SqliteResult<i64>;

    /// 更新存档元数据
    ///
    /// # 参数
    /// * [archive](\one-archive-core\src\main\java\com\cc01cc\onearchive\core\config\ArchiveConfig.java#L26-L26) - 存档元数据
    ///
    /// # 返回值
    /// 返回操作结果
    fn update_archive_metadata(&self, archive: &super::schema::ArchiveMetadata)
    -> SqliteResult<()>;

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
    ) -> SqliteResult<Option<super::schema::ArchiveMetadata>>;

    /// 根据名称查找存档元数据
    ///
    /// # 参数
    ///
    /// # 返回值
    /// 返回存档元数据
    fn find_archive_metadata_by_name(
        &self,
        name: &str,
    ) -> SqliteResult<Option<super::schema::ArchiveMetadata>>;

    /// 根据状态查找存档元数据
    ///
    /// # 参数
    ///
    /// # 返回值
    /// 返回存档元数据列表
    fn find_archive_metadata_by_status(
        &self,
        status: Option<super::constants::ArchiveStatus>,
    ) -> SqliteResult<Vec<super::schema::ArchiveMetadata>>;
}

/// 数据库存档资源操作 trait
pub trait ArchiveAssetOperations {
    /// 插入存档资源
    ///
    /// # 参数
    /// * `asset` - 存档资源
    ///
    /// # 返回值
    /// 返回插入记录的 ID
    fn insert_archive_asset(&self, asset: &super::schema::ArchiveAsset) -> SqliteResult<i64>;

    /// 更新存档资源
    ///
    /// # 参数
    /// * `asset` - 存档资源
    ///
    /// # 返回值
    /// 返回操作结果
    fn update_archive_asset(&self, asset: &super::schema::ArchiveAsset) -> SqliteResult<()>;

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
    ) -> SqliteResult<Option<super::schema::ArchiveAsset>>;

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
    ) -> SqliteResult<Vec<super::schema::ArchiveAsset>>;

    /// 根据状态查找存档资源
    ///
    /// # 参数
    ///
    /// # 返回值
    /// 返回存档资源列表
    fn find_archive_assets_by_status(
        &self,
        status: Option<super::constants::AssetStatus>,
    ) -> SqliteResult<Vec<super::schema::ArchiveAsset>>;

    /// 根据 asset hash 查找存档资源
    fn find_archive_asset_by_asset_hash(
        &self,
        asset_hash: &str,
    ) -> SqliteResult<Option<super::schema::ArchiveAsset>>;
}

/// 数据库文件与存档资源映射操作 trait
pub trait MapFileAssetOperations {
    /// 插入文件与存档资源映射
    ///
    /// # 参数
    /// * [map](\one-archive-core\src\main\java\com\cc01cc\onearchive\core\mapper\InfoRootRowMapper.java#L26-L36) - 映射信息
    ///
    /// # 返回值
    /// 返回插入记录的 ID
    fn insert_map_file_asset(&self, map: &super::schema::MapFileAsset) -> SqliteResult<i64>;

    /// 更新文件与存档资源映射
    ///
    /// # 参数
    /// * [map](\one-archive-core\src\main\java\com\cc01cc\onearchive\core\mapper\InfoRootRowMapper.java#L26-L36) - 映射信息
    ///
    /// # 返回值
    /// 返回操作结果
    fn update_map_file_asset(&self, map: &super::schema::MapFileAsset) -> SqliteResult<()>;

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
    ) -> SqliteResult<Option<super::schema::MapFileAsset>>;

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
    ) -> SqliteResult<Vec<super::schema::MapFileAsset>>;

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
    ) -> SqliteResult<Vec<super::schema::MapFileAsset>>;

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
    ) -> SqliteResult<Vec<super::schema::MapFileAsset>>;

    /// 根据状态查找文件与存档资源映射
    ///
    /// # 参数
    ///
    /// # 返回值
    /// 返回映射信息列表
    fn find_map_file_asset_by_status(
        &self,
        status: Option<super::constants::MapFileAssetStatus>,
    ) -> SqliteResult<Vec<super::schema::MapFileAsset>>;
}

/// 数据库视图操作 trait
pub trait ViewOperations {
    /// 根据根目录 ID 查找所有视图文件
    fn find_view_files_by_root_id(&self, root_id: i64) -> SqliteResult<Vec<ViewFile>>;

    /// 根据目录 ID 查找视图文件
    fn find_view_files_by_directory_id(&self, directory_id: i64) -> SqliteResult<Vec<ViewFile>>;

    /// 根据 root_id 和 file status 获取视图文件
    fn find_view_files_by_root_id_and_status(
        &self,
        root_id: i64,
        status: Option<FileStatus>,
    ) -> SqliteResult<Vec<ViewFile>>;

    /// 根据 hash 和 status 获取视图文件
    fn find_view_files_by_hash_and_status(
        &self,
        hash: &str,
        status: Option<FileStatus>,
    ) -> SqliteResult<Vec<ViewFile>>;

    /// 根据存档 ID 查找视图资源
    fn find_view_assets_by_archive_id(
        &self,
        archive_id: i64,
    ) -> SqliteResult<Vec<super::schema::ViewAsset>>;

    /// 根据文件 ID 查找视图资源
    fn find_view_assets_by_file_id(
        &self,
        file_id: i64,
    ) -> SqliteResult<Vec<super::schema::ViewAsset>>;
}

/// 数据库表状态操作 trait
pub trait StatusOperations {
    /// 更新表记录状态
    fn mark_table_status_by_id(
        &self,
        table_name: DatabaseTableName,
        id: i64,
        status: &str,
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
