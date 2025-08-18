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
import org.apache.commons.compress.archivers.tar.TarArchiveEntry;
import org.apache.commons.compress.archivers.tar.TarArchiveOutputStream;

import java.io.FileInputStream;
import java.io.IOException;
import java.io.InputStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.atomic.AtomicLong;

/**
 * 存档管理器，负责创建和管理存档文件
 */
@Slf4j
public class ArchiveManager {

    // 当前存档编号计数器
    private static final AtomicLong archiveCounter = new AtomicLong(1);

    /**
     * 创建一个新的tar存档文件，自动命名
     *
     * @param archiveDirectory 存档目录路径
     * @return 新创建的tar文件路径
     * @throws IOException 如果创建过程中发生IO错误
     */
    private static Path createNewArchiveFile(String archiveDirectory, String archiveName) throws IOException {
        Path archiveDirPath = Path.of(archiveDirectory);

        // 确保存档目录存在且为空
        if (!Files.exists(archiveDirPath)) {
            Files.createDirectories(archiveDirPath);
        } else if (!Files.isDirectory(archiveDirPath)) {
            throw new IOException("存档路径不是一个目录: " + archiveDirectory);
        }

        // 生成存档文件名
        long counter = archiveCounter.getAndIncrement();
        String archiveFileName = String.format("%s_%03d.tar", archiveName, counter);
        Path archiveFilePath = archiveDirPath.resolve(archiveFileName);

        // 创建空的tar文件
        try (TarArchiveOutputStream tarOutput = new TarArchiveOutputStream(Files.newOutputStream(archiveFilePath))) {
            tarOutput.setLongFileMode(TarArchiveOutputStream.LONGFILE_GNU);
            // 空的tar文件只需要正确关闭输出流即可
            log.info("成功创建空的tar存档文件: {}", archiveFilePath);
        }

        return archiveFilePath;
    }

    private static void addAssetToArchive(TarArchiveOutputStream tarOutput, byte[] chunkData, ArchiveAsset asset) throws IOException {
        // 创建tar条目，使用asset名称作为文件名
        TarArchiveEntry entry = new TarArchiveEntry(asset.getAssetName());
        entry.setSize(asset.getAssetSize());
        entry.setModTime(asset.getAssetMtime());

        // 添加条目到tar文件
        tarOutput.putArchiveEntry(entry);

        // 直接写入字节数组数据
        tarOutput.write(chunkData, 0, chunkData.length);

        // 完成条目写入
        tarOutput.closeArchiveEntry();
    }


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
    public static void addFilesToArchives(
            String rootPath,
            String archiveDirectory,
            long archiveLimitSize,
            DatabaseAccessor databaseAccessor) throws IOException {
        // 查找根目录ID
        long rootId = databaseAccessor.findRootIdByPath(rootPath);
        if (rootId == -1) {
            log.error("根目录 {} 不存在于数据库中", rootPath);
            return;
        }

        // 获取该根目录下的所有文件
        List<ViewFile> fileList = databaseAccessor.findViewFilesByRootId(rootId);
        log.info("找到 {} 个文件需要添加到存档", fileList.size());

        // 初始化存档相关变量
        TarArchiveOutputStream tarOutput = null;
        Path currentArchiveFile = null;
        long currentArchiveSize = 0;
        Long archiveId = null;

        try {
            // 遍历文件并添加到存档
            for (ViewFile viewFile : fileList) {
                Path filePath = Path.of(viewFile.getRootPath(), viewFile.getDirectoryPath(), viewFile.getFileName());

                if (Files.notExists(filePath)) {
                    log.warn("文件不存在: {}", filePath);
                    continue;
                }

                // 计算文件的hash值，用于重命名文件避免重复
                String fileHash;
                try (InputStream fis = Files.newInputStream(filePath)) {
                    fileHash = DigestUtils.sha256Hex(fis);
                }

                // 查询是否存在相同文件, 如果不存在, 才添加存档
                FileInfo fileByHash = databaseAccessor.findFileByHash(fileHash);
                List<Long> assetIds = new ArrayList<>();

                if (fileByHash == null) {
                    long fileOffset = 0;
                    long fileRemainingSize = viewFile.getFileSize();

                    while (fileRemainingSize > 0) {
                        // 如果当前存档已满或者还没有创建存档，则创建新存档
                        if (currentArchiveSize >= archiveLimitSize || tarOutput == null) {
                            // 关闭当前存档（如果存在）
                            if (tarOutput != null) {
                                if (archiveId != null) {
                                    ArchiveMetadata archiveMetadataById = databaseAccessor.findArchiveMetadataById(archiveId);
                                    if (archiveMetadataById != null) {
                                        archiveMetadataById.setStatus("HEALTH");
                                        archiveMetadataById.setUpdatedAt(System.currentTimeMillis());
                                        databaseAccessor.updateArchiveMetadata(archiveMetadataById);
                                    }
                                }
                                tarOutput.finish();
                                tarOutput.close();
                            }

                            // 创建新存档
                            currentArchiveFile = createNewArchiveFile(archiveDirectory, "archive");
                            tarOutput = new TarArchiveOutputStream(Files.newOutputStream(currentArchiveFile));
                            tarOutput.setLongFileMode(TarArchiveOutputStream.LONGFILE_GNU);
                            currentArchiveSize = 0;

                            ArchiveMetadata metadata = new ArchiveMetadata();
                            metadata.setName(currentArchiveFile.getFileName().toString());
                            metadata.setArchiveLimitSize(archiveLimitSize);
                            metadata.setIsCompressed(0);
                            metadata.setIsEncrypted(0);
                            metadata.setStatus("WAIT_TO_ARCHIVE");
                            metadata.setCreatedAt(System.currentTimeMillis());
                            metadata.setUpdatedAt(System.currentTimeMillis());
                            archiveId = databaseAccessor.insertArchiveMetadata(metadata);
                        }

                        // 计算本次可以写入的数据量
                        long chunkSize = Math.min(fileRemainingSize, archiveLimitSize - currentArchiveSize);

                        // 创建输入流，从文件的指定偏移量开始读取
                        byte[] chunkData;
                        try (InputStream fileInputStream = new FileInputStream(filePath.toFile())) {
                            // 跳过已经处理的部分
                            fileInputStream.skipNBytes(fileOffset);

                            // 读取整个块数据到字节数组中
                            chunkData = new byte[(int) chunkSize];
                            int totalBytesRead = 0;
                            int bytesRead;
                            while (totalBytesRead < chunkSize &&
                                    (bytesRead = fileInputStream.read(chunkData, totalBytesRead, (int) (chunkSize - totalBytesRead))) != -1) {
                                totalBytesRead += bytesRead;
                            }

                            if (totalBytesRead != chunkSize) {
                                throw new IOException("无法读取完整的数据块，期望: " + chunkSize + ", 实际: " + totalBytesRead);
                            }
                        }

                        // 计算 hash 值
                        String hash = DigestUtils.sha256Hex(chunkData);

                        ArchiveAsset newAsset = new ArchiveAsset();
                        newAsset.setAssetName(hash);
                        newAsset.setAssetHash(hash);
                        newAsset.setAssetSize(chunkSize);
                        newAsset.setAssetMtime(viewFile.getFileMtime());
                        newAsset.setArchiveId(archiveId);
                        newAsset.setRelativePath(hash);
                        newAsset.setStatus("HEALTH");

                        long assetId = databaseAccessor.insertArchiveAsset(newAsset);
                        assetIds.add(assetId);

                        // 将数据块添加到当前存档
                        addAssetToArchive(tarOutput, chunkData, newAsset);

                        // 更新状态
                        fileOffset += chunkSize;
                        fileRemainingSize -= chunkSize;
                        currentArchiveSize += chunkSize;
                    }
                } else {
                    List<FileVolumeAsset> fileVolumeAssets = databaseAccessor.findFileVolumeAssetByFileId(fileByHash.getId());

                    // 如果存在相同的资产，则使用已有的assetId
                    for (FileVolumeAsset fileVolumeAsset : fileVolumeAssets) {
                        assetIds.add(fileVolumeAsset.getAssetId());
                    }
                }

                // 为当前文件创建文件索引
                for (long assetId : assetIds) {
                    // 插入文件索引
                    FileVolumeAsset fileVolumeAsset = new FileVolumeAsset();
                    fileVolumeAsset.setFileId(viewFile.getFileId());
                    fileVolumeAsset.setAssetId(assetId);
                    fileVolumeAsset.setStatus("HEALTH");
                    fileVolumeAsset.setCreatedAt(System.currentTimeMillis());
                    fileVolumeAsset.setUpdatedAt(System.currentTimeMillis());
                    databaseAccessor.insertFileVolumeAsset(fileVolumeAsset);
                }

                log.info("文件 {} 已添加到存档", filePath);
            }
        } finally {
            // 确保输出流被正确关闭
            if (tarOutput != null) {
                try {
                    if (archiveId != null) {
                        ArchiveMetadata archiveMetadataById = databaseAccessor.findArchiveMetadataById(archiveId);
                        if (archiveMetadataById != null) {
                            archiveMetadataById.setStatus("HEALTH");
                            archiveMetadataById.setUpdatedAt(System.currentTimeMillis());
                            databaseAccessor.updateArchiveMetadata(archiveMetadataById);
                        }
                    }
                    tarOutput.finish();
                    tarOutput.close();
                } catch (IOException e) {
                    log.error("关闭存档输出流时出错", e);
                }
            }
        }

        log.info("所有文件已添加到存档目录 {}", archiveDirectory);
    }


}
