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

package com.cc01cc.onearchive.core.mapper;

import com.cc01cc.onearchive.core.entity.ArchiveMetadata;
import org.jdbi.v3.core.mapper.RowMapper;
import org.jdbi.v3.core.statement.StatementContext;

import java.sql.ResultSet;
import java.sql.SQLException;

public class ArchiveMetadataRowMapper implements RowMapper<ArchiveMetadata> {
    @Override
    public ArchiveMetadata map(ResultSet rs, StatementContext ctx) throws SQLException {
        ArchiveMetadata archiveMetadata = new ArchiveMetadata();
        archiveMetadata.setId(rs.getLong("id"));
        archiveMetadata.setName(rs.getString("archive_name"));
        archiveMetadata.setLimitSize(rs.getLong("archive_limit_size"));
        archiveMetadata.setHash(rs.getString("archive_hash"));
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
