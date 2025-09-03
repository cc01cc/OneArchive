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
    HEALTH,
    UPDATING,
    UNARCHIVED,
}

impl RootStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            RootStatus::HEALTH => "HEALTH",
            RootStatus::UPDATING => "UPDATING",
            RootStatus::UNARCHIVED => "UNARCHIVED",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "HEALTH" => RootStatus::HEALTH,
            "UPDATING" => RootStatus::UPDATING,
            "UNARCHIVED" => RootStatus::UNARCHIVED,
            _ => RootStatus::HEALTH, // 默认值
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
    HEALTH,
    UNARCHIVED,
    WAIT_TO_DELETE,
}

impl FileStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            FileStatus::HEALTH => "HEALTH",
            FileStatus::UNARCHIVED => "UNARCHIVED",
            FileStatus::WAIT_TO_DELETE => "WAIT_TO_DELETE",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "HEALTH" => FileStatus::HEALTH,
            "UNARCHIVED" => FileStatus::UNARCHIVED,
            "WAIT_TO_DELETE" => FileStatus::WAIT_TO_DELETE,
            _ => FileStatus::HEALTH, // 默认值
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
    HEALTH,
    UNARCHIVED,
    WAIT_TO_DELETE,
}

impl DirectoryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            DirectoryStatus::HEALTH => "HEALTH",
            DirectoryStatus::UNARCHIVED => "UNARCHIVED",
            DirectoryStatus::WAIT_TO_DELETE => "WAIT_TO_DELETE",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "HEALTH" => DirectoryStatus::HEALTH,
            "UNARCHIVED" => DirectoryStatus::UNARCHIVED,
            "WAIT_TO_DELETE" => DirectoryStatus::WAIT_TO_DELETE,
            _ => DirectoryStatus::HEALTH, // 默认值
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
    HEALTH,
    UNCOMPLETED,
}

impl ArchiveStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ArchiveStatus::HEALTH => "HEALTH",
            ArchiveStatus::UNCOMPLETED => "UNCOMPLETED",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "HEALTH" => ArchiveStatus::HEALTH,
            "UNCOMPLETED" => ArchiveStatus::UNCOMPLETED,
            _ => ArchiveStatus::HEALTH, // 默认值
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
    HEALTH,
    UNARCHIVED,
}

impl AssetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            AssetStatus::HEALTH => "HEALTH",
            AssetStatus::UNARCHIVED => "UNARCHIVED",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "HEALTH" => AssetStatus::HEALTH,
            "UNARCHIVED" => AssetStatus::UNARCHIVED,
            _ => AssetStatus::HEALTH, // 默认值
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

/// 文件卷资源状态枚举
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FileVolumeAssetStatus {
    HEALTH,
    UNARCHIVED,
}

impl FileVolumeAssetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            FileVolumeAssetStatus::HEALTH => "HEALTH",
            FileVolumeAssetStatus::UNARCHIVED => "UNARCHIVED",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "HEALTH" => FileVolumeAssetStatus::HEALTH,
            "UNARCHIVED" => FileVolumeAssetStatus::UNARCHIVED,
            _ => FileVolumeAssetStatus::HEALTH, // 默认值
        }
    }
}

impl FromSql for FileVolumeAssetStatus {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let s: String = FromSql::column_result(value)?;
        Ok(FileVolumeAssetStatus::from_str(&s))
    }
}

impl ToSql for FileVolumeAssetStatus {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(self.as_str()))
    }
}

/// 文件资源状态枚举
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MapFileAssetStatus {
    HEALTH,
    UNARCHIVED,
}
impl MapFileAssetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            MapFileAssetStatus::HEALTH => "HEALTH",
            MapFileAssetStatus::UNARCHIVED => "UNARCHIVED",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "HEALTH" => MapFileAssetStatus::HEALTH,
            "UNARCHIVED" => MapFileAssetStatus::UNARCHIVED,
            _ => MapFileAssetStatus::HEALTH, // 默认值
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
