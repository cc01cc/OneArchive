//! 归档服务的具体实现
//! 实现目录扫描、统计等具体功能

use anyhow::Result as AnyResult;
use anyhow::anyhow;
use log::info;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;
use std::time::SystemTime;
use walkdir::WalkDir;

use crate::mod_database::constants::{DirectoryStatus, FileStatus, RootStatus};

use crate::mod_archive::models::{DirectoryStatistics, ScanProgress};
use crate::mod_archive::traits::{DirectoryScanOperations, DirectoryStatisticsOperations};
use crate::mod_database::schema::{InfoDirectory, InfoFile, InfoRoot};
use crate::mod_database::traits::DirectoryOperations;
use crate::mod_database::traits::FileOperations;
use crate::mod_database::traits::RootOperations;
use crate::mod_database::traits::StatusOperations;

/// 归档服务实现结构体
pub struct ArchiveServices;

impl ArchiveServices {
    /// 创建新的归档服务实例
    pub fn new() -> Self {
        ArchiveServices
    }
}

impl Default for ArchiveServices {
    fn default() -> Self {
        Self::new()
    }
}

/// 计算文件哈希值
fn calculate_file_hash(file_path: &Path) -> AnyResult<String> {
    let mut file = fs::File::open(file_path)?;
    let mut hasher = Sha256::new();
    std::io::copy(&mut file, &mut hasher)?;
    let hash = hasher.finalize();
    Ok(format!("{:x}", hash))
}

impl DirectoryStatisticsOperations for ArchiveServices {
    /// 获取目录统计信息
    fn get_directory_statistics(&self, start_path: &Path) -> AnyResult<DirectoryStatistics> {
        let mut stats = DirectoryStatistics::default();
        let mut max_depth = 0u32;

        for entry in WalkDir::new(start_path).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            let depth = entry.depth() as u32;

            if depth > max_depth {
                max_depth = depth;
            }

            if path.is_dir() {
                stats.directory_count += 1;
            } else if path.is_file() {
                stats.file_count += 1;
                if let Ok(metadata) = fs::metadata(path) {
                    stats.total_size += metadata.len();
                }
            }
        }

        stats.max_depth = max_depth;
        Ok(stats)
    }
}

impl DirectoryScanOperations for ArchiveServices {
    /**
     * 扫描目录并更新数据库
     *
     * 此方法会扫描指定目录，并将扫描结果保存到数据库中。状态管理遵循以下规范：
     * 1. 首次扫描时，新创建的根目录、目录和文件记录状态应为 WaitToArchive
     * 2. 对于已存在的记录，在扫描开始时会先标记为 WaitToDelete
     * 3. 扫描过程中发现的已存在记录会根据情况更新状态：WaitToArchive
     * 4. 扫描结束后，未被更新的记录（仍为 WaitToDelete 状态）表示已删除
     */

    fn scan_and_save_directory_with_events<D, F>(
        &self,
        start_path: &Path,
        database: &D,
        progress_callback: Option<F>,
    ) -> AnyResult<()>
    where
        D: RootOperations + DirectoryOperations + FileOperations + StatusOperations,
        F: Fn(ScanProgress),
    {
        info!("开始扫描目录：{:?}", start_path);

        // 获取目录统计信息
        let stats = self.get_directory_statistics(start_path)?;
        let total_size = stats.total_size;
        let mut processed_size = 0u64;

        // 获取或创建根目录记录
        let root_path_str = start_path
            .to_str()
            .ok_or_else(|| anyhow!("Invalid UTF-8 in path"))?
            .to_string();

        let root_name = start_path
            .file_name()
            .ok_or_else(|| anyhow!("无法获取根目录名称"))?
            .to_str()
            .ok_or_else(|| anyhow!("根目录名称不是有效的 UTF-8"))?
            .to_string();

        // 查找或创建根目录
        let root_id = if let Some(root_info) = database.find_root_info_by_path(&root_path_str)? {
            info!("找到现有根目录记录：{:?}", root_info);

            // 将现有目录和文件标记为待删除
            database.mark_directories_as_wait_to_delete(root_info.id.unwrap())?;
            database.mark_files_as_wait_to_delete(root_info.id.unwrap())?;

            root_info.id.unwrap()
        } else {
            // 创建新的根目录记录
            let root_info = InfoRoot::new(
                root_path_str.clone(),
                root_name.clone(),
                RootStatus::WaitToArchive,
            );
            let root_id = database.add_root_directory(
                &root_info.root_path,
                &root_info.root_name,
                root_info.status.as_str(),
            )?;
            info!("创建新的根目录记录，ID: {}", root_id);

            root_id
        };
        let original_root_status = RootStatus::WaitToArchive;

        // 遍历目录结构
        for entry in walkdir::WalkDir::new(start_path)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();

            if path.is_dir() {
                let relative_path = path.strip_prefix(start_path)?.to_string_lossy().to_string();

                let metadata = fs::metadata(path)?;
                let mtime = metadata
                    .modified()?
                    .duration_since(SystemTime::UNIX_EPOCH)?
                    .as_millis() as i64;
                let dir_name = path.file_name().unwrap().to_string_lossy().to_string();

                let exist_dir = database.find_directory_by_path(root_id, &relative_path)?;
                if let Some(mut tmp_dir) = exist_dir {
                    // 更新现有目录的 mtime 和状态
                    tmp_dir.directory_mtime = mtime;
                    tmp_dir.status = DirectoryStatus::WaitToArchive;
                    database.update_directory(&tmp_dir)?;
                    info!("更新现有目录：{}", relative_path);
                } else {
                    // 创建新的目录记录，首次创建的目录状态应为 WaitToArchive
                    let info_dir = InfoDirectory::new(
                        root_id,
                        dir_name,
                        mtime,
                        Some(relative_path.clone()),
                        DirectoryStatus::WaitToArchive,
                    );
                    let _id = database.insert_directory(&info_dir)?;
                    info!("插入新目录：{}, ID: {}", relative_path, _id);
                }

                // Update progress
                if let Some(ref cb) = progress_callback {
                    let progress = processed_size as f64 / total_size as f64 * 100.0;
                    cb(ScanProgress {
                        processed: processed_size,
                        total: total_size,
                        message: format!("正在扫描目录：{}", relative_path),
                        progress,
                    });
                }
            } else if path.is_file() {
                let parent_path = path.parent().unwrap();
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

                // Find directory ID
                let directory_id = database
                    .find_directory_by_path(root_id, &relative_path)?
                    .ok_or_else(|| anyhow!("文件所属目录不存在：{:?}", relative_path))?
                    .id
                    .unwrap();
                let exist_file = database.find_file_by_name(directory_id, &file_name)?;

                if let Some(mut tmp_file) = exist_file {
                    if tmp_file.file_hash != file_hash {
                        tmp_file.file_hash = file_hash.clone();
                        tmp_file.status = FileStatus::WaitToArchive;
                        database.update_file(&tmp_file)?;
                        info!("文件内容变更，标记为待存档：{}", file_name);
                    } else {
                        // 所有扫描的文件都标记为 WaitToArchive 状态（新设计）
                        tmp_file.status = FileStatus::WaitToArchive;
                        database.update_file(&tmp_file)?;
                        info!("文件未变更，标记为待存档：{}", file_name);
                    }
                } else {
                    // 创建新的文件记录，首次创建的文件状态应为 WaitToArchive
                    let info_file = InfoFile::new(
                        directory_id,
                        file_name.clone(),
                        file_size as i64,
                        mtime,
                        file_hash,
                        FileStatus::WaitToArchive,
                    );

                    // TODO: Check if file already exists in database and update accordingly
                    let _id = database.insert_file(&info_file)?;
                    info!("插入文件：{}, ID: {}", file_name, _id);
                }

                // Update progress
                processed_size += file_size;
                if let Some(ref cb) = progress_callback {
                    let progress = processed_size as f64 / total_size as f64 * 100.0;
                    cb(ScanProgress {
                        processed: processed_size,
                        total: total_size,
                        message: format!("正在扫描文件：{}", file_name),
                        progress,
                    });
                }
            }
        }

        // 更新根目录状态为原始状态
        database.update_root_status(root_id, original_root_status.as_str())?;
        info!("目录扫描完成");
        Ok(())
    }
}
