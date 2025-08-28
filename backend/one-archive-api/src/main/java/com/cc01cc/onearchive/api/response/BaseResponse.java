
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

package com.cc01cc.onearchive.api.response;

import jakarta.validation.constraints.NotNull;
import lombok.Data;

@Data
public class BaseResponse<T> {

    @NotNull
    private int code;
    @NotNull
    private String message;
    @NotNull
    private T data;


    public BaseResponse(int code, String message, T data) {
        this.code = code;
        this.message = message;
        this.data = data;
    }

    /**
     * 创建成功的响应
     */
    public static <T> BaseResponse<T> success(T data) {
        return new BaseResponse<>(200, "Success", data);
    }

    /**
     * 创建成功的响应，带自定义消息
     */
    public static <T> BaseResponse<T> success(String message, T data) {
        return new BaseResponse<>(200, message, data);
    }

    /**
     * 创建错误响应
     */
    public static <T> BaseResponse<T> error(int code, String message) {
        return new BaseResponse<>(code, message, null);
    }

    /**
     * 创建错误响应
     */
    public static <T> BaseResponse<T> error(int code, String message, T data) {
        return new BaseResponse<>(code, message, data);
    }

    /**
     * 创建404响应
     */
    public static <T> BaseResponse<T> notFound() {
        return new BaseResponse<>(404, "Not Found", null);
    }

    /**
     * 创建500响应
     */
    public static <T> BaseResponse<T> error(String message) {
        return new BaseResponse<>(500, message, null);
    }
}

