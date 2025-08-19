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
        String dbUrl = "jdbc:sqlite:" + "one_archive.sqlite";

        String testRootDir = "W:\\zeolab\\w05-P";
        String testArchiveDir = "W:\\zeolab\\test-archive";

        // 初始化数据库
        DatabaseInitializer dbInitializer = new DatabaseInitializer(dbUrl);
        dbInitializer.initializeDatabase();

        DatabaseAccessor databaseAccessor = new DatabaseAccessor(dbUrl);
        DirectoryScanner.scanAndSaveDirectory(Path.of(testRootDir), databaseAccessor);

        // 添加文件到tar文件
        ArchiveIn.archive(testRootDir, testArchiveDir, 1024 * 1024L * 1024, databaseAccessor);
        String testUnArchiveDir = "W:\\zeolab\\test-unarchive";
        ArchiveOut.unArchive(testRootDir, testArchiveDir, testUnArchiveDir, databaseAccessor);
    }
}
