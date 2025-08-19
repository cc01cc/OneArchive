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

package com.cc01cc.project;

import com.cc01cc.project.dao.DatabaseAccessor;
import com.cc01cc.project.dto.*;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.codec.digest.DigestUtils;
import org.apache.commons.compress.archivers.tar.TarArchiveOutputStream;

import java.io.IOException;
import java.io.InputStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.List;

/**
 * 存档管理器，负责创建和管理存档文件
 */
@Slf4j
public class ArchiveIn {

    /**
     * 将指定根目录下的所有文件添加到存档中
     *
     * @param rootPath         根目录路径
     * @param archiveDirectory 存档目录路径
     * @param databaseAccessor 数据库访问器
     * @throws IOException 如果操作过程中发生IO错误
     *                     <p>
     *                     获取根目录所有文件的列表(包括文件路径, 以及文件大小)
     *                     如果文件大小超过存档限制大小，则临时跳过, 最后统一分卷处理大文件
     *                     执行存档
     */
    public static void archive(
            String rootPath,
            String archiveDirectory,
            long archiveLimitSize,
            DatabaseAccessor databaseAccessor) throws IOException {
        Path rootAbsolutePath = Path.of(rootPath).toAbsolutePath();
        // 查找根目录ID
        Long rootId = databaseAccessor.findRootIdByPath(rootAbsolutePath.toString());
        if (rootId == null) {
            log.error("根目录 {} 不存在于数据库中", rootPath);
            return;
        }

        // 获取该根目录下的所有文件
        List<ViewFile> fileList = databaseAccessor.findViewFilesByRootId(rootId);
        log.info("找到 {} 个文件需要添加到存档", fileList.size());

        // 初始化存档上下文
        ArchiveContext context = ArchiveContext.builder()
                .archiveLimitSize(archiveLimitSize)
                .archiveDirectory(archiveDirectory)
                .databaseAccessor(databaseAccessor)
                .archivePrefix("archive")
                .archiveCounter(1)
                .build();

        try {
            // 遍历文件并添加到存档
            processFiles(fileList, context);
        } finally {
            // 确保输出流被正确关闭
            closeTarOutputAndDoneArchive(context.getTarOutput(), context.getArchiveId(), databaseAccessor);
        }

        log.info("所有文件已添加到存档目录 {}", archiveDirectory);
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
        String fileHash = calculateFileHash(filePath);

        // 检查是否已存在相同文件
        FileInfo fileByHash = context.getDatabaseAccessor().findHealthFileByHash(fileHash);

        List<Long> assetIds;
        FileProcessingStrategy strategy;
        if (fileByHash == null) {
            // 处理新文件
            strategy = new NewFileProcessingStrategy();
            assetIds = strategy.process(viewFile, filePath, context);
        } else {
            // 处理已存在的文件
            strategy = new ExistingFileProcessingStrategy();
            assetIds = strategy.process(viewFile, filePath, context);
        }

        // 创建文件索引
        createFileIndexes(viewFile, assetIds, context);
        addHashAndMarkFileHealthy(context.getDatabaseAccessor(), viewFile.getFileId(), fileHash);

        log.info("文件 {} 已添加到存档", filePath);
    }

    static void addHashAndMarkFileHealthy(DatabaseAccessor databaseAccessor, Long fileId, String fileHash) {
        // 添加 hash，更新文件状态为健康
        FileInfo fileInfo = databaseAccessor.findFileInfoById(fileId);
        fileInfo.setStatus("HEALTH");
        fileInfo.setHash(fileHash);
        fileInfo.setUpdatedAt(System.currentTimeMillis() / 1000);

        // 更新数据库
        databaseAccessor.updateFileInfo(fileInfo);
    }

    private static String calculateFileHash(Path filePath) throws IOException {
        try (InputStream fis = Files.newInputStream(filePath)) {
            return DigestUtils.sha256Hex(fis);
        }
    }

    private static void createFileIndexes(ViewFile viewFile, List<Long> assetIds, ArchiveContext context) throws IOException {
        long volumeOrder = 1;
        for (long assetId : assetIds) {
            // 插入文件索引
            FileVolumeAsset fileVolumeAsset = new FileVolumeAsset();
            fileVolumeAsset.setFileId(viewFile.getFileId());
            fileVolumeAsset.setAssetId(assetId);
            fileVolumeAsset.setStatus("HEALTH");
            fileVolumeAsset.setVolumeOrder(volumeOrder++);

            context.getDatabaseAccessor().insertFileVolumeAsset(fileVolumeAsset);
        }
    }


    static void closeTarOutputAndDoneArchive(TarArchiveOutputStream tarOutput, Long archiveId, DatabaseAccessor databaseAccessor) {
        if (tarOutput != null) {
            try {
                if (archiveId != null) {
                    ArchiveMetadata archiveMetadataById = databaseAccessor.findArchiveMetadataById(archiveId);
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
        archiveMetadataById.setStatus("HEALTH");
        archiveMetadataById.setUpdatedAt(System.currentTimeMillis() / 1000);
        databaseAccessor.updateArchiveMetadata(archiveMetadataById);
    }

}
