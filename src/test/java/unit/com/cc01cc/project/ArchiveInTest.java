/// *
// * Copyright 2025. Zheng, Yihong (ZEO, github.com/cc01cc)
// *
// * Licensed under the Apache License, Version 2.0 (the "License");
// * you may not use this file except in compliance with the License.
// * You may obtain a copy of the License at
// *
// *     http://www.apache.org/licenses/LICENSE-2.0
// *
// * Unless required by applicable law or agreed to in writing, software
// * distributed under the License is distributed on an "AS IS" BASIS,
// * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// * See the License for the specific language governing permissions and
// * limitations under the License.
// */
//
//package unit.com.cc01cc.project;
//
//import com.cc01cc.project.ArchiveIn;
//import com.cc01cc.project.dao.DatabaseAccessor;
//import com.cc01cc.project.dto.ViewFile;
//import org.junit.jupiter.api.AfterEach;
//import org.junit.jupiter.api.BeforeEach;
//import org.junit.jupiter.api.Test;
//import org.mockito.*;
//import java.io.IOException;
//import java.util.Collections;
//import java.util.List;
//
//import static org.mockito.Mockito.*;
//
/// **
// * ArchiveIn.archive 方法的单元测试
// */
//class ArchiveInTest {
//
//    @Mock
//    private DatabaseAccessor databaseAccessor;
//
//    @InjectMocks
//    private ArchiveIn archiveIn;
//
//    private AutoCloseable closeable;
//
//    @BeforeEach
//    void setUp() {
//        closeable = MockitoAnnotations.openMocks(this);
//    }
//
//    @AfterEach
//    void tearDown() throws Exception {
//        closeable.close();
//    }
//
//    /**
//     * 测试用例：根目录不存在
//     * 预期结果：记录错误日志并直接返回
//     */
//    @Test
//    void testArchive_RootPathNotFound() throws IOException {
//        String rootPath = "/nonexistent";
//        String archiveDirectory = "/archive";
//        long archiveLimitSize = 1024L;
//
//        when(databaseAccessor.findRootIdByPath(rootPath)).thenReturn(null);
//
//        ArchiveIn.archive(rootPath, archiveDirectory, archiveLimitSize, databaseAccessor);
//
//        verify(databaseAccessor, times(1)).findRootIdByPath(rootPath);
//        verify(databaseAccessor, never()).findViewFilesByRootId(anyLong());
//    }
//
//    /**
//     * 测试用例：根目录存在但无文件
//     * 预期结果：不执行任何文件处理逻辑
//     */
//    @Test
//    void testArchive_NoFilesFound() throws IOException {
//        String rootPath = "/root";
//        String archiveDirectory = "/archive";
//        long archiveLimitSize = 1024L;
//        Long rootId = 1L;
//
//        when(databaseAccessor.findRootIdByPath(rootPath)).thenReturn(rootId);
//        when(databaseAccessor.findViewFilesByRootId(rootId)).thenReturn(Collections.emptyList());
//
//        ArchiveIn.archive(rootPath, archiveDirectory, archiveLimitSize, databaseAccessor);
//
//        verify(databaseAccessor, times(1)).findRootIdByPath(rootPath);
//        verify(databaseAccessor, times(1)).findViewFilesByRootId(rootId);
//    }
//
//    /**
//     * 测试用例：根目录存在且有多个文件
//     * 预期结果：正常处理所有文件，并关闭资源
//     */
//    @Test
//    void testArchive_FilesExist() throws IOException {
//        String rootPath = "/root";
//        String archiveDirectory = "/archive";
//        long archiveLimitSize = 1024L;
//        Long rootId = 1L;
//
//        ViewFile viewFile1 = new ViewFile();
//        viewFile1.setFileId(1L);
//        viewFile1.setRootPath("/root");
//        viewFile1.setDirectoryPath("/dir");
//        viewFile1.setFileName("file1.txt");
//
//        ViewFile viewFile2 = new ViewFile();
//        viewFile2.setFileId(2L);
//        viewFile2.setRootPath("/root");
//        viewFile2.setDirectoryPath("/dir");
//        viewFile2.setFileName("file2.txt");
//
//        List<ViewFile> fileList = List.of(viewFile1, viewFile2);
//
//        when(databaseAccessor.findRootIdByPath(rootPath)).thenReturn(rootId);
//        when(databaseAccessor.findViewFilesByRootId(rootId)).thenReturn(fileList);
//
//        // Mock静态方法调用
//        try (MockedStatic<ArchiveIn> mockedArchiveIn = mockStatic(ArchiveIn.class)) {
//            mockedArchiveIn.when(() -> ArchiveIn.processFiles(anyList(), any()))
//                    .thenCallRealMethod(); // 允许调用真实方法，但内部依赖会被mock
//            mockedArchiveIn.when(() -> ArchiveIn.closeTarOutputAndDoneArchive(any(), any(), any()))
//                    .thenCallRealMethod();
//
//            ArchiveIn.archive(rootPath, archiveDirectory, archiveLimitSize, databaseAccessor);
//
//            mockedArchiveIn.verify(() -> ArchiveIn.processFiles(fileList, any()));
//            mockedArchiveIn.verify(() -> ArchiveIn.closeTarOutputAndDoneArchive(any(), any(), eq(databaseAccessor)));
//        }
//    }
//
//    /**
//     * 测试用例：文件处理过程中抛出IOException
//     * 预期结果：异常被捕获，资源仍被正确关闭
//     */
//    @Test
//    void testArchive_IOExceptionDuringProcessing() throws IOException {
//        String rootPath = "/root";
//        String archiveDirectory = "/archive";
//        long archiveLimitSize = 1024L;
//        Long rootId = 1L;
//
//        ViewFile viewFile = new ViewFile();
//        viewFile.setFileId(1L);
//        viewFile.setRootPath("/root");
//        viewFile.setDirectoryPath("/dir");
//        viewFile.setFileName("file.txt");
//
//        when(databaseAccessor.findRootIdByPath(rootPath)).thenReturn(rootId);
//        when(databaseAccessor.findViewFilesByRootId(rootId)).thenReturn(List.of(viewFile));
//
//        // Mock静态方法调用
//        try (MockedStatic<ArchiveIn> mockedArchiveIn = mockStatic(ArchiveIn.class)) {
//            mockedArchiveIn.when(() -> ArchiveIn.processSingleFile(any(), any()))
//                    .thenThrow(new IOException("模拟IO异常"));
//            mockedArchiveIn.when(() -> ArchiveIn.closeTarOutputAndDoneArchive(any(), any(), any()))
//                    .thenCallRealMethod();
//
//            ArchiveIn.archive(rootPath, archiveDirectory, archiveLimitSize, databaseAccessor);
//
//            mockedArchiveIn.verify(() -> ArchiveIn.closeTarOutputAndDoneArchive(any(), any(), eq(databaseAccessor)));
//        }
//    }
//}
