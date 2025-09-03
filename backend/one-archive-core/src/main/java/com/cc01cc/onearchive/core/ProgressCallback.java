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

<<<<<<<< HEAD:backend/one-archive-core/src/main/java/com/cc01cc/onearchive/core/ProgressCallback.java
package com.cc01cc.onearchive.core;

/**
 * 进度回调接口，用于实时报告操作进度
 */
@FunctionalInterface
public interface ProgressCallback {
    /**
     * 更新进度
     *
     * @param progressInfo 进度信息对象
     */
    void updateProgress(ProgressInfo progressInfo);
========
package com.cc01cc.onearchive.api.service;

import com.cc01cc.onearchive.core.entity.InfoRoot;

import java.util.List;

public interface InfoService {
    public List<InfoRoot> getAllInfoRoots();
>>>>>>>> internal/develop/main:backend/one-archive-api/src/main/java/com/cc01cc/onearchive/api/service/InfoService.java
}
