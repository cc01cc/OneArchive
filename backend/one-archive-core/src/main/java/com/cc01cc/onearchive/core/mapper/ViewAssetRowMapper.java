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

import com.cc01cc.onearchive.core.entity.ViewAsset;
import org.jdbi.v3.core.mapper.RowMapper;
import org.jdbi.v3.core.statement.StatementContext;

import java.sql.ResultSet;
import java.sql.SQLException;

public class ViewAssetRowMapper implements RowMapper<ViewAsset> {
    @Override
    public ViewAsset map(ResultSet rs, StatementContext ctx) throws SQLException {
        ViewAsset viewAsset = new ViewAsset();
        viewAsset.setFileId(rs.getLong("file_id"));
        viewAsset.setAssetId(rs.getLong("asset_id"));
        viewAsset.setAssetName(rs.getString("asset_name"));
        viewAsset.setAssetSize(rs.getLong("asset_size"));
        viewAsset.setAssetHash(rs.getString("asset_hash"));
        viewAsset.setAssetMtime(rs.getLong("asset_mtime"));
        viewAsset.setAssetRelativePath(rs.getString("asset_relative_path"));
        viewAsset.setArchiveId(rs.getLong("archive_id"));
        viewAsset.setArchiveName(rs.getString("archive_name"));
        viewAsset.setVolumeOrder(rs.getLong("volume_order"));
        return viewAsset;
    }
}
