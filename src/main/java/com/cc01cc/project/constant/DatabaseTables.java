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

package com.cc01cc.project.constant;

/**
 * 数据库表名常量类
 */
public class DatabaseTables {
    public static final String ROOT_INDEX = "root_index";
    public static final String DIRECTORY_INDEX = "directory_index";
    public static final String FILE_INDEX = "file_index";
    public static final String ARCHIVE_METADATA = "archive_metadata";
    public static final String ARCHIVE_ASSET = "archive_asset";
    public static final String FILE_VOLUME_ASSET = "file_volume_asset";
    public static final String DIRECTORY_TREE = "directory_tree";

    // 视图
    public static final String VIEW_FILE = "v_file";
    public static final String VIEW_ASSET = "v_asset";

    private DatabaseTables() {
        // 私有构造函数防止实例化
    }
}
