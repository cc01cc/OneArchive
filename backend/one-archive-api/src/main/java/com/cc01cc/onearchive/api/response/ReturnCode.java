/*
 * 版权所有 © 2025. cc01cc(zeo) 郑一弘
 *
 * 所有权利保留。
 *
 * 未经 郑一弘 明确书面授权，不得以任何方式构建，复制、修改、发布、传输、分发、出售或租赁本代码及其任何部分。
 *
 * 本代码副本已授权给 杭州零一创研科技有限公司 使用。
 *
 * 杭州零一创研科技有限公司 (lycy.tech) 可以在授权协议的许可范围内，使用本代码进行商业目的，包括构建、修改，不得进行二次分发。
 *
 * 本授权的有效期为一年，杭州零一创研科技有限公司 应在授权到期前 30 内进行续签，否则授权自动终止。
 *
 * Copyright © 2025. cc01cc(zeo) Yihong Zheng
 *
 * All rights reserved.
 *
 * No part of this code may be built, copied, modified, published, transmitted, distributed, sold or leased in any way without the express written permission of Yihong Zheng.
 *
 * A copy of this code has been licensed to Hangzhou Ling Yi Chuang Yan Technology Co., Ltd. for use.
 *
 * Hangzhou Ling Yi Chuang Yan Technology Co., Ltd. (lycy.tech) may use this code for commercial purposes within the scope of the license agreement, including building and modifying, but may not redistribute it.
 *
 * The term of this license is one year. Hangzhou Ling Yi Chuang Yan Technology Co., Ltd. shall renew the license within 30 days before the expiration of the license, otherwise the license will automatically terminate.
 */

package com.cc01cc.onearchive.api.response;

import lombok.Getter;

/**
 * 返回给前端的状态码
 *
 * @author cc01cc(zeo) Yihong Zheng
 */
@Getter
public enum ReturnCode {

    SUCCESS(20000, "成功"),
    PARAMS_ERROR(40001, "请求参数错误"),
    NULL_EXIST(40200, "请求数据为空"),
    NULL_EXIST_DATABASE(40201, "数据库数据为空"),
    NOT_LOGIN(40101, "未登录"),
    NO_AUTH(40103, "未认证"),
    NOT_REGISTER(40104, "未注册"),
    NO_ACCESS(40301, "无访问权限"),
    ACCOUNT_PASSWORD_ERROR(40102, "账号或密码错误"),
    SYSTEM_ERROR(50001, "系统错误"),
    SERVER_ERROR(50002, "服务器错误"),
    RESPONSE_PACK_ERROR(50101, "返回数据封装错误");


    private final int code;
    private final String msg;

    ReturnCode(int code, String msg) {
        this.code = code;
        this.msg = msg;
    }

}
