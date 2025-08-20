# OneArchive

OneArchive 是一个文件归档系统，旨在帮助用户高效地管理和备份大量文件。

本系统的核心功能是将零散的小文件捆绑为单一文件, 将大体积文件分割为多个卷, 以方便传输与存储.通过自动去重、分卷存储和增量备份等技术，最大化存储效率并确保数据完整性。

## 1. 功能特性

- **智能去重**: 自动识别重复文件，避免重复存储
- **化零为整**: 支持将大量零碎文件归档成单个文件
- **分卷存储**: 自动将大文件分割成指定大小的卷进行存储
- **增量备份**: 只备份发生变化的文件，提高备份效率
- **数据完整性**: 使用SHA-256哈希校验确保文件完整性
- **灵活恢复**: 支持按目录恢复文件
- **数据库管理**: 使用SQLite数据库管理文件索引和元数据

## 2. 安装与使用

### 2.1. 环境要求

- Java 21+
- Maven 3.6+

### 2.2. 构建项目

```bash
bash
mvn clean install
```

```plantuml
@startuml 存档流程活动图
start

:初始化存档上下文(ArchiveContext);
:扫描目录获取文件列表(DirectoryScanner);

fork
  :获取文件元信息(大小、修改时间);
fork again
  :计算文件哈希(DigestUtils.sha256Hex);
end fork

if (文件大小 > 存档限制) then (是)
  :文件分卷处理;
  :生成分卷资产ID;
else (否)
  :生成单个资产ID;
endif

if (存在相同哈希资产?) then (是)
  :关联已有资产ID到文件索引;
else (否)
  :创建新资产记录(archive_asset表);
  :写入文件数据到Tar存档(TarArchiveOutputStream);
  :更新存档大小和状态;
endif

:更新文件索引状态(file_index表);
:提交事务(DatabaseAccessor);

stop
@enduml
```

## 3. 许可证

- 本项目采用 Apache License 2.0 许可证，详情请见 [LICENSE](LICENSE) 文件。

## 4. 开发计划

### v0.1.0 release

- [ ] 支持软链接/硬链接处理
- [ ] 支持断点续传功能
- [ ] 完善存档或解档中断异常处理机制

## 5. 贡献

欢迎提交Issue和Pull Request来改进本项目。

- 本项目 CLA
  详见 [Contributor License Agreement v1 By ZEO](https://gist.github.com/cc01cc/96a194266c6ffbf9f2b8e2c0a5cf97f2)
