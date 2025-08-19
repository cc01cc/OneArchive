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

package com.cc01cc.project;

import com.cc01cc.project.dto.ArchiveContext;
import com.cc01cc.project.dto.FileInfo;
import com.cc01cc.project.dto.FileVolumeAsset;
import com.cc01cc.project.dto.ViewFile;
import org.apache.commons.codec.digest.DigestUtils;

import java.io.IOException;
import java.io.InputStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;

// 处理已存在文件的策略
class ExistingFileProcessingStrategy implements FileProcessingStrategy {
    @Override
    public List<Long> process(ViewFile viewFile, Path filePath, ArchiveContext context) throws IOException {
        // 对于已存在的文件，我们只需要获取其关联的资产ID
        FileInfo fileByHash = context.getDatabaseAccessor().findHealthFileByHash(calculateFileHash(filePath));
        List<FileVolumeAsset> fileVolumeAssets = context.getDatabaseAccessor()
                .findFileVolumeAssetByFileId(fileByHash.getId());

        List<Long> assetIds = new ArrayList<>();
        for (FileVolumeAsset fileVolumeAsset : fileVolumeAssets) {
            assetIds.add(fileVolumeAsset.getAssetId());
        }

        return assetIds;
    }

    private String calculateFileHash(Path filePath) throws IOException {
        try (InputStream fis = Files.newInputStream(filePath)) {
            return DigestUtils.sha256Hex(fis);
        }
    }
}