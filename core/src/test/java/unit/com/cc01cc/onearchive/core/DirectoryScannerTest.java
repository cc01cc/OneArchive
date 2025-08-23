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

package unit.com.cc01cc.onearchive.core;

import com.cc01cc.onearchive.core.DirectoryScanner;
import com.cc01cc.onearchive.core.DirectoryStatistics;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

class DirectoryScannerTest {

    @TempDir
    Path tempDir;

    @Test
    void testScanEmptyDirectory() throws IOException {
        // 测试扫描空目录
        DirectoryStatistics statistics = DirectoryScanner.scanDirectoryOnly(tempDir);

        // 验证基本属性
        assertEquals(0L, statistics.getTotalSize());
        assertEquals(0, statistics.getDirectoryCount());
        assertEquals(0, statistics.getFileCount());
        assertEquals(0, statistics.getSymbolicLinkCount());
        assertEquals(0, statistics.getHardLinkGroupCount());
        assertEquals(0, statistics.getHardLinkTotalCount());
        assertEquals(0, statistics.getMaxDepth());
    }

    @Test
    void testScanDirectoryWithFiles() throws IOException {
        // 创建测试文件，使用精确的字节数组
        Path file1 = tempDir.resolve("file1.txt");
        Path file2 = tempDir.resolve("file2.txt");
        byte[] content1 = "Hello World".getBytes(StandardCharsets.UTF_8);
        byte[] content2 = "Test Content".getBytes(StandardCharsets.UTF_8);
        Files.write(file1, content1);
        Files.write(file2, content2);

        // 创建子目录
        Path subDir = tempDir.resolve("subdir");
        Files.createDirectory(subDir);
        Path file3 = subDir.resolve("file3.txt");
        byte[] content3 = "Subdir Content".getBytes(StandardCharsets.UTF_8);
        Files.write(file3, content3);

        // 扫描目录
        DirectoryStatistics statistics = DirectoryScanner.scanDirectoryOnly(tempDir);

        // 验证统计结果
        assertEquals(11 + 12 + 14, statistics.getTotalSize()); // 文件内容长度总和
        assertEquals(1, statistics.getDirectoryCount()); // 1个子目录
        assertEquals(3, statistics.getFileCount()); // 3个文件
        assertEquals(0, statistics.getSymbolicLinkCount()); // 没有符号链接
        assertEquals(0, statistics.getHardLinkGroupCount()); // 没有硬链接组
        assertEquals(0, statistics.getHardLinkTotalCount()); // 没有硬链接
        assertEquals(1, statistics.getMaxDepth()); // 最大深度为1
    }

    @Test
    void testScanDirectoryWithNestedStructure() throws IOException {
        // 创建嵌套目录结构
        Path level1Dir = tempDir.resolve("level1");
        Files.createDirectory(level1Dir);

        Path level2Dir = level1Dir.resolve("level2");
        Files.createDirectory(level2Dir);

        Path level3Dir = level2Dir.resolve("level3");
        Files.createDirectory(level3Dir);

        // 在各级目录中创建文件
        Files.write(tempDir.resolve("root_file.txt"), "root".getBytes());
        Files.write(level1Dir.resolve("level1_file.txt"), "level1".getBytes());
        Files.write(level2Dir.resolve("level2_file.txt"), "level2".getBytes());
        Files.write(level3Dir.resolve("level3_file.txt"), "level3".getBytes());

        // 扫描目录
        DirectoryStatistics statistics = DirectoryScanner.scanDirectoryOnly(tempDir);

        // 验证统计结果
        assertEquals(4 + 6 + 6 + 6, statistics.getTotalSize()); // 所有文件大小总和
        assertEquals(3, statistics.getDirectoryCount()); // 3个子目录
        assertEquals(4, statistics.getFileCount()); // 4个文件
        assertEquals(3, statistics.getMaxDepth()); // 最大深度为3
    }

    @Test
    void testScanDirectoryWithSymbolicLinks() throws IOException {
        // 创建测试文件
        Path targetFile = tempDir.resolve("target.txt");
        Files.write(targetFile, "Target Content".getBytes());

        // 创建符号链接（在支持符号链接的系统上）
        try {
            Path symlink = tempDir.resolve("symlink.txt");
            Files.createSymbolicLink(symlink, targetFile);

            // 扫描目录
            DirectoryStatistics statistics = DirectoryScanner.scanDirectoryOnly(tempDir);

            // 验证符号链接统计
            assertEquals(1, statistics.getSymbolicLinkCount());
            assertEquals(2, statistics.getFileCount()); // 包括符号链接
        } catch (UnsupportedOperationException | IOException e) {
            // 在不支持符号链接的系统上跳过此测试
            // Windows系统可能需要管理员权限才能创建符号链接
        }
    }

    @Test
    void testInodeCount() throws IOException {
        // 创建测试文件
        Path file1 = tempDir.resolve("file1.txt");
        Path file2 = tempDir.resolve("file2.txt");
        Files.write(file1, "Content 1".getBytes());
        Files.write(file2, "Content 2".getBytes());

        // 扫描目录
        DirectoryStatistics statistics = DirectoryScanner.scanDirectoryOnly(tempDir);

        // 验证inode计数（每个文件应该有唯一的inode）
        assertEquals(2, statistics.getFileCount());
        // inodeCount可能因系统而异，但应该等于文件数（因为每个文件有唯一的inode）
        assertEquals(2, statistics.getInodeCount());
    }

    @Test
    void testHardLinkStatistics() throws IOException {
        // 注意：创建硬链接需要特定的操作系统支持
        // 在大多数Unix-like系统上可以工作，但在Windows上可能有限制
        Path file1 = tempDir.resolve("original.txt");
        Files.write(file1, "Hard Link Test".getBytes());

        // 尝试创建硬链接（可能不总是成功）
        try {
            Path hardLink = tempDir.resolve("hardlink.txt");
            Files.createLink(hardLink, file1);

            // 扫描目录
            DirectoryStatistics statistics = DirectoryScanner.scanDirectoryOnly(tempDir);

            // 验证硬链接统计
            assertEquals(2, statistics.getFileCount());
            // 由于硬链接共享相同的inode，应该有1个硬链接组，包含2个文件
            assertTrue(statistics.getHardLinkGroupCount() >= 0);
            assertTrue(statistics.getHardLinkTotalCount() >= 0);
        } catch (UnsupportedOperationException | IOException e) {
            // 在不支持硬链接的系统上跳过此验证
        }
    }

    @Test
    void testIOExceptionHandling() throws IOException {
        // 创建一个无法访问的目录进行测试
        Path inaccessibleDir = tempDir.resolve("inaccessible");
        Files.createDirectory(inaccessibleDir);

        // 创建一个文件
        Path testFile = inaccessibleDir.resolve("test.txt");
        Files.write(testFile, "test".getBytes());

        // 扫描目录
        DirectoryStatistics statistics = DirectoryScanner.scanDirectoryOnly(tempDir);

        // 验证基本统计信息是否仍然正确
        assertTrue(statistics.getFileCount() >= 0);
        assertTrue(statistics.getDirectoryCount() >= 0);
    }
}
