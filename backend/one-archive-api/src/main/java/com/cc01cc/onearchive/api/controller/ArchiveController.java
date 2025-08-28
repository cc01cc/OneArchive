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

package com.cc01cc.onearchive.api.controller;

import com.cc01cc.onearchive.api.ArchiveRequest;
import com.cc01cc.onearchive.api.service.ArchiveService;
import com.cc01cc.onearchive.api.task.ArchiveTask;
import com.cc01cc.onearchive.api.task.TaskManager;
import lombok.AllArgsConstructor;
import org.springframework.http.ResponseEntity;
import org.springframework.http.MediaType;
import org.springframework.web.bind.annotation.*;
import org.springframework.web.servlet.mvc.method.annotation.SseEmitter;

import java.util.UUID;

@RestController
@RequestMapping("/api/v1/archive")
@AllArgsConstructor
public class ArchiveController {

    private final ArchiveService archiveService;
    private final TaskManager taskManager;

    @PostMapping(produces = MediaType.TEXT_EVENT_STREAM_VALUE)
    public SseEmitter archive(
            @RequestBody
            ArchiveRequest archiveRequest) {
        // 生成任务ID
        String taskId = UUID.randomUUID().toString();

        // 创建SSE emitter
        SseEmitter emitter = new SseEmitter(Long.MAX_VALUE);
        
        // 创建归档任务
        ArchiveTask task = new ArchiveTask(
                taskId,
                archiveRequest.getRootDir(),
                archiveRequest.getArchiveDir(),
                archiveRequest.getArchivePrefix(),
                archiveRequest.getDbPath(),
                archiveRequest.getArchiveLimitSize(),
                archiveService
        );

        // 设置 emitter
        task.setEmitter(emitter);

        // 添加任务到管理器
        taskManager.addTask(task);

        // 启动任务
        task.start();

        return emitter;
    }
}