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

import com.cc01cc.project.archive.ArchiveContext;
import com.cc01cc.project.config.ArchiveConfig;
import com.cc01cc.project.config.ConfigManager;
import com.cc01cc.project.dao.DatabaseAccessor;
import com.cc01cc.project.dao.DatabaseInitializer;
import lombok.extern.slf4j.Slf4j;

import java.io.IOException;
import java.nio.file.Path;

/**
 * @author cc01cc
 * @createDate 2025-07-21 21:41
 */

@Slf4j
public class OneArchive {
    public static void main(String[] args) throws IOException {
        log.info("Hello, World!");

        // 从配置文件加载参数
        ArchiveConfig config = ConfigManager.loadConfig();

        DirectoryStatistics directoryStatistics = DirectoryScanner.scanDirectoryOnly(Path.of(config.getRootDir()));
        log.info("目录统计结果: {}", directoryStatistics);

        String dbUrl = "jdbc:sqlite:" + config.getSqlitePath();
        String testRootDir = config.getRootDir();
        String testArchiveDir = config.getArchive().getDirectory();
        String testUnArchiveDir = config.getUnarchive().getDirectory();
        long archiveLimitSize = config.getArchive().getLimitSize();

        // 初始化数据库
        DatabaseInitializer dbInitializer = new DatabaseInitializer(dbUrl);
        dbInitializer.initializeDatabase();

        DatabaseAccessor databaseAccessor = new DatabaseAccessor(dbUrl);

        // 扫描目录并保存到数据库
        DirectoryScanner.scanAndSaveDirectory(Path.of(testRootDir), databaseAccessor);

//        // 添加文件到tar文件
//        ArchiveIn.archive(testRootDir, testArchiveDir, archiveLimitSize, databaseAccessor);
//
//        // 解档文件
//        ArchiveOut.unArchive(testRootDir, testArchiveDir, testUnArchiveDir, databaseAccessor);
        // 初始化存档上下文
        ArchiveContext context = ArchiveContext.builder()
                .archiveLimitSize(archiveLimitSize)
                .archiveDirectory(testArchiveDir)
                .databaseAccessor(databaseAccessor)
                .archivePrefix("archive")
                .archiveCounter(1)
                .build();
    }
}