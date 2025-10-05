/// 通用API响应类型
export interface ApiResponse<T> {
    success: boolean;
    data?: T;
    error?: string;
}

/// 分页参数
export interface PaginationParams {
    page: number;
    pageSize: number;
}

/// 分页响应
export interface PaginatedResponse<T> {
    data: T[];
    total: number;
    page: number;
    pageSize: number;
    totalPages: number;
}
