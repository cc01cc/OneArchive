//! 数据库操作 trait 定义
//! 定义各种数据库操作的接口
//! 如果 传入参数超过 2 个, 则使用结构化参数 params

use crate::mod_database::schema::CreateArchiveMetadataParams;

use super::constants::{DatabaseTableName, DirectoryStatus, FileStatus};
use super::schema::{InfoDirectory, InfoFile, InfoRoot, ViewFile};
use rusqlite::{Connection, Result as SqliteResult};

/// 数据库根目录操作 trait
pub trait RootOperations {
    /// 添加根目录信息
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
    /// # 返回值
    /// 返回插入记录的 ID
    fn insert_directory(&self, directory: &InfoDirectory) -> SqliteResult<i64>;

    /// 更新目录信息
    ///
    /// # 返回值
    /// 返回操作结果
    fn update_directory(&self, directory: &InfoDirectory) -> SqliteResult<()>;

    /// 根据路径查找目录 ID
    ///
    /// # 返回值
    /// 返回目录 ID
    fn find_directory_by_path(
        &self,
        root_id: i64,
        path: &str,
    ) -> SqliteResult<Option<InfoDirectory>>;
    /// 根据 ID 查找目录
    ///
    /// # 参数
    ///
    /// # 返回值
    /// 返回目录信息
    fn find_directory_by_id(&self, id: i64) -> SqliteResult<Option<InfoDirectory>>;
    /// 根据状态和根目录 ID 查找目录
    fn find_directories_by_status_and_root_id(
        &self,
        root_id: i64,
        status: Option<DirectoryStatus>,
    ) -> SqliteResult<Vec<InfoDirectory>>;

    /// 查找某个目录的直接父目录
    fn find_parent_directory(
        &self,
        root_id: i64,
        directory_path: &str,
    ) -> SqliteResult<Option<InfoDirectory>>;

    /// 查找某个目录下的所有直接子目录
    fn find_child_directories(
        &self,
        root_id: i64,
        parent_path: &str,
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


/// 数据库归档数据块操作 trait
pub trait ArchiveChunkOperations {
    /// 插入归档数据块
    ///
    /// # 参数
    /// * `chunk` - 归档数据块
    ///
    /// # 返回值
    /// 返回插入记录的 ID
    fn insert_archive_chunk(&self, chunk: &super::schema::ArchiveChunk) -> SqliteResult<i64>;

    /// 更新归档数据块
    ///
    /// # 参数
    /// * `chunk` - 归档数据块
    ///
    /// # 返回值
    /// 返回操作结果
    fn update_archive_chunk(&self, chunk: &super::schema::ArchiveChunk) -> SqliteResult<()>;

    /// 根据 ID 查找归档数据块
    ///
    /// # 参数
    ///
    /// # 返回值
    /// 返回归档数据块
    fn find_archive_chunk_by_id(
        &self,
        id: i64,
    ) -> SqliteResult<Option<super::schema::ArchiveChunk>>;

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
    ) -> SqliteResult<Vec<super::schema::ArchiveChunk>>;

    /// 根据状态查找归档数据块
    ///
    /// # 参数
    ///
    /// # 返回值
    /// 返回归档数据块列表
    fn find_archive_chunks_by_status(
        &self,
        status: Option<super::constants::ChunkStatus>,
    ) -> SqliteResult<Vec<super::schema::ArchiveChunk>>;

    /// 根据 chunk hash 查找归档数据块
    fn find_archive_chunk_by_chunk_hash(
        &self,
        chunk_hash: &str,
    ) -> SqliteResult<Option<super::schema::ArchiveChunk>>;
}

/// 数据库文件与归档数据块映射操作 trait
pub trait MapFileChunkOperations {
    /// 插入文件与归档数据块映射
    ///
    /// # 返回值
    /// 返回插入记录的 ID
    fn insert_map_file_chunk(&self, map: &super::schema::MapFileChunk) -> SqliteResult<i64>;

    /// 更新文件与归档数据块映射
    ///
    /// # 返回值
    /// 返回操作结果
    fn update_map_file_chunk(&self, map: &super::schema::MapFileChunk) -> SqliteResult<()>;

    /// 根据 ID 查找文件与归档数据块映射
    ///
    /// # 参数
    ///
    /// # 返回值
    /// 返回映射信息
    fn find_map_file_chunk_by_id(
        &self,
        id: i64,
    ) -> SqliteResult<Option<super::schema::MapFileChunk>>;

    /// 根据文件 ID 查找文件与归档数据块映射
    ///
    /// # 参数
    /// * `file_id` - 文件 ID
    ///
    /// # 返回值
    /// 返回映射信息列表
    fn find_map_file_chunk_by_file_id(
        &self,
        file_id: i64,
    ) -> SqliteResult<Vec<super::schema::MapFileChunk>>;

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
    ) -> SqliteResult<Vec<super::schema::MapFileChunk>>;

    /// 根据数据块 ID 查找文件与归档数据块映射
    ///
    /// # 参数
    /// * `chunk_id` - 数据块 ID
    ///
    /// # 返回值
    /// 返回映射信息列表
    fn find_map_file_chunk_by_chunk_id(
        &self,
        chunk_id: i64,
    ) -> SqliteResult<Vec<super::schema::MapFileChunk>>;

    /// 根据状态查找文件与归档数据块映射
    ///
    /// # 参数
    ///
    /// # 返回值
    /// 返回映射信息列表
    fn find_map_file_chunk_by_status(
        &self,
        status: Option<super::constants::MapFileChunkStatus>,
    ) -> SqliteResult<Vec<super::schema::MapFileChunk>>;
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

    /// 根据归档 ID 查找视图数据块
    fn find_view_chunks_by_archive_id(
        &self,
        archive_id: i64,
    ) -> SqliteResult<Vec<super::schema::ViewChunk>>;

    /// 根据文件 ID 查找视图数据块
    fn find_view_chunks_by_file_id(
        &self,
        file_id: i64,
    ) -> SqliteResult<Vec<super::schema::ViewChunk>>;
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
