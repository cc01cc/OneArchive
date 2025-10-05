use serde::{Deserialize, Serialize};

/// 通用API响应结构体
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

/// 分页参数结构体
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PaginationParams {
    pub page: i64,
    pub page_size: i64,
}

/// 通用分页响应
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
    pub total_pages: i64,
}
