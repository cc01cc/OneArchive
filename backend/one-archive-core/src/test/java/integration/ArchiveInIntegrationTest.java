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

// ArchiveInIntegrationTest.java
package integration;

import com.cc01cc.onearchive.core.DirectoryScanner;
import com.cc01cc.onearchive.core.archive.ArchiveContext;
import com.cc01cc.onearchive.core.archive.ArchiveCore;
import com.cc01cc.onearchive.core.archive.ArchiveIn;
import com.cc01cc.onearchive.core.archive.ArchiveOut;
import com.cc01cc.onearchive.core.dao.DatabaseAccessor;
import com.cc01cc.onearchive.core.dao.DatabaseInitializer;
import org.junit.jupiter.api.*;

import java.io.BufferedOutputStream;
import java.io.IOException;
import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.sql.Connection;
import java.sql.PreparedStatement;
import java.sql.ResultSet;
import java.util.List;
import java.util.Random;

import static org.junit.jupiter.api.Assertions.*;

@TestMethodOrder(MethodOrderer.OrderAnnotation.class)
class ArchiveInIntegrationTest {

    private static final String TEST_DB_PATH = "test_archive.sqlite";
    private static final String TEST_DB_URL = "jdbc:sqlite:" + TEST_DB_PATH;
    private static final String TEST_SOURCE_DIR = "test-source";
    private static final String TEST_ARCHIVE_DIR = "test-archive";
    private static final String TEST_RESTORE_DIR = "test-restore";

    private static DatabaseAccessor databaseAccessor;

    @BeforeAll
    static void setUpAll() throws IOException {
        // 初始化测试数据库
        DatabaseInitializer initializer = new DatabaseInitializer(TEST_DB_URL);
        initializer.initializeDatabase();
        databaseAccessor = new DatabaseAccessor(TEST_DB_URL);

        // 创建测试目录和文件
        Files.createDirectories(Path.of(TEST_SOURCE_DIR));
        Files.createDirectories(Path.of(TEST_ARCHIVE_DIR));
        Files.createDirectories(Path.of(TEST_RESTORE_DIR));

        // 创建普通测试文件
        Files.write(Path.of(TEST_SOURCE_DIR, "test1.txt"),
                "This is test file 1 content".getBytes());
        Files.write(Path.of(TEST_SOURCE_DIR, "test2.txt"),
                "This is test file 2 content".getBytes());

        // 创建大文件用于测试分卷功能 (大小超过存档限制)
        createLargeTestFile();
    }

    private static void createLargeTestFile() throws IOException {
        Path largeFile = Path.of(TEST_SOURCE_DIR, "large_file.dat");
        try (BufferedOutputStream bos = new BufferedOutputStream(Files.newOutputStream(largeFile))) {
            // 创建一个大小超过存档限制的大文件 (2MB)
            byte[] buffer = new byte[1024]; // 1KB buffer
            Random random = new Random(12345); // 使用固定种子确保可重复性

            for (int i = 0; i < 2048; i++) { // 写入2048次，总共2MB
                random.nextBytes(buffer);
                bos.write(buffer);
            }
        }
    }

    @AfterAll
    static void tearDownAll() throws IOException {
        // 清理文件系统
        deleteDirectory(TEST_SOURCE_DIR);
        deleteDirectory(TEST_ARCHIVE_DIR);
        deleteDirectory(TEST_RESTORE_DIR);
        Files.deleteIfExists(Path.of("test_archive.sqlite"));
    }

    private static void deleteDirectory(String dirPath) throws IOException {
        Path path = Path.of(dirPath);
        if (Files.exists(path)) {
            try (var files = Files.walk(path)) {
                files.sorted((a, b) -> b.compareTo(a))
                        .forEach(file -> {
                            try {
                                Files.delete(file);
                            } catch (IOException e) {
                                throw new RuntimeException(e);
                            }
                        });
            }
        }
    }

    public static void manualCleanup() throws IOException {
        // 清理目录
        deleteDirectory(TEST_SOURCE_DIR);
        deleteDirectory(TEST_ARCHIVE_DIR);
        deleteDirectory(TEST_RESTORE_DIR);

        // 清理数据库文件
        try {
            Files.deleteIfExists(Path.of("test_archive.sqlite"));
            Files.deleteIfExists(Path.of("test_archive.sqlite-journal"));
            Files.deleteIfExists(Path.of("test_archive.db"));
            Files.deleteIfExists(Path.of("test_archive.db-journal"));
            System.out.println("已清理所有测试数据库文件");
        } catch (IOException e) {
            System.err.println("清理数据库文件时出错: " + e.getMessage());
        }

        System.out.println("手动清理完成");
    }

    @Test
    @Order(1)
    void testScanAndSaveDirectory() throws IOException {
        // 1. 执行目录扫描和保存
        DirectoryScanner.scanAndSaveDirectory(Path.of(TEST_SOURCE_DIR), databaseAccessor);

        // 2. 验证 info_root 表记录
        assertRootIndexRecords();

        // 3. 验证 info_directory 表记录
        assertDirectoryIndexRecords();

        // 4. 验证 info_file 表记录
        assertFileIndexRecords();
    }

    private void assertRootIndexRecords() throws IOException {
        try (Connection conn = databaseAccessor.getConnection();
             PreparedStatement stmt = conn.prepareStatement("SELECT id, root_path FROM info_root WHERE root_path = ?")) {
            Path testSourceDirPath = Path.of(TEST_SOURCE_DIR);
            stmt.setString(1, testSourceDirPath.toAbsolutePath().toString());
            try (ResultSet rs = stmt.executeQuery()) {
                assertTrue(rs.next(), "应该存在根目录记录");
                long rootId = rs.getLong("id");
                String rootPath = rs.getString("root_path");

                assertEquals(testSourceDirPath.toAbsolutePath().toString(), rootPath, "根目录路径应该匹配");
                assertTrue(rootId > 0, "根目录ID应该大于0");
            }
        } catch (Exception e) {
            fail("验证根目录记录时发生错误: " + e.getMessage());
        }
    }

    private void assertDirectoryIndexRecords() throws IOException {
        try (Connection conn = databaseAccessor.getConnection();
             PreparedStatement stmt = conn.prepareStatement(
                     "SELECT COUNT(*) as dirCount FROM info_directory d " +
                             "JOIN info_root r ON d.root_id = r.id " +
                             "WHERE r.root_path = ?")) {
            stmt.setString(1, Path.of(TEST_SOURCE_DIR).toAbsolutePath().toString());
            try (ResultSet rs = stmt.executeQuery()) {
                assertTrue(rs.next());
                int dirCount = rs.getInt("dirCount");
                // 根目录本身也会作为一个目录记录存在
                assertEquals(1, dirCount, "应该只有一个目录记录（根目录）");
            }
        } catch (Exception e) {
            fail("验证目录记录时发生错误: " + e.getMessage());
        }
    }

    private void assertFileIndexRecords() throws IOException {
        try (Connection conn = databaseAccessor.getConnection();
             PreparedStatement stmt = conn.prepareStatement(
                     "SELECT f.id, f.file_name, f.file_size FROM info_file f " +
                             "JOIN info_directory d ON f.directory_id = d.id " +
                             "JOIN info_root r ON d.root_id = r.id " +
                             "WHERE r.root_path = ?")) {
            stmt.setString(1, Path.of(TEST_SOURCE_DIR).toAbsolutePath().toString());
            try (ResultSet rs = stmt.executeQuery()) {
                int fileCount = 0;
                while (rs.next()) {
                    fileCount++;
                    String fileName = rs.getString("file_name");

                    assertTrue(fileName.equals("test1.txt") || fileName.equals("test2.txt") || fileName.equals("large_file.dat"),
                            "文件名应该是test1.txt、test2.txt或large_file.dat");
                    long fileSize = rs.getLong("file_size");
                    assertTrue(fileSize > 0, "文件大小应该大于0");

                    // 验证大文件大小
                    if (fileName.equals("large_file.dat")) {
                        assertEquals(2 * 1024 * 1024, fileSize, "大文件大小应该是2MB");
                    }
                }

                assertEquals(3, fileCount, "应该有三个文件记录");
            }
        } catch (Exception e) {
            fail("验证文件记录时发生错误: " + e.getMessage());
        }
    }

    @Test
    @Order(2)
    void testFullArchiveFileInDbProcess() throws Exception {
        ArchiveContext context = ArchiveContext.builder()
                .archiveLimitSize(512L * 1024L)
                .archiveDirectory(TEST_ARCHIVE_DIR)
                .databaseAccessor(databaseAccessor)
                .archivePrefix("archive")
                .archiveCounter(1)
                .build();
        // 2. 执行存档 (使用较小的存档限制来强制分卷)
        ArchiveIn.archiveFileInDb(TEST_SOURCE_DIR, context); // 512KB限制

        // 3. 验证数据库记录
        assertDatabaseRecords();

        // 4. 验证存档文件
        assertArchiveFileInDbFiles();

        // 5. 验证分卷功能
        assertVolumeProcessing();
    }

    private void assertDatabaseRecords() throws Exception {
        // 验证元数据表
        try (Connection conn = databaseAccessor.getConnection();
             PreparedStatement stmt = conn.prepareStatement("SELECT COUNT(*) as count FROM archive_metadata");
             ResultSet rs = stmt.executeQuery()) {
            assertTrue(rs.next());
            assertTrue(rs.getInt("count") > 0, "应该创建存档元数据");
        }

        // 验证资产表
        try (Connection conn = databaseAccessor.getConnection();
             PreparedStatement stmt = conn.prepareStatement("SELECT COUNT(*) as count FROM archive_asset");
             ResultSet rs = stmt.executeQuery()) {
            assertTrue(rs.next());
            int assetCount = rs.getInt("count");
            assertTrue(assetCount > 0, "应该创建资产记录");
        }

        // 验证文件与资源映射表
        try (Connection conn = databaseAccessor.getConnection();
             PreparedStatement stmt = conn.prepareStatement("SELECT COUNT(*) as count FROM map_file_asset");
             ResultSet rs = stmt.executeQuery()) {
            assertTrue(rs.next());
            int mappingCount = rs.getInt("count");
            assertTrue(mappingCount > 0, "应该创建文件与资源映射记录");
        }

        // 验证view_asset视图
        try (Connection conn = databaseAccessor.getConnection();
             PreparedStatement stmt = conn.prepareStatement("SELECT COUNT(*) as count FROM view_asset");
             ResultSet rs = stmt.executeQuery()) {
            assertTrue(rs.next());
            assertTrue(rs.getInt("count") > 0, "view_asset视图应该包含记录");
        }
    }

    private void assertArchiveFileInDbFiles() throws IOException {
        Path archiveDir = Path.of(TEST_ARCHIVE_DIR);
        assertTrue(Files.exists(archiveDir), "存档目录应该存在");

        try (var files = Files.list(archiveDir)) {
            long count = files.count();
            assertTrue(count > 0, "应该创建存档文件");
        }

        // 验证存档文件大小合理性
        try (var files = Files.list(archiveDir)) {
            files.filter(Files::isRegularFile)
                    .forEach(file -> {
                        try {
                            long size = Files.size(file);
                            assertTrue(size > 0, "存档文件大小应该大于0");
                            // 1024 * 5 模拟头文件的开销
                            assertTrue(size <= 512L * 1024L + 1024L * 5, "存档文件大小应该在合理范围内: " + size);
                        } catch (IOException e) {
                            fail("无法获取文件大小: " + e.getMessage());
                        }
                    });
        }
    }

    @Test
    @Order(3)
    void testArchiveFileInDbAndRestore() throws IOException {
        // 1. 解档
        ArchiveOut.unArchive(TEST_SOURCE_DIR, TEST_ARCHIVE_DIR, TEST_RESTORE_DIR, databaseAccessor);

        // 2. 验证文件完整性
        assertFileIntegrity();
    }

    private void assertVolumeProcessing() throws Exception {
        // 验证大文件是否被分卷处理
        try (Connection conn = databaseAccessor.getConnection();
             PreparedStatement stmt = conn.prepareStatement(
                     "SELECT f.file_name, COUNT(fva.id) as volume_count " +
                             "FROM info_file f " +
                             "JOIN map_file_asset fva ON f.id = fva.file_id " +
                             "JOIN info_directory d ON f.directory_id = d.id " +
                             "JOIN info_root r ON d.root_id = r.id " +
                             "WHERE r.root_path = ? AND f.file_name = ? " +
                             "GROUP BY f.id, f.file_name")) {
            stmt.setString(1, Path.of(TEST_SOURCE_DIR).toAbsolutePath().toString());
            stmt.setString(2, "large_file.dat");

            try (ResultSet rs = stmt.executeQuery()) {
                if (rs.next()) {
                    int volumeCount = rs.getInt("volume_count");
                    assertTrue(volumeCount > 1, "大文件应该被分卷处理");
                } else {
                    fail("未找到大文件的分卷记录");
                }
            }
        }

        // 验证分卷记录是否正确存储
        try (Connection conn = databaseAccessor.getConnection();
             PreparedStatement stmt = conn.prepareStatement(
                     "SELECT COUNT(*) as count FROM map_file_asset fva " +
                             "JOIN info_file f ON fva.file_id = f.id " +
                             "JOIN info_directory d ON f.directory_id = d.id " +
                             "JOIN info_root r ON d.root_id = r.id " +
                             "WHERE r.root_path = ? AND f.file_name = ?")) {
            stmt.setString(1, Path.of(TEST_SOURCE_DIR).toAbsolutePath().toString());
            stmt.setString(2, "large_file.dat");

            try (ResultSet rs = stmt.executeQuery()) {
                assertTrue(rs.next());
                int volumeCount = rs.getInt("count");
                assertTrue(volumeCount > 1, "应该有多条分卷记录");
            }
        }
    }

    private void assertFileIntegrity() throws IOException {
        // 验证源文件和恢复文件内容一致
        Path originalFile1 = Path.of(TEST_SOURCE_DIR, "test1.txt");
        Path restoredFile1 = Path.of(TEST_RESTORE_DIR, "test1.txt");

        Path originalFile2 = Path.of(TEST_SOURCE_DIR, "test2.txt");
        Path restoredFile2 = Path.of(TEST_RESTORE_DIR, "test2.txt");

        Path originalLargeFile = Path.of(TEST_SOURCE_DIR, "large_file.dat");
        Path restoredLargeFile = Path.of(TEST_RESTORE_DIR, "large_file.dat");

        assertTrue(Files.exists(restoredFile1), "恢复的test1.txt文件应该存在");
        assertTrue(Files.exists(restoredFile2), "恢复的test2.txt文件应该存在");
        assertTrue(Files.exists(restoredLargeFile), "恢复的large_file.dat文件应该存在");

        assertArrayEquals(Files.readAllBytes(originalFile1),
                Files.readAllBytes(restoredFile1),
                "test1.txt文件内容应该一致");

        assertArrayEquals(Files.readAllBytes(originalFile2),
                Files.readAllBytes(restoredFile2),
                "test2.txt文件内容应该一致");

        // 验证大文件内容一致性
        assertEquals(Files.size(originalLargeFile), Files.size(restoredLargeFile), "大文件大小应该一致");

        // 采样验证大文件内容 (避免内存问题)
        try (var originalChannel = Files.newByteChannel(originalLargeFile);
             var restoredChannel = Files.newByteChannel(restoredLargeFile)) {

            byte[] originalBuffer = new byte[4096];
            byte[] restoredBuffer = new byte[4096];

            // 检查文件开头
            originalChannel.position(0);
            restoredChannel.position(0);
            originalChannel.read(ByteBuffer.wrap(originalBuffer));
            restoredChannel.read(ByteBuffer.wrap(restoredBuffer));
            assertArrayEquals(originalBuffer, restoredBuffer, "大文件开头内容应该一致");

            // 检查文件中间部分
            long middlePosition = Files.size(originalLargeFile) / 2;
            originalChannel.position(middlePosition);
            restoredChannel.position(middlePosition);
            originalChannel.read(ByteBuffer.wrap(originalBuffer));
            restoredChannel.read(ByteBuffer.wrap(restoredBuffer));
            assertArrayEquals(originalBuffer, restoredBuffer, "大文件中间内容应该一致");

            // 检查文件结尾
            long endPosition = Files.size(originalLargeFile) - 4096;
            originalChannel.position(endPosition);
            restoredChannel.position(endPosition);
            originalChannel.read(ByteBuffer.wrap(originalBuffer));
            restoredChannel.read(ByteBuffer.wrap(restoredBuffer));
            assertArrayEquals(originalBuffer, restoredBuffer, "大文件结尾内容应该一致");
        }
    }

    @Test
    @Order(4)
    void testGetRootDirList() throws IOException {

        // 调用getRootDirList方法
        List<String> rootDirs = ArchiveCore.getRootDirList(TEST_DB_PATH);

        // 验证结果
        assertNotNull(rootDirs, "根目录列表不应为null");
        assertFalse(rootDirs.isEmpty(), "根目录列表不应为空");
        assertTrue(rootDirs.contains(Path.of(TEST_SOURCE_DIR).toAbsolutePath().toString()),
                "根目录列表应包含测试源目录");
    }
}
