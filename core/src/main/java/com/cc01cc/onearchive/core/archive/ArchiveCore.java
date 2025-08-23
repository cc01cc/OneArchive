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

import com.cc01cc.onearchive.core.dao.DatabaseAccessor;
import com.cc01cc.onearchive.core.entity.InfoRoot;
import com.cc01cc.onearchive.core.entity.ViewFile;
import lombok.extern.slf4j.Slf4j;

import java.nio.file.Path;
import java.util.List;

@Slf4j
public class ArchiveCore {
    public static List<String> getRootDirList(String sqlitePath) {
        String dbUrl = "jdbc:sqlite:" + sqlitePath;
        DatabaseAccessor databaseAccessor = new DatabaseAccessor(dbUrl);
        return databaseAccessor.getHealthRootDirList().stream().map(InfoRoot::getPath).toList();
    }

    public static List<ViewFile> getViewFileList(String rootDir, DatabaseAccessor databaseAccessor) {
        Path rootAbsolutePath = Path.of(rootDir).toAbsolutePath();
        Long rootId = databaseAccessor.findRootIdByPath(rootAbsolutePath.toString());
        if (rootId == null) {
            log.error("根目录 {} 不存在于数据库中", rootDir);
            return null;
        }
        return databaseAccessor.findViewFilesByRootId(rootId);
    }
}
