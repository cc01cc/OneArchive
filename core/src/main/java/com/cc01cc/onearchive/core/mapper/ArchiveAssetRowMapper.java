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

import com.cc01cc.onearchive.core.entity.ArchiveAsset;
import org.jdbi.v3.core.mapper.RowMapper;
import org.jdbi.v3.core.statement.StatementContext;

import java.sql.ResultSet;
import java.sql.SQLException;

public class ArchiveAssetRowMapper implements RowMapper<ArchiveAsset> {
    @Override
    public ArchiveAsset map(ResultSet rs, StatementContext ctx) throws SQLException {
        ArchiveAsset archiveAsset = new ArchiveAsset();
        archiveAsset.setId(rs.getLong("id"));
        archiveAsset.setArchiveId(rs.getLong("archive_id"));
        archiveAsset.setName(rs.getString("asset_name"));
        archiveAsset.setSize(rs.getLong("asset_size"));
        archiveAsset.setHash(rs.getString("asset_hash"));
        archiveAsset.setMtime(rs.getLong("asset_mtime"));
        archiveAsset.setRelativePath(rs.getString("asset_relative_path"));
        archiveAsset.setStatus(rs.getString("status"));
        archiveAsset.setCreatedAt(rs.getLong("created_at"));
        archiveAsset.setUpdatedAt(rs.getLong("updated_at"));
        return archiveAsset;
    }
}
