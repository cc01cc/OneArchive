//! 解档服务实现

use anyhow::Result as AnyResult;
use anyhow::anyhow;
use log::debug;
use log::info;
use log::warn;
use std::fs;
use std::io::Write;
use std::path::Path;

// 添加 tar 相关依赖
use std::fs::File;
use std::io::BufReader;
use tar::{Archive as TarArchive, EntryType};

use crate::mod_database::constants::AssetStatus;
use crate::mod_database::constants::DirectoryStatus;
use crate::mod_database::constants::FileStatus;
use crate::mod_database::trait_database::DirectoryOperations;
use crate::mod_database::trait_database::{
    ArchiveAssetOperations, ArchiveMetadataOperations, MapFileAssetOperations, ViewOperations,
};
use crate::mod_extract::model_extract::{ExtractProgress, ExtractTask};
use crate::mod_extract::trait_extract::ExtractOperations;

/// 解档服务实现
pub struct ExtractService;

impl ExtractService {
    pub fn new() -> Self {
        Self
    }

    /// 从存档中提取单个资产
    fn extract_single_asset(
        &self,
        archive_directory: &str,
        archive_name: &str,
        asset_relative_path: &str,
        target_file_path: &Path,
    ) -> AnyResult<()> {
        // 确保目标文件的目录存在
        if let Some(parent) = target_file_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
                warn!("创建了新的文件目录: {}", parent.display());
            }
        }

        let archive_path = Path::new(archive_directory).join(archive_name);
        let archive_file = File::open(&archive_path)?;
        let mut archive = TarArchive::new(BufReader::new(archive_file));

        // 遍历 tar 中的条目
        for entry in archive.entries()? {
            let mut entry = entry?;
            // 检查条目是否是我们要找的资产
            if entry.path()?.to_str() == Some(asset_relative_path) {
                // 创建目标文件
                let mut target_file = File::create(target_file_path)?;

                // 复制数据
                std::io::copy(&mut entry, &mut target_file)?;
                break;
            }
        }

        Ok(())
    }
  /// 从多个分卷存档中提取并合并资产
    fn extract_volume_assets(
        &self,
        archive_directory: &str,
        archive_assets: &[(String, String)], // Vec of (archive_name, asset_relative_path)
        target_file_path: &Path,
    ) -> AnyResult<()> {
        // 确保目标文件的目录存在
        if let Some(parent) = target_file_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
                warn!("创建了新的文件目录: {}", parent.display());
            }
        }

        // 创建目标文件
        let mut target_file = File::create(target_file_path)?;

        // 按顺序处理每个分卷
        for (archive_name, asset_relative_path) in archive_assets {
            let archive_path = Path::new(archive_directory).join(archive_name);
            let archive_file = File::open(&archive_path)?;
            let mut archive = TarArchive::new(BufReader::new(archive_file));

            // 查找对应的资产并追加到目标文件
            for entry in archive.entries()? {
                let mut entry = entry?;
                if entry.path()?.to_str() == Some(asset_relative_path) {
                    std::io::copy(&mut entry, &mut target_file)?;
                    break;
                }
            }
        }

        Ok(())
    }
}

impl ExtractOperations for ExtractService {
    fn extract_archive<D>(&self, task: &ExtractTask, database: &D) -> AnyResult<ExtractProgress>
    where
        D: ArchiveAssetOperations
        +DirectoryOperations
            + MapFileAssetOperations
            + ArchiveMetadataOperations
            + ViewOperations,
    {
        info!(
            "开始解档任务：root_id={}, target_path={}",
            task.root_id, task.target_path
        );

        // 检查目标路径是否存在，如果不存在则创建
        let target_path = Path::new(&task.target_path);
        if !target_path.exists() {
            fs::create_dir_all(target_path)?;
        }

        // 检查目标路径是否为目录
        if !target_path.is_dir() {
            return Err(anyhow!("目标路径不是目录：{}", task.target_path));
        }

        // 检查目标路径是否为空目录(存档目录可以是非空, 但是解档的目标目录暂不支持非空)
        if target_path.read_dir()?.next().is_some() {
            let first_file_name = target_path
                .read_dir()?
                .next()
                .and_then(|entry| entry.ok())
                .and_then(|entry| entry.file_name().into_string().ok())
                .unwrap_or_else(|| "Unknown file".to_string());
            return Err(anyhow!(
                "目标路径为非空目录：{}\n{}",
                task.target_path,
                first_file_name
            ));
        }

        // 需要先处理每个 directory
        let dir_list = database.find_directories_by_status_and_root_id(task.root_id, Some(DirectoryStatus::Health))?;
        for directory in dir_list {
            // 构造目标目录路径
            let directory_path = target_path
                .join(directory.directory_path.as_deref().unwrap_or(""));
            debug!("正在处理目录：{}", directory_path.display());

            // 创建目标目录
            if !directory_path.exists() {
                fs::create_dir_all(&directory_path)?;
            }
        }

        // 获取指定存档的文件列表
        // TODO 这里只获取了状态为 Health 的 file 需要考虑其他状态的 file 如何处理
        let file_list = database
            .find_view_files_by_root_id_and_status(task.root_id, Some(FileStatus::Health))?;

        if file_list.is_empty() {
            return Err(anyhow!("未找到任何文件：{}", task.root_id));
        }

        let mut progress = ExtractProgress {
            total_files: file_list.len(),
            processed_files: 0,
            current_file: None,
            completed: false,
            error: None,
        };

        // 处理每个文件
        for file in file_list {
            // 构造目标文件路径
            // let file_name = file.file_name.clone();
            let file_path = target_path
                .join(file.directory_path.as_deref().unwrap_or(""))
                .join(&file.file_name);

            // 检查是否需要覆盖已存在的文件
            // TODO 此处保留，以后增量解档的时候可以用
            if file_path.exists() && !task.overwrite {
                warn!("文件已存在：{}", file_path.display());
                // 跳过已存在的文件
                progress.processed_files += 1;
                continue;
            }

            let asset_list = database.find_view_assets_by_file_id(file.file_id)?;

            // 根据资产数量决定处理方式
            match asset_list.len() {
                0 => {
                    // 没有资产，记录错误但继续处理
                    progress.error = Some(format!("文件 ID {} 没有关联的资产", file.file_id));
                }
                1 => {
                    if AssetStatus::from_str(&asset_list[0].asset_status) != AssetStatus::Health {
                        progress.error =
                            Some(format!("资产状态不健康：{}", asset_list[0].asset_status));
                    }

                    // 单个资产，直接提取
                    if let Err(e) = self.extract_single_asset(
                        &task.archive_directory,
                        &asset_list[0].archive_name,
                        &asset_list[0].asset_relative_path,
                        &file_path,
                    ) {
                        progress.error = Some(format!("提取文件失败：{}", e));
                    }
                }
                _ => {
                    // 多个资产，需要合并分卷
                    let asset_info: Vec<(String, String)> = asset_list
                        .iter()
                        .map(|asset| {
                            (
                                asset.archive_name.clone(),
                                asset.asset_relative_path.clone(),
                            )
                        })
                        .collect();

                    if let Err(e) =
                        self.extract_volume_assets(&task.archive_directory, &asset_info, &file_path)
                    {
                        progress.error = Some(format!("提取分卷文件失败：{}", e));
                    }
                }
            }

            progress.processed_files += 1;
            debug!("已解档文件：{}", file_path.display());
        }

        progress.completed = true;
        info!("解档任务完成：root_id={}", task.root_id);

        Ok(progress)
    }
}
