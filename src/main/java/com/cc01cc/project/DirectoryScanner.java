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

import com.cc01cc.project.constant.DatabaseTables;
import com.cc01cc.project.constant.DirectoryStatus;
import com.cc01cc.project.constant.FileStatus;
import com.cc01cc.project.constant.RootStatus;
import com.cc01cc.project.dao.DatabaseAccessor;
import com.cc01cc.project.entity.DirectoryInfo;
import com.cc01cc.project.entity.FileInfo;
import com.cc01cc.project.entity.RootIndex;
import lombok.NonNull;
import lombok.extern.slf4j.Slf4j;
import oshi.SystemInfo;
import oshi.software.os.OSFileStore;

import java.io.IOException;
import java.nio.file.FileVisitResult;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.SimpleFileVisitor;
import java.nio.file.attribute.BasicFileAttributes;
import java.util.*;

/**
 * 目录扫描器，负责扫描目录结构并保存到数据库
 */
@Slf4j
public class DirectoryScanner {

    private DirectoryScanner() {
    }

    /**
     * 扫描目录并写入或更新数据库
     * <p>
     * 1. 将 rootPath 标记为 更新中
     * 2. 将所有 dir 以及 file 标记为 待删除
     * 3. 扫描目录, 已经存在的子目录更新mtime 然后还原回原来的状态; 已存在的文件还原回原来的状态; 新的文件标记为 未存档
     *
     * @param startPath        根目录路径
     * @param databaseAccessor 数据库访问器
     * @throws IOException 如果扫描过程中发生IO错误
     */
    public static void scanAndSaveDirectory(Path startPath, DatabaseAccessor databaseAccessor) throws IOException {
        // 根目录-需要转为绝对路径
        String rootAbsolutePath = startPath.toAbsolutePath().toString();
        RootIndex rootIndex = databaseAccessor.findRootInfoByPath(rootAbsolutePath);
        long rootId;
        List<DirectoryInfo> dirListInDb = new ArrayList<>();
        List<FileInfo> fileListInDb = new ArrayList<>();
        if (rootIndex == null) {
            // 如果根目录不存在，则创建根目录
            log.info("根目录{}不存在，正在创建...", rootAbsolutePath);
            rootId = databaseAccessor.insertRootPath(rootAbsolutePath);
        } else {
            rootId = rootIndex.getId();

            // 获取现有的目录和文件列表(需要在状态变更前获取, 后续需要还原状态)
            dirListInDb = databaseAccessor.findDirectoriesByStatusAndRootId(rootId, null);
            fileListInDb = databaseAccessor.findFilesByStatusRootId(rootId, null);

            log.info("开始更新根目录 {}", rootAbsolutePath);
            databaseAccessor.updateStatusById(DatabaseTables.ROOT_INDEX, rootId, RootStatus.UPDATING);

            log.info("将根目录 {} 下的所有目录标记为待删除", rootAbsolutePath);
            List<DirectoryInfo> directoryInfoList = databaseAccessor.findDirectoriesByStatusAndRootId(rootId, null);
            if (directoryInfoList != null && !directoryInfoList.isEmpty()) {
                databaseAccessor.batchUpdateStatusById(
                        DatabaseTables.DIRECTORY_INDEX,
                        directoryInfoList.stream().map(DirectoryInfo::getId).toList(),
                        DirectoryStatus.WAIT_TO_DELETE
                );
            }

            log.info("将根目录 {} 下的所有状态 HEALTH 文件标记为待删除, 其他状态如 ", rootAbsolutePath);
            List<FileInfo> fileInfoList = databaseAccessor.findFilesByStatusRootId(rootId, null);
            if (fileInfoList != null && !fileInfoList.isEmpty()) {
                databaseAccessor.batchUpdateStatusById(
                        DatabaseTables.FILE_INDEX,
                        fileInfoList.stream().map(FileInfo::getId).toList(),
                        FileStatus.WAIT_TO_DELETE
                );
            }
            log.info("所有目录和文件状态更新完成");
        }
        List<DirectoryInfo> existingDirectories = dirListInDb;
        List<FileInfo> existingFiles = fileListInDb;

        Files.walkFileTree(startPath, new SimpleFileVisitor<Path>() {
            @Override
            public FileVisitResult preVisitDirectory(Path dir, BasicFileAttributes attrs) throws IOException {
                String relativePath = startPath.relativize(dir).toString();

                DirectoryInfo directoryInfo = new DirectoryInfo();
                directoryInfo.setRootId(rootId);
                directoryInfo.setName(dir.getFileName().toString());
                directoryInfo.setPath(relativePath);
                directoryInfo.setMtime(attrs.lastModifiedTime().toMillis());
                directoryInfo.setStatus(DirectoryStatus.HEALTH);

                // 统一处理逻辑：无论首次扫描还是增量更新
                if (existingDirectories != null && !existingDirectories.isEmpty()) {
                    Optional<DirectoryInfo> existingDir = existingDirectories.stream()
                            .filter(dirUnit -> dirUnit.getPath().equals(relativePath))
                            .findFirst();
                    if (existingDir.isPresent()) {
                        directoryInfo.setId(existingDir.get().getId());
                        directoryInfo.setStatus(existingDir.get().getStatus());
                        databaseAccessor.updateDirectory(directoryInfo);
                        log.info("更新已存在的目录: {}", relativePath);
                    } else {
                        long id = databaseAccessor.insertDirectory(directoryInfo);
                        log.info("插入新目录: {}, ID: {}", relativePath, id);
                    }
                } else {
                    long id = databaseAccessor.insertDirectory(directoryInfo);
                    log.info("插入目录: {}, ID: {}", relativePath, id);
                }

                return FileVisitResult.CONTINUE;
            }

            @Override
            public FileVisitResult visitFile(@NonNull Path file, @NonNull BasicFileAttributes attrs) throws IOException {
                String relativePath = startPath.relativize(file).getParent() != null ?
                        startPath.relativize(file).getParent().toString() : "";
                long directoryId = databaseAccessor.findDirectoryIdByPath(rootId, relativePath);
                String fileName = file.getFileName().toString();
                String fileHash = CommonTool.calculateFileHash(file.toAbsolutePath());

                FileInfo fileInfo = new FileInfo();
                fileInfo.setName(fileName);
                fileInfo.setSize(attrs.size());
                fileInfo.setMtime(attrs.lastModifiedTime().toMillis());
                fileInfo.setDirectoryId(directoryId);
                fileInfo.setHash(fileHash);
                // 统一处理逻辑：无论首次扫描还是增量更新
                if (existingFiles != null && !existingFiles.isEmpty()) {
                    Optional<FileInfo> existingFile = existingFiles.stream()
                            .filter(fileUnit ->
                                    fileUnit.getDirectoryId().equals(directoryId) &&
                                            fileUnit.getName().equals(fileName) &&
                                            Objects.equals(fileUnit.getHash(), fileHash))
                            .findFirst();
                    if (existingFile.isPresent()) {
                        // 文件已存在，更新文件状态(从 wait_to_delete -> health/unarchived(原来的状态))
                        fileInfo.setId(existingFile.get().getId());
                        fileInfo.setStatus(existingFile.get().getStatus());
                        databaseAccessor.updateFileInfo(fileInfo);
                        log.info("更新已存在的文件: {}", fileName);
                    } else {
                        // 文件不存在，插入新文件
                        fileInfo.setStatus(FileStatus.UNARCHIVED);
                        long id = databaseAccessor.insertFile(fileInfo);
                        log.info("插入新文件: {}, ID: {}", fileName, id);
                    }
                } else {
                    fileInfo.setStatus(FileStatus.UNARCHIVED);
                    long id = databaseAccessor.insertFile(fileInfo);
                    log.info("插入文件: {}, ID: {}", fileName, id);
                }

                return FileVisitResult.CONTINUE;
            }
        });
    }

    /**
     * 扫描目录但不写入数据库，仅返回目录统计信息
     *
     * @param startPath 起始路径
     * @return 目录统计信息
     * @throws IOException 如果扫描过程中发生IO错误
     */
    public static DirectoryStatistics scanDirectoryOnly(Path startPath) throws IOException {
        DirectoryStatistics statistics = new DirectoryStatistics();

        // 使用OSHI获取文件系统信息
        SystemInfo si = new SystemInfo();
        List<OSFileStore> fileStores = si.getOperatingSystem().getFileSystem().getFileStores();
        log.debug("文件系统信息: ");
        for (OSFileStore fs : fileStores) {
            log.debug("  文件系统: {} (类型: {}, 总大小: {}, 可用空间: {})", fs.getName(), fs.getType(), fs.getTotalSpace(), fs.getUsableSpace());
        }

        // 用于统计硬链接的映射表（inode -> 引用计数）
        Map<Object, Integer> inodeMap = new HashMap<>();

        Files.walkFileTree(startPath, new SimpleFileVisitor<Path>() {
            private int currentDepth = 0;

            @Override
            public FileVisitResult preVisitDirectory(Path dir, BasicFileAttributes attrs) throws IOException {
                // 计算当前深度
                currentDepth = dir.getNameCount() - startPath.getNameCount();

                if (currentDepth > statistics.getMaxDepth()) {
                    statistics.setMaxDepth(currentDepth);
                }

                // 根目录不计入子目录统计
                if (!dir.equals(startPath)) {
                    statistics.setDirectoryCount(statistics.getDirectoryCount() + 1);
                }

                return FileVisitResult.CONTINUE;
            }

            @Override
            public FileVisitResult visitFile(@NonNull Path file, @NonNull BasicFileAttributes attrs) throws IOException {
                // 统计文件大小
                statistics.setTotalSize(statistics.getTotalSize() + attrs.size());

                // 增加文件计数
                statistics.setFileCount(statistics.getFileCount() + 1);

                // 检查是否为符号链接
                if (Files.isSymbolicLink(file)) {
                    statistics.setSymbolicLinkCount(statistics.getSymbolicLinkCount() + 1);
                }

                // 统计硬链接信息
                try {
                    // 获取文件的inode（在Unix-like系统上）或文件标识符（Windows上）
                    // FIXME 硬链接的统计存在异常, fileKey 返回 null
                    Object fileKey = attrs.fileKey();
                    if (fileKey != null) {
                        inodeMap.put(fileKey, inodeMap.getOrDefault(fileKey, 0) + 1);
                    }
                } catch (Exception e) {
                    log.warn("无法获取文件的inode信息: {}", file, e);
                }

                return FileVisitResult.CONTINUE;
            }

            @Override
            public FileVisitResult visitFileFailed(Path file, IOException exc) throws IOException {
                // 即使访问文件失败，也记录该文件的存在
                log.warn("无法访问文件: {}", file, exc);
                statistics.setFileCount(statistics.getFileCount() + 1);
                return FileVisitResult.CONTINUE;
            }
        });

        // 统计硬链接信息
        int hardLinkGroupCount = 0;
        int hardLinkTotalCount = 0;
        int inodeCount = 0;
        for (Map.Entry<Object, Integer> entry : inodeMap.entrySet()) {
            if (entry.getValue() > 1) { // 只有引用计数大于1的才是硬链接
                hardLinkGroupCount++;
                hardLinkTotalCount += entry.getValue();
            } else {
                inodeCount++;
            }
        }
        statistics.setHardLinkGroupCount(hardLinkGroupCount);
        statistics.setHardLinkTotalCount(hardLinkTotalCount);
        statistics.setInodeCount(inodeCount);
        return statistics;
    }


    /**
     * 扫描目录并检查文件hash
     * <p>
     * 1. 获取所有状态为 待校验 的文件
     * 2. 获取文件对应的hash
     * 5. 比较文件hash, 如果不一致, 则更新文件信息 标记为待更新; 否则更新文件信息 mtime 等后, 标记为 HEALTH(volume中的mtime暂不用更新, 没影响)
     *
     * @param path
     * @param databaseAccessor
     */
    public static void scanAndCheckFileHash(Path path, DatabaseAccessor databaseAccessor) {
    }

}