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

package unit.com.cc01cc.project.dao;

import com.cc01cc.project.dao.DatabaseInitializer;
import org.junit.jupiter.api.AfterEach;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;

import java.io.File;
import java.sql.*;

import static org.junit.jupiter.api.Assertions.*;

/**
 * DatabaseInitializer 的单元测试类
 */
class DatabaseInitializerTest {

    private static final String TEST_DB_URL = "jdbc:sqlite:test_database.db";
    private DatabaseInitializer databaseInitializer;
    private Connection connection;

    @BeforeEach
    void setUp() {
        databaseInitializer = new DatabaseInitializer(TEST_DB_URL);
        databaseInitializer.initializeDatabase();

        try {
            connection = DriverManager.getConnection(TEST_DB_URL);
        } catch (SQLException e) {
            fail("无法建立数据库连接: " + e.getMessage());
        }
    }

    @AfterEach
    void tearDown() {
        try {
            if (connection != null && !connection.isClosed()) {
                connection.close();
            }

            // 删除测试数据库文件
            File dbFile = new File("test_database.db");
            if (dbFile.exists()) {
                dbFile.delete();
            }

            File dbJournalFile = new File("test_database.db-journal");
            if (dbJournalFile.exists()) {
                dbJournalFile.delete();
            }
        } catch (SQLException e) {
            fail("清理数据库连接失败: " + e.getMessage());
        }
    }

    @Test
    @DisplayName("测试数据库初始化成功")
    void testInitializeDatabaseSuccess() {
        assertNotNull(databaseInitializer);
        assertDoesNotThrow(() -> databaseInitializer.initializeDatabase());
    }

    @Test
    @DisplayName("测试 root_index 表是否正确创建")
    void testRootIndexTableCreation() throws SQLException {
        assertTrue(tableExists("root_index"));

        // 检查表结构
        try (Statement stmt = connection.createStatement();
             ResultSet rs = stmt.executeQuery("PRAGMA table_info(root_index)")) {

            assertTrue(rs.next());
            assertEquals("id", rs.getString("name"));
            assertTrue(rs.next());
            assertEquals("root_path", rs.getString("name"));
            assertTrue(rs.next());
            assertEquals("status", rs.getString("name"));
            assertTrue(rs.next());
            assertEquals("created_at", rs.getString("name"));
            assertTrue(rs.next());
            assertEquals("updated_at", rs.getString("name"));
        }
    }

    @Test
    @DisplayName("测试 directory_index 表是否正确创建")
    void testDirectoryIndexTableCreation() throws SQLException {
        assertTrue(tableExists("directory_index"));
    }

    @Test
    @DisplayName("测试 directory_tree 表是否正确创建")
    void testDirectoryTreeTableCreation() throws SQLException {
        assertTrue(tableExists("directory_tree"));
    }

    @Test
    @DisplayName("测试 file_index 表是否正确创建")
    void testFileIndexTableCreation() throws SQLException {
        assertTrue(tableExists("file_index"));
    }

    @Test
    @DisplayName("测试 archive_metadata 表是否正确创建")
    void testArchiveMetadataTableCreation() throws SQLException {
        assertTrue(tableExists("archive_metadata"));
    }

    @Test
    @DisplayName("测试 archive_asset 表是否正确创建")
    void testArchiveAssetTableCreation() throws SQLException {
        assertTrue(tableExists("archive_asset"));
    }


    @Test
    @DisplayName("测试 v_file 视图是否正确创建")
    void testFilesByRootViewCreation() throws SQLException {
        assertTrue(viewExists("v_file"));
    }

    @Test
    @DisplayName("测试所有必需的表和视图都已创建")
    void testAllTablesAndViewsCreated() throws SQLException {
        String[] expectedTables = {
                "root_index", "directory_index", "directory_tree", "file_index",
                "archive_metadata", "archive_asset", "file_volume_asset"
        };

        String[] expectedViews = {
                "v_file"
        };

        for (String table : expectedTables) {
            assertTrue(tableExists(table), "表 " + table + " 未创建");
        }

        for (String view : expectedViews) {
            assertTrue(viewExists(view), "视图 " + view + " 未创建");
        }
    }

    /**
     * 检查指定名称的表是否存在
     *
     * @param tableName 表名
     * @return 如果表存在返回true，否则返回false
     */
    private boolean tableExists(String tableName) throws SQLException {
        try (Statement stmt = connection.createStatement();
             ResultSet rs = stmt.executeQuery(
                     "SELECT name FROM sqlite_master WHERE type='table' AND name='" + tableName + "'")) {
            return rs.next();
        }
    }

    /**
     * 检查指定名称的视图是否存在
     *
     * @param viewName 视图名
     * @return 如果视图存在返回true，否则返回false
     */
    private boolean viewExists(String viewName) throws SQLException {
        try (Statement stmt = connection.createStatement();
             ResultSet rs = stmt.executeQuery(
                     "SELECT name FROM sqlite_master WHERE type='view' AND name='" + viewName + "'")) {
            return rs.next();
        }
    }
}
