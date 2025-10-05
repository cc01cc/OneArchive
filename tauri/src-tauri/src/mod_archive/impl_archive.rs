//! 归档功能的具体实现

use crate::mod_archive::trait_archive::{ArchiveContext, ArchiveOperations};
use crate::mod_archive::utils_archive::{
    ArchiveProgress, create_new_archive_file_path, get_file_chunk_bytes, update_archive_progress,
};
use crate::mod_database::constants::DatabaseTableName;
use crate::mod_database::constants::DirectoryStatus;
use crate::mod_database::constants::RootStatus;
use crate::mod_database::constants::{ArchiveStatus, ChunkStatus, FileStatus, MapFileChunkStatus};
use crate::mod_database::dao_database::dao_archive_chunk::ArchiveChunkDao;
use crate::mod_database::dao_database::dao_archive_metadata::ArchiveMetadataDao;
use crate::mod_database::dao_database::dao_info_directory::InfoDirectoryDao;
use crate::mod_database::dao_database::dao_info_file::InfoFileDao;
use crate::mod_database::dao_database::dao_info_root::InfoRootDao;
use crate::mod_database::dao_database::dao_map_file_chunk::MapFileChunkDao;
use crate::mod_database::dao_database::dao_view_file::ViewFileDao;
use crate::mod_database::impl_database::Database;
use crate::mod_database::schema::{
    ArchiveChunk, CreateArchiveMetadataParams, MapFileChunk, ViewFile,
};
use crate::mod_database::trait_database::StatusOperations;
use crate::utils::{EventType, ProgressEvent, calculate_file_hash, calculate_sha256};

use anyhow::Result as AnyResult;
use anyhow::anyhow;
use log::info;
use log::warn;
use std::default::Default;
use std::fs::{self, File};
use std::path::Path;
use tar::Header;

/// 归档服务实现结构体
#[derive(Default)]
pub struct ArchiveServices;

impl ArchiveServices {
    /// 保存文件与数据块的映射关系到数据库
    ///
    /// 该函数将文件与其对应的数据块建立映射关系，并保存到数据库的映射表中。
    /// 每个数据块都有一个卷序号，用于标识其在文件中的顺序位置。
    ///
    /// # 参数
    /// * `file_info` - 文件信息，包含文件 ID 等信息
    /// * `chunk_ids` - 数据块 ID 列表，按顺序排列
    /// * `database` - 数据库操作接口
    ///
    /// # 返回值
    /// 返回操作结果
    fn save_map_file_chunk_list(
        file_info: &ViewFile, chunk_ids: &[i64], database: &Database,
    ) -> AnyResult<()> {
        // TODO 需要测试分卷顺序是否有影响
        let mut volume_order: i32 = 1;

        for &chunk_id in chunk_ids {
            // 插入文件索引
            let map_file_chunk = MapFileChunk::new(
                file_info.file_id,
                chunk_id,
                volume_order,
                MapFileChunkStatus::Health,
            );

            // 在实际实现中，需要将映射信息保存到数据库
            MapFileChunkDao::new(database.conn.clone()).insert(&map_file_chunk)?;

            volume_order += 1;
        }

        // 更新文件状态为健康
        // database.update_file_status(file_info.id.unwrap(), FileStatus::Healthy.as_str())?

        Ok(())
    }

    /// 处理单个文件的归档
    fn process_single_file(
        &self, file_info: &ViewFile, file_path: &Path, context: &mut ArchiveContext,
    ) -> AnyResult<()> {
        info!("处理文件：{:?}", file_path);

        // 计算文件 hash
        let file_hash = calculate_file_hash(file_path)?;

        // 检查是否已存在相同文件
        let view_file_dao = ViewFileDao::new(context.database.conn.clone());
        let health_file_list =
            view_file_dao.find_by_hash_and_status(&file_hash, Some(FileStatus::Health))?;

        let chunk_ids = if health_file_list.is_empty() {
            // 处理新文件
            FileProcessingStrategy::process_new_file(file_info, file_path, context)?
        } else {
            // 处理已存在的文件
            FileProcessingStrategy::process_existing_file(&file_hash, &context.database)?
        };

        let _ = Self::save_map_file_chunk_list(file_info, &chunk_ids, &context.database)?;

        // 更新 file Status
        context.database.mark_table_status_by_id(
            DatabaseTableName::InfoFile,
            file_info.file_id,
            FileStatus::Health.as_str(),
        )?;
        info!("文件 {:?} 已添加到归档", file_path);
        Ok(())
    }
    /// 验证根目录并准备归档环境
    fn validate_and_prepare(
        &self, root_dir: &str, context: &mut ArchiveContext,
    ) -> AnyResult<Option<i64>> {
        info!("开始执行归档操作，根目录：{}", root_dir);

        // 校验 Root 是否存在在数据库中，即是否扫描过 Root
        let root_dao = InfoRootDao::new(context.database.conn.clone());
        let root_absolute_path = Path::new(root_dir).canonicalize()?;
        let root_id =
            match root_dao.find_root_info_by_path(&root_absolute_path.to_string_lossy())? {
                Some(root_info) => match root_info.id {
                    Some(id) => Some(id),
                    None => return Err(anyhow!("根目录 ID 不存在")),
                },
                None => {
                    info!("根目录 {} 不存在于数据库中", root_dir);
                    return Ok(None);
                }
            };

        // 确保归档目录存在且为空
        let archive_directory = &context.archive_directory;
        let archive_dir_path = Path::new(archive_directory);
        if archive_dir_path.exists() {
            if !archive_dir_path.is_dir() {
                return Err(anyhow!("归档路径不是一个目录：{}", archive_directory));
            }
            // 检查目录是否为空
            let mut entries = fs::read_dir(archive_dir_path)
                .map_err(|e| anyhow!("无法读取归档目录 {}: {}", archive_directory, e))?;
            if entries.next().is_some() {
                return Err(anyhow!("归档目录不为空：{}", archive_directory));
            }
        } else {
            // 创建归档目录
            fs::create_dir_all(archive_dir_path)
                .map_err(|e| anyhow!("无法创建归档目录 {}: {}", archive_directory, e))?;
        }

        Ok(root_id)
    }

    /// 获取需要归档的文件列表
    fn get_files_to_archive(
        &self, database: &Database, root_id: i64,
    ) -> AnyResult<(Vec<ViewFile>, u64, usize)> {
        let view_file_dao = ViewFileDao::new(database.conn.clone());
        // 查找所有需要归档的文件 (状态为 WaitToArchive)
        let file_list = view_file_dao
            .find_by_root_id_and_status(root_id, Some(FileStatus::WaitToArchive))?;

        if file_list.is_empty() {
            return Err(anyhow!("没有找到需要归档的文件"));
        }

        info!("找到 {} 个文件需要添加到归档", file_list.len());

        // 计算总大小用于进度跟踪
        let total_size: u64 = file_list.iter().map(|f| f.file_size as u64).sum();
        let total_files = file_list.len();

        info!("开始归档文件，总大小：{} bytes, 文件数：{}", total_size, total_files);

        Ok((file_list, total_size, total_files))
    }

    /// 初始化进度信息
    fn initialize_progress(&self, total_files: usize, total_size: u64) -> ArchiveProgress {
        ArchiveProgress {
            total_files,
            processed_files: 0,
            processed_bytes: 0,
            total_bytes: total_size,
            current_file: None,
            completed: false,
            error: None,
        }
    }

    /// 处理所有文件的归档
    fn process_files_to_archive<F>(
        &self, file_list: &[ViewFile], context: &mut ArchiveContext,
        progress: &mut ArchiveProgress, progress_callback: &Option<F>,
    ) -> AnyResult<()>
    where
        F: Fn(ProgressEvent<ArchiveProgress>),
    {
        let mut processed_files = 0;
        let mut processed_size: u64 = 0;

        // 发送初始进度
        update_archive_progress(progress_callback, progress, "开始归档任务", EventType::Start);

        // 遍历文件并添加到归档
        for file_info in file_list {
            // 拼接 file 的绝对路径
            let file_path = match &file_info.directory_path {
                Some(dir) if !dir.is_empty() => {
                    Path::new(&file_info.root_path).join(dir).join(&file_info.file_name)
                }
                // 在 Root 下的文件，即 dir 为 none
                _ => Path::new(&file_info.root_path).join(&file_info.file_name),
            };

            if !file_path.exists() {
                warn!("文件不存在，跳过：{:?}", file_path);
                continue;
            }

            // 更新当前文件信息
            progress.current_file = Some(file_info.file_name.clone());
            update_archive_progress(
                progress_callback,
                progress,
                "发送初始进度",
                EventType::Progress,
            );

            // ! MAIN 处理单个文件
            self.process_single_file(&file_info, &file_path, context)?;

            // 更新已处理大小和文件数
            processed_size += file_info.file_size as u64;
            processed_files += 1;

            // 更新进度
            progress.processed_files = processed_files;
            progress.processed_bytes = processed_size;
            update_archive_progress(
                progress_callback,
                progress,
                &format!("已完成 {}/{} 个文件的归档", processed_files, file_list.len()),
                EventType::Progress,
            );

            info!("已完成 {}/{} 个文件的归档", processed_files, file_list.len());
        }

        progress.completed = true;
        update_archive_progress(progress_callback, progress, "归档任务完成", EventType::Complete);

        Ok(())
    }

    /// 完成归档后的清理工作
    fn finalize_archive(&self, root_id: i64, context: &ArchiveContext) -> AnyResult<()> {
        info!("所有文件已添加到归档目录 {}", context.archive_directory);

        let directory_dao = InfoDirectoryDao::new(context.database.conn.clone());
        let file_dao = InfoFileDao::new(context.database.conn.clone());
        // TODO 更新 directory 状态
        // 获取所有目录
        let directories = directory_dao.find_by_status_and_root_id(root_id, None)?;

        // 遍历每个目录并检查其下的文件状态
        // FIXME 每个文件不仅仅有直接的归属目录，还有上级目录的再上级目录，直至递归到根目录
        for mut directory in directories {
            // 获取该目录下的所有文件
            let files = file_dao
                .find_files_by_status_and_root_id(root_id, None)?
                .into_iter()
                .filter(|file| file.directory_id == directory.id.unwrap_or(0))
                .collect::<Vec<_>>();

            // 检查是否有文件不是健康状态
            let all_files_healthy = files.iter().all(|file| file.status == FileStatus::Health);

            // 根据文件状态更新目录状态
            if all_files_healthy {
                directory.status = DirectoryStatus::Health;
            } else {
                directory.status = DirectoryStatus::ErrorArchivingFailed; // 或其他适当的异常状态
            }

            // 更新目录状态到数据库
            directory_dao.update_directory(&directory)?;
        }

        // 更新 Root 状态
        // FIXME 需要调整 逻辑，计算自 file 和 dir
        let _ = context.database.mark_table_status_by_id(
            DatabaseTableName::InfoRoot,
            root_id,
            RootStatus::Health.as_str(),
        );

        Ok(())
    }
}

impl ArchiveOperations for ArchiveServices {
    /// 将指定根目录下的所有文件添加到归档中
    /// 对应 Java 中 ArchiveIn.java 的 archiveFileInDb 方法
    ///
    /// # 参数
    /// * `root_dir` - 根目录路径
    /// * `context` - 归档上下文
    /// * `database` - 数据库实例
    /// * `progress_callback` - 进度回调函数（可选）
    ///
    /// # 返回值
    /// 返回操作结果
    fn archive<F>(
        &self, root_dir: &str, context: &mut ArchiveContext, progress_callback: Option<F>,
    ) -> AnyResult<()>
    where
        F: Fn(ProgressEvent<ArchiveProgress>),
    {
        // 1. 验证和准备工作
        let root_id = self.validate_and_prepare(root_dir, context)?;

        // 如果根目录不存在于数据库中，直接返回
        let root_id = match root_id {
            Some(id) => id,
            None => {
                info!("没有找到根目录 {} 下需要归档的文件", root_dir);
                return Ok(());
            }
        };

        // 2. 获取需要归档的文件列表
        let (file_list, total_size, total_files) =
            self.get_files_to_archive(&context.database, root_id)?;

        if file_list.is_empty() {
            info!("没有找到根目录 {} 下需要归档的文件", root_dir);
            return Ok(());
        }

        // 3. 初始化进度信息
        let mut progress = self.initialize_progress(total_files, total_size);

        // 4. 处理文件归档
        self.process_files_to_archive(&file_list, context, &mut progress, &progress_callback)?;

        // 5. 完成后处理
        self.finalize_archive(root_id, context)?;

        Ok(())
    }
}

/// 文件处理策略实现
struct FileProcessingStrategy;

impl FileProcessingStrategy {
    /// 处理新文件 (不写入 map db, 会写入 chunk db)
    fn process_new_file(
        view_file: &ViewFile, file_path: &Path, archive_context: &mut ArchiveContext,
    ) -> AnyResult<Vec<i64>> {
        let mut chunk_ids = Vec::new();

        // 处理非空文件
        if view_file.file_size > 0 {
            Self::process_non_empty_file(view_file, file_path, archive_context, &mut chunk_ids)?;
        }

        // 特殊处理空文件（文件大小为 0）
        if view_file.file_size == 0 {
            // 确保归档可用（确保 current_archive_id 已设置）
            Self::prepare_archive_if_needed(archive_context)?;
            let empty_data = vec![];
            Self::process_chunk_data(empty_data, view_file, archive_context, &mut chunk_ids)?;
        }

        Ok(chunk_ids)
    }

    /// 处理非空文件
    fn process_non_empty_file(
        view_file: &ViewFile, file_path: &Path, archive_context: &mut ArchiveContext,
        chunk_ids: &mut Vec<i64>,
    ) -> AnyResult<()> {
        let mut file_offset: i64 = 0;
        let mut file_remaining_size = view_file.file_size;

        // 用于跟踪文件内部进度
        let _file_processed_size: i64 = 0;

        // 在处理文件的循环中，应该处理文件大小为 0 的情况
        while file_remaining_size > 0 {
            Self::prepare_archive_if_needed(archive_context)?;

            // 计算本次可以处理的数据量
            let chunk_size = std::cmp::min(file_remaining_size, archive_context.archive_limit_size);

            // 读取数据块
            let chunk_data = get_file_chunk_bytes(file_path, file_offset, chunk_size)?;

            // 处理数据块
            Self::process_chunk_data(chunk_data, view_file, archive_context, chunk_ids)?;

            // 如果文件大小超过归档限制大小，则分卷处理
            // 更新偏移量和剩余大小
            file_offset += chunk_size;
            file_remaining_size -= chunk_size;

            archive_context.current_archive_size = std::fs::metadata(
                archive_context
                    .current_archive_file_path
                    .as_ref()
                    .ok_or_else(|| anyhow!("当前归档文件路径不存在"))?,
            )?
            .len() as i64;

            info!("处理了 {} 字节的数据块", chunk_size);
        }

        Ok(())
    }

    /// 如果需要，准备新的归档
    fn prepare_archive_if_needed(archive_context: &mut ArchiveContext) -> AnyResult<()> {
        if archive_context.need_new_archive() {
            archive_context.reset_current();

            // 创建新的归档数据库记录
            Self::create_new_archive_db_record(archive_context)?;

            // 创建新的归档文件
            let archive_file =
                File::create(archive_context.current_archive_file_path.as_ref().unwrap())?;
            let archive_tar = tar::Builder::new(archive_file);
            archive_context.current_archive_tar_builder = Some(archive_tar);

            // 更新归档文件计数
            archive_context.archive_counter += 1;
        }
        Ok(())
    }

    /// 处理数据块
    fn process_chunk_data(
        chunk_data: Vec<u8>, view_file: &ViewFile, archive_context: &mut ArchiveContext,
        chunk_ids: &mut Vec<i64>,
    ) -> AnyResult<()> {
        // 计算 hash(需要考虑不同块，即不同分卷，有相同内容的可能)
        let hash = calculate_sha256(&chunk_data);

        let archive_chunk_dao = ArchiveChunkDao::new(archive_context.database.conn.clone());
        // 检查是否已存在相同 hash 的数据块
        if let Some(existing_chunk) = archive_chunk_dao.find_by_chunk_hash(&hash)? {
            // 如果存在相同的数据块，则直接使用已有的数据块 ID，避免重复创建
            chunk_ids.push(existing_chunk.id.unwrap());
            info!("发现重复数据块，使用已有的数据块 ID: {:?}", existing_chunk.id);
        } else {
            let mut new_chunk = ArchiveChunk::new(
                archive_context.current_archive_id.unwrap(),
                hash.clone(),
                chunk_data.len() as i64,
                hash.clone(),
                view_file.file_mtime,
                hash.clone(),
                ChunkStatus::WaitToArchive,
            );
            // 将数据块添加到归档中
            Self::add_chunk_to_archive(
                &mut archive_context.current_archive_tar_builder,
                &chunk_data,
                &new_chunk,
            )?;

            new_chunk.status = ChunkStatus::Health;

            // 插入数据块到数据库
            let chunk_id = archive_chunk_dao.insert_archive_chunk(&new_chunk);
            new_chunk.id = Some(chunk_id?);
            chunk_ids.push(new_chunk.id.unwrap());
        }

        Ok(())
    }

    /// 处理已存在文件 (不写入 db)
    fn process_existing_file(file_hash: &str, database: &Database) -> AnyResult<Vec<i64>> {
        let file_dao = ViewFileDao::new(database.conn.clone());
        // 查找具有相同哈希值的健康文件
        // TODO 当 非 Health 的 chunk 存在时，name 现在使用 hash 是否会冲突？
        let file_by_hash_list =
            file_dao.find_by_hash_and_status(file_hash, Some(FileStatus::Health))?;

        // 使用第一个匹配的文件 ID 来查找关联的数据块
        let first_matching_file_id = file_by_hash_list
            .first()
            .map(|f| f.file_id)
            .ok_or_else(|| anyhow!("找不到匹配的文件 ID"))?;
        let map_dao = MapFileChunkDao::new(database.conn.clone());

        let map_file_chunk_list = map_dao.find_by_file_id_ordered(first_matching_file_id)?;

        info!("链接已存在的文件 hash: {}", file_hash);

        // 获取已存在的数据块 ID
        let chunk_ids: Vec<i64> =
            map_file_chunk_list.into_iter().map(|map_chunk| map_chunk.chunk_id).collect();

        Ok(chunk_ids)
    }

    /// 将数据块添加到归档中
    fn add_chunk_to_archive(
        tar_builder: &mut Option<tar::Builder<File>>, chunk_data: &[u8], chunk: &ArchiveChunk,
    ) -> AnyResult<()> {
        match tar_builder {
            Some(builder) => {
                // 创建 tar 条目，使用数据块名称作为文件名
                let mut header = Header::new_gnu();
                header.set_size(chunk.chunk_size as u64);
                header.set_mtime(chunk.chunk_mtime as u64);
                header.set_cksum();

                // 添加条目到 tar 文件
                builder.append_data(&mut header, &chunk.chunk_name, chunk_data)?;

                info!("成功添加数据块到归档：{}", chunk.chunk_name);
                Ok(())
            }
            None => Err(anyhow!("当前没有可用的归档构建器")),
        }
    }

    fn create_new_archive_db_record(context: &mut ArchiveContext) -> AnyResult<i64> {
        // 创建新的归档路径
        let archive_file_path = create_new_archive_file_path(
            &context.archive_directory,
            &context.archive_file_prefix,
            context.archive_counter as u32,
        );

        let archive_name = archive_file_path
            .file_name()
            .ok_or_else(|| anyhow!("无法获取归档文件名"))?
            .to_str()
            .ok_or_else(|| anyhow!("文件名包含无效字符"))?;

        println!("文件名：{}", archive_name);

        let new_archive_metadata = CreateArchiveMetadataParams {
            archive_uri: archive_file_path.to_string_lossy().to_string(),
            archive_name: archive_name.to_string(),
            archive_limit_size: context.archive_limit_size as u64,
            // TODO 暂不考虑压缩和加密
            is_compressed: 0,
            is_encrypted: 0,
            compression_algorithm: None,
            encryption_algorithm: None,
            status: ArchiveStatus::InArchiving,
        };
        let archive_id =
            ArchiveMetadataDao::new(context.database.conn.clone()).create(new_archive_metadata)?;

        context.current_archive_id = Some(archive_id);
        context.current_archive_file_path = Some(archive_file_path);

        Ok(archive_id)
    }
}
