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
import com.cc01cc.project.dto.DirectoryInfo;
import com.cc01cc.project.dto.FileInfo;
import lombok.NonNull;
import lombok.extern.slf4j.Slf4j;

import java.io.IOException;
import java.nio.file.FileVisitResult;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.SimpleFileVisitor;
import java.nio.file.attribute.BasicFileAttributes;

/**
 * 目录扫描器，负责扫描目录结构并保存到数据库
 */
@Slf4j
public class DirectoryScanner {

    private DirectoryScanner() {
    }

    /**
     * 扫描资源和写入资源一起执行, 每扫描到一个资源, 就写入数据库, 避免内存溢出
     *
     * @param startPath        起始路径
     * @param databaseAccessor 数据库访问器
     * @throws IOException 如果扫描过程中发生IO错误
     */
    public static void scanAndSaveDirectory(Path startPath, DatabaseAccessor databaseAccessor) throws IOException {
        // 根目录-需要转为绝对路径
        String rootAbsolutePath = startPath.toAbsolutePath().toString();
        Long tempId = databaseAccessor.findRootIdByPath(rootAbsolutePath);
        // 插入根目录
        if (tempId == null) {
            tempId = databaseAccessor.insertRootPath(rootAbsolutePath);
        }
        long rootId = tempId;

        Files.walkFileTree(startPath, new SimpleFileVisitor<Path>() {
            @Override
            public FileVisitResult preVisitDirectory(Path dir, BasicFileAttributes attrs) throws IOException {
                DirectoryInfo directoryInfo = new DirectoryInfo();
                directoryInfo.setName(dir.getFileName().toString());
                directoryInfo.setPath(startPath.relativize(dir).toString());
                directoryInfo.setMtime(attrs.lastModifiedTime().toMillis());
                directoryInfo.setRootId(rootId);
                long id = databaseAccessor.insertDirectory(directoryInfo);

                log.info("dir is directory: {}", attrs.isDirectory());
                log.info("dir name: {}", dir.getFileName().toString());
                log.info("dir full path: {}", dir.toAbsolutePath());
                log.info("dir mtime: {}", attrs.lastModifiedTime().toMillis());
                log.info("dir id: {}", id);

                return FileVisitResult.CONTINUE;
            }

            @Override
            public FileVisitResult visitFile(@NonNull Path file, @NonNull BasicFileAttributes attrs) {
                FileInfo fileInfo = new FileInfo();
                fileInfo.setName(file.getFileName().toString());
                fileInfo.setSize(attrs.size());
                fileInfo.setMtime(attrs.lastModifiedTime().toMillis());
                String relativePath = startPath.relativize(file).getParent() != null ?
                        startPath.relativize(file).getParent().toString() : "";
                long directoryId = databaseAccessor.findDirectoryIdByPath(rootId, relativePath);
                fileInfo.setDirectoryId(directoryId);
                fileInfo.setStatus("WAIT_TO_ARCHIVE");
                long id = databaseAccessor.insertFile(fileInfo);

                log.info("file id: {}", id);
                log.info("file name: {}", file.getFileName().toString());
                log.info("file full path: {}", file.toAbsolutePath());
                log.info("file size: {}", attrs.size());
                log.info("file mtime: {}", attrs.lastModifiedTime().toMillis());

                return FileVisitResult.CONTINUE;
            }
        });
    }
}