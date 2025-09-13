use rusqlite::ToSql;
use rusqlite::types::{FromSql, FromSqlResult, ToSqlOutput, ValueRef};
use serde::{Deserialize, Serialize};
/// 数据库表名常量
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatabaseTableName {
    InfoRoot,
    InfoDirectory,
    InfoFile,
    ArchiveMetadata,
    ArchiveAsset,
    MapFileAsset,
    DirectoryTree,
    ViewFile,
    ViewAsset,
}

impl DatabaseTableName {
    /// 获取表名字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            DatabaseTableName::InfoRoot => "info_root",
            DatabaseTableName::InfoDirectory => "info_directory",
            DatabaseTableName::InfoFile => "info_file",
            DatabaseTableName::ArchiveMetadata => "archive_metadata",
            DatabaseTableName::ArchiveAsset => "archive_asset",
            DatabaseTableName::MapFileAsset => "map_file_asset",
            DatabaseTableName::DirectoryTree => "directory_tree",
            DatabaseTableName::ViewFile => "view_file",
            DatabaseTableName::ViewAsset => "view_asset",
        }
    }

    /// 根据字符串查找表名
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "info_root" => Some(DatabaseTableName::InfoRoot),
            "info_directory" => Some(DatabaseTableName::InfoDirectory),
            "info_file" => Some(DatabaseTableName::InfoFile),
            "archive_metadata" => Some(DatabaseTableName::ArchiveMetadata),
            "archive_asset" => Some(DatabaseTableName::ArchiveAsset),
            "map_file_asset" => Some(DatabaseTableName::MapFileAsset),
            "directory_tree" => Some(DatabaseTableName::DirectoryTree),
            "view_file" => Some(DatabaseTableName::ViewFile),
            "view_asset" => Some(DatabaseTableName::ViewAsset),
            _ => None,
        }
    }
}

/// 根目录状态枚举
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RootStatus {
    Health,
    WaitToScan,
    WaitToArchive,
    InScanning,
    InArchiving,
    ErrorScanningFailed,
    ErrorArchivingFailed,
    WaitToDelete,
}

impl RootStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            RootStatus::Health => "Health",
            RootStatus::WaitToScan => "WAIT_TO_SCAN",
            RootStatus::WaitToArchive => "WaitToArchive",
            RootStatus::InScanning => "IN_SCANNING",
            RootStatus::InArchiving => "IN_ARCHIVING",
            RootStatus::ErrorScanningFailed => "ERROR_SCANNING_FAILED",
            RootStatus::ErrorArchivingFailed => "ERROR_ARCHIVING_FAILED",
            RootStatus::WaitToDelete => "WAIT_TO_DELETE",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "Health" => RootStatus::Health,
            "WAIT_TO_SCAN" => RootStatus::WaitToScan,
            "WaitToArchive" => RootStatus::WaitToArchive,
            "IN_SCANNING" => RootStatus::InScanning,
            "IN_ARCHIVING" => RootStatus::InArchiving,
            "ERROR_SCANNING_FAILED" => RootStatus::ErrorScanningFailed,
            "ERROR_ARCHIVING_FAILED" => RootStatus::ErrorArchivingFailed,
            "WAIT_TO_DELETE" => RootStatus::WaitToDelete,
            _ => RootStatus::Health, // 默认值
        }
    }
}

impl FromSql for RootStatus {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let s: String = FromSql::column_result(value)?;
        Ok(RootStatus::from_str(&s))
    }
}

impl ToSql for RootStatus {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(self.as_str()))
    }
}

/// 文件状态枚举
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FileStatus {
    Health,
    WaitToArchive,
    WaitToDelete,
    InScanning,
    InArchiving,
    ErrorScanningFailed,
    ErrorArchivingFailed,
}

impl FileStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            FileStatus::Health => "Health",
            FileStatus::WaitToArchive => "WaitToArchive",
            FileStatus::WaitToDelete => "WAIT_TO_DELETE",
            FileStatus::InScanning => "IN_SCANNING",
            FileStatus::InArchiving => "IN_ARCHIVING",
            FileStatus::ErrorScanningFailed => "ERROR_SCANNING_FAILED",
            FileStatus::ErrorArchivingFailed => "ERROR_ARCHIVING_FAILED",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "Health" => FileStatus::Health,
            "WaitToArchive" => FileStatus::WaitToArchive,
            "WAIT_TO_DELETE" => FileStatus::WaitToDelete,
            "IN_SCANNING" => FileStatus::InScanning,
            "IN_ARCHIVING" => FileStatus::InArchiving,
            "ERROR_SCANNING_FAILED" => FileStatus::ErrorScanningFailed,
            "ERROR_ARCHIVING_FAILED" => FileStatus::ErrorArchivingFailed,
            _ => FileStatus::Health, // 默认值
        }
    }
}

impl FromSql for FileStatus {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let s: String = FromSql::column_result(value)?;
        Ok(FileStatus::from_str(&s))
    }
}

impl ToSql for FileStatus {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(self.as_str()))
    }
}

/// 目录状态枚举
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DirectoryStatus {
    Health,
    WaitToArchive,
    WaitToDelete,
    InScanning,
    InArchiving,
    ErrorScanningFailed,
    ErrorArchivingFailed,
}

impl DirectoryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            DirectoryStatus::Health => "Health",
            DirectoryStatus::WaitToArchive => "WaitToArchive",
            DirectoryStatus::WaitToDelete => "WAIT_TO_DELETE",
            DirectoryStatus::InScanning => "IN_SCANNING",
            DirectoryStatus::InArchiving => "IN_ARCHIVING",
            DirectoryStatus::ErrorScanningFailed => "ERROR_SCANNING_FAILED",
            DirectoryStatus::ErrorArchivingFailed => "ERROR_ARCHIVING_FAILED",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "Health" => DirectoryStatus::Health,
            "WaitToArchive" => DirectoryStatus::WaitToArchive,
            "WAIT_TO_DELETE" => DirectoryStatus::WaitToDelete,
            "IN_SCANNING" => DirectoryStatus::InScanning,
            "IN_ARCHIVING" => DirectoryStatus::InArchiving,
            "ERROR_SCANNING_FAILED" => DirectoryStatus::ErrorScanningFailed,
            "ERROR_ARCHIVING_FAILED" => DirectoryStatus::ErrorArchivingFailed,
            _ => DirectoryStatus::Health, // 默认值
        }
    }
}

impl FromSql for DirectoryStatus {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let s: String = FromSql::column_result(value)?;
        Ok(DirectoryStatus::from_str(&s))
    }
}

impl ToSql for DirectoryStatus {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(self.as_str()))
    }
}

/// 归档状态枚举
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ArchiveStatus {
    Health,
    InArchiving,
}

impl ArchiveStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ArchiveStatus::Health => "Health",
            ArchiveStatus::InArchiving => "InArchiving",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "Health" => ArchiveStatus::Health,
            "InArchiving" => ArchiveStatus::InArchiving,
            _ => ArchiveStatus::Health, // 默认值
        }
    }
}

impl FromSql for ArchiveStatus {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let s: String = FromSql::column_result(value)?;
        Ok(ArchiveStatus::from_str(&s))
    }
}

impl ToSql for ArchiveStatus {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(self.as_str()))
    }
}

/// 资源状态枚举
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AssetStatus {
    Health,
    WaitToArchive,
}

impl AssetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            AssetStatus::Health => "Health",
            AssetStatus::WaitToArchive => "WaitToArchive",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "Health" => AssetStatus::Health,
            "WaitToArchive" => AssetStatus::WaitToArchive,
            _ => AssetStatus::Health, // 默认值
        }
    }
}

impl FromSql for AssetStatus {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let s: String = FromSql::column_result(value)?;
        Ok(AssetStatus::from_str(&s))
    }
}

impl ToSql for AssetStatus {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(self.as_str()))
    }
}

/// 文件资源状态枚举
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MapFileAssetStatus {
    Health,
    WaitToArchive,
}
impl MapFileAssetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            MapFileAssetStatus::Health => "Health",
            MapFileAssetStatus::WaitToArchive => "WaitToArchive",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "Health" => MapFileAssetStatus::Health,
            "WaitToArchive" => MapFileAssetStatus::WaitToArchive,
            _ => MapFileAssetStatus::Health, // 默认值
        }
    }
}
impl FromSql for MapFileAssetStatus {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let s: String = FromSql::column_result(value)?;
        Ok(MapFileAssetStatus::from_str(&s))
    }
}
impl ToSql for MapFileAssetStatus {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(self.as_str()))
    }
}
