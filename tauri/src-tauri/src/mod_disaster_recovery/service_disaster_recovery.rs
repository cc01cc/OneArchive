//! 灾备恢复数据库模块
//! 包含依赖数据库的业务逻辑
//!
//! ## 模块职责说明
//!
//! ### 核心协调器
//! - `decoder`: 灾备恢复解码器，协调恢复过程的主要入口组件
//! - `encoder`: 灾备组编码器，协调编码过程的主要入口组件
//!
//! ### 数据处理组件
//! - `data_saver`: 数据保存器，负责去除对齐填充、文件写入等数据处理操作
//! - `shard_processor`: 分片处理器，负责生成和保存 Reed-Solomon 分片数据
//! - `shard_loader`: 分片加载器与归档文件读取器，负责加载分片和读取归档文件
//!
//! ### 业务逻辑组件
//! - `recovery_planner`: 恢复规划器，负责分析恢复组信息和准备恢复数据
//! - `recovery_executor`: 恢复执行器，负责执行 Reed-Solomon 恢复操作
//! - `group_creator`: 灾备组创建器，负责创建灾备组记录
//! - `shard_mapping_service`: 分片映射服务，提供分片与归档文件的映射关系
//!
//! ### 基础设施组件
//! - `file_integrity_validator`: 文件完整性校验器，基于 SHA256 的验证功能

pub mod service_data_saver;
pub mod service_decoder;
pub mod service_encoder;
pub mod service_file_integrity_validator;
pub mod service_group_creator;
pub mod service_recovery_executor;
pub mod service_recovery_planner;
pub mod service_shard_loader;
pub mod service_shard_mapping_service;
pub mod service_shard_processor;

// 重新导出重要的 trait，方便外部使用
pub use service_shard_loader::{ArchiveReader, ShardLoader};
pub use service_shard_processor::ShardProcessor;