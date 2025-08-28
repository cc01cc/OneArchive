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

// src/main/java/com/cc01cc/project/ConfigManager.java
package com.cc01cc.onearchive.core.config;

import com.fasterxml.jackson.databind.ObjectMapper;
import lombok.extern.slf4j.Slf4j;

import java.io.IOException;
import java.io.InputStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;

@Slf4j
public class ConfigManager {

    private static final String CONFIG_FILE = "config.json";

    public static ArchiveConfig loadConfig() throws IOException {
        ObjectMapper mapper = new ObjectMapper();

        // 首先尝试从当前工作目录加载配置文件
        Path configPath = Paths.get(CONFIG_FILE);
        if (Files.exists(configPath)) {
            log.info("从文件加载配置: {}", configPath.toAbsolutePath());
            return mapper.readValue(configPath.toFile(), ArchiveConfig.class);
        }

        // 如果找不到文件，则尝试从classpath加载
        try (InputStream inputStream = ConfigManager.class.getClassLoader().getResourceAsStream(CONFIG_FILE)) {
            if (inputStream != null) {
                log.info("从classpath加载配置: {}", CONFIG_FILE);
                return mapper.readValue(inputStream, ArchiveConfig.class);
            }
        }

        throw new IOException("无法找到配置文件: " + CONFIG_FILE);
    }
}
