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
import org.jdbi.v3.sqlobject.customizer.Bind;
import org.jdbi.v3.sqlobject.customizer.BindBean;
import org.jdbi.v3.sqlobject.customizer.Define;
import org.jdbi.v3.sqlobject.statement.GetGeneratedKeys;
import org.jdbi.v3.sqlobject.statement.SqlBatch;
import org.jdbi.v3.sqlobject.statement.SqlQuery;
import org.jdbi.v3.sqlobject.statement.SqlUpdate;

import java.util.List;

public interface CommonDao {

    @SqlUpdate("update <tableName> set status = :status where id = :id")
    int updateStatusById(@Define("tableName") String tableName, @Bind("id") Long id, @Bind("status") String status);

    @SqlBatch("update <tableName> set status = :status where id = :id")
    int[] batchUpdateStatusById(@Define("tableName") String tableName, @Bind("id") List<Long> ids, @Bind("status") String status);

    @SqlQuery("SELECT * FROM info_file WHERE file_hash = :fileHash AND status = :status")
    List<InfoFile> findFileByHashAndStatus(@Bind("fileHash") String fileHash, @Bind("status") String status);

    @SqlQuery("""
            SELECT 
                f.root_id,
                f.root_path,
                f.root_status,
                f.directory_id,
                f.directory_path,
                f.directory_mtime,
                f.directory_status,
                f.file_id,
                f.file_name,
                f.file_size,
                f.file_mtime,
                f.file_hash,
                f.file_status
            FROM view_file f
            WHERE f.root_id = :rootId
            AND (:status IS NULL OR f.file_status = :status)
            """)
    List<ViewFile> findViewFilesWithStatusAndRootId(@Bind("rootId") long rootId, @Bind("status") String status);

    @SqlQuery("""
            SELECT
                f.id,
                f.directory_id,
                f.file_name,
                f.file_size,
                f.file_mtime,
                f.file_hash,
                f.status,
                f.created_at,
                f.updated_at
            FROM info_file f
            JOIN info_directory d ON f.directory_id = d.id
            JOIN info_root r ON d.root_id = r.id
            WHERE r.id = :rootId
            AND (:status IS NULL OR f.status = :status)
            """)
    List<InfoFile> findFilesByRootId(@Bind("rootId") long rootId, @Bind("status") String status);

    @SqlQuery("""
            SELECT
                d.id,
                d.root_id,
                d.directory_name,
                d.directory_path,
                d.directory_mtime,
                d.status,
                d.created_at,
                d.updated_at
            FROM info_directory d
            JOIN info_root r ON d.root_id = r.id
            WHERE r.id = :rootId
            AND (:status IS NULL OR d.status = :status)
            """)
    List<InfoDirectory> findDirectoriesByRootId(@Bind("rootId") long rootId, @Bind("status") String status);

    @SqlUpdate("""
            UPDATE info_directory
            SET root_id = :infoDirectory.rootId,
                directory_name = :infoDirectory.name,
                directory_mtime = :infoDirectory.mtime,
                directory_path = :infoDirectory.path,
                status = :infoDirectory.status,
                updated_at = strftime('%s', 'now')
            WHERE id = :infoDirectory.id
            """)
    int updateDirectory(@BindBean("infoDirectory") InfoDirectory infoDirectory);

    @SqlUpdate("""
            UPDATE info_file
            SET directory_id = :infoFile.directoryId,
                file_name = :infoFile.name,
                file_size = :infoFile.size,
                file_mtime = :infoFile.mtime,
                file_hash = :infoFile.hash,
                status = :infoFile.status,
                updated_at = strftime('%s', 'now')
            WHERE id = :infoFile.id
            """)
    int updateFileInfo(@BindBean("infoFile") InfoFile infoFile);

    @SqlUpdate("""
            INSERT INTO info_directory(root_id, directory_name, directory_mtime, directory_path, status, created_at, updated_at)
            VALUES (:infoDirectory.rootId, :infoDirectory.name, :infoDirectory.mtime, :infoDirectory.path, :infoDirectory.status, strftime('%s', 'now'), strftime('%s', 'now'))
            """)
    @GetGeneratedKeys
    long insertDirectory(@BindBean("infoDirectory") InfoDirectory infoDirectory);

    @SqlUpdate("""
            INSERT INTO info_file(directory_id, file_name, file_size, file_mtime, file_hash, status, created_at, updated_at)
            VALUES (:infoFile.directoryId, :infoFile.name, :infoFile.size, :infoFile.mtime, :infoFile.hash, :infoFile.status, strftime('%s', 'now'), strftime('%s', 'now'))
            """)
    @GetGeneratedKeys
    long insertFile(@BindBean("infoFile") InfoFile infoFile);

    @SqlUpdate("""
            INSERT INTO info_root(root_path, root_name, status, created_at, updated_at)
            VALUES (:infoRoot.path, :infoRoot.name, :infoRoot.status, strftime('%s', 'now'), strftime('%s', 'now'))
            """)
    @GetGeneratedKeys
    long insertRoot(@BindBean("infoRoot") InfoRoot infoRoot);

    @SqlQuery("SELECT * FROM info_root WHERE root_path = :rootPath")
    InfoRoot findRootByPath(@Bind("rootPath") String rootPath);

    @SqlQuery("SELECT * FROM info_root WHERE id = :id")
    InfoRoot findRootById(@Bind("id") Long id);

    @SqlQuery("SELECT * FROM info_root WHERE status = 'HEALTH'")
    List<InfoRoot> getHealthRootDirList();

    @SqlQuery("SELECT * FROM info_file WHERE id = :id")
    InfoFile findFileById(@Bind("id") Long id);

    @SqlQuery("SELECT * FROM view_asset WHERE file_id = :fileId")
    List<ViewAsset> findViewAssetByFileId(@Bind("fileId") Long fileId);

    @SqlQuery("SELECT * FROM view_file WHERE root_id = :rootId")
    List<ViewFile> findViewFilesByRootId(@Bind("rootId") Long rootId);

    @SqlQuery("SELECT id FROM info_directory WHERE directory_path = :path AND root_id = :rootId AND (:status IS NULL OR status = :status)")
    Long findDirectoryIdByPath(@Bind("rootId") long rootId, @Bind("path") String path, @Bind("status") String status);

    @SqlQuery("SELECT id FROM info_root WHERE root_path = :path")
    Long findRootIdByPath(@Bind("path") String path);

    @SqlUpdate("""
            INSERT INTO archive_metadata(archive_name, archive_limit_size, archive_hash, is_compressed, compressed_algorithm, is_encrypted, encryption_algorithm, status, created_at, updated_at)
            VALUES (:archiveMetadata.name, :archiveMetadata.limitSize, :archiveMetadata.hash, :archiveMetadata.isCompressed, :archiveMetadata.compressedAlgorithm, :archiveMetadata.isEncrypted, :archiveMetadata.encryptionAlgorithm, :archiveMetadata.status, strftime('%s', 'now'), strftime('%s', 'now'))
            """)
    @GetGeneratedKeys
    long insertArchiveMetadata(@BindBean("archiveMetadata") ArchiveMetadata archiveMetadata);

    @SqlUpdate("""
            INSERT INTO archive_asset(archive_id, asset_name, asset_size, asset_hash, asset_mtime, asset_relative_path, status, created_at, updated_at)
            VALUES (:archiveAsset.archiveId, :archiveAsset.name, :archiveAsset.size, :archiveAsset.hash, :archiveAsset.mtime, :archiveAsset.relativePath, :archiveAsset.status, strftime('%s', 'now'), strftime('%s', 'now'))
            """)
    @GetGeneratedKeys
    long insertArchiveAsset(@BindBean("archiveAsset") ArchiveAsset archiveAsset);

    @SqlUpdate("""
            INSERT INTO map_file_asset(file_id, asset_id, volume_order, status, created_at, updated_at)
            VALUES (:mapFileAsset.fileId, :mapFileAsset.assetId, :mapFileAsset.volumeOrder, :mapFileAsset.status, strftime('%s', 'now'), strftime('%s', 'now'))
            """)
    @GetGeneratedKeys
    Long insertMapFileAsset(@BindBean("mapFileAsset") MapFileAsset mapFileAsset);

    @SqlQuery("SELECT * FROM archive_asset WHERE asset_hash = :fileHash AND (:status IS NULL OR status = :status)")
    ArchiveAsset findArchiveAssetByHash(@Bind("fileHash") String fileHash, @Bind("status") String status);

    @SqlQuery("SELECT * FROM archive_metadata WHERE id = :id AND (:status IS NULL OR status = :status)")
    ArchiveMetadata findArchiveMetadataById(@Bind("id") long id, @Bind("status") String status);

    @SqlQuery("SELECT * FROM map_file_asset WHERE file_id = :id AND (:status IS NULL OR status = :status)")
    List<MapFileAsset> findMapFileAssetByFileId(@Bind("id") long id, @Bind("status") String status);

    @SqlUpdate("""
            UPDATE archive_metadata 
            SET archive_name = :archiveMetadata.name, 
                archive_limit_size = :archiveMetadata.limitSize,
                archive_hash = :archiveMetadata.hash,
                is_compressed = :archiveMetadata.isCompressed, 
                compressed_algorithm = :archiveMetadata.compressedAlgorithm, 
                is_encrypted = :archiveMetadata.isEncrypted, 
                encryption_algorithm = :archiveMetadata.encryptionAlgorithm, 
                status = :archiveMetadata.status, 
                updated_at = strftime('%s', 'now') 
            WHERE id = :archiveMetadata.id
            """)
    int updateArchiveMetadata(@BindBean("archiveMetadata") ArchiveMetadata archiveMetadata);

    @SqlUpdate("""
            UPDATE archive_asset 
            SET asset_name = :archiveAsset.name, 
                asset_size = :archiveAsset.size, 
                asset_hash = :archiveAsset.hash,
                asset_mtime = :archiveAsset.mtime, 
                asset_relative_path = :archiveAsset.relativePath, 
                status = :archiveAsset.status, 
                updated_at = strftime('%s', 'now') 
            WHERE id = :archiveAsset.id
            """)
    int updateArchiveAsset(@BindBean("archiveAsset") ArchiveAsset archiveAsset);
}
