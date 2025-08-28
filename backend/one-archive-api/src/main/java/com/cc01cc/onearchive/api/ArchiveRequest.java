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

<<<<<<<< HEAD:core/src/test/java/integration/CreateEnv.java
package integration;
========
package com.cc01cc.onearchive.api;
>>>>>>>> internal/develop/main:backend/one-archive-api/src/main/java/com/cc01cc/onearchive/api/ArchiveRequest.java

import com.cc01cc.onearchive.core.dao.DatabaseInitializer;

<<<<<<<< HEAD:core/src/test/java/integration/CreateEnv.java
public class CreateEnv {

    public static void createTable() {
        DatabaseInitializer databaseInitializer = new DatabaseInitializer("jdbc:sqlite:one_archive.sqlite");
        databaseInitializer.initializeDatabase();
    }

    public static void main(String[] args) {
        createTable();
    }
========
@Data
public class ArchiveRequest {
    String rootDir;
    String archiveDir;
    String archivePrefix;
    String dbPath;
    Long archiveLimitSize;
>>>>>>>> internal/develop/main:backend/one-archive-api/src/main/java/com/cc01cc/onearchive/api/ArchiveRequest.java
}
