/*
 * Copyright 2025. Zheng, Yihong (ZEO, github.com/cc01cc)
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

package com.cc01cc.onearchive.core.archive;

import com.cc01cc.onearchive.core.CommonTool;
import com.cc01cc.onearchive.core.DirectoryScanner;
import com.cc01cc.onearchive.core.constant.ArchiveStatus;
import com.cc01cc.onearchive.core.constant.FileStatus;
import com.cc01cc.onearchive.core.constant.FileVolumeAssetStatus;
import com.cc01cc.onearchive.core.dao.DatabaseAccessor;
import com.cc01cc.onearchive.core.entity.ArchiveMetadata;
import com.cc01cc.onearchive.core.entity.InfoFile;
import com.cc01cc.onearchive.core.entity.MapFileAsset;
import com.cc01cc.onearchive.core.entity.ViewFile;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.compress.archivers.tar.TarArchiveOutputStream;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.List;
import java.util.Objects;

/**
 * 存档管理器，负责创建和管理存档文件
 */
@Slf4j
public class ArchiveIn {

    public static void archive(
            String rootDir,
            String dbPath
    ) throws IOException {

        String dbUrl = "jdbc:sqlite:" + dbPath;
        DatabaseAccessor databaseAccessor = new DatabaseAccessor(dbUrl);

        // 检查 rootDir 是否已经存在
        DirectoryScanner.scanAndSaveDirectory(Path.of(rootDir), databaseAccessor);
    }

    /**
     * 将指定根目录下的所有文件添加到存档中
     *
     * @param rootDir          根目录路径
     * @param databaseAccessor 数据库访问器
     * @throws IOException 如果操作过程中发生IO错误
     *                     <p>
     *                     获取根目录所有文件的列表(包括文件路径, 以及文件大小)
     *                     如果文件大小超过存档限制大小，则临时跳过, 最后统一分卷处理大文件
     *                     执行存档
     */
    public static void archiveFileInDb(
            String rootDir,
            ArchiveContext context,
            DatabaseAccessor databaseAccessor) throws IOException {

        Path rootAbsolutePath = Path.of(rootDir).toAbsolutePath();
        Long rootId = databaseAccessor.findRootIdByPath(rootAbsolutePath.toString());

        String archiveDirectory = context.getArchiveDirectory();
        Path archiveDirPath = Path.of(archiveDirectory);

        // 校验配置参数: 确保存档目录存在且为空
        if (!Files.exists(archiveDirPath)) {
            Files.createDirectories(archiveDirPath);
        } else if (!Files.isDirectory(archiveDirPath)) {
            throw new IOException("存档路径不是一个目录: " + archiveDirectory);
        } else if (Files.list(archiveDirPath).findFirst().isPresent()) {
            throw new IOException("存档目录不为空: " + archiveDirectory);
        }

        if (rootId == null) {
            log.error("根目录 {} 不存在于数据库中", rootDir);
            return;
        }

        List<ViewFile> fileList = databaseAccessor.findViewFilesWithStatusAndRootId(rootId, FileStatus.UNARCHIVED);
        if (fileList == null) {
            log.warn("没有找到根目录 {} 下的文件", rootDir);
            return;
        }
        log.info("找到 {} 个文件需要添加到存档", fileList.size());

        try {
            // 遍历文件并添加到存档
            processFiles(fileList, context);
        } finally {
            // 确保输出流被正确关闭
            closeTarOutputAndDoneArchive(context.getTarOutput(), context.getArchiveId(), databaseAccessor);
        }

        log.info("所有文件已添加到存档目录 {}", context.getArchiveDirectory());

        //TODO 存档结束后删除 status 为 WAIT_TO_DELETE 的文件以及目录
    }

    static void processFiles(List<ViewFile> fileList, ArchiveContext context) throws IOException {
        for (ViewFile viewFile : fileList) {
            processSingleFile(viewFile, context);
        }
    }

    static void processSingleFile(ViewFile viewFile, ArchiveContext context) throws IOException {
        Path filePath = Path.of(viewFile.getRootPath(), viewFile.getDirectoryPath(), viewFile.getFileName());

        if (Files.notExists(filePath)) {
            log.warn("文件不存在: {}", filePath);
            return;
        }

        // 计算文件hash
        String fileHash = CommonTool.calculateFileHash(filePath);

        // 检查是否已存在相同文件
        List<InfoFile> healthFileListByHash = context.getDatabaseAccessor().findFileByHashAndStatus(fileHash, FileStatus.HEALTH);

        List<Long> assetIds;
        FileProcessingStrategy strategy;
        if (healthFileListByHash == null || healthFileListByHash.isEmpty()) {
            // 处理新文件
            strategy = new NewFileProcessingStrategy();
        } else {
            // 处理已存在的文件
            strategy = new ExistingFileProcessingStrategy();
        }
        assetIds = strategy.process(viewFile, filePath, context);

        // 创建文件索引
        createFileIndexes(viewFile, assetIds, context);
        checkHashAndMarkFileHealthy(context.getDatabaseAccessor(), viewFile.getFileId(), fileHash);

        log.info("文件 {} 已添加到存档", filePath);
    }

    static void checkHashAndMarkFileHealthy(DatabaseAccessor databaseAccessor, Long fileId, String fileHash) {
        // 添加 hash，更新文件状态为健康
        InfoFile infoFile = databaseAccessor.findFileInfoById(fileId);
        infoFile.setStatus(FileStatus.HEALTH);
        if (!Objects.equals(fileHash, infoFile.getHash())) {
            log.warn("文件 {} 的 hash 不一致，请检查", infoFile.getName());
        }
        infoFile.setUpdatedAt(System.currentTimeMillis() / 1000);

        // 更新数据库
        databaseAccessor.updateFileInfo(infoFile);
    }


    private static void createFileIndexes(ViewFile viewFile, List<Long> assetIds, ArchiveContext context) throws IOException {
        long volumeOrder = 1;
        for (long assetId : assetIds) {
            // 插入文件索引
            MapFileAsset mapFileAsset = new MapFileAsset();
            mapFileAsset.setFileId(viewFile.getFileId());
            mapFileAsset.setAssetId(assetId);
            mapFileAsset.setStatus(FileVolumeAssetStatus.HEALTH);
            mapFileAsset.setVolumeOrder(volumeOrder++);

            context.getDatabaseAccessor().insertFileVolumeAsset(mapFileAsset);
        }
    }

    static void closeTarOutputAndDoneArchive(TarArchiveOutputStream tarOutput, Long archiveId, DatabaseAccessor databaseAccessor) {
        if (tarOutput != null) {
            try {
                if (archiveId != null) {
                    // TODO 校验 status 逻辑
                    ArchiveMetadata archiveMetadataById = databaseAccessor.findArchiveMetadataById(archiveId, null);
                    if (archiveMetadataById != null) {
                        doneArchive(databaseAccessor, archiveMetadataById);
                    }
                }
                tarOutput.finish();
                tarOutput.close();
            } catch (IOException e) {
                log.error("关闭存档输出流时出错", e);
            }
        }
    }

    private static void doneArchive(DatabaseAccessor databaseAccessor, ArchiveMetadata archiveMetadataById) {
        archiveMetadataById.setStatus(ArchiveStatus.HEALTH);
        archiveMetadataById.setUpdatedAt(System.currentTimeMillis() / 1000);
        databaseAccessor.updateArchiveMetadata(archiveMetadataById);
    }

}
