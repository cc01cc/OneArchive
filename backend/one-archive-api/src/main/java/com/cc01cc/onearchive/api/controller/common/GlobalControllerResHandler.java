

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

import com.cc01cc.onearchive.api.response.BaseResponse;
import com.cc01cc.onearchive.api.response.ResponseUtils;
import com.cc01cc.onearchive.api.response.ReturnCode;
import com.fasterxml.jackson.core.JsonProcessingException;
import com.fasterxml.jackson.databind.ObjectMapper;
import jakarta.validation.ConstraintViolation;
import jakarta.validation.Validation;
import jakarta.validation.Validator;
import jakarta.validation.ValidatorFactory;
import org.springframework.core.MethodParameter;
import org.springframework.http.MediaType;
import org.springframework.http.ResponseEntity;
import org.springframework.http.converter.HttpMessageConverter;
import org.springframework.http.server.ServerHttpRequest;
import org.springframework.http.server.ServerHttpResponse;
import org.springframework.web.bind.annotation.RestControllerAdvice;
import org.springframework.web.servlet.mvc.method.annotation.ResponseBodyAdvice;

import java.util.Set;

/**
 * @author cc01cc(zeo) Yihong Zheng
 * @createDate 2023-12-05 15:21
 * @description 参考 <a href="https://juejin.cn/post/7153911112172322829">SpringBoot中这样编写 Controller 层代码</a>
 */
@RestControllerAdvice(basePackages = {"tech.lycy.p.controller"})
public class GlobalControllerResHandler implements ResponseBodyAdvice<Object> {

    public ResponseEntity<BaseResponse<Object>> handleControllerReturn(Object o) {
        return ResponseEntity.ok().body(ResponseUtils.success(o));
    }

    @Override
    public boolean supports(MethodParameter returnType, Class<? extends HttpMessageConverter<?>> converterType) {
        // response是 BaseResponse 类型，或者注释了 NotGlobalControllerResAdvice 都不进行包装
        return !(returnType.hasMethodAnnotation(NotGlobalControllerResAdvice.class));
    }

    @Override
    public Object beforeBodyWrite(
            Object data,
            MethodParameter returnType,
            MediaType selectedContentType,
            Class<? extends HttpMessageConverter<?>> selectedConverterType,
            ServerHttpRequest request,
            ServerHttpResponse response) {

        // 创建 Validator 对象
        Validator validator;
        try (ValidatorFactory factory = Validation.buildDefaultValidatorFactory()) {
            validator = factory.getValidator();
        } catch (Exception e) {
            throw new BusinessException(ReturnCode.SYSTEM_ERROR);
        }

        // 对 VO 对象进行约束校验
        Set<ConstraintViolation<Object>> violations = validator.validate(data);
        if (!violations.isEmpty()) {
            // 如果有约束违反，抛出异常或返回错误信息
            throw new BusinessException(ReturnCode.PARAMS_ERROR, violations.iterator().next().getMessage());
        }

        // String 类型不能直接包装
        if (returnType.getGenericParameterType().equals(String.class)) {
            ObjectMapper objectMapper = new ObjectMapper();
            try {
                // 将数据包装在 BaseResponse 后转换为json串进行返回
                return objectMapper.writeValueAsString(ResponseUtils.success(data));
            } catch (JsonProcessingException e) {
                throw new BusinessException(ReturnCode.RESPONSE_PACK_ERROR);
            }
        }

        if (returnType.getParameterType().isAssignableFrom(BaseResponse.class)) {
            return data;
        }
        // 会自动添加 response header，所以这里只需要返回 body 即可
        return ResponseUtils.success(data);
    }
}

