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

<<<<<<<< HEAD:core/src/main/java/com/cc01cc/onearchive/core/entity/InfoRoot.java
package com.cc01cc.onearchive.core.entity;

import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.annotation.JsonProperty;
import lombok.Data;
import org.jdbi.v3.core.mapper.reflect.ColumnName;

@Data
@JsonIgnoreProperties(ignoreUnknown = true)
public class InfoRoot {
    private Long id;
    
    @JsonProperty("root_name")
    private String name;
    
    @JsonProperty("root_path")
    private String path;
    
    private String status;
    private Long createdAt;
    private Long updatedAt;
    
    @ColumnName("root_name")
    public String getName() {
        return name;
    }
    
    @ColumnName("root_path")
    public String getPath() {
        return path;
    }
    
    @ColumnName("created_at")
    public Long getCreatedAt() {
        return createdAt;
    }
    
    @ColumnName("updated_at")
    public Long getUpdatedAt() {
        return updatedAt;
    }
}
========
package com.cc01cc.onearchive.api.service;

/**
 * @author Zheng, Yihong
 * @date 20250825
 */
public interface ArchiveService {

    void archive(
            String rootDir,
            String archiveDir,
            String archivePrefix,
            String dbPath,
            Long archiveLimitSize
    );
}
>>>>>>>> internal/develop/main:backend/one-archive-api/src/main/java/com/cc01cc/onearchive/api/service/ArchiveService.java
