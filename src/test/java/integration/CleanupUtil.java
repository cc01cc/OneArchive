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

// src/test/java/integration/CleanupUtil.java
package integration;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.sql.Connection;
import java.sql.DriverManager;
import java.sql.PreparedStatement;
import java.sql.SQLException;

/**
 * 集成测试清理工具类
 * 用于手动清理测试过程中产生的临时文件和目录
 */
public class CleanupUtil {

    private static final String[] TEST_DIRECTORIES = {
            "test-source",
            "test-archive",
            "test-restore"
    };

    private static final String[] TEST_DB_URLS = {
            "jdbc:sqlite:test_archive.sqlite",
            "jdbc:sqlite:test_archive.db"
    };

    /**
     * 手动清理所有测试相关的文件和目录
     */
    public static void cleanupAllTestResources() {
        // 删除测试目录
        for (String dir : TEST_DIRECTORIES) {
            deleteDirectory(dir);
        }

        // 清理测试数据库
        for (String dbUrl : TEST_DB_URLS) {
            cleanupDatabase(dbUrl);
        }

        System.out.println("已完成所有测试资源的清理");
    }

    /**
     * 删除指定目录及其所有内容
     *
     * @param dirPath 目录路径
     */
    private static void deleteDirectory(String dirPath) {
        Path path = Path.of(dirPath);
        if (Files.exists(path)) {
            try {
                // 递归删除目录及其内容
                java.nio.file.Files.walk(path)
                        .sorted((a, b) -> b.compareTo(a)) // 从 deepest first 开始删除
                        .forEach(file -> {
                            try {
                                Files.delete(file);
                            } catch (IOException e) {
                                System.err.println("无法删除文件: " + file + " - " + e.getMessage());
                            }
                        });
                System.out.println("已删除目录: " + dirPath);
            } catch (IOException e) {
                System.err.println("无法遍历目录: " + dirPath + " - " + e.getMessage());
            }
        } else {
            System.out.println("目录不存在，无需删除: " + dirPath);
        }
    }

    /**
     * 清理数据库中的所有数据
     *
     * @param dbUrl 数据库URL
     */
    private static void cleanupDatabase(String dbUrl) {
        String dbFile = dbUrl.replace("jdbc:sqlite:", "");
        Path dbPath = Path.of(dbFile);

        if (!Files.exists(dbPath)) {
            System.out.println("数据库文件不存在，无需清理: " + dbFile);
            return;
        }

        try (Connection conn = DriverManager.getConnection(dbUrl)) {
            // 禁用外键约束以避免删除顺序问题
            try (PreparedStatement stmt = conn.prepareStatement("PRAGMA foreign_keys = OFF")) {
                stmt.execute();
            }

            // 清空所有表的数据
            String[] tables = {
                    "file_volume_asset",
                    "archive_asset",
                    "archive_metadata",
                    "file_index",
                    "directory_tree",
                    "directory_index",
                    "root_index"
            };

            for (String table : tables) {
                try {
                    try (PreparedStatement stmt = conn.prepareStatement("DROP TABLE " + table)) {
                        stmt.executeUpdate();
                        System.out.println("已删除表 " + table);
                    }
                } catch (SQLException e) {
                    System.err.println("清空表 " + table + " 时出错: " + e.getMessage());
                }
            }

            // 重新启用外键约束
            try (PreparedStatement stmt = conn.prepareStatement("PRAGMA foreign_keys = ON")) {
                stmt.execute();
            }

            System.out.println("已清理数据库: " + dbFile);

        } catch (SQLException e) {
            System.err.println("无法连接到数据库或清理数据: " + dbUrl + " - " + e.getMessage());
            // 如果无法连接数据库，则尝试删除文件
            deleteFile(dbFile);
        }
    }

    /**
     * 删除指定文件
     *
     * @param filePath 文件路径
     */
    private static void deleteFile(String filePath) {
        try {
            Path path = Path.of(filePath);
            if (Files.exists(path)) {
                Files.delete(path);
                System.out.println("已删除文件: " + filePath);
            } else {
                System.out.println("文件不存在，无需删除: " + filePath);
            }

            // 同时尝试删除可能存在的 journal 文件
            Path journalPath = Path.of(filePath + "-journal");
            if (Files.exists(journalPath)) {
                Files.delete(journalPath);
                System.out.println("已删除 journal 文件: " + filePath + "-journal");
            }
        } catch (IOException e) {
            System.err.println("无法删除文件: " + filePath + " - " + e.getMessage());
        }
    }

    /**
     * 主方法，可以直接运行进行清理
     */
    public static void main(String[] args) {
        System.out.println("开始清理集成测试资源...");
        cleanupAllTestResources();
        System.out.println("清理完成。");
        System.out.println("注意：如果仍有文件无法删除，请确保没有测试进程正在运行。");
    }
}
