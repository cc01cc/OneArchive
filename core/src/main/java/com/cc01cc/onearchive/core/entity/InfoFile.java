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

package com.cc01cc.onearchive.core.entity;

import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.annotation.JsonProperty;
import lombok.Data;
import org.jdbi.v3.core.mapper.reflect.ColumnName;

@Data
@JsonIgnoreProperties(ignoreUnknown = true)
public class InfoFile {
    private Long id;
    private Long directoryId;
    
    @JsonProperty("file_name")
    private String name;
    
    @JsonProperty("file_size")
    private Long size;
    
    @JsonProperty("file_mtime")
    private Long mtime;
    
    @JsonProperty("file_hash")
    private String hash;
    
    private String status;
    private Long createdAt;
    private Long updatedAt;
    
    @ColumnName("file_name")
    public String getName() {
        return name;
    }
    
    @ColumnName("file_size")
    public Long getSize() {
        return size;
    }
    
    @ColumnName("file_mtime")
    public Long getMtime() {
        return mtime;
    }
    
    @ColumnName("file_hash")
    public String getHash() {
        return hash;
    }
    
    @ColumnName("directory_id")
    public Long getDirectoryId() {
        return directoryId;
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
