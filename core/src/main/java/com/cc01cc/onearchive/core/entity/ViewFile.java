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

// ViewFile.java
package com.cc01cc.onearchive.core.entity;

import lombok.Data;

@Data
public class ViewFile {
    private Long rootId;
    private String rootPath;
    private String rootStatus;
    private Long directoryId;
    private String directoryPath;
    private Long directoryMtime;
    private String directoryStatus;
    private Long fileId;
    private String fileName;
    private Long fileSize;
    private Long fileMtime;
    private String fileHash;
    private Long volumeCount;
    private String fileStatus;
}
