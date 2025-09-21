//! 数据库初始化模块
use log::info;
use rusqlite::{Connection, Result as SqliteResult};

/// 初始化数据库表结构
///
/// # 参数
/// * `conn` - 数据库连接引用
pub fn initialize_tables(conn: &Connection) -> SqliteResult<()> {
    // 创建根目录索引表 info_root
    conn.execute(
        "CREATE TABLE IF NOT EXISTS info_root (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        root_name TEXT,
        root_path TEXT NOT NULL UNIQUE,
        status TEXT NOT NULL DEFAULT 'HEALTH',
        created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
        updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
    )",
        [],
    )?;
    info!("已创建 info_root 表");

    // 创建目录索引表 info_directory
    conn.execute(
        "CREATE TABLE IF NOT EXISTS info_directory (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        root_id INTEGER NOT NULL,
        directory_name TEXT NOT NULL DEFAULT '',
        directory_mtime INTEGER NOT NULL DEFAULT 0,
        directory_path TEXT,
        status TEXT NOT NULL DEFAULT 'HEALTH',
        created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
        updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
        FOREIGN KEY (root_id) REFERENCES info_root(id) ON DELETE CASCADE,
        UNIQUE(root_id, directory_path)
    )",
        [],
    )?;
    info!("已创建 info_directory 表");

    // 创建文件索引表 info_file
    conn.execute(
        "CREATE TABLE IF NOT EXISTS info_file (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        directory_id INTEGER NOT NULL,
        file_name TEXT NOT NULL,
        file_size INTEGER NOT NULL DEFAULT 0,
        file_mtime INTEGER NOT NULL DEFAULT 0,
        file_hash TEXT,
        status TEXT NOT NULL DEFAULT 'HEALTH',
        created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
        updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
        FOREIGN KEY (directory_id) REFERENCES info_directory(id) ON DELETE CASCADE,
        UNIQUE(directory_id, file_name)
    )",
        [],
    )?;
    info!("已创建 info_file 表");

    // 创建归档元数据表 archive_metadata
    conn.execute(
        "CREATE TABLE IF NOT EXISTS archive_metadata (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        archive_name TEXT NOT NULL,
        archive_uri TEXT NOT NULL UNIQUE,
        archive_limit_size INTEGER NOT NULL,
        archive_hash TEXT,
        is_compressed INTEGER NOT NULL DEFAULT 0,
        compressed_algorithm TEXT,
        is_encrypted INTEGER NOT NULL DEFAULT 0,
        encryption_algorithm TEXT,
        status TEXT NOT NULL DEFAULT 'HEALTH',
        created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
        updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
    )",
        [],
    )?;
    info!("已创建 archive_metadata 表");

    // 创建归档内容表 archive_chunk
    conn.execute(
        "CREATE TABLE IF NOT EXISTS archive_chunk (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        archive_id INTEGER NOT NULL,
        chunk_name TEXT NOT NULL UNIQUE,
        chunk_size INTEGER NOT NULL DEFAULT 0,
        chunk_hash TEXT NOT NULL UNIQUE,
        chunk_mtime INTEGER NOT NULL DEFAULT 0,
        chunk_relative_path TEXT NOT NULL DEFAULT './',
        status TEXT NOT NULL DEFAULT 'HEALTH',
        created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
        updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
        FOREIGN KEY (archive_id) REFERENCES archive_metadata(id) ON DELETE CASCADE
    )",
        [],
    )?;
    info!("已创建 archive_chunk 表");

    // 创建文件与归档数据块表 map_file_chunk
    conn.execute(
        "CREATE TABLE IF NOT EXISTS map_file_chunk (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        file_id INTEGER NOT NULL,
        chunk_id INTEGER NOT NULL,
        volume_order INTEGER NOT NULL DEFAULT 1,
        status TEXT NOT NULL DEFAULT 'HEALTH',
        created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
        updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
        FOREIGN KEY (file_id) REFERENCES info_file(id) ON DELETE CASCADE,
        FOREIGN KEY (chunk_id) REFERENCES archive_chunk(id) ON DELETE CASCADE,
        UNIQUE(file_id, chunk_id)
    )",
        [],
    )?;
    info!("已创建 map_file_chunk 表");

    // 创建文件列表视图 view_file
    conn.execute(
        "CREATE VIEW IF NOT EXISTS view_file AS
            SELECT
                r.id as root_id,
                r.root_path,
                r.status as root_status,
                d.id as directory_id,
                d.directory_path,
                d.directory_mtime,
                d.status as directory_status,
                f.id as file_id,
                f.file_name,
                f.file_size,
                f.file_mtime,
                f.file_hash,
                f.status as file_status
            FROM info_root r
            JOIN info_directory d ON r.id = d.root_id
            JOIN info_file f ON d.id = f.directory_id",
        [],
    )?;
    info!("已创建 view_file 视图");

    // 创建数据块列表视图 view_chunk
    conn.execute(
        "CREATE VIEW IF NOT EXISTS view_chunk AS
            SELECT
                am.id as archive_id,
                am.archive_uri as archive_uri,
                am.archive_name as archive_name,
                am.status as archive_status,
                aa.id as chunk_id,
                aa.chunk_name,
                aa.chunk_size,
                aa.chunk_hash,
                aa.chunk_mtime,
                aa.chunk_relative_path as chunk_relative_path,
                aa.status as chunk_status,
                fva.file_id,
                fva.volume_order
            FROM archive_metadata am
            JOIN archive_chunk aa ON am.id = aa.archive_id
            LEFT JOIN map_file_chunk fva ON aa.id = fva.chunk_id",
        [],
    )?;
    info!("已创建 view_chunk 视图");

    Ok(())
}