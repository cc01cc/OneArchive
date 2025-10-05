# 后端数据库抽象层规范（Rust）

> 此规范为可选，短期内以快速实现功能为主，长期会考虑添加数据库抽象层，但是现在可以慢慢做起来

## 设计目标

- **隔离具体实现**：业务逻辑代码不得直接依赖 `rusqlite`、`sqlx`、`diesel` 或 `sea-orm` 等具体库。
- **平滑切换**：通过统一的 trait 接口，未来可在 `rusqlite`、`sqlx`、`diesel`、`sea-orm` 之间无缝切换。
- **零成本抽象**：优先使用泛型而非 trait object 以避免运行时开销，除非确实需要运行时多态。

## 核心模式：Repository 模式

- 采用 **Repository 模式** 作为数据访问层的抽象。
- 为每个聚合根（如 `User`, `Archive`）定义一个 trait，例如 `UserRepository`、`ArchiveRepository`。
- Trait 方法应使用**领域语言**，而非 SQL 术语（如 `find_by_email` 而非 `select_where_email`）。

## 错误处理

- 定义统一的 `DatabaseError` 枚举，将各数据库框架的错误（`rusqlite::Error`, `sqlx::Error` 等）封装其中。
- 使用 **条件编译** (`#[cfg(feature = "...")]`) 来包含不同数据库的错误变体，避免引入未使用的依赖。
- 所有 Repository 方法返回 `Result<T, DatabaseError>`。

## 实现与组织

- **接口定义**：在 `src/db/` 目录下定义所有 trait（如 `mod.rs`）。
- **具体实现**：为每种数据库创建独立的模块，如：
  - `src/db/rusqlite_impl/`
  - `src/db/sqlx_impl/` (通过 feature gate 控制)
- **依赖注入**：服务层通过泛型参数或 `Arc<dyn Trait>` 接收 Repository 实例。**优先使用泛型**以获得更好的性能和内联优化。

## 代码示例

### 1. 统一错误类型

```rust
#[derive(Debug)]
pub enum DatabaseError {
    Rusqlite(rusqlite::Error),
    #[cfg(feature = "sqlx")]
    Sqlx(sqlx::Error),
    // ... 其他数据库错误
    Custom(String),
}
```

### 2. Repository Trait

```rust
pub trait ArchiveRepository {
    fn find_by_id(&self, id: i64) -> Result<Option<Archive>, DatabaseError>;
    fn save(&self, archive: &Archive) -> Result<(), DatabaseError>;
}
```

### 3. 具体实现 (rusqlite)

```rust
pub struct RusqliteArchiveRepository {
    conn: Arc<Connection>,
}

impl ArchiveRepository for RusqliteArchiveRepository {
    // ... 实现
}
```

## 禁止事项

- **禁止**在业务逻辑（如 `service` 层）中直接使用 `rusqlite::Connection` 或任何具体数据库的 API。
- **禁止**在 trait 方法签名中使用具体数据库的类型（如 `rusqlite::Row`）。
- **禁止**在没有 feature gate 的情况下引入 `sqlx`、`diesel` 等可选依赖。
