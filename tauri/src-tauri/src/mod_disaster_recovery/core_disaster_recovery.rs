//! 灾备恢复核心模块
//! 包含不依赖数据库的核心逻辑

pub mod model_disaster_recovery;
pub mod core_reed_solomon_validator;
pub mod core_shard_alignment;