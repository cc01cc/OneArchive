//! 数据库模块，提供 SQLite 数据库连接和基本操作功能

pub mod constants_sql;
pub mod schema;
pub mod schema_recovery;
pub mod constants;
pub mod dao_database;
pub mod trait_database;
pub mod trait_recovery;
pub mod impl_database;
pub mod impl_initialize;
pub mod impl_recovery;
pub mod common;