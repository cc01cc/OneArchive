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

import com.cc01cc.onearchive.core.entity.InfoRoot;
import org.jdbi.v3.core.mapper.RowMapper;
import org.jdbi.v3.core.statement.StatementContext;

import java.sql.ResultSet;
import java.sql.SQLException;

public class InfoRootRowMapper implements RowMapper<InfoRoot> {
    @Override
    public InfoRoot map(ResultSet rs, StatementContext ctx) throws SQLException {
        InfoRoot infoRoot = new InfoRoot();
        infoRoot.setId(rs.getLong("id"));
        infoRoot.setName(rs.getString("root_name"));
        infoRoot.setPath(rs.getString("root_path"));
        infoRoot.setStatus(rs.getString("status"));
        infoRoot.setCreatedAt(rs.getLong("created_at"));
        infoRoot.setUpdatedAt(rs.getLong("updated_at"));
        return infoRoot;
    }
}
