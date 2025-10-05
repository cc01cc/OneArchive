use std::str::FromStr;

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
    ArchiveChunk,
    MapFileChunk,
    DirectoryTree,
    ViewFile,
    ViewChunk,
}

pub const STATUS_HEALTH: &str = "Health";

impl DatabaseTableName {
    /// 获取表名字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            DatabaseTableName::InfoRoot => "info_root",
            DatabaseTableName::InfoDirectory => "info_directory",
            DatabaseTableName::InfoFile => "info_file",
            DatabaseTableName::ArchiveMetadata => "archive_metadata",
            DatabaseTableName::ArchiveChunk => "archive_chunk",
            DatabaseTableName::MapFileChunk => "map_file_chunk",
            DatabaseTableName::DirectoryTree => "directory_tree",
            DatabaseTableName::ViewFile => "view_file",
            DatabaseTableName::ViewChunk => "view_chunk",
        }
    }

    /// 根据字符串查找表名
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "info_root" => Some(DatabaseTableName::InfoRoot),
            "info_directory" => Some(DatabaseTableName::InfoDirectory),
            "info_file" => Some(DatabaseTableName::InfoFile),
            "archive_metadata" => Some(DatabaseTableName::ArchiveMetadata),
            "archive_chunk" => Some(DatabaseTableName::ArchiveChunk),
            "map_file_chunk" => Some(DatabaseTableName::MapFileChunk),
            "directory_tree" => Some(DatabaseTableName::DirectoryTree),
            "view_file" => Some(DatabaseTableName::ViewFile),
            "view_chunk" => Some(DatabaseTableName::ViewChunk),
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
impl FromStr for RootStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(RootStatus::from_str(s))
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

/// 数据块状态枚举
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ChunkStatus {
    Health,
    WaitToArchive,
}

impl ChunkStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ChunkStatus::Health => "Health",
            ChunkStatus::WaitToArchive => "WaitToArchive",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "Health" => ChunkStatus::Health,
            "WaitToArchive" => ChunkStatus::WaitToArchive,
            _ => ChunkStatus::Health, // 默认值
        }
    }
}

impl FromSql for ChunkStatus {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let s: String = FromSql::column_result(value)?;
        Ok(ChunkStatus::from_str(&s))
    }
}

impl ToSql for ChunkStatus {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(self.as_str()))
    }
}

/// 文件数据块状态枚举
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MapFileChunkStatus {
    Health,
    WaitToArchive,
}
impl MapFileChunkStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            MapFileChunkStatus::Health => "Health",
            MapFileChunkStatus::WaitToArchive => "WaitToArchive",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "Health" => MapFileChunkStatus::Health,
            "WaitToArchive" => MapFileChunkStatus::WaitToArchive,
            _ => MapFileChunkStatus::Health, // 默认值
        }
    }
}
impl FromSql for MapFileChunkStatus {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let s: String = FromSql::column_result(value)?;
        Ok(MapFileChunkStatus::from_str(&s))
    }
}
impl ToSql for MapFileChunkStatus {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(self.as_str()))
    }
}

/// 灾备组状态枚举
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RecoveryGroupStatus {
    Health,
    Recycled,
}

impl RecoveryGroupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            RecoveryGroupStatus::Health => "Health",
            RecoveryGroupStatus::Recycled => "Recycled",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "Health" => RecoveryGroupStatus::Health,
            "Recycled" => RecoveryGroupStatus::Recycled,
            _ => RecoveryGroupStatus::Health, // 默认值
        }
    }
}

impl FromSql for RecoveryGroupStatus {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let s: String = FromSql::column_result(value)?;
        Ok(RecoveryGroupStatus::from_str(&s))
    }
}

impl ToSql for RecoveryGroupStatus {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(self.as_str()))
    }
}

/// 数据分片状态枚举
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RecoveryDataShardStatus {
    Health,
    Recycled,
}

impl RecoveryDataShardStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            RecoveryDataShardStatus::Health => "Health",
            RecoveryDataShardStatus::Recycled => "Recycled",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "Health" => RecoveryDataShardStatus::Health,
            "Recycled" => RecoveryDataShardStatus::Recycled,
            _ => RecoveryDataShardStatus::Health, // 默认值
        }
    }
}

impl FromSql for RecoveryDataShardStatus {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let s: String = FromSql::column_result(value)?;
        Ok(RecoveryDataShardStatus::from_str(&s))
    }
}

impl ToSql for RecoveryDataShardStatus {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(self.as_str()))
    }
}

/// 校验分片状态枚举
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RecoveryParityShardStatus {
    Health,
    Recycled,
}

impl RecoveryParityShardStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            RecoveryParityShardStatus::Health => "Health",
            RecoveryParityShardStatus::Recycled => "Recycled",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "Health" => RecoveryParityShardStatus::Health,
            "Recycled" => RecoveryParityShardStatus::Recycled,
            _ => RecoveryParityShardStatus::Health, // 默认值
        }
    }
}

impl FromSql for RecoveryParityShardStatus {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let s: String = FromSql::column_result(value)?;
        Ok(RecoveryParityShardStatus::from_str(&s))
    }
}

impl ToSql for RecoveryParityShardStatus {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(self.as_str()))
    }
}

/// 灾备组归档文件映射状态枚举
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RecoveryGroupArchiveStatus {
    Health,
    Recycled,
}

impl RecoveryGroupArchiveStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            RecoveryGroupArchiveStatus::Health => "Health",
            RecoveryGroupArchiveStatus::Recycled => "Recycled",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "Health" => RecoveryGroupArchiveStatus::Health,
            "Recycled" => RecoveryGroupArchiveStatus::Recycled,
            _ => RecoveryGroupArchiveStatus::Health, // 默认值
        }
    }
}

impl FromSql for RecoveryGroupArchiveStatus {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let s: String = FromSql::column_result(value)?;
        Ok(RecoveryGroupArchiveStatus::from_str(&s))
    }
}

impl ToSql for RecoveryGroupArchiveStatus {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(self.as_str()))
    }
}

/// 归档文件与数据分片映射状态枚举
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MapArchiveDataShardStatus {
    Health,
    Recycled,
}

impl MapArchiveDataShardStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            MapArchiveDataShardStatus::Health => "Health",
            MapArchiveDataShardStatus::Recycled => "Recycled",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "Health" => MapArchiveDataShardStatus::Health,
            "Recycled" => MapArchiveDataShardStatus::Recycled,
            _ => MapArchiveDataShardStatus::Health, // 默认值
        }
    }
}

impl FromSql for MapArchiveDataShardStatus {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let s: String = FromSql::column_result(value)?;
        Ok(MapArchiveDataShardStatus::from_str(&s))
    }
}

impl ToSql for MapArchiveDataShardStatus {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(self.as_str()))
    }
}
