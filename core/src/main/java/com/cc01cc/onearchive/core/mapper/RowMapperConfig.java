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

package com.cc01cc.onearchive.core.mapper;

import org.jdbi.v3.core.Jdbi;

/**
 * 用于注册所有 RowMapper 的配置类
 */
public class RowMapperConfig {
    
    /**
     * 注册所有自定义的 RowMapper
     * @param jdbi Jdbi 实例
     */
    public static void registerRowMappers(Jdbi jdbi) {
        jdbi.registerRowMapper(new InfoFileRowMapper());
        jdbi.registerRowMapper(new InfoDirectoryRowMapper());
        jdbi.registerRowMapper(new InfoRootRowMapper());
        jdbi.registerRowMapper(new ViewFileRowMapper());
        jdbi.registerRowMapper(new ArchiveAssetRowMapper());
        jdbi.registerRowMapper(new ArchiveMetadataRowMapper());
        jdbi.registerRowMapper(new MapFileAssetRowMapper());
        jdbi.registerRowMapper(new ViewAssetRowMapper());
    }
}
