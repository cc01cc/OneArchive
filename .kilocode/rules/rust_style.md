# Rust 代码规范

## 模块组织

- 采用 Rust 2018+ 的模块系统，**不再使用 `mod.rs`**。
- 模块 `foo` 应定义在 `src/foo.rs`，其子模块 `foo::bar` 定义在 `src/foo/bar.rs`。
- 避免将所有逻辑塞入单个 `.rs` 文件；当日益增大时，应拆分为多个逻辑子模块。

## 子模块命名规范

- 所有子模块文件必须使用统一的前缀格式：`前缀_模块名.rs`，以确保命名一致性和功能区分。
- 标准前缀定义：
  - `api_`: 接口定义。
  - `impl_`: 具体实现代码（结构体 impl、函数实现）。
  - `trait_`: Trait 定义。
  - `model_`: 数据模型、结构体、枚举定义。
  - `utils_`: 工具函数、辅助代码。
  - `schema_`: 数据库 schema 定义。
  - `constants_`: 常量定义。
  - `common_`: 通用代码、共享逻辑。
  - `dao_`: DAO 层实现。
  - `core_`: 核心逻辑。
  - `service_`: 服务层实现。
- 通用顶级模块：`api`
- 非通用顶级模块保持 `mod_` 前缀，子模块统一使用上述前缀。
- 目录下文件也遵循相同规则，避免无前缀或混合命名。

## 测试文件命名规范

- 测试文件（位于 `tests/` 目录下的集成测试）统一使用 `test_` 前缀，如 `test_archive.rs`、`test_database.rs`。
- 这是 Rust 标准约定，Cargo 会自动识别并运行这些文件，无需额外前缀。
- 测试代码中的模块引用需与源码一致，确保路径更新同步。

## 函数与结构

- 单个函数体（不含注释和空行）**不应超过 50 行**。理想长度为 25 行以内。
- 公共逻辑（如错误处理、日志、工具函数）应提取到独立模块（如 `utils.rs`、`error.rs`）。
- 优先使用 `Result` 和 `Option` 进行错误和空值处理，避免 panic。
- 结构体和枚举应合理拆分，避免“上帝对象”。
- 可以添加 log debug, info 等辅助排查问题，或者开发; 也可以修改 err 暴露方式，以排查 bug

## 代码风格

- 所有公开 API 必须有文档注释（`///`）。
- 避免不必要的 `clone()`，优先使用引用或 `Cow`。
- 注释很重要，但是不要大篇幅的添加注释

## 关联外链

> 以下 URL 链接可以获取更加详细的内容

- <https://doc.rust-lang.org/stable/book/index.html>
- <https://github.com/rusqlite/rusqlite>
- <https://docs.rs/rusqlite/latest/rusqlite/>
