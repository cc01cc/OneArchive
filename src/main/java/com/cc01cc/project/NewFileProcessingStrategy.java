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
import com.cc01cc.project.dto.ArchiveAsset;
import com.cc01cc.project.dto.ArchiveContext;
import com.cc01cc.project.dto.ArchiveMetadata;
import com.cc01cc.project.dto.ViewFile;
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

@Slf4j
// 处理新文件的策略
class NewFileProcessingStrategy implements FileProcessingStrategy {
    /**
     * 创建一个新的tar存档文件，自动命名
     *
     * @param archiveDirectory 存档目录路径
     * @return 新创建的tar文件路径
     * @throws IOException 如果创建过程中发生IO错误
     */
    private static Path createNewArchiveFile(String archiveDirectory, String archiveName, long archiveCounter) throws IOException {
        Path archiveDirPath = Path.of(archiveDirectory);

        // 确保存档目录存在且为空
        if (!Files.exists(archiveDirPath)) {
            Files.createDirectories(archiveDirPath);
        } else if (!Files.isDirectory(archiveDirPath)) {
            throw new IOException("存档路径不是一个目录: " + archiveDirectory);
        }

        // 生成存档文件名
        String archiveFileName = String.format("%s_%03d.tar", archiveName, archiveCounter);
        Path archiveFilePath = archiveDirPath.resolve(archiveFileName);

        // 创建空的tar文件
        try (TarArchiveOutputStream tarOutput = new TarArchiveOutputStream(Files.newOutputStream(archiveFilePath))) {
            tarOutput.setLongFileMode(TarArchiveOutputStream.LONGFILE_GNU);
            // 空的tar文件只需要正确关闭输出流即可
            log.info("成功创建空的tar存档文件: {}", archiveFilePath);
        }

        return archiveFilePath;
    }

    @Override
    public List<Long> process(ViewFile viewFile, Path filePath, ArchiveContext context) throws IOException {
        List<Long> assetIds = new ArrayList<>();
        long fileOffset = 0;
        long fileRemainingSize = viewFile.getFileSize();

        while (fileRemainingSize > 0) {
            // 确保存档可用
            ensureArchiveAvailable(context, fileRemainingSize);

            // 计算本次可以写入的数据量, 需要考虑512字节的Tar头
            long chunkSize = Math.min(fileRemainingSize,
                    context.getArchiveLimitSize() - context.getCurrentArchiveSize() - 512L);

            // 读取数据块
            byte[] chunkData = getFileChunkBytes(filePath, fileOffset, chunkSize);

            // 计算hash
            String hash = DigestUtils.sha256Hex(chunkData);

            // 创建资产
            ArchiveAsset newAsset = new ArchiveAsset();
            newAsset.setAssetName(hash);
            newAsset.setAssetHash(hash);
            newAsset.setAssetSize(chunkSize);
            newAsset.setAssetMtime(viewFile.getFileMtime());
            newAsset.setRelativePath(hash);
            newAsset.setArchiveId(context.getArchiveId());
            newAsset.setStatus("WAIT_TO_ARCHIVE");
            // 插入资产
            long assetId = context.getDatabaseAccessor().insertArchiveAsset(newAsset);

            newAsset.setId(assetId);

            assetIds.add(assetId);

            // 添加到存档
            addAssetToArchive(context.getTarOutput(), chunkData, newAsset);
            log.info("成功添加资产到存档: {}", assetId);

            markAssetAsHealthy(context.getDatabaseAccessor(), newAsset);

            // 更新 file 状态
            fileOffset += chunkSize;
            fileRemainingSize -= chunkSize;
            // 更新 context 状态
            // 由于 tar 会添加文件头信息, 所以需要减去文件头信息大小 或 直接读取当前文件大小
            long newCurrentArchiveFileSize = Files.size(context.getCurrentArchiveFile());
            context.setCurrentArchiveSize(newCurrentArchiveFileSize);
        }

        return assetIds;
    }

    private void ensureArchiveAvailable(ArchiveContext context, long requiredSize) throws IOException {
        if (!context.needNewArchive(requiredSize)) {
            return;
        }

        // 关闭当前存档
        closeCurrentArchive(context);

        // 创建新存档
        createNewArchive(context);
    }

    private void closeCurrentArchive(ArchiveContext context) throws IOException {
        if (context.getTarOutput() != null) {
            try {
                if (context.getArchiveId() != null) {
                    ArchiveMetadata archiveMetadata = context.getDatabaseAccessor()
                            .findArchiveMetadataById(context.getArchiveId());
                    if (archiveMetadata != null) {
                        // TODO 数据库写入存档的 hash
                        markArchiveAsHealthy(context.getDatabaseAccessor(), archiveMetadata);
                    }
                }
                context.getTarOutput().finish();
                context.getTarOutput().close();
            } finally {
                context.setTarOutput(null);
            }
        }
    }

    private void createNewArchive(ArchiveContext context) throws IOException {
        // 重置存档信息
        context.reset();

        // 创建新存档文件
        context.setCurrentArchiveFile(
                createNewArchiveFile(context.getArchiveDirectory(), context.getArchivePrefix(), context.getArchiveCounter()));

        // 初始化tar输出流
        context.setTarOutput(
                new TarArchiveOutputStream(Files.newOutputStream(context.getCurrentArchiveFile())));
        context.getTarOutput().setLongFileMode(TarArchiveOutputStream.LONGFILE_GNU);

        // 创建存档元数据
        ArchiveMetadata metadata = new ArchiveMetadata();

        metadata.setName(context.getCurrentArchiveFile().getFileName().toString());
        metadata.setArchiveLimitSize(context.getArchiveLimitSize());
        metadata.setStatus("WAIT_TO_ARCHIVE");

        long archiveId = context.getDatabaseAccessor().insertArchiveMetadata(metadata);
        context.setArchiveId(archiveId);
        context.setArchiveCounter(context.getArchiveCounter() + 1);
    }

    private void markArchiveAsHealthy(DatabaseAccessor databaseAccessor, ArchiveMetadata archiveMetadata) {
        archiveMetadata.setStatus("HEALTH");
        archiveMetadata.setUpdatedAt(System.currentTimeMillis() / 1000);
        databaseAccessor.updateArchiveMetadata(archiveMetadata);
    }

    private void markAssetAsHealthy(DatabaseAccessor databaseAccessor, ArchiveAsset asset) {
        asset.setStatus("HEALTH");
        asset.setUpdatedAt(System.currentTimeMillis() / 1000);
        databaseAccessor.updateArchiveAsset(asset);
    }

    private void addAssetToArchive(TarArchiveOutputStream tarOutput, byte[] chunkData, ArchiveAsset asset) throws IOException {
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

    private byte[] getFileChunkBytes(Path filePath, long fileOffset, long chunkSize) throws IOException {
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
        return chunkData;
    }
}
