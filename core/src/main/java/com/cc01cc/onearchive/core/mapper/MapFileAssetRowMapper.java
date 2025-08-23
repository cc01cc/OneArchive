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

import com.cc01cc.onearchive.core.entity.MapFileAsset;
import org.jdbi.v3.core.mapper.RowMapper;
import org.jdbi.v3.core.statement.StatementContext;

import java.sql.ResultSet;
import java.sql.SQLException;

public class MapFileAssetRowMapper implements RowMapper<MapFileAsset> {
    @Override
    public MapFileAsset map(ResultSet rs, StatementContext ctx) throws SQLException {
        MapFileAsset mapFileAsset = new MapFileAsset();
        mapFileAsset.setId(rs.getLong("id"));
        mapFileAsset.setFileId(rs.getLong("file_id"));
        mapFileAsset.setAssetId(rs.getLong("asset_id"));
        mapFileAsset.setVolumeOrder(rs.getLong("volume_order"));
        mapFileAsset.setStatus(rs.getString("status"));
        mapFileAsset.setCreatedAt(rs.getLong("created_at"));
        mapFileAsset.setUpdatedAt(rs.getLong("updated_at"));
        return mapFileAsset;
    }
}
