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

package com.cc01cc.onearchive.core;

import com.cc01cc.onearchive.core.archive.ArchiveIn;
import com.cc01cc.onearchive.core.archive.ArchiveOut;
import com.cc01cc.onearchive.core.dao.DatabaseAccessor;
import com.cc01cc.onearchive.core.dao.DatabaseInitializer;
import lombok.extern.slf4j.Slf4j;

import java.io.IOException;
import java.nio.file.Path;



/**
 * 归档器类
 * 封装归档和解档的核心业务逻辑
 *
 * 这是OneArchive框架对外提供的主要API入口。
 * 所有归档和解档操作都应通过此类进行，以确保正确的工作流程和数据一致性。
 */
@Slf4j
public class OneArchive {

    /**
     * 初始化数据库
     * 创建所需的表结构和视图
     *
     * @param dbPath 数据库路径
     */
    public static void initializeDatabase(String dbPath) {
        String dbUrl = "jdbc:sqlite:" + dbPath;
        DatabaseInitializer initializer = new DatabaseInitializer(dbUrl);
        initializer.initializeDatabase();
        log.info("数据库初始化完成: {}", dbPath);
    }

    /**
     * 执行归档操作
     *
     * @param rootDir 根目录
     * @param archiveDir 存档目录
     * @param archivePrefix 存档文件前缀
     * @param dbPath  数据库路径
     * @param archiveLimitSize 存档大小限制（字节）
     * @throws IOException IO异常
     */
    public static void performArchive(String rootDir, String archiveDir, String archivePrefix, String dbPath, Long archiveLimitSize) throws IOException {
        performArchive(rootDir, archiveDir, archivePrefix, dbPath, archiveLimitSize, null);
    }

    /**
     * 执行归档操作（支持进度回调）
     *
     * @param rootDir 根目录
     * @param archiveDir 存档目录
     * @param archivePrefix 存档文件前缀
     * @param dbPath  数据库路径
     * @param archiveLimitSize 存档大小限制（字节）
     * @param callback 进度回调接口
     * @throws IOException IO异常
     */
    public static void performArchive(String rootDir, String archiveDir, String archivePrefix, String dbPath, Long archiveLimitSize, ProgressCallback callback) throws IOException {
        // 可以在这里添加前置检查、日志记录等操作
        log.info("开始执行归档操作: rootDir={}, archiveDir={}", rootDir, archiveDir);
        ArchiveIn.archive(rootDir, archiveDir, archivePrefix, dbPath, archiveLimitSize, callback);
        log.info("归档操作完成");
    }

    /**
     * 执行解档操作
     *
     * @param rootDir             根目录
     * @param archiveDir          存档目录
     * @param unarchiveTargetPath 解档目标路径
     * @param databaseAccessor    数据库访问器
     */
    public static void performUnarchive(String rootDir, String archiveDir, String unarchiveTargetPath, DatabaseAccessor databaseAccessor) {
        log.info("开始执行解档操作: rootDir={}, archiveDir={}", rootDir, archiveDir);
        ArchiveOut.unArchive(rootDir, archiveDir, unarchiveTargetPath, databaseAccessor);
        log.info("解档操作完成");
    }

    /**
     * 扫描并保存目录信息
     *
     * @param rootPath         根路径
     * @param databaseAccessor 数据库访问器
     * @throws IOException IO异常
     */
    public static void scanAndSaveDirectory(Path rootPath, DatabaseAccessor databaseAccessor) throws IOException {
        DirectoryScanner.scanAndSaveDirectory(rootPath, databaseAccessor);
    }
}