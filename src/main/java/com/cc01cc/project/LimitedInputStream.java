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

import java.io.IOException;
import java.io.InputStream;

/**
 * 限制读取数据量的输入流装饰器
 */
public class LimitedInputStream extends InputStream {
    private final InputStream inputStream;
    private long remaining;

    public LimitedInputStream(InputStream inputStream, long limit) {
        this.inputStream = inputStream;
        this.remaining = limit;
    }

    @Override
    public int read() throws IOException {
        if (remaining <= 0) {
            return -1;
        }
        int result = inputStream.read();
        if (result != -1) {
            remaining--;
        }
        return result;
    }

    @Override
    public int read(byte[] b, int off, int len) throws IOException {
        if (remaining <= 0) {
            return -1;
        }
        int maxLen = (int) Math.min(len, remaining);
        int result = inputStream.read(b, off, maxLen);
        if (result > 0) {
            remaining -= result;
        }
        return result;
    }
}
