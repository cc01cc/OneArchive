package com.cc01cc.onearchive.api.response;


/**
 * 返回给前端的工具类
 *
 * @author cc01cc(zeo) Yihong Zheng
 */
public class ResponseUtils {
    private ResponseUtils() {
        throw new IllegalStateException("Utility class");
    }

    public static <T> BaseResponse<T> success(T data) {
        return new BaseResponse<>(ReturnCode.SUCCESS.getCode(), "ok", data);
    }

    public static <T> BaseResponse<T> success() {
        return new BaseResponse<>(ReturnCode.SUCCESS.getCode(), "ok", null);
    }

    public static <T> BaseResponse<T> error(ReturnCode returnCode) {
        return new BaseResponse<>(returnCode.getCode(), returnCode.getMsg(), null);
    }

    public static <T> BaseResponse<T> error(int code, String msg, T data) {
        return new BaseResponse<>(code, msg, data);
    }

    public static <T> BaseResponse<T> error(ReturnCode returnCode, T data) {
        return new BaseResponse<>(returnCode.getCode(), returnCode.getMsg(), data);
    }
}
