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

<<<<<<< HEAD
<<<<<<<< HEAD:core/src/main/java/com/cc01cc/onearchive/core/archive/FileProcessingStrategy.java
// FileProcessingStrategy.java
package com.cc01cc.onearchive.core.archive;

=======
<<<<<<<< HEAD:backend/one-archive-core/src/test/java/integration/CreateEnv.java
package integration;

import com.cc01cc.onearchive.core.dao.DatabaseInitializer;
>>>>>>> internal/develop/main
========
package com.cc01cc.onearchive.core.archive;

import com.cc01cc.onearchive.core.ProgressCallback;
import com.cc01cc.onearchive.core.ProgressInfo;
<<<<<<< HEAD
>>>>>>>> internal/develop/main:backend/one-archive-core/src/main/java/com/cc01cc/onearchive/core/archive/FileProcessingStrategy.java
import com.cc01cc.onearchive.core.entity.ViewFile;

import java.io.IOException;
import java.nio.file.Path;
import java.util.List;

interface FileProcessingStrategy {
    List<Long> process(ViewFile viewFile, Path filePath, ArchiveContext context, ProgressCallback callback) throws IOException;
}



=======
import com.cc01cc.onearchive.core.entity.ViewFile;
>>>>>>>> internal/develop/main:backend/one-archive-core/src/main/java/com/cc01cc/onearchive/core/archive/FileProcessingStrategy.java

public class CreateEnv {

<<<<<<<< HEAD:backend/one-archive-core/src/test/java/integration/CreateEnv.java
    public static void createTable() {
        DatabaseInitializer databaseInitializer = new DatabaseInitializer("jdbc:sqlite:one_archive.sqlite");
        databaseInitializer.initializeDatabase();
    }

    public static void main(String[] args) {
        createTable();
    }
========
interface FileProcessingStrategy {
    List<Long> process(ViewFile viewFile, Path filePath, ArchiveContext context, ProgressCallback callback) throws IOException;
>>>>>>>> internal/develop/main:backend/one-archive-core/src/main/java/com/cc01cc/onearchive/core/archive/FileProcessingStrategy.java
}
>>>>>>> internal/develop/main
