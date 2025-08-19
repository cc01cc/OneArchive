/*
 * Copyright 2025. Zheng, Yihong (ZEO, github.com/cc01cc)
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

package com.cc01cc.project.dao;

import com.cc01cc.project.dto.*;
import lombok.extern.slf4j.Slf4j;

import java.sql.*;
import java.util.ArrayList;
import java.util.List;

/**
 * 数据库访问对象类
 * 用于处理数据库的增删改查操作
 */
@Slf4j
public class DatabaseAccessor {

    private final String dbUrl;

    public DatabaseAccessor(String dbUrl) {
        this.dbUrl = dbUrl;
        // 测试数据库连接
        try (Connection conn = DriverManager.getConnection(dbUrl)) {
            if (conn != null) {
                log.info("数据库连接成功: {}", dbUrl);
            }
        } catch (SQLException e) {
            log.error("数据库连接失败", e);
            throw new RuntimeException("数据库连接失败", e);
        }
    }

    /**
     * 获取数据库连接
     *
     * @return 数据库连接
     * @throws SQLException SQL异常
     */
    public Connection getConnection() throws SQLException {
        return DriverManager.getConnection(dbUrl);
    }

    /**
     * 插入根目录信息到 root_index 表
     *
     * @param rootPath 根目录路径
     * @return 插入记录的ID
     */
    public long insertRootPath(String rootPath) {
        String sql = """
                INSERT INTO root_index(root_path, status, created_at, updated_at)
                VALUES (?, 'HEALTH', strftime('%s', 'now'), strftime('%s', 'now'))
                """;

        try (
                Connection conn = getConnection();
                PreparedStatement pstmt = conn.prepareStatement(sql, Statement.RETURN_GENERATED_KEYS)) {

            pstmt.setString(1, rootPath);

            int affectedRows = pstmt.executeUpdate();

            if (affectedRows > 0) {
                try (ResultSet generatedKeys = pstmt.getGeneratedKeys()) {
                    if (generatedKeys.next()) {
                        long id = generatedKeys.getLong(1);
                        log.info("插入根目录信息成功，ID: {}", id);
                        return id;
                    }
                }
            }
        } catch (SQLException e) {
            log.error("插入根目录信息时发生错误", e);
        }
        return -1;
    }

    /**
     * 插入目录信息到 directory_index 表
     *
     * @param directoryInfo 目录信息
     * @return 插入记录的ID
     */
    public long insertDirectory(DirectoryInfo directoryInfo) {
        String sql = """
                INSERT INTO directory_index(root_id, directory_name, directory_mtime, directory_path, status, created_at, updated_at)
                VALUES (?, ?, ?, ?, 'HEALTH', strftime('%s', 'now'), strftime('%s', 'now'))
                """;

        try (
                Connection conn = getConnection();
                PreparedStatement pstmt = conn.prepareStatement(sql, Statement.RETURN_GENERATED_KEYS)) {

            pstmt.setLong(1, directoryInfo.getRootId());
            pstmt.setString(2, directoryInfo.getName());
            pstmt.setLong(3, directoryInfo.getMtime());
            pstmt.setString(4, directoryInfo.getPath());

            int affectedRows = pstmt.executeUpdate();

            if (affectedRows > 0) {
                try (ResultSet generatedKeys = pstmt.getGeneratedKeys()) {
                    if (generatedKeys.next()) {
                        long id = generatedKeys.getLong(1);
                        log.info("插入目录信息成功，ID: {}", id);
                        return id;
                    }
                }
            }
        } catch (SQLException e) {
            log.error("插入目录信息时发生错误", e);
        }
        return -1;
    }

    /**
     * 插入文件信息到 file_index 表
     *
     * @param fileInfo 文件信息
     * @return 插入记录的ID
     */
    public long insertFile(FileInfo fileInfo) {
        String sql = """
                INSERT INTO file_index(directory_id, file_name, file_size, file_mtime, file_hash, volume_count, status, created_at, updated_at)
                VALUES (?, ?, ?, ?, ?, ?, ?, strftime('%s', 'now'), strftime('%s', 'now'))
                """;

        try (
                Connection conn = getConnection();
                PreparedStatement pstmt = conn.prepareStatement(sql, Statement.RETURN_GENERATED_KEYS)) {

            pstmt.setLong(1, fileInfo.getDirectoryId());
            pstmt.setString(2, fileInfo.getName());
            pstmt.setLong(3, fileInfo.getSize());
            pstmt.setLong(4, fileInfo.getMtime());
            pstmt.setString(5, fileInfo.getHash());
            pstmt.setLong(6, fileInfo.getVolumeCount());
            pstmt.setString(7, fileInfo.getStatus());

            int affectedRows = pstmt.executeUpdate();

            if (affectedRows > 0) {
                try (ResultSet generatedKeys = pstmt.getGeneratedKeys()) {
                    if (generatedKeys.next()) {
                        long id = generatedKeys.getLong(1);
                        log.info("插入文件信息成功，ID: {}", id);
                        return id;
                    }
                }
            }
        } catch (SQLException e) {
            log.error("插入文件信息时发生错误", e);
        }
        return -1;
    }

    /**
     * 批量插入目录树关系到 directory_tree 表
     *
     * @param ancestorId   祖先目录ID
     * @param descendantId 后代目录ID
     * @param depth        深度
     */
    public void insertDirectoryTreeRelation(long ancestorId, long descendantId, int depth) {
        String sql = """
                INSERT OR IGNORE INTO directory_tree(ancestor_id, descendant_id, depth, status, created_at, updated_at)
                VALUES (?, ?, ?, 'HEALTH', strftime('%s', 'now'), strftime('%s', 'now'))
                """;

        try (
                Connection conn = getConnection();
                PreparedStatement pstmt = conn.prepareStatement(sql)) {

            pstmt.setLong(1, ancestorId);
            pstmt.setLong(2, descendantId);
            pstmt.setInt(3, depth);

            pstmt.executeUpdate();
            log.info("插入目录树关系成功: ancestor_id={}, descendant_id={}, depth={}", ancestorId, descendantId, depth);
        } catch (SQLException e) {
            log.error("插入目录树关系时发生错误", e);
        }
    }

    /**
     * 插入存档元数据到 archive_metadata 表
     *
     * @param archiveMetadata 存档元数据
     * @return 插入记录的ID
     */
    public long insertArchiveMetadata(ArchiveMetadata archiveMetadata) {
        String sql = """
                INSERT INTO archive_metadata(name, archive_limit_size, archive_hash, is_compressed, compressed_algorithm, is_encrypted, encryption_algorithm, status, created_at, updated_at)
                VALUES (?, ?, ?, ?, ?, ?, ?, 'HEALTH', strftime('%s', 'now'), strftime('%s', 'now'))
                """;

        try (
                Connection conn = getConnection();
                PreparedStatement pstmt = conn.prepareStatement(sql, Statement.RETURN_GENERATED_KEYS)) {

            pstmt.setString(1, archiveMetadata.getName());
            pstmt.setLong(2, archiveMetadata.getArchiveLimitSize());
            pstmt.setString(3, archiveMetadata.getArchiveHash());
            pstmt.setInt(4, archiveMetadata.getIsCompressed() != null ? archiveMetadata.getIsCompressed() : 0);
            pstmt.setString(5, archiveMetadata.getCompressedAlgorithm());
            pstmt.setInt(6, archiveMetadata.getIsEncrypted() != null ? archiveMetadata.getIsEncrypted() : 0);
            pstmt.setString(7, archiveMetadata.getEncryptionAlgorithm());

            int affectedRows = pstmt.executeUpdate();

            if (affectedRows > 0) {
                try (ResultSet generatedKeys = pstmt.getGeneratedKeys()) {
                    if (generatedKeys.next()) {
                        long id = generatedKeys.getLong(1);
                        log.info("插入存档元数据成功，ID: {}", id);
                        return id;
                    }
                }
            }
        } catch (SQLException e) {
            log.error("插入存档元数据时发生错误", e);
        }
        return -1;
    }

    /**
     * 插入存档资源到 archive_asset 表
     *
     * @param archiveAsset 存档资源
     * @return 插入记录的ID
     */
    public long insertArchiveAsset(ArchiveAsset archiveAsset) {
        String sql = """
                INSERT INTO archive_asset(archive_id, asset_name, asset_size, asset_hash, asset_mtime, relative_path, status, created_at, updated_at)
                VALUES (?, ?, ?, ?, ?, ?, ?, strftime('%s', 'now'), strftime('%s', 'now'))
                """;

        try (
                Connection conn = getConnection();
                PreparedStatement pstmt = conn.prepareStatement(sql, Statement.RETURN_GENERATED_KEYS)) {

            pstmt.setLong(1, archiveAsset.getArchiveId());
            pstmt.setString(2, archiveAsset.getAssetName());
            pstmt.setLong(3, archiveAsset.getAssetSize());
            pstmt.setString(4, archiveAsset.getAssetHash());
            pstmt.setLong(5, archiveAsset.getAssetMtime());
            pstmt.setString(6, archiveAsset.getRelativePath());
            pstmt.setString(7, archiveAsset.getStatus());

            int affectedRows = pstmt.executeUpdate();

            if (affectedRows > 0) {
                try (ResultSet generatedKeys = pstmt.getGeneratedKeys()) {
                    if (generatedKeys.next()) {
                        long id = generatedKeys.getLong(1);
                        log.info("插入存档资源成功，ID: {}", id);
                        return id;
                    }
                }
            }
        } catch (SQLException e) {
            log.error("插入存档资源时发生错误", e);
        }
        return -1;
    }

    /**
     * 插入文件与存档资源映射到 file_archive_asset 表
     *
     * @param fileVolumeAsset 文件与存档资源映射
     * @return 插入记录的ID
     */
    public Long insertFileVolumeAsset(FileVolumeAsset fileVolumeAsset) {
        String sql = """
                INSERT INTO file_volume_asset(file_id, asset_id, volume_order, status, created_at, updated_at)
                VALUES (?, ?, ?, ?, strftime('%s', 'now'), strftime('%s', 'now'))
                """;

        try (
                Connection conn = getConnection();
                PreparedStatement pstmt = conn.prepareStatement(sql, Statement.RETURN_GENERATED_KEYS)) {

            pstmt.setLong(1, fileVolumeAsset.getFileId());
            pstmt.setLong(2, fileVolumeAsset.getAssetId());
            pstmt.setLong(3, fileVolumeAsset.getVolumeOrder());
            pstmt.setString(4, fileVolumeAsset.getStatus());

            int affectedRows = pstmt.executeUpdate();

            if (affectedRows > 0) {
                try (ResultSet generatedKeys = pstmt.getGeneratedKeys()) {
                    if (generatedKeys.next()) {
                        long id = generatedKeys.getLong(1);
                        log.info("插入文件与存档资源映射成功，ID: {}", id);
                        return id;
                    }
                }
            }
        } catch (SQLException e) {
            log.error("插入文件与存档资源映射时发生错误", e);
        }
        return null;
    }

    public Long findRootIdByPath(String path) {
        String sql = "SELECT id FROM root_index WHERE root_path = ?";

        try (
                Connection conn = getConnection();
                PreparedStatement pstmt = conn.prepareStatement(sql)) {

            pstmt.setString(1, path);

            try (ResultSet rs = pstmt.executeQuery()) {
                if (rs.next()) {
                    return rs.getLong("id");
                }
            }
        } catch (SQLException e) {
            log.error("查询根目录ID时发生错误", e);
        }
        return null;
    }

    /**
     * 根据完整路径查找目录ID
     *
     * @param path 完整路径
     * @return 目录ID，如果未找到返回-1
     */
    public long findDirectoryIdByPath(long rootId, String path) {
        String sql = "SELECT id FROM directory_index WHERE directory_path = ? AND root_id = ? AND status = 'HEALTH'";

        try (
                Connection conn = getConnection();
                PreparedStatement pstmt = conn.prepareStatement(sql)) {

            pstmt.setString(1, path);
            pstmt.setLong(2, rootId);

            try (ResultSet rs = pstmt.executeQuery()) {
                if (rs.next()) {
                    return rs.getLong("id");
                }
            }
        } catch (SQLException e) {
            log.error("查询目录ID时发生错误", e);
        }
        return -1;
    }

    /**
     * 根据根目录ID查找该目录下的所有文件
     *
     * @param rootId 根目录ID
     * @return 文件信息列表
     */
    public List<FileInfo> findFilesByRootId(long rootId) {
        String sql = """
                SELECT 
                    f.id as file_id,
                    f.file_name,
                    f.file_size,
                    f.file_mtime,
                    f.file_hash,
                    f.directory_id,
                    f.volume_count,
                    d.directory_path,
                    r.root_path
                FROM file_index f
                JOIN directory_index d ON f.directory_id = d.id
                JOIN root_index r ON d.root_id = r.id
                WHERE r.id = ? AND f.status = 'HEALTH'
                """;

        List<FileInfo> files = new ArrayList<>();
        try (
                Connection conn = getConnection();
                PreparedStatement pstmt = conn.prepareStatement(sql)) {

            pstmt.setLong(1, rootId);

            try (ResultSet rs = pstmt.executeQuery()) {
                while (rs.next()) {
                    FileInfo fileInfo = new FileInfo();
                    fileInfo.setId(rs.getLong("id"));
                    fileInfo.setDirectoryId(rs.getLong("directory_id"));
                    fileInfo.setName(rs.getString("file_name"));
                    fileInfo.setSize(rs.getLong("file_size"));
                    fileInfo.setMtime(rs.getLong("file_mtime"));
                    fileInfo.setHash(rs.getString("file_hash"));
                    fileInfo.setVolumeCount(rs.getInt("volume_count"));
                    fileInfo.setStatus(rs.getString("status"));
                    fileInfo.setCreatedAt(rs.getLong("created_at"));
                    fileInfo.setUpdatedAt(rs.getLong("updated_at"));

                    files.add(fileInfo);
                }
            }
        } catch (SQLException e) {
            log.error("查询根目录下的文件时发生错误", e);
        }
        return files;
    }

    /**
     * 根据文件哈希查找存档资源
     *
     * @param fileHash 文件哈希
     * @return 存档资源信息
     */
    public ArchiveAsset findArchiveAssetByHash(String fileHash) {
        String sql = "SELECT * FROM archive_asset WHERE asset_hash = ? AND status = 'HEALTH'";

        try (
                Connection conn = getConnection();
                PreparedStatement pstmt = conn.prepareStatement(sql)) {
            pstmt.setString(1, fileHash);
            try (ResultSet rs = pstmt.executeQuery()) {
                if (rs.next()) {
                    ArchiveAsset archiveAsset = new ArchiveAsset();
                    archiveAsset.setId(rs.getLong("id"));
                    archiveAsset.setArchiveId(rs.getLong("archive_id"));
                    archiveAsset.setAssetName(rs.getString("asset_name"));
                    archiveAsset.setAssetSize(rs.getLong("asset_size"));
                    archiveAsset.setAssetHash(rs.getString("asset_hash"));
                    archiveAsset.setAssetMtime(rs.getLong("asset_mtime"));
                    archiveAsset.setRelativePath(rs.getString("relative_path"));
                    archiveAsset.setStatus(rs.getString("status"));
                    archiveAsset.setCreatedAt(rs.getLong("created_at"));
                    archiveAsset.setUpdatedAt(rs.getLong("updated_at"));
                    return archiveAsset;
                }
            }
        } catch (SQLException e) {
            log.error("查询存档资源时发生错误", e);
        }
        return null;
    }

    public ArchiveMetadata findArchiveMetadataById(long archiveId) {
        String sql = "SELECT * FROM archive_metadata WHERE id = ? AND status = 'HEALTH'";

        try (
                Connection conn = getConnection();
                PreparedStatement pstmt = conn.prepareStatement(sql)) {
            pstmt.setLong(1, archiveId);
            try (ResultSet rs = pstmt.executeQuery()) {
                if (rs.next()) {
                    ArchiveMetadata archiveMetadata = new ArchiveMetadata();
                    archiveMetadata.setId(rs.getLong("id"));
                    archiveMetadata.setName(rs.getString("name"));
                    archiveMetadata.setArchiveLimitSize(rs.getLong("archive_limit_size"));
                    archiveMetadata.setArchiveHash(rs.getString("archive_hash"));
                    archiveMetadata.setIsCompressed(rs.getInt("is_compressed"));
                    archiveMetadata.setCompressedAlgorithm(rs.getString("compressed_algorithm"));
                    archiveMetadata.setIsEncrypted(rs.getInt("is_encrypted"));
                    archiveMetadata.setEncryptionAlgorithm(rs.getString("encryption_algorithm"));
                    archiveMetadata.setStatus(rs.getString("status"));
                    archiveMetadata.setCreatedAt(rs.getLong("created_at"));
                    archiveMetadata.setUpdatedAt(rs.getLong("updated_at"));
                    return archiveMetadata;
                }
            }
        } catch (SQLException e) {
            log.error("查询存档元数据时发生错误", e);
        }
        return null;
    }

    public FileInfo findHealthFileByHash(String fileHash) {

        String sql = "SELECT * FROM file_index WHERE file_hash = ?";

        try (Connection conn = getConnection();
             PreparedStatement pstmt = conn.prepareStatement(sql)) {
            pstmt.setString(1, fileHash);


            try (ResultSet rs = pstmt.executeQuery()) {
                if (rs.next()) {
                    FileInfo fileInfo = new FileInfo();
                    fileInfo.setId(rs.getLong("id"));
                    fileInfo.setDirectoryId(rs.getLong("directory_id"));
                    fileInfo.setName(rs.getString("file_name"));
                    fileInfo.setSize(rs.getLong("file_size"));
                    fileInfo.setMtime(rs.getLong("file_mtime"));
                    fileInfo.setHash(rs.getString("file_hash"));
                    fileInfo.setVolumeCount(rs.getInt("volume_count"));
                    fileInfo.setStatus(rs.getString("status"));
                    fileInfo.setCreatedAt(rs.getLong("created_at"));
                    fileInfo.setUpdatedAt(rs.getLong("updated_at"));
                    return fileInfo;
                }
            }
        } catch (SQLException e) {
            log.error("查询文件时发生错误", e);
        }
        return null;
    }

    public List<FileVolumeAsset> findFileVolumeAssetByFileId(long id) {

        String sql = "SELECT * FROM file_volume_asset WHERE file_id = ? AND status = 'HEALTH'";
        try (Connection conn = getConnection();
             PreparedStatement stmt = conn.prepareStatement(sql)) {

            stmt.setLong(1, id);
            try (ResultSet rs = stmt.executeQuery()) {
                List<FileVolumeAsset> fileVolumeAssets = new ArrayList<>();
                while (rs.next()) {
                    FileVolumeAsset fileVolumeAsset = new FileVolumeAsset();
                    fileVolumeAsset.setId(rs.getLong("id"));
                    fileVolumeAsset.setFileId(rs.getLong("file_id"));
                    fileVolumeAsset.setAssetId(rs.getLong("asset_id"));
                    fileVolumeAsset.setVolumeOrder(rs.getLong("volume_order"));
                    fileVolumeAsset.setStatus(rs.getString("status"));
                    fileVolumeAsset.setCreatedAt(rs.getLong("created_at"));
                    fileVolumeAsset.setUpdatedAt(rs.getLong("updated_at"));
                    fileVolumeAssets.add(fileVolumeAsset);
                }
                return fileVolumeAssets;
            }

        } catch (SQLException e) {
            log.error("查询文件时发生错误", e);
        }
        return null;
    }

    public List<ViewFile> findViewFilesByRootId(long rootId) {
        String sql = "SELECT * FROM v_file WHERE root_id = ?";
        try (Connection conn = getConnection();
             PreparedStatement stmt = conn.prepareStatement(sql)) {
            stmt.setLong(1, rootId);
            try (ResultSet rs = stmt.executeQuery()) {
                List<ViewFile> files = new ArrayList<>();
                while (rs.next()) {
                    ViewFile file = new ViewFile();
                    file.setRootId(rs.getLong("root_id"));
                    file.setRootPath(rs.getString("root_path"));
                    file.setRootStatus(rs.getString("root_status"));
                    file.setDirectoryId(rs.getLong("directory_id"));
                    file.setDirectoryPath(rs.getString("directory_path"));
                    file.setDirectoryMtime(rs.getLong("directory_mtime"));
                    file.setDirectoryStatus(rs.getString("directory_status"));
                    file.setFileId(rs.getLong("file_id"));
                    file.setFileName(rs.getString("file_name"));
                    file.setFileSize(rs.getLong("file_size"));
                    file.setFileMtime(rs.getLong("file_mtime"));
                    file.setFileHash(rs.getString("file_hash"));
                    file.setVolumeCount(rs.getLong("volume_count"));
                    file.setFileStatus(rs.getString("file_status"));

                    files.add(file);
                }
                return files;
            }
        } catch (SQLException e) {
            log.error("查询文件时发生错误", e);
        }
        return null;
    }

    public void updateArchiveMetadata(ArchiveMetadata archiveMetadataById) {
        try (Connection connection = getConnection();
             PreparedStatement preparedStatement = connection.prepareStatement(
                     "UPDATE archive_metadata SET name = ?, archive_limit_size = ?, " +
                             "compressed_algorithm = ?, is_compressed = ?, encryption_algorithm = ?, " +
                             "is_encrypted = ?, status = ?, updated_at = ? WHERE id = ?")) {
            preparedStatement.setString(1, archiveMetadataById.getName());
            preparedStatement.setLong(2, archiveMetadataById.getArchiveLimitSize());
            preparedStatement.setString(3, archiveMetadataById.getCompressedAlgorithm());
            preparedStatement.setInt(4, archiveMetadataById.getIsCompressed());
            preparedStatement.setString(5, archiveMetadataById.getEncryptionAlgorithm());
            preparedStatement.setInt(6, archiveMetadataById.getIsEncrypted());
            preparedStatement.setString(7, archiveMetadataById.getStatus());
            preparedStatement.setLong(8, archiveMetadataById.getCreatedAt());
            preparedStatement.setLong(9, archiveMetadataById.getUpdatedAt());
            preparedStatement.executeUpdate();
        } catch (SQLException e) {
            log.error("更新存档元数据时发生错误", e);
        }
    }

    public List<ViewAsset> findViewAssetByFileId(Long fileId) {
        String sql = "SELECT * FROM v_asset WHERE file_id = ?";
        try (Connection conn = getConnection();
             PreparedStatement stmt = conn.prepareStatement(sql)) {
            stmt.setLong(1, fileId);
            try (ResultSet rs = stmt.executeQuery()) {
                List<ViewAsset> viewAssets = new ArrayList<>();
                while (rs.next()) {
                    ViewAsset viewAsset = new ViewAsset();
                    viewAsset.setArchiveId(rs.getLong("archive_id"));
                    viewAsset.setArchiveName(rs.getString("archive_name"));
                    viewAsset.setArchiveStatus(rs.getString("archive_status"));
                    viewAsset.setAssetId(rs.getLong("asset_id"));
                    viewAsset.setAssetName(rs.getString("asset_name"));
                    viewAsset.setAssetSize(rs.getLong("asset_size"));
                    viewAsset.setAssetHash(rs.getString("asset_hash"));
                    viewAsset.setAssetMtime(rs.getLong("asset_mtime"));
                    viewAsset.setAssetRelativePath(rs.getString("asset_relative_path"));
                    viewAsset.setAssetStatus(rs.getString("asset_status"));
                    viewAsset.setFileId(rs.getLong("file_id"));
                    viewAsset.setVolumeOrder(rs.getLong("volume_order"));
                    viewAssets.add(viewAsset);
                }
                return viewAssets;
            }
        } catch (SQLException e) {
            throw new RuntimeException(e);
        }
    }

    public void updateArchiveAsset(ArchiveAsset asset) {
        try (Connection connection = getConnection();
             PreparedStatement preparedStatement = connection.prepareStatement(
                     "UPDATE archive_asset SET asset_name = ?, asset_size = ?, asset_hash = ?, " +
                             "asset_mtime = ?, relative_path = ?, status = ?, updated_at = ? WHERE id = ?")) {
            preparedStatement.setString(1, asset.getAssetName());
            preparedStatement.setLong(2, asset.getAssetSize());
            preparedStatement.setString(3, asset.getAssetHash());
            preparedStatement.setLong(4, asset.getAssetMtime());
            preparedStatement.setString(5, asset.getRelativePath());
            preparedStatement.setString(6, asset.getStatus());
            preparedStatement.setLong(7, asset.getUpdatedAt());
            preparedStatement.setLong(8, asset.getId());
            preparedStatement.executeUpdate();
        } catch (SQLException e) {
            log.error("更新存档资产时发生错误", e);
        }
    }

    public void updateFileInfo(FileInfo fileInfo) {
        try (Connection connection = getConnection();
             PreparedStatement preparedStatement = connection.prepareStatement(
                     "UPDATE file_index SET id = ?, directory_id = ?,file_hash = ?, file_name=?, file_size =?, file_mtime =?, status = ?, updated_at = ? WHERE id = ?"
             )) {
            preparedStatement.setLong(1, fileInfo.getId());
            preparedStatement.setLong(2, fileInfo.getDirectoryId());
            preparedStatement.setString(3, fileInfo.getHash());
            preparedStatement.setString(4, fileInfo.getName());
            preparedStatement.setLong(5, fileInfo.getSize());
            preparedStatement.setLong(6, fileInfo.getMtime());
            preparedStatement.setString(7, fileInfo.getStatus());
            preparedStatement.setLong(8, fileInfo.getUpdatedAt());
            preparedStatement.setLong(9, fileInfo.getId());
            preparedStatement.executeUpdate();

        } catch (SQLException e) {
            log.error("更新文件信息时发生错误", e);
        }

    }

    public FileInfo findFileInfoById(Long fileId) {
        String sql = "SELECT * FROM file_index WHERE id = ?";
        try (Connection connection = getConnection(); PreparedStatement preparedStatement = connection.prepareStatement(sql)) {
            preparedStatement.setLong(1, fileId);
            try (ResultSet rs = preparedStatement.executeQuery()) {
                if (rs.next()) {
                    FileInfo fileInfo = new FileInfo();
                    fileInfo.setId(rs.getLong("id"));
                    fileInfo.setDirectoryId(rs.getLong("directory_id"));
                    fileInfo.setName(rs.getString("file_name"));
                    fileInfo.setSize(rs.getLong("file_size"));
                    fileInfo.setMtime(rs.getLong("file_mtime"));
                    fileInfo.setHash(rs.getString("file_hash"));
                    fileInfo.setVolumeCount(rs.getInt("volume_count"));
                    fileInfo.setStatus(rs.getString("status"));
                    fileInfo.setCreatedAt(rs.getLong("created_at"));
                    fileInfo.setUpdatedAt(rs.getLong("updated_at"));
                    return fileInfo;
                }
            }
        } catch (SQLException e) {
            log.error("查询文件信息时发生错误", e);
        }
        return null;
    }
}
