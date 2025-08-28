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

package com.cc01cc.onearchive.core.archive;

import com.cc01cc.onearchive.core.constant.FileStatus;
import com.cc01cc.onearchive.core.entity.InfoFile;
import com.cc01cc.onearchive.core.entity.MapFileAsset;
import com.cc01cc.onearchive.core.entity.ViewFile;
import com.cc01cc.onearchive.core.ProgressCallback;
import com.cc01cc.onearchive.core.ProgressInfo;
import org.apache.commons.codec.digest.DigestUtils;

import java.io.IOException;
import java.io.InputStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;

// 处理已存在文件的策略
public class ExistingFileProcessingStrategy implements FileProcessingStrategy {
    @Override
    public List<Long> process(ViewFile viewFile, Path filePath, ArchiveContext context, ProgressCallback callback) throws IOException {
        // 对于已存在的文件，我们只需要创建索引，不需要实际复制数据
        // 这里可以快速完成，所以进度更新可以简化
        
        if (callback != null) {
            ProgressInfo progressInfo = new ProgressInfo(
                0, 
                viewFile.getFileSize(), 
                0, 
                1, 
                "链接已存在的文件: " + viewFile.getFileName()
            );
            callback.updateProgress(progressInfo);
        }
        
        // 对于已存在的文件，我们只需要获取其关联的资产ID
        InfoFile fileByHash = context.getDatabaseAccessor().findFileByHashAndStatus(calculateFileHash(filePath), FileStatus.HEALTH).getFirst();
        // TODO 校验 status 逻辑
        List<MapFileAsset> mapFileAssets = context.getDatabaseAccessor()
                .findFileVolumeAssetByFileId(fileByHash.getId(), null);

        List<Long> assetIds = new ArrayList<>();
        for (MapFileAsset mapFileAsset : mapFileAssets) {
            assetIds.add(mapFileAsset.getAssetId());
        }
        
        if (callback != null) {
            ProgressInfo progressInfo = new ProgressInfo(
                viewFile.getFileSize(), 
                viewFile.getFileSize(), 
                1, 
                1, 
                "完成链接文件: " + viewFile.getFileName()
            );
            callback.updateProgress(progressInfo);
        }
        
        return assetIds;
    }

    private String calculateFileHash(Path filePath) throws IOException {
        try (InputStream fis = Files.newInputStream(filePath)) {
            return DigestUtils.sha256Hex(fis);
        }
    }
}