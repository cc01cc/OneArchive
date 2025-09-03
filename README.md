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

```plantuml
@startuml
title scanAndSaveDirectory 流程图

start

partition "初始化阶段" {
    :获取根目录绝对路径;
    :查询数据库中是否存在该根目录;
    
    if ("根目录不存在?") then (是)
        :创建新的根目录;
        :获取新根目录ID;
    else (否)
        :获取根目录ID;
        :获取现有目录和文件列表;
        :将根目录状态更新为 UPDATING;
        :将所有目录标记为 WAIT_TO_DELETE;
        :将所有文件标记为 WAIT_TO_DELETE;
    endif
}

:扫描目录统计信息(用于进度计算);
:初始化已处理大小计数器;

if ("是否提供回调函数?") then (是)
    :调用回调函数报告初始进度;
endif

partition "目录遍历阶段" {
    :开始遍历目录树;

    note right: 使用 Files.walkFileTree 方法
    
    :访问目录 (preVisitDirectory);
    
    partition "处理目录" {
        :计算相对路径;
        :创建 InfoDirectory 对象;
        :设置目录属性 (rootId, name, path, mtime, status);
        
        if ("是否存在相同目录?") then (是)
            :使用现有目录ID;
            :恢复目录状态;
            :更新数据库中的目录信息;
        else (否)
            :插入新目录到数据库;
        endif
    }
    
    :访问文件 (visitFile);
    
    partition "处理文件" {
        :计算文件相对路径;
        :获取父目录ID;
        :计算文件哈希值;
        :创建 InfoFile 对象;
        :设置文件属性 (name, size, mtime, directoryId, hash);
        
        if ("是否存在相同文件?") then (是)
            :使用现有文件ID;
            :恢复文件状态;
            :更新数据库中的文件信息;
        else (否)
            :设置文件状态为 UNARCHIVED;
            :插入新文件到数据库;
        endif
        
        :更新已处理大小;
        
        if ("是否提供回调函数且总大小>0?") then (是)
            :计算进度百分比;
            :调用回调函数更新进度;
        endif
    }
    
    :结束目录遍历;
}

if ("是否提供回调函数?") then (是)
    :调用回调函数报告完成进度;
endif

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

### v0.1.1 release

- [ ] API: 支持往指定的根目录指定路径添加指定文件(单一/批量添加)

## 5. 贡献

欢迎提交Issue和Pull Request来改进本项目。

- 本项目 CLA
  详见 [Contributor License Agreement v1 By ZEO](https://gist.github.com/cc01cc/96a194266c6ffbf9f2b8e2c0a5cf97f2)
