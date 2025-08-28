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

package com.cc01cc.onearchive.api.controller.common;

import com.cc01cc.onearchive.api.response.ReturnCode;
import lombok.Getter;

@Getter
public class BusinessException extends RuntimeException {

    private final int code; // 异常编码，用于标识不同类型的异常
    private final String data; // 异常描述，详细说明异常的情况


    public BusinessException(String msg, int code, String data) {
        super(msg);
        this.code = code;
        this.data = data;
    }


    public BusinessException(ReturnCode returnCode) {
        super(returnCode.getMsg());
        this.code = returnCode.getCode();
        this.data = "";
    }

    public BusinessException(ReturnCode returnCode, String data) {
        super(returnCode.getMsg());
        this.code = returnCode.getCode();
        this.data = data;
    }
}
