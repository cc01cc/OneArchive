//! 数据库查询语句定义
//! 将所有 SQL 查询语句集中管理，提高可维护性
//! 当参数超过 2 个时，必须使用命名参数

/// 归档元数据相关查询
pub mod archive_metadata {
    // 创建归档元数据
    pub const INSERT: &str = r#"
        INSERT INTO archive_metadata 
        (archive_name, archive_uri, archive_hash, archive_size, archive_limit_size, is_compressed, compressed_algorithm, 
         is_encrypted, encryption_algorithm, status)
        VALUES (:archive_name, :archive_uri, :archive_hash, :archive_size, :archive_limit_size, :is_compressed, :compressed_algorithm, 
                :is_encrypted, :encryption_algorithm, :status)
    "#;

    // 根据 ID 查找归档元数据
    pub const SELECT_BY_ID: &str = r#"
        SELECT id, archive_name, archive_uri, archive_limit_size, archive_hash, 
               archive_size, is_compressed, compressed_algorithm, is_encrypted, 
               encryption_algorithm, status, created_at, updated_at 
        FROM archive_metadata 
        WHERE id = ?1
    "#;

    // 根据 URI 查找归档元数据
    pub const SELECT_BY_URI: &str = r#"
        SELECT id, archive_name, archive_uri, archive_limit_size, archive_hash, 
               archive_size, is_compressed, compressed_algorithm, is_encrypted, 
               encryption_algorithm, status, created_at, updated_at 
        FROM archive_metadata 
        WHERE archive_uri = ?1
    "#;

    // 更新归档元数据状态
    pub const UPDATE_STATUS: &str = r#"
        UPDATE archive_metadata 
        SET status = ?1, updated_at = strftime('%s', 'now')
        WHERE id = ?2
    "#;

    pub const UPDATE: &str = r#"
        UPDATE archive_metadata 
        SET archive_name = :archive_name, archive_uri = :archive_uri, archive_limit_size = :archive_limit_size, archive_hash = :archive_hash, 
             archive_size = :archive_size, is_compressed = :is_compressed, compressed_algorithm = :compressed_algorithm, 
             is_encrypted = :is_encrypted, encryption_algorithm = :encryption_algorithm, status = :status, updated_at = strftime('%s', 'now')
        WHERE id = :id
    "#;

    // 删除归档元数据
    pub const DELETE_BY_ID: &str = r#"
        DELETE FROM archive_metadata 
        WHERE id = ?1
    "#;

    // 根据状态 ROOT ID 查找归档元数据
    pub const SELECT_BY_STATUS_AND_ROOT_ID: &str = r#"
    SELECT DISTINCT am.id, am.archive_name, am.archive_uri, am.archive_limit_size, am.archive_hash, am.is_compressed, am.compressed_algorithm, am.is_encrypted, am.encryption_algorithm, am.status, am.created_at, am.updated_at
             FROM archive_metadata am
             JOIN archive_chunk ac ON am.id = ac.archive_id
             JOIN map_file_chunk mfc ON ac.id = mfc.chunk_id
             JOIN info_file f ON mfc.file_id = f.id
             JOIN info_directory d ON f.directory_id = d.id
             WHERE d.root_id = ?1 AND (?2 IS NULL OR am.status = ?2)
             "#;
}

/// 归档数据块相关查询
pub mod archive_chunk {
    pub const INSERT: &str = r#"
        INSERT INTO archive_chunk 
        (archive_id, chunk_name, chunk_size, chunk_hash, chunk_mtime, chunk_relative_path, status)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
    "#;

    pub const UPDATE: &str = r#"
        UPDATE archive_chunk 
        SET archive_id = ?1, chunk_name = ?2, chunk_size = ?3, chunk_hash = ?4, 
            chunk_mtime = ?5, chunk_relative_path = ?6, status = ?7, updated_at = strftime('%s', 'now')
        WHERE id = ?8
    "#;

    pub const SELECT_BY_ID: &str = r#"
        SELECT id, archive_id, chunk_name, chunk_size, chunk_hash, chunk_mtime, chunk_relative_path, status, created_at, updated_at 
        FROM archive_chunk 
        WHERE id = ?1
    "#;

    pub const SELECT_BY_ARCHIVE_ID: &str = r#"
        SELECT id, archive_id, chunk_name, chunk_size, chunk_hash, chunk_mtime, chunk_relative_path, status, created_at, updated_at 
        FROM archive_chunk 
        WHERE archive_id = ?1
    "#;

    pub const SELECT_BY_STATUS: &str = r#"
        SELECT id, archive_id, chunk_name, chunk_size, chunk_hash, chunk_mtime, chunk_relative_path, status, created_at, updated_at 
        FROM archive_chunk 
        WHERE (?1 IS NULL OR status = ?1)
    "#;

    pub const SELECT_ALL: &str = r#"
        SELECT id, archive_id, chunk_name, chunk_size, chunk_hash, chunk_mtime, chunk_relative_path, status, created_at, updated_at 
        FROM archive_chunk
    "#;

    pub const SELECT_BY_CHUNK_HASH: &str = r#"
        SELECT id, archive_id, chunk_name, chunk_size, chunk_hash, chunk_mtime, chunk_relative_path, status, created_at, updated_at 
        FROM archive_chunk 
        WHERE chunk_hash = ?1
    "#;
}

/// 文件与归档数据块映射相关查询
pub mod map_file_chunk {
    pub const INSERT: &str = r#"
        INSERT INTO map_file_chunk 
        (file_id, chunk_id, volume_order, status)
        VALUES (?1, ?2, ?3, ?4)
    "#;

    pub const UPDATE: &str = r#"
        UPDATE map_file_chunk 
        SET file_id = ?1, chunk_id = ?2, volume_order = ?3, status = ?4, updated_at = strftime('%s', 'now')
        WHERE id = ?5
    "#;

    pub const SELECT_BY_ID: &str = r#"
        SELECT id, file_id, chunk_id, volume_order, status, created_at, updated_at 
        FROM map_file_chunk 
        WHERE id = ?1
    "#;

    pub const SELECT_BY_FILE_ID: &str = r#"
        SELECT id, file_id, chunk_id, volume_order, status, created_at, updated_at 
        FROM map_file_chunk 
        WHERE file_id = ?1
    "#;

    pub const SELECT_BY_FILE_ID_ORDERED: &str = r#"
        SELECT id, file_id, chunk_id, volume_order, status, created_at, updated_at 
        FROM map_file_chunk 
        WHERE file_id = ?1 
        ORDER BY volume_order ASC
    "#;

    pub const SELECT_BY_CHUNK_ID: &str = r#"
        SELECT id, file_id, chunk_id, volume_order, status, created_at, updated_at 
        FROM map_file_chunk 
        WHERE chunk_id = ?1
    "#;

    pub const SELECT_BY_STATUS: &str = r#"
        SELECT id, file_id, chunk_id, volume_order, status, created_at, updated_at 
        FROM map_file_chunk 
        WHERE (?1 IS NULL OR status = ?1)
    "#;

    pub const SELECT_BY_FILE_ID_CHUNK_ID_VOLUME_ORDER: &str = r#"
        SELECT id, file_id, chunk_id, volume_order, status, created_at, updated_at 
        FROM map_file_chunk 
        WHERE file_id = ?1 AND chunk_id = ?2 AND volume_order = ?3
    "#;
}

/// 灾备组相关查询
pub mod recovery_group {
    // 创建灾备组
    pub const INSERT: &str = r#"
        INSERT INTO disaster_recovery_group
        (group_id, num_data_shards, num_parity_shards, shard_size, data_shards_stored, status, last_verified)
        VALUES (:group_id, :num_data_shards, :num_parity_shards, :shard_size, :data_shards_stored, :status, :last_verified)
        RETURNING id
    "#;

    // 根据组 ID 查找灾备组
    pub const SELECT_BY_GROUP_ID: &str = r#"
        SELECT id, group_id, num_data_shards, num_parity_shards, shard_size, 
               data_shards_stored, status, last_verified, created_at, updated_at
        FROM disaster_recovery_group 
        WHERE group_id = ?1 AND (?2 IS NULL OR status = ?2)
    "#;

    // 根据 ID 查找灾备组
    pub const SELECT_BY_ID: &str = r#"
        SELECT id, group_id, num_data_shards, num_parity_shards, shard_size, 
               data_shards_stored, status, last_verified, created_at, updated_at
        FROM disaster_recovery_group 
        WHERE id = ?1 AND (?2 IS NULL OR status = ?2)
    "#;

    // 更新灾备组状态
    pub const UPDATE_STATUS: &str = r#"
        UPDATE disaster_recovery_group 
        SET status = ?1, updated_at = strftime('%s', 'now')
        WHERE id = ?2 AND (?3 IS NULL OR status = ?3)
    "#;

    // 删除灾备组
    pub const DELETE_BY_ID: &str = r#"
        DELETE FROM disaster_recovery_group 
        WHERE id = ?1 AND (?2 IS NULL OR status = ?2)
    "#;
}

/// 灾备数据分片相关查询
pub mod recovery_data_shard {
    // 创建数据分片
    pub const INSERT: &str = r#"
        INSERT INTO disaster_recovery_data_shard 
        (group_id, shard_index, shard_path, shard_hash, status)
        VALUES (?1, ?2, ?3, ?4, 'HEALTH')
        RETURNING id
    "#;

    // 根据组 ID 查找所有数据分片
    pub const SELECT_BY_GROUP_ID: &str = r#"
        SELECT id, group_id, shard_index, shard_path, shard_hash, 
               status, last_verified, created_at, updated_at
        FROM disaster_recovery_data_shard 
        WHERE group_id = ?1
        ORDER BY shard_index
    "#;

    // 根据 ID 查找数据分片
    pub const SELECT_BY_ID: &str = r#"
        SELECT id, group_id, shard_index, shard_path, shard_hash, 
               status, last_verified, created_at, updated_at
        FROM disaster_recovery_data_shard 
        WHERE id = ?1
    "#;

    // 更新数据分片状态
    pub const UPDATE_STATUS: &str = r#"
        UPDATE disaster_recovery_data_shard 
        SET status = ?1, updated_at = strftime('%s', 'now')
        WHERE id = ?2
    "#;

    // 删除数据分片
    pub const DELETE_BY_ID: &str = r#"
        DELETE FROM disaster_recovery_data_shard 
        WHERE id = ?1
    "#;
}

/// 灾备校验分片相关查询
pub mod recovery_parity_shard {
    // 创建校验分片
    pub const INSERT: &str = r#"
        INSERT INTO disaster_recovery_parity_shard 
        (group_id, shard_index, shard_path, shard_hash, status)
        VALUES (?1, ?2, ?3, ?4, 'HEALTH')
        RETURNING id
    "#;

    // 根据组 ID 查找所有校验分片
    pub const SELECT_BY_GROUP_ID: &str = r#"
        SELECT id, group_id, shard_index, shard_path, shard_hash, 
               status, last_verified, created_at, updated_at
        FROM disaster_recovery_parity_shard 
        WHERE group_id = ?1
        ORDER BY shard_index
    "#;

    // 根据 ID 查找校验分片
    pub const SELECT_BY_ID: &str = r#"
        SELECT id, group_id, shard_index, shard_path, shard_hash, 
               status, last_verified, created_at, updated_at
        FROM disaster_recovery_parity_shard 
        WHERE id = ?1
    "#;

    // 更新校验分片状态
    pub const UPDATE_STATUS: &str = r#"
        UPDATE disaster_recovery_parity_shard 
        SET status = ?1, updated_at = strftime('%s', 'now')
        WHERE id = ?2
    "#;

    // 删除校验分片
    pub const DELETE_BY_ID: &str = r#"
        DELETE FROM disaster_recovery_parity_shard 
        WHERE id = ?1
    "#;
}

/// 灾备组与归档文件映射相关查询
pub mod recovery_archive {
    // 创建灾备组与归档文件映射
    pub const INSERT: &str = r#"
        INSERT INTO map_recovery_group_archive 
        (group_id, archive_id, status)
        VALUES (?1, ?2, 'HEALTH')
        RETURNING id
    "#;

    // 根据组 ID 查找所有关联的归档文件
    pub const SELECT_BY_GROUP_ID: &str = r#"
        SELECT id, group_id, archive_id, created_at, updated_at
        FROM map_recovery_group_archive 
        WHERE group_id = ?1
    "#;

    // 根据归档 ID 查找所有关联的灾备组
    pub const SELECT_BY_ARCHIVE_ID: &str = r#"
        SELECT id, group_id, archive_id, created_at, updated_at
        FROM map_recovery_group_archive 
        WHERE archive_id = ?1
    "#;

    // 删除灾备组与归档文件的映射关系
    pub const DELETE_BY_GROUP_AND_ARCHIVE_ID: &str = r#"
        DELETE FROM map_recovery_group_archive 
        WHERE group_id = ?1 AND archive_id = ?2
    "#;
}

/// 根目录相关查询
pub mod info_root {
    // 插入或替换根目录信息
    pub const INSERT_OR_REPLACE: &str = r#"
        INSERT OR REPLACE INTO info_root 
        (root_path, root_name, status) 
        VALUES (?1, ?2, ?3)
    "#;

    // 根据路径查找根目录信息
    pub const SELECT_BY_PATH: &str = r#"
        SELECT id, root_name, root_path, status, created_at, updated_at 
        FROM info_root 
        WHERE root_path = ?1
    "#;

    // 更新根目录状态
    pub const UPDATE_STATUS: &str = r#"
        UPDATE info_root 
        SET status = ?1, updated_at = strftime('%s', 'now') 
        WHERE id = ?2
    "#;

    // 查找所有根目录信息
    pub const SELECT_ALL: &str = r#"
        SELECT id, root_name, root_path, status, created_at, updated_at 
        FROM info_root
    "#;
}

/// 目录相关查询
pub mod info_directory {
    // 将指定根目录下的所有目录标记为待删除状态
    pub const UPDATE_WAIT_TO_DELETE: &str = r#"
        UPDATE info_directory 
        SET status = ?1, updated_at = strftime('%s', 'now') 
        WHERE root_id = ?2
    "#;

    // 插入目录信息
    pub const INSERT: &str = r#"
        INSERT INTO info_directory 
        (root_id, directory_name, directory_mtime, directory_path, status) 
        VALUES (?1, ?2, ?3, ?4, ?5)
    "#;

    // 更新目录信息
    pub const UPDATE: &str = r#"
        UPDATE info_directory 
        SET directory_name = ?1, directory_mtime = ?2, directory_path = ?3, status = ?4, updated_at = strftime('%s', 'now') 
        WHERE id = ?5
    "#;

    // 根据路径查找目录
    pub const SELECT_BY_PATH: &str = r#"
        SELECT id, root_id, directory_name, directory_mtime, directory_path, status, created_at, updated_at 
        FROM info_directory 
        WHERE root_id = ?1 AND directory_path = ?2
    "#;

    // 根据 ID 查找目录
    pub const SELECT_BY_ID: &str = r#"
        SELECT id, root_id, directory_name, directory_mtime, directory_path, status, created_at, updated_at 
        FROM info_directory 
        WHERE id = ?1
    "#;

    // 根据状态和根目录 ID 查找目录
    pub const SELECT_BY_STATUS_AND_ROOT_ID: &str = r#"
        SELECT id, root_id, directory_name, directory_mtime, directory_path, status, created_at, updated_at 
        FROM info_directory 
        WHERE root_id = ?1 AND (?2 IS NULL OR status = ?2)
    "#;

    // 根据根目录 ID 查找所有目录
    pub const SELECT_BY_ROOT_ID: &str = r#"
        SELECT id, root_id, directory_name, directory_mtime, directory_path, status, created_at, updated_at 
        FROM info_directory 
        WHERE root_id = ?1
    "#;

    // 查找某个目录下的所有直接子目录
    pub const SELECT_CHILD_DIRECTORIES: &str = r#"
        SELECT id, root_id, directory_name, directory_mtime, directory_path, status, created_at, updated_at
        FROM info_directory
        WHERE root_id = ?1 AND directory_path LIKE 
            CASE 
                WHEN ?2 = '' THEN '%'
                ELSE ?2 || '/%'
            END
        AND directory_path NOT LIKE 
            CASE 
                WHEN ?2 = '' THEN '%/%'
                ELSE ?2 || '/%/%'
            END
    "#;
}

/// 文件相关查询
pub mod info_file {
    // 将指定根目录下的所有文件标记为待删除状态
    pub const UPDATE_WAIT_TO_DELETE: &str = r#"
        UPDATE info_file 
        SET status = ?1, updated_at = strftime('%s', 'now') 
        WHERE directory_id IN (SELECT id FROM info_directory WHERE root_id = ?2)
    "#;

    // 根据名称查找文件
    pub const SELECT_BY_NAME: &str = r#"
        SELECT id, directory_id, file_name, file_size, file_mtime, file_hash, status, created_at, updated_at 
        FROM info_file 
        WHERE directory_id = ?1 AND file_name = ?2
    "#;

    // 根据状态和根目录 ID 查找文件
    pub const SELECT_BY_STATUS_AND_ROOT_ID: &str = r#"
        SELECT * FROM info_file
        WHERE directory_id IN (SELECT id FROM info_directory WHERE root_id = ?1) AND (?2 IS NULL OR status = ?2)
    "#;

    // 根据根目录 ID 查找所有文件
    pub const SELECT_BY_ROOT_ID: &str = r#"
        SELECT * FROM info_file
        WHERE directory_id IN (SELECT id FROM info_directory WHERE root_id = ?1)
    "#;

    // 插入文件信息
    pub const INSERT: &str = r#"
        INSERT INTO info_file 
        (directory_id, file_name, file_size, file_mtime, file_hash, status) 
        VALUES (?1, ?2, ?3, ?4, ?5, ?6)
    "#;

    // 更新文件信息
    pub const UPDATE: &str = r#"
        UPDATE info_file 
        SET file_name = ?1, file_size = ?2, file_mtime = ?3, file_hash = ?4, status = ?5, updated_at = strftime('%s', 'now') 
        WHERE id = ?6
    "#;
}

/// 文件视图相关查询
pub mod view_file {
    // 根据根目录 ID 查找所有视图文件
    pub const SELECT_BY_ROOT_ID: &str = r#"
        SELECT root_id, root_path, root_status, directory_id, directory_path, directory_mtime, directory_status,
               file_id, file_name, file_size, file_mtime, file_hash, file_status
        FROM view_file 
        WHERE root_id = ?1
    "#;

    // 根据目录 ID 查找视图文件
    pub const SELECT_BY_DIRECTORY_ID: &str = r#"
        SELECT root_id, root_path, root_status, directory_id, directory_path, directory_mtime, directory_status,
               file_id, file_name, file_size, file_mtime, file_hash, file_status
        FROM view_file 
        WHERE directory_id = ?1
    "#;

    // 根据 root_id 和 status 获取视图文件
    pub const SELECT_BY_ROOT_ID_AND_STATUS: &str = r#"
        SELECT root_id, root_path, root_status, directory_id, directory_path, directory_mtime, directory_status,
               file_id, file_name, file_size, file_mtime, file_hash, file_status
        FROM view_file 
        WHERE root_id = ?1 AND (?2 IS NULL OR file_status = ?2)
    "#;

    // 根据 hash 和 status 获取视图文件
    pub const SELECT_BY_HASH_AND_STATUS: &str = r#"
        SELECT root_id, root_path, root_status, directory_id, directory_path, directory_mtime, directory_status,
               file_id, file_name, file_size, file_mtime, file_hash, file_status
        FROM view_file 
        WHERE file_hash = ?1 AND (?2 IS NULL OR file_status = ?2)
    "#;
}

/// 数据块视图相关查询
pub mod view_chunk {
    // 根据归档 ID 查找视图数据块
    pub const SELECT_BY_ARCHIVE_ID: &str = r#"
        SELECT archive_id, archive_uri, archive_name, archive_status, chunk_id, chunk_name, chunk_size, chunk_hash, 
               chunk_mtime, chunk_relative_path, chunk_status, file_id, volume_order
        FROM view_chunk 
        WHERE archive_id = ?1
    "#;

    // 根据文件 ID 查找视图数据块
    pub const SELECT_BY_FILE_ID: &str = r#"
        SELECT archive_id, archive_uri, archive_name, archive_status, chunk_id, chunk_name, chunk_size, chunk_hash, 
               chunk_mtime, chunk_relative_path, chunk_status, file_id, volume_order
        FROM view_chunk 
        WHERE file_id = ?1
    "#;
}
/// 归档文件与数据分片映射相关查询
pub mod archive_shard_map {
    // 创建映射记录
    pub const INSERT: &str = r#"
        INSERT INTO map_archive_data_shard
        (shard_id, archive_id, archive_order, original_archive_hash, status)
        VALUES (?1, ?2, ?3, ?4, 'HEALTH')
        RETURNING id
    "#;

    // 根据分片 ID 查找映射记录
    pub const SELECT_BY_SHARD_ID: &str = r#"
        SELECT id, shard_id, archive_id, archive_order, original_archive_hash, status, created_at, updated_at
        FROM map_archive_data_shard 
        WHERE shard_id = ?1
    "#;

    // 根据归档文件 ID 查找映射记录
    pub const SELECT_BY_ARCHIVE_ID: &str = r#"
        SELECT id, shard_id, archive_id, archive_order, original_archive_hash, status, created_at, updated_at
        FROM map_archive_data_shard 
        WHERE archive_id = ?1 AND (?2 IS NULL OR status = ?2)
    "#;

    pub const SELECT_BY_ARCHIVE_ID_AND_SHARD_ID: &str = r#"
        SELECT id, shard_id, archive_id, archive_order, original_archive_hash, status, created_at, updated_at
        FROM map_archive_data_shard 
        WHERE archive_id = ?1 AND shard_id = ?2 AND (?3 IS NULL OR status = ?3)
    "#;

    // 此查询已废弃，因为 map_archive_data_shard 表不再包含 group_id 字段
    // pub const SELECT_BY_ARCHIVE_ID_AND_GROUP_ID: &str = r#"
    //     SELECT id, shard_id, archive_id, archive_order, original_archive_hash, status, created_at, updated_at
    //     FROM map_archive_data_shard
    //     WHERE archive_id = ?1 AND group_id = ?2 AND (?3 IS NULL OR status = ?3)
    // "#;
    // 更新映射记录状态
    pub const UPDATE_STATUS: &str = r#"
        UPDATE map_archive_data_shard 
        SET status = ?1, updated_at = strftime('%s', 'now')
        WHERE shard_id = ?2
    "#;

    // 删除映射记录
    pub const DELETE_BY_SHARD_ID: &str = r#"
        DELETE FROM map_archive_data_shard 
        WHERE shard_id = ?1
    "#;
}

/// 灾备组与归档文件映射相关 SQL 查询语句
pub mod map_archive_recovery_group {
    /// 插入新的灾备组与归档文件映射
    pub const INSERT: &str = r#"
    INSERT INTO map_recovery_group_archive (group_id, archive_id, status)
    VALUES (?1, ?2, 'HEALTH')
    RETURNING id
"#;

    /// 根据组 ID 查询所有关联的归档文件映射
    pub const SELECT_BY_GROUP_ID: &str = r#"
    SELECT id, group_id, archive_id, status, created_at, updated_at
    FROM map_recovery_group_archive 
    WHERE group_id = ?1
"#;

    /// 根据归档 ID 查询所有关联的灾备组映射
    pub const SELECT_BY_ARCHIVE_ID: &str = r#"
    SELECT id, group_id, archive_id, status, created_at, updated_at
    FROM map_recovery_group_archive 
    WHERE archive_id = ?1
"#;

    /// 删除指定的灾备组与归档文件映射
    pub const DELETE_BY_GROUP_AND_ARCHIVE_ID: &str = r#"
    DELETE FROM map_recovery_group_archive 
    WHERE group_id = ?1 AND archive_id = ?2
"#;

    /// 根据组 ID 删除所有关联的归档文件映射
    pub const DELETE_BY_GROUP_ID: &str = r#"
    DELETE FROM map_recovery_group_archive 
    WHERE group_id = ?1
"#;

    /// 根据归档 ID 删除所有关联的灾备组映射
    pub const DELETE_BY_ARCHIVE_ID: &str = r#"
    DELETE FROM map_recovery_group_archive 
    WHERE archive_id = ?1
"#;

    /// 检查指定的灾备组与归档文件映射是否存在
    pub const COUNT_BY_GROUP_AND_ARCHIVE_ID: &str = r#"
    SELECT COUNT(*) 
    FROM map_recovery_group_archive 
    WHERE group_id = ?1 AND archive_id = ?2
"#;
}
