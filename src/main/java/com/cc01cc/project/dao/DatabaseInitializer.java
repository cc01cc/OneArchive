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

package com.cc01cc.project.dao;

import lombok.extern.slf4j.Slf4j;

import java.sql.Connection;
import java.sql.DriverManager;
import java.sql.SQLException;
import java.sql.Statement;

/**
 * 数据库初始化类
 * 用于在程序第一次运行时创建SQLite数据库和表结构
 */
@Slf4j
public class DatabaseInitializer {

    private final String dbUrl;

    public DatabaseInitializer(String dbUrl) {
        this.dbUrl = dbUrl;
    }

    /**
     * 初始化SQLite数据库并创建所需的表
     */
    public void initializeDatabase() {
        try (Connection conn = DriverManager.getConnection(dbUrl)) {
            if (conn != null) {
                log.info("成功连接到SQLite数据库: {}", dbUrl);
                createTables(conn);
            }
        } catch (SQLException e) {
            log.error("初始化数据库时发生错误", e);
            throw new RuntimeException("数据库初始化失败", e);
        }
    }

    /**
     * 创建数据库表
     *
     * @param conn 数据库连接
     * @throws SQLException SQL异常
     */
    private void createTables(Connection conn) throws SQLException {
        Statement stmt = conn.createStatement();

        // 创建根目录索引表 root_index
        String createRootIndexTable = """
                CREATE TABLE IF NOT EXISTS root_index (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    root_path TEXT NOT NULL UNIQUE,
                    status TEXT NOT NULL DEFAULT 'HEALTH',
                    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
                    updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
                );
                """;
        stmt.execute(createRootIndexTable);
        log.info("已创建 root_index 表");

        // 创建目录索引表 directory_index
        String createDirectoryIndexTable = """
                CREATE TABLE IF NOT EXISTS directory_index (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    root_id INTEGER NOT NULL,
                    directory_name TEXT NOT NULL DEFAULT '',
                    directory_mtime INTEGER NOT NULL DEFAULT 0,
                    directory_path TEXT,
                    status TEXT NOT NULL DEFAULT 'HEALTH',
                    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
                    updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
                    FOREIGN KEY (root_id) REFERENCES root_index(id),
                    UNIQUE (root_id, directory_path)
                );
                """;
        stmt.execute(createDirectoryIndexTable);
        log.info("已创建 directory_index 表");

        // 创建目录树关系表 - 闭包表 (directory_tree)
        String createDirectoryTreeTable = """
                CREATE TABLE IF NOT EXISTS directory_tree (
                    ancestor_id INTEGER NOT NULL,
                    descendant_id INTEGER NOT NULL,
                    depth INTEGER NOT NULL,
                    status TEXT NOT NULL DEFAULT 'HEALTH',
                    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
                    updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
                    PRIMARY KEY (ancestor_id, descendant_id),
                    FOREIGN KEY (ancestor_id) REFERENCES directory_index(id),
                    FOREIGN KEY (descendant_id) REFERENCES directory_index(id)
                );
                """;
        stmt.execute(createDirectoryTreeTable);
        log.info("已创建 directory_tree 表");

        // 创建文件索引表 file_index
        String createFileIndexTable = """
                CREATE TABLE IF NOT EXISTS file_index (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    directory_id INTEGER NOT NULL,
                    file_name TEXT NOT NULL,
                    file_size INTEGER NOT NULL DEFAULT 0,
                    file_mtime INTEGER NOT NULL DEFAULT 0,
                    file_hash TEXT,
                    volume_count INTEGER ,
                    status TEXT NOT NULL DEFAULT 'HEALTH',
                    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
                    updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
                    UNIQUE (directory_id, file_name, file_hash),
                    FOREIGN KEY (directory_id) REFERENCES directory_index(id)
                );
                """;
        stmt.execute(createFileIndexTable);
        log.info("已创建 file_index 表");

        // 创建存档元数据表 (archive_metadata)
        String createArchiveMetadataTable = """
                CREATE TABLE IF NOT EXISTS archive_metadata (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    name TEXT UNIQUE NOT NULL,
                    archive_limit_size INTEGER NOT NULL,
                    archive_hash TEXT,
                    is_compressed INTEGER NOT NULL DEFAULT 0,
                    compressed_algorithm TEXT,
                    is_encrypted INTEGER NOT NULL DEFAULT 0,
                    encryption_algorithm TEXT,
                    status TEXT NOT NULL DEFAULT 'HEALTH',
                    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
                    updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
                );
                """;
        stmt.execute(createArchiveMetadataTable);
        log.info("已创建 archive_metadata 表");

        // 创建存档内容表 (archive_asset)
        String createArchiveAssetTable = """
                CREATE TABLE IF NOT EXISTS archive_asset (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    archive_id INTEGER NOT NULL,
                    asset_name TEXT NOT NULL UNIQUE,
                    asset_size INTEGER NOT NULL DEFAULT 0,
                    asset_hash TEXT NOT NULL,
                    asset_mtime INTEGER NOT NULL DEFAULT 0,
                    relative_path TEXT NOT NULL DEFAULT './',
                    status TEXT NOT NULL DEFAULT 'HEALTH',
                    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
                    updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
                    FOREIGN KEY (archive_id) REFERENCES archive_metadata(id)
                );
                """;
        stmt.execute(createArchiveAssetTable);
        log.info("已创建 archive_asset 表");

        // 创建文件与资源映射表 file_volume_asset
        String createFileVolumeAssetTable = """
                CREATE TABLE IF NOT EXISTS file_volume_asset (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    file_id INTEGER NOT NULL,
                    asset_id INTEGER NOT NULL,
                    volume_order INTEGER NOT NULL DEFAULT 1,
                    status TEXT NOT NULL DEFAULT 'HEALTH',
                    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
                    updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
                    FOREIGN KEY (file_id) REFERENCES file_index(id),
                    FOREIGN KEY (asset_id) REFERENCES archive_asset(id)
                );
                """;
        stmt.execute(createFileVolumeAssetTable);
        log.info("已创建 file_volume_asset 表");

        // 创建视图：按根目录查看文件 (file list view)
        String createFilesByRootView = """
                CREATE VIEW IF NOT EXISTS v_file AS
                SELECT
                    r.id as root_id,
                    r.root_path,
                    r.status as root_status,
                    d.id as directory_id,
                    d.directory_path,
                    d.directory_mtime,
                    d.status as directory_status,
                    f.id as file_id,
                    f.file_name,
                    f.file_size,
                    f.file_mtime,
                    f.file_hash,
                    f.volume_count,
                    f.status as file_status
                FROM root_index r
                JOIN directory_index d ON r.id = d.root_id
                JOIN file_index f ON d.id = f.directory_id
                """;
        stmt.execute(createFilesByRootView);
        log.info("已创建 v_file 视图");

        // 创建视图：资源列表视图 (asset list view)
        String createAssetListView = """
                CREATE VIEW IF NOT EXISTS v_asset AS
                SELECT
                    am.id as archive_id,
                    am.name as archive_name,
                    am.status as archive_status,
                    aa.id as asset_id,
                    aa.asset_name,
                    aa.asset_size,
                    aa.asset_hash,
                    aa.asset_mtime,
                    aa.relative_path as asset_relative_path,
                    aa.status as asset_status,
                    fva.file_id,
                    fva.volume_order
                FROM archive_metadata am
                JOIN archive_asset aa ON am.id = aa.archive_id
                LEFT JOIN file_volume_asset fva ON aa.id = fva.asset_id
                """;
        stmt.execute(createAssetListView);
        log.info("已创建 v_asset 视图");

    }
}
