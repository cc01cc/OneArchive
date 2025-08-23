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

import com.cc01cc.onearchive.core.DirectoryScanner;
import com.cc01cc.onearchive.core.dao.DatabaseAccessor;
import lombok.extern.slf4j.Slf4j;

import java.io.IOException;
import java.nio.file.Path;

/**
 * 归档服务类
 * 封装归档和解档的核心业务逻辑
 */
@Slf4j
public class ArchiveService {

    /**
     * 执行归档操作
     *
     * @param rootDir 根目录
     * @param dbPath  数据库路径
     * @param context 归档上下文
     * @throws IOException IO异常
     */
    public void performArchive(String rootDir, String dbPath, ArchiveContext context) throws IOException {
        ArchiveIn.archive(rootDir, dbPath);
    }

    /**
     * 执行解档操作
     *
     * @param rootDir             根目录
     * @param archiveDir          存档目录
     * @param unarchiveTargetPath 解档目标路径
     * @param databaseAccessor    数据库访问器
     */
    public void performUnarchive(String rootDir, String archiveDir, String unarchiveTargetPath, DatabaseAccessor databaseAccessor) {
        ArchiveOut.unArchive(rootDir, archiveDir, unarchiveTargetPath, databaseAccessor);
    }

    /**
     * 扫描并保存目录信息
     *
     * @param rootPath         根路径
     * @param databaseAccessor 数据库访问器
     * @throws IOException IO异常
     */
    public void scanAndSaveDirectory(Path rootPath, DatabaseAccessor databaseAccessor) throws IOException {
        DirectoryScanner.scanAndSaveDirectory(rootPath, databaseAccessor);
    }
}
