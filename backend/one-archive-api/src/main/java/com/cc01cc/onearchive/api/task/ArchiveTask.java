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

package com.cc01cc.onearchive.api.task;

import com.cc01cc.onearchive.api.service.ArchiveService;
import com.cc01cc.onearchive.core.OneArchive;
import com.cc01cc.onearchive.core.ProgressInfo;
import lombok.Getter;
import org.springframework.http.MediaType;
import org.springframework.web.servlet.mvc.method.annotation.SseEmitter;

import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.util.Map;
import java.util.concurrent.ConcurrentHashMap;

// TODO 优化为线程池, 以及使用守护线程
public class ArchiveTask extends Thread {
    
    private static final ConcurrentHashMap<String, SseEmitter> taskEmitters = new ConcurrentHashMap<>();
    
    @Getter
    private final String taskId;
    
    private final String rootDir;
    private final String archiveDir;
    private final String archivePrefix;
    private final String dbPath;
    private final Long archiveLimitSize;
    private final ArchiveService archiveService;
    
    @Getter
    private volatile String status = "PENDING"; // PENDING, RUNNING, COMPLETED, FAILED
    
    @Getter
    private volatile int progress = 0; // 0-100
    
    @Getter
    private volatile String message = "任务等待中";
    
    @Getter
    private Long startTime;
    
    @Getter
    private Long endTime;
    
    @Getter
    private Object result;
    
    public ArchiveTask(String taskId, String rootDir, String archiveDir, String archivePrefix, 
                       String dbPath, Long archiveLimitSize, ArchiveService archiveService) {
        this.taskId = taskId;
        this.rootDir = rootDir;
        this.archiveDir = archiveDir;
        this.archivePrefix = archivePrefix;
        this.dbPath = dbPath;
        this.archiveLimitSize = archiveLimitSize;
        this.archiveService = archiveService;
        this.setName("ArchiveTask-" + taskId);
    }
    
    public void setEmitter(SseEmitter emitter) {
        taskEmitters.put(taskId, emitter);
        
        // 添加完成和超时回调
        emitter.onCompletion(() -> taskEmitters.remove(taskId));
        emitter.onTimeout(() -> {
            taskEmitters.remove(taskId);
            try {
                emitter.complete();
            } catch (Exception e) {
                // 忽略
            }
        });

        // 发送初始连接成功事件
        try {
            emitter.send(SseEmitter.event()
                    .name("CONNECTED")
                    .data(Map.of(
                        "taskId", taskId,
                        "message", "Connected to archive task"
                    ), MediaType.APPLICATION_JSON));
        } catch (IOException e) {
            taskEmitters.remove(taskId);
        }
    }
    
    @Override
    public void run() {
        try {
            status = "RUNNING";
            message = "归档任务开始执行";
            startTime = System.currentTimeMillis();
            
            // 通过SSE发送状态更新
            sendSseUpdate();

            OneArchive.initializeDatabase(dbPath);
            
            // 执行归档操作，使用支持进度回调的接口
            OneArchive.performArchive(
                    rootDir, 
                    archiveDir, 
                    archivePrefix, 
                    dbPath, 
                    archiveLimitSize,
                    (progressInfo) -> {
                        // 根据ProgressInfo更新进度和消息
                        ArchiveTask.this.progress = (int) ((progressInfo.getProcessedBytes() * 100) / Math.max(1, progressInfo.getTotalBytes()));
                        ArchiveTask.this.message = progressInfo.getMessage();
                        sendSseUpdate();
                    }
            );
            
            status = "COMPLETED";
            progress = 100;
            message = "归档任务执行完成";
            endTime = System.currentTimeMillis();
            result = Map.of("success", true, "message", "归档完成");
            
            // 通过SSE发送完成事件
            sendSseCompleted();
        } catch (Exception e) {
            status = "FAILED";
            message = "归档任务执行失败: " + e.getMessage();
            endTime = System.currentTimeMillis();
            result = Map.of("success", false, "error", e.getMessage());
            
            // 通过SSE发送失败事件
            sendSseFailed();
            
            e.printStackTrace();
        }
    }
    
    public void updateProgress(int progress, String message) {
        this.progress = progress;
        this.message = message;
        
        // 通过SSE发送状态更新
        sendSseUpdate();
    }
    
    public long getDuration() {
        if (startTime == null) return 0;
        if (endTime == null) return System.currentTimeMillis() - startTime;
        return endTime - startTime;
    }
    
    private void sendSseUpdate() {
        SseEmitter emitter = taskEmitters.get(taskId);
        if (emitter != null) {
            try {
                emitter.send(SseEmitter.event()
                        .name("TASK_UPDATE")
                        .data(Map.of(
                            "taskId", taskId,
                            "status", status,
                            "progress", progress,
                            "message", message,
                            "timestamp", System.currentTimeMillis()
                        ), MediaType.APPLICATION_JSON));
            } catch (IOException e) {
                taskEmitters.remove(taskId);
                try {
                    emitter.complete();
                } catch (Exception ex) {
                    // 忽略
                }
            }
        }
    }

    private void sendSseCompleted() {
        SseEmitter emitter = taskEmitters.get(taskId);
        if (emitter != null) {
            try {
                emitter.send(SseEmitter.event()
                        .name("TASK_COMPLETED")
                        .data(Map.of(
                            "taskId", taskId,
                            "status", status,
                            "progress", progress,
                            "message", message,
                            "result", result,
                            "duration", getDuration(),
                            "timestamp", System.currentTimeMillis()
                        ), MediaType.APPLICATION_JSON));
                emitter.complete(); // 完成连接
            } catch (IOException e) {
                // 忽略
            } finally {
                taskEmitters.remove(taskId);
            }
        }
    }

    private void sendSseFailed() {
        SseEmitter emitter = taskEmitters.get(taskId);
        if (emitter != null) {
            try {
                emitter.send(SseEmitter.event()
                        .name("TASK_FAILED")
                        .data(Map.of(
                            "taskId", taskId,
                            "status", status,
                            "progress", progress,
                            "message", message,
                            "error", result,
                            "duration", getDuration(),
                            "timestamp", System.currentTimeMillis()
                        ), MediaType.APPLICATION_JSON));
                emitter.complete(); // 完成连接
            } catch (IOException e) {
                // 忽略
            } finally {
                taskEmitters.remove(taskId);
            }
        }
    }
}

