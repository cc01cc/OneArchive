//! 存档功能的具体实现

use anyhow::Result as AnyResult;
use anyhow::anyhow;
use chrono::Utc;
use log::info;
use log::warn;
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use std::path::PathBuf;
use std::time::SystemTime;
use tar::Header;

use crate::mod_archive::traits_archive::{ArchiveContext, ArchiveOperations};
use crate::mod_database::constants::DatabaseTableName;
use crate::mod_database::constants::DirectoryStatus;
use crate::mod_database::constants::RootStatus;
use crate::mod_database::constants::{ArchiveStatus, AssetStatus, FileStatus, MapFileAssetStatus};
use crate::mod_database::schema::{ArchiveAsset, ArchiveMetadata, MapFileAsset, ViewFile};
use crate::mod_database::traits::{
    ArchiveAssetOperations, ArchiveMetadataOperations, FileOperations, MapFileAssetOperations,
    RootOperations, StatusOperations, ViewOperations,DirectoryOperations
};

/// 存档服务实现结构体
pub struct ArchiveInServices;

impl ArchiveInServices {
    /// 创建新的归档服务实例
    pub fn new() -> Self {
        ArchiveInServices
    }
}

impl ArchiveOperations for ArchiveInServices {
    /// 将指定根目录下的所有文件添加到存档中
    /// 对应 Java 中 ArchiveIn.java 的 archiveFileInDb 方法
    ///
    /// # 参数
    /// * `root_dir` - 根目录路径
    /// * `context` - 归档上下文
    /// * `database` - 数据库实例
    ///
    /// # 返回值
    /// 返回操作结果
    fn archive_file_in_db<D>(
        &self,
        root_dir: &str,
        context: &mut ArchiveContext,
        // TODO 需要调整 database 输入的逻辑，database 应该直接放到存档目录中，减少用户参数输入
        database: &D,
    ) -> AnyResult<()>
    where
        D: ViewOperations
            + FileOperations
            + DirectoryOperations
            + MapFileAssetOperations
            + ArchiveAssetOperations
            + ArchiveMetadataOperations
            + RootOperations
            + StatusOperations,
    {
        info!("开始执行存档操作，根目录：{}", root_dir);

        // 校验 Root 是否存在在数据库中，即是否扫描过 Root
        let root_absolute_path = Path::new(root_dir).canonicalize()?;
        let root_id =
            match database.find_root_info_by_path(&root_absolute_path.to_string_lossy())? {
                Some(root_info) => match root_info.id {
                    Some(id) => id,
                    None => return Err(anyhow!("根目录 ID 不存在")),
                },
                None => {
                    info!("根目录 {} 不存在于数据库中", root_dir);
                    return Ok(());
                }
            };

        // 确保存档目录存在且为空
        let archive_directory = &context.archive_directory;
        let archive_dir_path = Path::new(archive_directory);
        if archive_dir_path.exists() {
            if !archive_dir_path.is_dir() {
                return Err(anyhow!("存档路径不是一个目录：{}", archive_directory));
            }
            // 检查目录是否为空
            let mut entries = fs::read_dir(archive_dir_path)
                .map_err(|e| anyhow!("无法读取存档目录 {}: {}", archive_directory, e))?;
            if entries.next().is_some() {
                return Err(anyhow!("存档目录不为空：{}", archive_directory));
            }
        } else {
            // 创建存档目录
            fs::create_dir_all(archive_dir_path)
                .map_err(|e| anyhow!("无法创建存档目录 {}: {}", archive_directory, e))?;
        }

        // 查找所有需要存档的文件 (状态为 WaitToArchive)
        let file_list = database
            .find_view_files_by_root_id_and_status(root_id, Some(FileStatus::WaitToArchive))?;

        if file_list.is_empty() {
            info!("没有找到根目录 {} 下需要存档的文件", root_dir);
            return Ok(());
        }

        info!("找到 {} 个文件需要添加到存档", file_list.len());

        // 计算总大小用于进度跟踪
        let total_size: u64 = file_list.iter().map(|f| f.file_size as u64).sum();
        let total_files = file_list.len();
        let mut processed_files = 0;
        let mut processed_size: u64 = 0;

        info!(
            "开始归档文件，总大小：{} bytes, 文件数：{}",
            total_size, total_files
        );

        // 遍历文件并添加到存档
        for file_info in file_list {
            // 拼接 file 的绝对路径
            let file_path = match &file_info.directory_path {
                Some(dir) if !dir.is_empty() => Path::new(&file_info.root_path)
                    .join(dir)
                    .join(&file_info.file_name),
                // 在 Root 下的文件，即 dir 为 none
                _ => Path::new(&file_info.root_path).join(&file_info.file_name),
            };

            if !file_path.exists() {
                warn!("文件不存在，跳过：{:?}", file_path);
                continue;
            }

            // 处理单个文件
            self.process_single_file(&file_info, &file_path, context, database)?;

            // 更新已处理大小和文件数
            processed_size += file_info.file_size as u64;
            processed_files += 1;

            info!("已完成 {}/{} 个文件的归档", processed_files, total_files);
        }

        info!("所有文件已添加到存档目录 {}", context.archive_directory);

        // 更新 directory 状态
        // 获取所有目录
        let directories = database.find_directories_by_status_and_root_id(root_id, None)?;

        // 遍历每个目录并检查其下的文件状态
        // FIXME 每个文件不仅仅有直接的归属目录, 还有上级目录的再上级目录, 直至递归到根目录
        for mut directory in directories {
            // 获取该目录下的所有文件
            let files = database
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
            database.update_directory(&directory)?;
        }

        // 更新 Root 状态
        // FIXME 需要调整 逻辑, 计算自 file 和 dir
        let _ = database.mark_table_status_by_id(
            DatabaseTableName::InfoRoot,
            root_id,
            RootStatus::Health.as_str(),
        );
        Ok(())
    }
}

impl ArchiveInServices {
    /// 处理单个文件的归档
    fn process_single_file<D>(
        &self,
        file_info: &ViewFile,
        file_path: &Path,
        context: &mut ArchiveContext,
        database: &D,
    ) -> AnyResult<()>
    where
        D: ViewOperations
            + FileOperations
            + MapFileAssetOperations
            + ArchiveAssetOperations
            + ArchiveMetadataOperations
            + StatusOperations,
    {
        info!("处理文件：{:?}", file_path);

        // 计算文件 hash
        let file_hash = self.calculate_file_hash(file_path)?;

        // 检查是否已存在相同文件
        let health_file_list =
            database.find_view_files_by_hash_and_status(&file_hash, Some(FileStatus::Health))?;

        let mut asset_ids: Option<Vec<i64>> = None;
        if health_file_list.is_empty() {
            // 处理新文件
            let strategy = NewFileProcessingStrategy;
            asset_ids = Some(strategy.process(file_info, file_path, context, database)?);
        } else {
            // 处理已存在的文件
            let strategy = ExistingFileProcessingStrategy;
            asset_ids = Some(strategy.process(&file_hash, database)?);
        }

        self.save_file_indexes(file_info, asset_ids.as_ref().unwrap(), database);
        // 更新 file Status
        database.mark_table_status_by_id(
            DatabaseTableName::InfoFile,
            file_info.file_id,
            FileStatus::Health.as_str(),
        )?;
        info!("文件 {:?} 已添加到存档", file_path);
        Ok(())
    }

    /// 计算文件哈希值
    fn calculate_file_hash(&self, file_path: &Path) -> AnyResult<String> {
        let mut file = File::open(file_path)?;
        let mut hasher = Sha256::new();
        std::io::copy(&mut file, &mut hasher)?;
        let hash = hasher.finalize();
        Ok(format!("{:x}", hash))
    }

    /// 创建文件索引
    fn save_file_indexes<D>(
        &self,
        file_info: &ViewFile,
        asset_ids: &[i64],
        database: &D,
    ) -> AnyResult<()>
    where
        D: MapFileAssetOperations,
    {
        // TODO 需要测试分卷顺序是否有影响
        let mut volume_order: i32 = 1;

        for &asset_id in asset_ids {
            // 插入文件索引
            let map_file_asset = MapFileAsset::new(
                file_info.file_id,
                asset_id,
                volume_order,
                MapFileAssetStatus::Health,
            );

            // 在实际实现中，需要将映射信息保存到数据库
            database.insert_map_file_asset(&map_file_asset)?;

            volume_order += 1;
        }

        // 更新文件状态为健康
        // database.update_file_status(file_info.id.unwrap(), FileStatus::Healthy.as_str())?

        Ok(())
    }
}

impl Default for ArchiveInServices {
    fn default() -> Self {
        Self::new()
    }
}

/// 获取文件数据块
fn get_file_chunk_bytes(file_path: &Path, offset: i64, size: i64) -> AnyResult<Vec<u8>> {
    let mut file = File::open(file_path)?;
    let mut buffer = vec![0u8; size as usize];

    // 跳转到指定偏移量
    file.seek(SeekFrom::Start(offset as u64))?;

    // 读取指定大小的数据
    file.read_exact(&mut buffer)?;

    Ok(buffer)
}

/// 创建新的归档文件路径
fn create_new_archive_file_path(context: &mut ArchiveContext) -> AnyResult<PathBuf> {
    let archive_dir = Path::new(&context.archive_directory);
    let timestamp = Utc::now().format("%Y%m%d%H%M%S");
    let archive_file_name = format!(
        "{}_{}_{}.tar",
        &context.archive_file_prefix, &timestamp, &context.archive_counter
    );
    context.current_archive_file_path = Some(archive_dir.join(&archive_file_name));
    Ok(archive_dir.join(&archive_file_name))
}

/// 计算数据的 SHA256 哈希值
fn calculate_sha256(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let hash = hasher.finalize();
    format!("{:x}", hash)
}

/// 处理新文件的策略实现
struct NewFileProcessingStrategy;

impl NewFileProcessingStrategy {
    /// 处理新文件的主要方法
    fn process<D>(
        &self,
        view_file: &ViewFile,
        file_path: &Path,
        archive_context: &mut ArchiveContext,
        database: &D,
    ) -> AnyResult<Vec<i64>>
    where
        D: ViewOperations + FileOperations + ArchiveMetadataOperations + ArchiveAssetOperations,
    {
        let mut asset_ids = Vec::new();
        let mut file_offset: i64 = 0;
        let mut file_remaining_size = view_file.file_size;

        // 用于跟踪文件内部进度
        let mut file_processed_size: i64 = 0;

        while file_remaining_size > 0 {
            if archive_context.need_new_archive() {
                archive_context.reset_current();

                // 创建新的存档路径
                let archive_file_path = create_new_archive_file_path(archive_context)?;

                // 创建新的归档数据库记录
                let archive_id: i64 =
                    self.create_new_archive_db_record(archive_context, database)?;

                // 创建新的归档文件
                let archive_file = File::create(archive_file_path)?;
                let archive_tar = tar::Builder::new(archive_file);
                archive_context.current_archive_tar_builder = Some(archive_tar);

                // 更新归档文件计数
                archive_context.archive_counter += 1;
            }

            // 计算本次可以处理的数据量
            let chunk_size = std::cmp::min(file_remaining_size, archive_context.archive_limit_size);

            // 读取数据块
            let chunk_data = get_file_chunk_bytes(file_path, file_offset, chunk_size)?;

            // 计算 hash(需要考虑不同块，即不同分卷，有相同内容的可能)
            let hash = calculate_sha256(&chunk_data);

            // 检查是否已存在相同 hash 的资产
            if let Some(existing_asset) = database.find_archive_asset_by_asset_hash(&hash)? {
                // 如果存在相同的资产，则直接使用已有的资产 ID，避免重复创建
                asset_ids.push(existing_asset.id.unwrap());
                info!("发现重复资产，使用已有的资产 ID: {:?}", existing_asset.id);
            } else {
                // 创建资产
                let mut new_asset = ArchiveAsset::new(
                    archive_context.current_archive_id.unwrap(),
                    hash.clone(),
                    chunk_size,
                    hash.clone(),
                    view_file.file_mtime,
                    hash.clone(),
                    AssetStatus::WaitToArchive,
                );
                // 将资产添加到存档中
                self.add_asset_to_archive(archive_context, &chunk_data, &new_asset)?;

                new_asset.status = AssetStatus::Health;

                // 插入资产到数据库
                let asset_id = database.insert_archive_asset(&new_asset);
                new_asset.id = Some(asset_id?);
                asset_ids.push(new_asset.id.unwrap());
            }

            // 如果文件大小超过存档限制大小，则分卷处理
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

        Ok(asset_ids)
    }

    /// 将资产添加到归档中
    fn add_asset_to_archive(
        &self,
        context: &mut ArchiveContext,
        chunk_data: &[u8],
        asset: &ArchiveAsset,
    ) -> AnyResult<()> {
        if let Some(mut tar_builder) = context.current_archive_tar_builder.take() {
            // 创建 tar 条目，使用资产名称作为文件名
            let mut header = Header::new_gnu();
            header.set_size(asset.asset_size as u64);
            header.set_mtime(asset.asset_mtime as u64);
            header.set_cksum();

            // 添加条目到 tar 文件
            tar_builder.append_data(&mut header, &asset.asset_name, chunk_data)?;

            // 重新存储 tar builder
            context.current_archive_tar_builder = Some(tar_builder);

            info!("成功添加资产到归档：{}", asset.asset_name);
        } else {
            return Err(anyhow!("当前没有可用的归档构建器"));
        }

        Ok(())
    }

    fn create_new_archive_db_record<D>(
        &self,
        context: &mut ArchiveContext,
        database: &D,
    ) -> AnyResult<i64>
    where
        D: ArchiveMetadataOperations,
    {
        let file_path = context
            .current_archive_file_path
            .as_ref()
            .ok_or_else(|| anyhow!("当前没有设置存档文件"))?;

        let file_name = file_path
            .file_name()
            .ok_or_else(|| anyhow!("无法获取文件名"))?
            .to_str()
            .ok_or_else(|| anyhow!("文件名包含无效字符"))?;

        println!("文件名：{}", file_name);

        let metadata = ArchiveMetadata::new(
            file_path.to_string_lossy().to_string(),
            file_name.to_string(),
            context.archive_limit_size,
            // TODO 暂不考虑压缩和加密
            0,
            0,
            ArchiveStatus::InArchiving,
        );

        let archive_id = database.insert_archive_metadata(&metadata)?;
        context.current_archive_id = Some(archive_id);
        Ok(archive_id)
    }
}

/// 处理已存在文件的策略实现
struct ExistingFileProcessingStrategy;

impl ExistingFileProcessingStrategy {
    /// 处理已存在文件的主要方法
    fn process<D>(&self, file_hash: &str, database: &D) -> AnyResult<Vec<i64>>
    where
        D: MapFileAssetOperations + ViewOperations,
    {
        // 查找具有相同哈希值的健康文件
        // TODO 当 非 Health 的 asset 存在时，name 现在使用 hash 是否会冲突？
        let file_by_hash_list =
            database.find_view_files_by_hash_and_status(file_hash, Some(FileStatus::Health))?;

        // 使用第一个匹配的文件 ID 来查找关联的资产
        let first_matching_file_id = file_by_hash_list
            .first()
            .map(|f| f.file_id)
            .ok_or_else(|| anyhow!("找不到匹配的文件 ID"))?;

        let map_file_asset_list =
            database.find_map_file_asset_by_file_id_ordered(first_matching_file_id)?;

        info!("链接已存在的文件 hash: {}", file_hash);

        // 获取已存在的资产 ID
        let asset_ids: Vec<i64> = map_file_asset_list
            .into_iter()
            .map(|map_asset| map_asset.asset_id)
            .collect();

        Ok(asset_ids)
    }
}
