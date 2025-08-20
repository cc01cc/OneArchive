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

import com.cc01cc.project.entity.DirectoryInfo;
import com.cc01cc.project.entity.FileInfo;
import com.cc01cc.project.entity.ViewFile;
import org.jdbi.v3.sqlobject.config.RegisterBeanMapper;
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

    @SqlQuery("SELECT * FROM file_index WHERE file_hash = :fileHash AND status = :status")
    @RegisterBeanMapper(FileInfo.class)
    List<FileInfo> findFileByHashAndStatus(@Bind("fileHash") String fileHash, @Bind("status") String status);

    @SqlQuery("""
            SELECT 
                f.root_id as rootId,
                f.root_path as rootPath,
                f.root_status as rootStatus,
                f.directory_id as directoryId,
                f.directory_path as directoryPath,
                f.directory_mtime as directoryMtime,
                f.directory_status as directoryStatus,
                f.file_id as fileId,
                f.file_name as fileName,
                f.file_size as fileSize,
                f.file_mtime as fileMtime,
                f.file_hash as fileHash,
                f.file_status as fileStatus
            FROM v_file f
            WHERE f.root_id = :rootId
            AND (:status IS NULL OR f.file_status = :status)
            """)
    @RegisterBeanMapper(ViewFile.class)
    List<ViewFile> findViewFilesWithStatusAndRootId(@Bind("rootId") long rootId, @Bind("status") String status);

    @SqlQuery("""
            SELECT
                f.id as id,
                f.directory_id as directoryId,
                f.file_name as name,
                f.file_size as size,
                f.file_mtime as mtime,
                f.file_hash as hash,
                f.status as status,
                f.created_at as createdAt,
                f.updated_at as updatedAt
            FROM file_index f
            JOIN directory_index d ON f.directory_id = d.id
            JOIN root_index r ON d.root_id = r.id
            WHERE r.id = :rootId
            AND (:status IS NULL OR f.status = :status)
            """)
    @RegisterBeanMapper(FileInfo.class)
    List<FileInfo> findFilesByRootId(@Bind("rootId") long rootId, @Bind("status") String status);

    @SqlQuery("""
            SELECT
                d.id as id,
                d.root_id as rootId,
                d.directory_name as name,
                d.directory_path as path,
                d.status as status,
                d.created_at as createdAt,
                d.updated_at as updatedAt
            FROM directory_index d
            JOIN root_index r ON d.root_id = r.id
            WHERE r.id = :rootId
            AND (:status IS NULL OR d.status = :status)
            """)
    @RegisterBeanMapper(DirectoryInfo.class)
    List<DirectoryInfo> findDirectoriesByRootId(@Bind("rootId") long rootId, @Bind("status") String status);

    @SqlUpdate("""
            UPDATE directory_index
            SET root_id = :directoryInfo.rootId,
                directory_name = :directoryInfo.name,
                directory_mtime = :directoryInfo.mtime,
                directory_path = :directoryInfo.path,
                status = :directoryInfo.status,
                updated_at = strftime('%s', 'now')
            WHERE id = :directoryInfo.id
            """)
    int updateDirectory(@BindBean("directoryInfo") DirectoryInfo directoryInfo);

    @SqlUpdate("""
            UPDATE file_index
            SET directory_id = :fileInfo.directoryId,
                file_name = :fileInfo.name,
                file_size = :fileInfo.size,
                file_mtime = :fileInfo.mtime,
                file_hash = :fileInfo.hash,
                status = :fileInfo.status,
                updated_at = strftime('%s', 'now')
            WHERE id = :fileInfo.id
            """)
    int updateFileInfo(@BindBean("fileInfo") FileInfo fileInfo);

    @SqlUpdate("""
            INSERT INTO directory_index(root_id, directory_name, directory_mtime, directory_path, status, created_at, updated_at)
            VALUES (:directoryInfo.rootId, :directoryInfo.name, :directoryInfo.mtime, :directoryInfo.path, :directoryInfo.status, strftime('%s', 'now'), strftime('%s', 'now'))
            """)
    @GetGeneratedKeys
    long insertDirectory(@BindBean("directoryInfo") DirectoryInfo directoryInfo);
}
