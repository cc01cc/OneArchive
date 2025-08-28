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

/**
 * 进度信息类，用于封装归档操作的进度信息
 */
public class ProgressInfo {
    private long processedBytes;
    private long totalBytes;
    private int processedFiles;
    private int totalFiles;
    private String message;
    
    public ProgressInfo() {
    }
    
    public ProgressInfo(long processedBytes, long totalBytes, int processedFiles, int totalFiles, String message) {
        this.processedBytes = processedBytes;
        this.totalBytes = totalBytes;
        this.processedFiles = processedFiles;
        this.totalFiles = totalFiles;
        this.message = message;
    }
    
    // Getters and setters
    public long getProcessedBytes() {
        return processedBytes;
    }
    
    public void setProcessedBytes(long processedBytes) {
        this.processedBytes = processedBytes;
    }
    
    public long getTotalBytes() {
        return totalBytes;
    }
    
    public void setTotalBytes(long totalBytes) {
        this.totalBytes = totalBytes;
    }
    
    public int getProcessedFiles() {
        return processedFiles;
    }
    
    public void setProcessedFiles(int processedFiles) {
        this.processedFiles = processedFiles;
    }
    
    public int getTotalFiles() {
        return totalFiles;
    }
    
    public void setTotalFiles(int totalFiles) {
        this.totalFiles = totalFiles;
    }
    
    public String getMessage() {
        return message;
    }
    
    public void setMessage(String message) {
        this.message = message;
    }
    
    @Override
    public String toString() {
        return "ProgressInfo{" +
                "processedBytes=" + processedBytes +
                ", totalBytes=" + totalBytes +
                ", processedFiles=" + processedFiles +
                ", totalFiles=" + totalFiles +
                ", message='" + message + '\'' +
                '}';
    }
}
