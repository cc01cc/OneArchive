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

import com.cc01cc.onearchive.core.dao.DatabaseAccessor;
import com.cc01cc.onearchive.core.entity.ViewAsset;
import com.cc01cc.onearchive.core.entity.ViewFile;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.compress.archivers.tar.TarArchiveEntry;
import org.apache.commons.compress.archivers.tar.TarArchiveInputStream;

import java.io.IOException;
import java.io.OutputStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.attribute.FileTime;
import java.util.List;


/**
 * 解档
 */
@Slf4j
public class ArchiveOut {

    /**
     * 1. 根据 rootPath 获取所有的 文件以及关联的 asset
     * 2. 根据 file 和 asset 信息 从 archive 中解压文件
     *
     * @param rootDir
     * @param archiveDir          存放存档的目录
     * @param unarchiveTargetPath
     */
    public static void unArchive(String rootDir, String archiveDir, String unarchiveTargetPath, DatabaseAccessor databaseAccessor) {
        List<ViewFile> viewFileList = ArchiveCore.getViewFileList(rootDir, databaseAccessor);
        if (viewFileList == null) {
            log.warn("没有找到根目录 {} 下的文件", rootDir);
            return;
        }
        log.info("找到 {} 个文件需要解压", viewFileList.size());

        for (ViewFile viewFile : viewFileList) {
            try {
                unArchiveFile(viewFile, archiveDir, unarchiveTargetPath, databaseAccessor);
            } catch (IOException e) {
                throw new RuntimeException(e);
            }
        }

    }


    private static void unArchiveFile(ViewFile viewFile, String archiveDir, String unarchiveTargetPath, DatabaseAccessor databaseAccessor) throws IOException {
        // 获取文件关联的资产信息
        List<ViewAsset> viewAssets = databaseAccessor.findViewAssetByFileId(viewFile.getFileId());

        if (viewAssets == null || viewAssets.isEmpty()) {
            log.warn("文件 {} 没有找到关联的资产", viewFile.getFileName());
            return;
        }

        // 创建目标目录
        Path targetDir = Path.of(unarchiveTargetPath, viewFile.getDirectoryPath());
        Files.createDirectories(targetDir);

        // 目标文件路径
        Path targetFilePath = targetDir.resolve(viewFile.getFileName());
        // 如果只有一个资产，直接解压
        if (viewAssets.size() == 1) {
            unArchiveSingleFile(archiveDir, viewAssets, targetFilePath);
        } else {
            unArchiveVolumeFile(viewAssets, archiveDir, targetFilePath);
        }
        // 设置文件的修改时间
        FileTime fileTime = FileTime.fromMillis(viewFile.getFileMtime());
        Files.setLastModifiedTime(targetFilePath, fileTime);

        log.info("成功解压文件: {}", targetFilePath);
    }

    private static void unArchiveSingleFile(String archiveDir, List<ViewAsset> viewAssetList, Path targetFilePath) throws IOException {

        // 解压目录, 以及相对路径
        ViewAsset viewAsset = viewAssetList.getFirst();
        String archiveName = viewAsset.getArchiveName();
        String archivePath = archiveDir + "/" + archiveName;

        // 使用Apache Commons Compress解压tar文件
        try (TarArchiveInputStream tarInput = new TarArchiveInputStream(Files.newInputStream(Path.of(archivePath)))) {
            TarArchiveEntry entry;
            while ((entry = tarInput.getNextEntry()) != null) {
                if (entry.getName().equals(viewAsset.getAssetRelativePath())) {
                    // 找到对应的资产，写入文件
                    try (OutputStream out = Files.newOutputStream(targetFilePath)) {
                        tarInput.transferTo(out);
                    }
                    break;
                }
            }
        }


    }

    private static void unArchiveVolumeFile(List<ViewAsset> viewAssetList, String archiveDir, Path targetFilePath) throws IOException {
        // 处理分卷文件
        try (OutputStream out = Files.newOutputStream(targetFilePath)) {
            for (ViewAsset viewAsset : viewAssetList) {
                String archiveName = viewAsset.getArchiveName();
                String archivePath = archiveDir + "/" + archiveName;

                // 从对应的存档中提取资产数据
                try (TarArchiveInputStream tarInput = new TarArchiveInputStream(Files.newInputStream(Path.of(archivePath)))) {
                    TarArchiveEntry entry;
                    while ((entry = tarInput.getNextEntry()) != null) {
                        if (entry.getName().equals(viewAsset.getAssetRelativePath())) {
                            // 找到对应的资产，追加写入文件
                            tarInput.transferTo(out);
                            break;
                        }
                    }
                } catch (IOException e) {
                    throw new RuntimeException(e);
                }
            }
        }
    }
}