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

import com.cc01cc.project.archive.ArchiveService;
import com.cc01cc.project.config.ConfigManager;
import com.cc01cc.project.dao.DatabaseAccessor;
import com.cc01cc.project.dao.DatabaseInitializer;
import lombok.extern.slf4j.Slf4j;

import java.io.IOException;
import java.nio.file.Path;

/**
 * OneArchive核心库入口类
 * 提供归档和解档的核心功能API
 *
 * @author cc01cc
 */
@Slf4j
public class OneArchive {

    /**
     * 初始化数据库
     *
     * @param dbUrl 数据库URL
     * @return DatabaseInitializer实例
     */
    public static DatabaseInitializer initializeDatabase(String dbUrl) {
        DatabaseInitializer dbInitializer = new DatabaseInitializer(dbUrl);
        dbInitializer.initializeDatabase();
        return dbInitializer;
    }

    /**
     * 创建数据库访问器
     *
     * @param dbUrl 数据库URL
     * @return DatabaseAccessor实例
     */
    public static DatabaseAccessor createDatabaseAccessor(String dbUrl) {
        return new DatabaseAccessor(dbUrl);
    }

    /**
     * 扫描目录并保存到数据库
     *
     * @param rootPath         根目录路径
     * @param databaseAccessor 数据库访问器
     * @throws IOException IO异常
     */
    public static void scanAndSaveDirectory(Path rootPath, DatabaseAccessor databaseAccessor) throws IOException {
        DirectoryScanner.scanAndSaveDirectory(rootPath, databaseAccessor);
    }

    /**
     * 获取归档服务实例
     *
     * @return ArchiveService实例
     */
    public static ArchiveService getArchiveService() {
        return new ArchiveService();
    }

    /**
     * 获取配置管理器实例
     *
     * @return ConfigManager实例
     */
    public static ConfigManager getConfigManager() {
        return new ConfigManager();
    }

}
