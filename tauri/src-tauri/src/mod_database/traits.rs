//! 数据库操作 trait 定义
//! 定义各种数据库操作的接口

use super::schema::{InfoRoot, InfoDirectory, InfoFile, ViewFile};
use super::constants::{DatabaseTableName, DirectoryStatus, FileStatus};
use rusqlite::{Connection, Result as SqliteResult};

/// 数据库根目录操作 trait
pub trait RootOperations {
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
    ) -> SqliteResult<i64>;

    /// 根据路径查找根目录信息
    ///
    /// # 参数
    /// * `root_path` - 根目录路径
    ///
    /// # 返回值
    /// 返回根目录信息
    fn find_root_info_by_path(
        &self,
        root_path: &str,
    ) -> SqliteResult<Option<InfoRoot>>;

    /// 更新根目录状态
    ///
    /// # 参数
    /// * `root_id` - 根目录 ID
    /// * `status` - 新状态
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
    /// * `directory` - 目录信息
    ///
    /// # 返回值
    /// 返回插入记录的 ID
    fn insert_directory(&self, directory: &InfoDirectory) -> SqliteResult<i64>;

    /// 更新目录信息
    ///
    /// # 参数
    /// * `directory` - 目录信息
    ///
    /// # 返回值
    /// 返回操作结果
    fn update_directory(&self, directory: &InfoDirectory) -> SqliteResult<()>;

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
    fn find_file_by_name(
        &self,
        directory_id: i64,
        name: &str,
    ) -> SqliteResult<Option<InfoFile>>;

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

/// 数据库视图操作 trait
pub trait ViewOperations {
    /// 根据根目录 ID 查找所有视图文件
    fn find_view_files_by_root_id(
        &self,
        root_id: i64,
    ) -> SqliteResult<Vec<ViewFile>>;

    /// 根据目录 ID 查找视图文件
    fn find_view_files_by_directory_id(
        &self,
        directory_id: i64,
    ) -> SqliteResult<Vec<ViewFile>>;
}

/// 数据库表状态操作 trait
pub trait StatusOperations {
    /// 更新表记录状态
    fn mark_table_status(
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
