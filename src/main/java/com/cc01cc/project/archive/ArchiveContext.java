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

package com.cc01cc.project.archive;

import com.cc01cc.project.dao.DatabaseAccessor;
import lombok.Builder;
import lombok.Data;
import org.apache.commons.compress.archivers.tar.TarArchiveOutputStream;

import java.nio.file.Path;

/**
 * 存档上下文，用于管理当前存档过程中的状态
 */
@Data
@Builder
public class ArchiveContext {
    private TarArchiveOutputStream tarOutput;
    private Path currentArchiveFile;
    private long currentArchiveSize;
    private Long archiveId;
    private long archiveLimitSize;
    private String archiveDirectory;
    private DatabaseAccessor databaseAccessor;
    private long archiveCounter;
    private String archivePrefix;

    // 检查是否需要创建新存档
    public boolean needNewArchive(long requiredSize) {
        return currentArchiveSize >= archiveLimitSize || tarOutput == null;
    }

    // 重置存档状态
    public void reset() {
        currentArchiveSize = 0;
        currentArchiveFile = null;
        archiveId = null;
        tarOutput = null;
    }
}
