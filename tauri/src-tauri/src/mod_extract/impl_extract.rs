//! 解档服务实现
use anyhow::Result as AnyResult;
use anyhow::anyhow;
use log::debug;
use log::info;
use log::warn;
use std::fs;
use std::path::Path;

use std::fs::File;
use std::io::BufReader;
use tar::Archive as TarArchive;

use crate::mod_database::constants::ChunkStatus;
use crate::mod_database::constants::DirectoryStatus;
use crate::mod_database::constants::FileStatus;
use crate::mod_database::trait_database::DirectoryOperations;
use crate::mod_database::trait_database::{
    ArchiveChunkOperations, ArchiveMetadataOperations, MapFileChunkOperations, ViewOperations,
};
use crate::mod_extract::model_extract::{ExtractProgress, ExtractTask};
use crate::mod_extract::trait_extract::ExtractOperations;

/// 归档文件提取器
/// 用于从归档中提取单个文件或分卷文件
struct ArchiveExtractor;

impl ArchiveExtractor {
    /// 从归档中提取单个数据块
    fn extract_single_chunk(
        archive_uri: &str,
        chunk_relative_path: &str,
        target_file_path: &Path,
    ) -> AnyResult<()> {
        // 确保目标文件的目录存在
        if let Some(parent) = target_file_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
                warn!("创建了新的文件目录：{}", parent.display());
            }
        }

        let archive_path = Path::new(archive_uri);
        let archive_file = File::open(&archive_path)?;
        let mut archive = TarArchive::new(BufReader::new(archive_file));

        for entry in archive.entries_with_seek()? {
            let mut entry = entry?;
            // 检查条目是否是我们要找的数据块
            if entry.path()?.to_str() == Some(chunk_relative_path) {
                // 创建目标文件
                let mut target_file = File::create(target_file_path)?;

                std::io::copy(&mut entry, &mut target_file)?;
                break;
            }
        }

        Ok(())
    }

    /// 从多个分卷归档中提取并合并数据块
    fn extract_volume_chunks(
        archive_chunks: &[(String, String)], // Vec of (archive_uri, chunk_relative_path)
        target_file_path: &Path,
    ) -> AnyResult<()> {
        // 确保目标文件的目录存在
        if let Some(parent) = target_file_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
                warn!("创建了新的文件目录：{}", parent.display());
            }
        }

        // 验证所有归档文件都存在
        for (archive_uri, _) in archive_chunks {
            let archive_path = Path::new(archive_uri);
            if !archive_path.exists() {
                return Err(anyhow!("归档文件不存在：{}", archive_uri));
            }
        }

        // 创建目标文件
        let mut target_file = File::create(target_file_path)?;

        // 按顺序处理每个分卷
        for (archive_uri, chunk_relative_path) in archive_chunks {
            let archive_path = Path::new(archive_uri);
            let archive_file = File::open(&archive_path)?;
            let mut archive = TarArchive::new(BufReader::new(archive_file));

            // 查找对应的数据块并追加到目标文件
            for entry in archive.entries_with_seek()? {
                let mut entry = entry?;
                if entry.path()?.to_str() == Some(chunk_relative_path) {
                    std::io::copy(&mut entry, &mut target_file)?;
                    break;
                }
            }

            info!("已处理分卷文件：{}", archive_uri);
        }

        Ok(())
    }
}

pub struct ExtractService;

impl ExtractOperations for ExtractService {
    fn extract_archive<D, F>(
        &mut self,
        task: &ExtractTask,
        database: &D,
        progress_callback: Option<F>,
    ) -> AnyResult<ExtractProgress>
    where
        D: ArchiveChunkOperations
            + DirectoryOperations
            + MapFileChunkOperations
            + ArchiveMetadataOperations
            + ViewOperations,
        F: Fn(ExtractProgress),
    {
        info!(
            "开始解档任务：root_id={}, target_path={}",
            task.root_id, task.target_path
        );

        // 验证和准备目标路径
        let target_path = Self::prepare_target_path(task)?;

        // 创建目录结构
        Self::create_directory_structure(task, database, &target_path, &progress_callback)?;

        // 获取需要解档的文件列表
        let file_list = Self::get_files_to_extract(task, database)?;

        // 初始化进度信息
        let mut progress = ExtractProgress {
            total_files: file_list.len(),
            processed_files: 0,
            current_file: None,
            completed: false,
            error: None,
        };

        // 发送初始进度
        Self::update_progress(&progress_callback, &progress);

        // 处理每个文件
        Self::process_files(
            task,
            database,
            &file_list,
            &target_path,
            &mut progress,
            &progress_callback,
        )?;

        progress.completed = true;
        Self::update_progress(&progress_callback, &progress);

        info!("解档任务完成：root_id={}", task.root_id);
        info!("总共处理了 {} 个文件", progress.processed_files);

        Ok(progress)
    }
}

impl ExtractService {
    pub fn new() -> Self {
        Self
    }

    /// 更新进度并调用回调函数
    fn update_progress<F>(progress_callback: &Option<F>, progress: &ExtractProgress)
    where
        F: Fn(ExtractProgress),
    {
        if let Some(cb) = progress_callback {
            cb(progress.clone());
        }
    }

    /// 验证并准备目标路径
    fn prepare_target_path(task: &ExtractTask) -> AnyResult<std::path::PathBuf> {
        let target_path = Path::new(&task.target_path);

        // 检查目标路径是否存在，如果不存在则创建
        if !target_path.exists() {
            fs::create_dir_all(target_path)?;
        }

        // 检查目标路径是否为目录
        if !target_path.is_dir() {
            return Err(anyhow!("目标路径不是目录：{}", task.target_path));
        }

        Ok(target_path.to_path_buf())
    }

    /// 创建目录结构
    fn create_directory_structure<D, F>(
        task: &ExtractTask,
        database: &D,
        target_path: &Path,
        progress_callback: &Option<F>,
    ) -> AnyResult<()>
    where
        D: DirectoryOperations,
        F: Fn(ExtractProgress),
    {
        let dir_list = database
            .find_directories_by_status_and_root_id(task.root_id, Some(DirectoryStatus::Health))?;

        info!("正在创建 {} 个目录结构", dir_list.len());
        for (index, directory) in dir_list.iter().enumerate() {
            // 构造目标目录路径
            let directory_path =
                target_path.join(directory.directory_path.as_deref().unwrap_or(""));
            debug!("正在处理目录：{}", directory_path.display());

            // 创建目标目录
            if !directory_path.exists() {
                fs::create_dir_all(&directory_path)?;
            }

            // 更新进度
            if let Some(cb) = progress_callback {
                cb(ExtractProgress {
                    total_files: dir_list.len(),
                    processed_files: index + 1,
                    current_file: Some(format!("创建目录：{}", directory_path.display())),
                    completed: false,
                    error: None,
                });
            }
        }
        info!("已完成目录结构创建");

        Ok(())
    }

    /// 获取需要解档的文件列表
    fn get_files_to_extract<D>(
        task: &ExtractTask,
        database: &D,
    ) -> AnyResult<Vec<crate::mod_database::schema::ViewFile>>
    where
        D: ViewOperations,
    {
        // 获取指定归档的文件列表
        // TODO 这里只获取了状态为 Health 的 file 需要考虑其他状态的 file 如何处理
        let file_list = database
            .find_view_files_by_root_id_and_status(task.root_id, Some(FileStatus::Health))?;

        if file_list.is_empty() {
            return Err(anyhow!("未找到任何文件：{}", task.root_id));
        }

        info!("找到 {} 个文件需要解档", file_list.len());
        Ok(file_list)
    }

    /// 处理文件列表
    fn process_files<D, F>(
        task: &ExtractTask,
        database: &D,
        file_list: &[crate::mod_database::schema::ViewFile],
        target_path: &Path,
        progress: &mut ExtractProgress,
        progress_callback: &Option<F>,
    ) -> AnyResult<()>
    where
        D: MapFileChunkOperations + ViewOperations,
        F: Fn(ExtractProgress),
    {
        // 处理每个文件
        for (index, file) in file_list.iter().enumerate() {
            info!(
                "正在处理文件 {}/{}: {}",
                index + 1,
                file_list.len(),
                file.file_name
            );

            // 更新当前文件信息
            progress.current_file = Some(file.file_name.clone());
            Self::update_progress(progress_callback, progress);

            Self::process_single_file(
                task,
                database,
                file,
                target_path,
                progress,
                progress_callback,
            )?;
            progress.processed_files += 1;

            info!(
                "已完成 {}/{} 个文件的解档",
                progress.processed_files, progress.total_files
            );
            Self::update_progress(progress_callback, progress);
        }

        Ok(())
    }

    /// 处理单个文件
    fn process_single_file<D, F>(
        task: &ExtractTask,
        database: &D,
        file: &crate::mod_database::schema::ViewFile,
        target_path: &Path,
        progress: &mut ExtractProgress,
        progress_callback: &Option<F>,
    ) -> AnyResult<()>
    where
        D: MapFileChunkOperations + ViewOperations,
        F: Fn(ExtractProgress),
    {
        // 构造目标文件路径
        let file_path = target_path
            .join(file.directory_path.as_deref().unwrap_or(""))
            .join(&file.file_name);

        // 检查是否需要覆盖已存在的文件
        if file_path.exists() && !task.overwrite {
            warn!("文件已存在，跳过处理：{}", file_path.display());
            // 即使跳过文件，也需要更新进度
            Self::update_progress(progress_callback, progress);
            return Ok(());
        }

        let chunk_list = database.find_view_chunks_by_file_id(file.file_id)?;

        // 根据数据块数量决定处理方式
        match chunk_list.len() {
            0 => {
                // 没有数据块，记录错误但继续处理
                progress.error = Some(format!("文件 ID {} 没有关联的数据块", file.file_id));
                warn!("{}", progress.error.as_ref().unwrap());
                Self::update_progress(progress_callback, progress);
            }
            1 => {
                info!("正在提取单个数据块文件：{}", file.file_name);
                Self::process_single_chunk(
                    chunk_list[0].clone(),
                    &file_path,
                    progress,
                    progress_callback,
                )?;
            }
            _ => {
                info!(
                    "正在提取分卷数据块文件：{} (共{}个分卷)",
                    file.file_name,
                    chunk_list.len()
                );
                Self::process_volume_chunks(&chunk_list, &file_path, progress, progress_callback)?;
            }
        }

        debug!("已解档文件：{}", file_path.display());
        Ok(())
    }

    /// 处理单个数据块
    fn process_single_chunk<F>(
        chunk: crate::mod_database::schema::ViewChunk,
        file_path: &Path,
        progress: &mut ExtractProgress,
        progress_callback: &Option<F>,
    ) -> AnyResult<()>
    where
        F: Fn(ExtractProgress),
    {
        if ChunkStatus::from_str(&chunk.chunk_status) != ChunkStatus::Health {
            progress.error = Some(format!("数据块状态不健康：{}", chunk.chunk_status));
            warn!("{}", progress.error.as_ref().unwrap());
            Self::update_progress(progress_callback, progress);
        }

        // 单个数据块，直接提取
        if let Err(e) = ArchiveExtractor::extract_single_chunk(
            // todo 后续需要处理 archive_uri 可能是 网址不是本地路径
            &chunk.archive_uri,
            &chunk.chunk_relative_path,
            file_path,
        ) {
            progress.error = Some(format!("提取文件失败：{}", e));
            warn!("{}", progress.error.as_ref().unwrap());
            Self::update_progress(progress_callback, progress);
        }

        Ok(())
    }

    /// 处理分卷数据块
    fn process_volume_chunks<F>(
        chunk_list: &[crate::mod_database::schema::ViewChunk],
        file_path: &Path,
        progress: &mut ExtractProgress,
        progress_callback: &Option<F>,
    ) -> AnyResult<()>
    where
        F: Fn(ExtractProgress),
    {
        // 多个数据块，需要合并分卷
        let chunk_info: Vec<(String, String)> = chunk_list
            .iter()
            .map(|chunk| (chunk.archive_uri.clone(), chunk.chunk_relative_path.clone()))
            .collect();

        if let Err(e) = ArchiveExtractor::extract_volume_chunks(&chunk_info, file_path) {
            progress.error = Some(format!("提取分卷文件失败：{}", e));
            warn!("{}", progress.error.as_ref().unwrap());
            Self::update_progress(progress_callback, progress);
        }

        Ok(())
    }
}