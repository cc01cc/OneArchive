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

package com.cc01cc.project.dto;

import lombok.Data;
import lombok.extern.slf4j.Slf4j;

/**
 * @author cc01cc
 * @createDate 2025-08-13 3:07
 */
@Data
@Slf4j
public class DirectoryInfo {
    String status;
    long createdAt;
    long updatedAt;
    private long id;
    private long rootId;
    private String name;
    private long mtime;
    private String path;
}
