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

package com.cc01cc.onearchive.core.dao;

import com.cc01cc.onearchive.core.entity.*;
import com.cc01cc.onearchive.core.mapper.RowMapperConfig;
import lombok.extern.slf4j.Slf4j;
import org.jdbi.v3.core.Jdbi;
import org.jdbi.v3.sqlobject.SqlObjectPlugin;

import java.sql.Connection;
import java.sql.DriverManager;
import java.sql.SQLException;
import java.util.List;

/**
 * 数据库访问对象类
 * 用于处理数据库的增删改查操作
 */
@Slf4j
public class DatabaseAccessor {

    private final String dbUrl;
    private final Jdbi jdbi;

    public DatabaseAccessor(String dbUrl) {
        this.dbUrl = dbUrl;
        this.jdbi = Jdbi.create(dbUrl);
        // 注册SqlObjectPlugin插件以支持DAO接口
        this.jdbi.installPlugin(new SqlObjectPlugin());
        // 注册所有自定义的 RowMapper
        RowMapperConfig.registerRowMappers(this.jdbi);
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
     * 插入根目录信息到 info_root 表
     *
     * @param rootPath 根目录路径
     * @return 插入记录的ID
     */
    public long insertRootPath(String rootPath) {
        InfoRoot infoRoot = new InfoRoot();
        infoRoot.setPath(rootPath);
        infoRoot.setStatus("HEALTH");
        try {
            return jdbi.withExtension(CommonDao.class, dao -> dao.insertRoot(infoRoot));
        } catch (Exception e) {
            log.error("插入根目录信息时发生错误", e);
            return -1;
        }
    }

    /**
     * 插入文件信息到 info_file 表
     *
     * @param infoFile 文件信息
     * @return 插入记录的ID
     */
    public long insertFile(InfoFile infoFile) {
        try {
            return jdbi.withExtension(CommonDao.class, dao -> dao.insertFile(infoFile));
        } catch (Exception e) {
            log.error("插入文件信息时发生错误", e);
            return -1;
        }
    }


    /**
     * 插入存档元数据到 archive_metadata 表
     *
     * @param archiveMetadata 存档元数据
     * @return 插入记录的ID
     */
    public long insertArchiveMetadata(ArchiveMetadata archiveMetadata) {
        try {
            return jdbi.withExtension(CommonDao.class, dao -> dao.insertArchiveMetadata(archiveMetadata));
        } catch (Exception e) {
            log.error("插入存档元数据时发生错误", e);
            return -1;
        }
    }

    /**
     * 插入存档资源到 archive_asset 表
     *
     * @param archiveAsset 存档资源
     * @return 插入记录的ID
     */
    public long insertArchiveAsset(ArchiveAsset archiveAsset) {
        try {
            return jdbi.withExtension(CommonDao.class, dao -> dao.insertArchiveAsset(archiveAsset));
        } catch (Exception e) {
            log.error("插入存档资源时发生错误", e);
            return -1;
        }
    }

    /**
     * 插入文件与存档资源映射到 map_file_asset 表
     *
     * @param mapFileAsset 文件与存档资源映射
     * @return 插入记录的ID
     */
    public Long insertFileVolumeAsset(MapFileAsset mapFileAsset) {
        try {
            return jdbi.withExtension(CommonDao.class, dao -> dao.insertMapFileAsset(mapFileAsset));
        } catch (Exception e) {
            log.error("插入文件与存档资源映射时发生错误", e);
            return null;
        }
    }

    public Long findRootIdByPath(String path) {
        try {
            return jdbi.withExtension(CommonDao.class, dao -> dao.findRootIdByPath(path));
        } catch (Exception e) {
            log.error("查询根目录ID时发生错误", e);
            return null;
        }
    }

    /**
     * 根据完整路径查找目录ID
     *
     * @param path 完整路径
     * @return 目录ID，如果未找到返回-1
     */
    public long findDirectoryIdByPath(long rootId, String path, String status) {
        try {
            Long id = jdbi.withExtension(CommonDao.class, dao -> dao.findDirectoryIdByPath(rootId, path, status));
            return id != null ? id : -1;
        } catch (Exception e) {
            log.error("查询目录ID时发生错误", e);
            return -1;
        }
    }

    /**
     * 根据根目录ID查找该目录下的所有文件
     *
     * @param rootId 根目录ID
     * @return 文件信息列表
     */
    public List<InfoFile> findFilesByStatusRootId(long rootId, String status) {
        return jdbi.withExtension(CommonDao.class, dao -> dao.findFilesByRootId(rootId, status));
    }

    public List<InfoDirectory> findDirectoriesByStatusAndRootId(long rootId, String status) {
        return jdbi.withExtension(CommonDao.class, dao -> dao.findDirectoriesByRootId(rootId, status));
    }

    /**
     * 根据文件哈希查找存档资源
     *
     * @param fileHash 文件哈希
     * @return 存档资源信息
     */
    public ArchiveAsset findArchiveAssetByHash(String fileHash, String status) {
        try {
            return jdbi.withExtension(CommonDao.class, dao -> dao.findArchiveAssetByHash(fileHash, status));
        } catch (Exception e) {
            log.error("查询存档资源时发生错误", e);
            return null;
        }
    }

    public ArchiveMetadata findArchiveMetadataById(long archiveId, String status) {
        try {
            return jdbi.withExtension(CommonDao.class, dao -> dao.findArchiveMetadataById(archiveId, status));
        } catch (Exception e) {
            log.error("查询存档元数据时发生错误", e);
            return null;
        }
    }

    /**
     * 根据文件哈希和状态查找文件
     *
     * @param fileHash 文件哈希值
     * @param status   文件状态
     * @return 文件信息，如果未找到返回null
     */
    public List<InfoFile> findFileByHashAndStatus(String fileHash, String status) {
        try {
            return jdbi.withExtension(CommonDao.class, dao -> dao.findFileByHashAndStatus(fileHash, status));
        } catch (Exception e) {
            log.error("根据哈希值和状态查询文件时发生错误", e);
            return null;
        }
    }

    public List<MapFileAsset> findFileVolumeAssetByFileId(long id, String status) {
        try {
            return jdbi.withExtension(CommonDao.class, dao -> dao.findMapFileAssetByFileId(id, status));
        } catch (Exception e) {
            log.error("查询文件时发生错误", e);
            return null;
        }
    }

    public List<ViewFile> findViewFilesByRootId(long rootId) {
        try {
            return jdbi.withExtension(CommonDao.class, dao -> dao.findViewFilesByRootId(rootId));
        } catch (Exception e) {
            log.error("查询文件时发生错误", e);
            return null;
        }
    }

    public void updateArchiveMetadata(ArchiveMetadata archiveMetadataById) {
        try {
            jdbi.withExtension(CommonDao.class, dao -> dao.updateArchiveMetadata(archiveMetadataById));
        } catch (Exception e) {
            log.error("更新存档元数据时发生错误", e);
        }
    }

    public List<ViewAsset> findViewAssetByFileId(Long fileId) {
        try {
            return jdbi.withExtension(CommonDao.class, dao -> dao.findViewAssetByFileId(fileId));
        } catch (Exception e) {
            log.error("查询资产时发生错误", e);
            throw new RuntimeException(e);
        }
    }

    public void updateArchiveAsset(ArchiveAsset asset) {
        try {
            jdbi.withExtension(CommonDao.class, dao -> dao.updateArchiveAsset(asset));
        } catch (Exception e) {
            log.error("更新存档资产时发生错误", e);
        }
    }

    public void updateFileInfo(InfoFile infoFile) {
        try {
            jdbi.withExtension(CommonDao.class, dao -> dao.updateFileInfo(infoFile));
        } catch (Exception e) {
            log.error("更新文件信息时发生错误", e);
        }
    }

    public InfoFile findFileInfoById(Long fileId) {
        try {
            return jdbi.withExtension(CommonDao.class, dao -> dao.findFileById(fileId));
        } catch (Exception e) {
            log.error("查询文件信息时发生错误", e);
            return null;
        }
    }

    /**
     * 插入目录信息到 info_directory 表
     *
     * @param infoDirectory 目录信息
     * @return 插入记录的ID
     */
    public long insertDirectory(InfoDirectory infoDirectory) {
        try {
            return jdbi.withExtension(CommonDao.class, dao -> dao.insertDirectory(infoDirectory));
        } catch (Exception e) {
            log.error("插入参数: {}", infoDirectory);
            log.error("插入目录信息时发生错误", e);
            return -1;
        }
    }

    public List<InfoRoot> getHealthRootDirList() {
        try {
            return jdbi.withExtension(CommonDao.class, dao -> dao.getHealthRootDirList());
        } catch (Exception e) {
            log.error("查询根目录信息时发生错误", e);
            return null;
        }
    }

    public List<ViewFile> findViewFilesWithStatusAndRootId(Long rootId, String status) {
        return jdbi.withExtension(CommonDao.class, dao -> dao.findViewFilesWithStatusAndRootId(rootId, status));
    }

    public InfoRoot findRootInfoByPath(String rootAbsolutePath) {
        try {
            return jdbi.withExtension(CommonDao.class, dao -> dao.findRootByPath(rootAbsolutePath));
        } catch (Exception e) {
            log.error("查询根目录信息时发生错误", e);
            return null;
        }
    }

    public void updateStatusById(String tableName, Long id, String status) {
        jdbi.withExtension(CommonDao.class, dao -> dao.updateStatusById(tableName, id, status));
    }

    public int[] batchUpdateStatusById(String tableName, List<Long> ids, String status) {
        return jdbi.withExtension(CommonDao.class, dao -> dao.batchUpdateStatusById(tableName, ids, status));
    }

    /**
     * 更新目录信息
     *
     * @param infoDirectory 目录信息
     * @return 更新记录数
     */
    public int updateDirectory(InfoDirectory infoDirectory) {
        return jdbi.withExtension(CommonDao.class, dao -> dao.updateDirectory(infoDirectory));
    }
}
