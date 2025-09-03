//! Directory scanning service

use log::{debug, error, info};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::database::Database;
use crate::database::constants::{
    self as status, ArchiveStatus, AssetStatus, DirectoryStatus, FileStatus, MapFileAssetStatus,
    RootStatus,
};

use crate::database::schema::{InfoDirectory, InfoFile, InfoRoot};

/// Directory statistics structure
#[derive(Debug)]
pub struct DirectoryStatistics {
    pub total_size: u64,
    pub directory_count: u32,
    pub file_count: u32,
    pub max_depth: u32,
}

impl Default for DirectoryStatistics {
    fn default() -> Self {
        Self {
            total_size: 0,
            directory_count: 0,
            file_count: 0,
            max_depth: 0,
        }
    }
}

/// Progress callback trait
pub trait ProgressCallback {
    fn update_progress(&self, processed: u64, total: u64, message: &str);
}

/* /// Dummy implementation of ProgressCallback
pub struct DummyProgressCallback;

impl ProgressCallback for DummyProgressCallback {
    fn update_progress(&self, _processed: u64, _total: u64, _message: &str) {
        // No-op
    }
}
 */
/// Scan directory and collect statistics without writing to database
pub fn scan_directory_only(start_path: &Path) -> Result<DirectoryStatistics, std::io::Error> {
    let mut stats = DirectoryStatistics::default();

    for entry in walkdir::WalkDir::new(start_path) {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            stats.file_count += 1;
            if let Ok(metadata) = fs::metadata(path) {
                stats.total_size += metadata.len();
            }
        } else if path.is_dir() && path != start_path {
            stats.directory_count += 1;

            let depth = path.components().count() - start_path.components().count();
            if depth as u32 > stats.max_depth {
                stats.max_depth = depth as u32;
            }
        }
    }

    Ok(stats)
}

/// Calculate SHA256 hash of a file
fn calculate_file_hash(file_path: &Path) -> Result<String, std::io::Error> {
    let mut file = fs::File::open(file_path)?;
    let mut hasher = Sha256::new();
    std::io::copy(&mut file, &mut hasher)?;
    let hash = hasher.finalize();
    Ok(format!("{:x}", hash))
}

/// Progress information structure for event-based communication
#[derive(serde::Serialize, Clone)]
pub struct ScanProgress {
    pub processed: u64,
    pub total: u64,
    pub message: String,
    pub progress: f64,
}

/// Scan directory and save to database
pub fn scan_and_save_directory(
    start_path: &Path,
    database: &Database,
) -> Result<(), Box<dyn std::error::Error>> {
    scan_and_save_directory_with_events(start_path, database, None::<fn(ScanProgress)>)
}

/**
  扫描目录并保存到数据库，支持进度回调
  (要求扫描时目录保持静态,不允许有文件变动)
*/
pub fn scan_and_save_directory_with_events<F>(
    start_path: &Path,
    database: &Database,
    progress_callback: Option<F>,
) -> Result<(), Box<dyn std::error::Error>>
where
    F: Fn(ScanProgress),
{
    // Get absolute path
    let root_absolute_path = start_path.canonicalize()?.to_string_lossy().to_string();

    // Find or create root directory
    let exist_info = match database.find_root_info_by_path(&root_absolute_path)? {
        Some(info_root) => {
            let root_id = info_root.id.expect("数据库中的根目录记录应该有有效的ID");

            info!("开始更新根目录 {}", root_absolute_path);
            database.update_root_status(root_id, RootStatus::UPDATING.as_str())?;
            // 获取现有的目录和文件列表(需要在状态变更前获取, 后续需要还原状态)
            let existing_directories =
                database.find_directories_by_status_and_root_id(root_id, None)?;
            let existing_files = database.find_files_by_status_and_root_id(root_id, None)?;

            info!("将根目录 {} 下的所有目录标记为待删除", root_absolute_path);
            database.mark_directories_as_wait_to_delete(root_id)?;

            info!("将根目录 {} 下的所有文件标记为待删除", root_absolute_path);
            database.mark_files_as_wait_to_delete(root_id)?;

            (root_id, Some(existing_directories), Some(existing_files))
        }
        None => {
            info!("根目录{}不存在，正在创建...", root_absolute_path);
            (
                database.add_root_directory(&root_absolute_path, &"A")?,
                None,
                None,
            )
        }
    };

    // Collect statistics for progress tracking
    let stats = scan_directory_only(start_path)?;
    let total_size = stats.total_size;
    let mut processed_size = 0u64;

    if let Some(ref cb) = progress_callback {
        cb(ScanProgress {
            processed: 0,
            total: total_size,
            message: format!("开始扫描目录: {}", start_path.display()),
            progress: 0.0,
        });
    } else {
        info!("开始扫描目录: {}", start_path.display());
    }

    // Walk the directory tree
    for entry in walkdir::WalkDir::new(start_path) {
        let entry = entry?;
        let path = entry.path();
        debug!("Visiting: {}", path.display());

        if path.is_dir() {
            let relative_path = path.strip_prefix(start_path)?.to_string_lossy().to_string();

            let metadata = fs::metadata(path)?;
            let mtime = metadata
                .modified()?
                .duration_since(SystemTime::UNIX_EPOCH)?
                .as_millis() as i64;

            let directory_name = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            // Try to find existing directory
            if let Some(ref existing_directories) = exist_info.1 {
                // 从 existing_directories 中查找匹配的目录
                if let Some(mut exist_directory) = existing_directories
                    .iter()
                    .find(|d| d.directory_path.as_ref() == Some(&relative_path))
                    .cloned()
                {
                    // Update existing directory with its original status
                    database.update_directory(&exist_directory)?;
                    info!("更新已存在的目录: {}", relative_path);
                } else {
                    let info_directory = InfoDirectory::new(
                        exist_info.0,
                        directory_name,
                        mtime,
                        Some(relative_path.clone()),
                        DirectoryStatus::HEALTH,
                    );
                    // Insert new directory
                    let _id = database.insert_directory(&info_directory)?;
                    info!("插入新目录: {}, ID: {}", relative_path, _id);
                }
            } else {
                // 如果没有 existing_directories（新根目录），则直接插入
                let info_directory = InfoDirectory::new(
                    exist_info.0,
                    directory_name,
                    mtime,
                    Some(relative_path.clone()),
                    DirectoryStatus::HEALTH,
                );
                // Insert new directory
                let _id = database.insert_directory(&info_directory)?;
                info!("插入新目录: {}, ID: {}", relative_path, _id);
            }
        } else if path.is_file() {
            // 1. dir + file name 不存在 => 新文件 => 插入
            // 2. dir + file name 存在 => 已存在的文件 => hash 不一致 => 重新存档 UNARCHIVED
            // 3. dir + file name 存在 => 已存在的文件 => hash 一致 => 恢复原来的状态
            let parent_path = path.parent().unwrap_or(start_path);
            let relative_path = parent_path
                .strip_prefix(start_path)?
                .to_string_lossy()
                .to_string();

            let metadata = fs::metadata(path)?;
            let file_size = metadata.len();
            let mtime = metadata
                .modified()?
                .duration_since(SystemTime::UNIX_EPOCH)?
                .as_millis() as i64;
            let file_name = path.file_name().unwrap().to_string_lossy().to_string();

            // Calculate file hash
            let file_hash = calculate_file_hash(path).ok();

            // Find directory
            let directory_for_file = database
                .find_directory_by_path(exist_info.0, &relative_path)?
                .expect(&format!("文件所属目录不存在: {:?}", relative_path));

            let mut file_inserted = false;
            if let Some(ref existing_files) = exist_info.2 {
                // 从 existing_files 中查找匹配的文件
                if let Some(mut exist_file) = existing_files
                    .iter()
                    .find(|f| {
                        f.directory_id == directory_for_file.id.unwrap() && f.file_name == file_name
                    })
                    .cloned()
                {
                    // Update existing file with its original status
                    if exist_file.file_hash != file_hash {
                        exist_file.file_hash = file_hash.clone();
                        exist_file.status = FileStatus::UNARCHIVED;
                        database.update_file(&exist_file)?;
                        info!("文件内容变更, 标记为待存档: {}", file_name);
                    } else {
                        // 恢复原来的状态
                        database.update_file(&exist_file)?;
                        info!("文件未变更, 恢复原状态: {}", file_name);
                    }
                    file_inserted = true;
                }
            }

            // 如果没有existing_files或者没有找到匹配的文件，则插入新文件
            if !file_inserted {
                let info_file = InfoFile::new(
                    directory_for_file.id.unwrap(),
                    file_name.clone(),
                    file_size as i64,
                    mtime,
                    file_hash,
                    FileStatus::UNARCHIVED,
                );

                // TODO: Check if file already exists in database and update accordingly
                let _id = database.insert_file(&info_file)?;
                info!("插入文件: {}, ID: {}", file_name, _id);
            }

            // Update progress
            processed_size += file_size;
            if let Some(ref cb) = progress_callback {
                let progress = (processed_size as f64 / total_size as f64 * 100.0);
                cb(ScanProgress {
                    processed: processed_size,
                    total: total_size,
                    message: format!("正在扫描文件: {}", file_name),
                    progress,
                });
            } else {
                info!("正在扫描文件: {}", file_name);
            }
        }
    }

    if let Some(ref cb) = progress_callback {
        cb(ScanProgress {
            processed: total_size,
            total: total_size,
            message: "目录扫描完成".to_string(),
            progress: 100.0,
        });
    } else {
        info!("目录扫描完成");
    }

    Ok(())
}
