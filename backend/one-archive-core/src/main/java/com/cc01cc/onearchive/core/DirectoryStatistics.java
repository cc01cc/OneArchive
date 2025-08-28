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

package com.cc01cc.onearchive.core;

import lombok.Data;
import lombok.ToString;

@ToString
@Data
public class DirectoryStatistics {
    /**
     * 目录总体积（字节）
     */
    private long totalSize = 0;

    /**
     * 子目录数量
     */
    private int directoryCount = 0;

    /**
     * 子文件数量
     */
    private int fileCount = 0;

    /**
     * inode数量
     */
    private int inodeCount = 0;

    /**
     * 软链接(符号链接)数量
     */
    private int symbolicLinkCount = 0;

    /**
     * 硬链接数量
     */
    private int hardLinkCount = 0;

    /**
     * 硬链接组数
     */
    private int hardLinkGroupCount = 0;

    /**
     * 硬链接总数
     */
    private int hardLinkTotalCount = 0;

    /**
     * 子目录最深的层级
     */
    private int maxDepth = 0;
}
