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

package com.cc01cc.onearchive.core.mapper;// ... existing code ...
import com.cc01cc.onearchive.core.entity.InfoFile;
import org.jdbi.v3.core.mapper.RowMapper;
import org.jdbi.v3.core.statement.StatementContext;

import java.sql.ResultSet;
import java.sql.SQLException;

public class InfoFileRowMapper implements RowMapper<InfoFile> {
    @Override
    public InfoFile map(ResultSet rs, StatementContext ctx) throws SQLException {
        InfoFile infoFile = new InfoFile();
        infoFile.setId(rs.getLong("id"));
        infoFile.setDirectoryId(rs.getLong("directory_id"));
        infoFile.setName(rs.getString("file_name"));
        infoFile.setSize(rs.getLong("file_size"));
        infoFile.setMtime(rs.getLong("file_mtime"));
        infoFile.setHash(rs.getString("file_hash"));
        infoFile.setStatus(rs.getString("status"));
        infoFile.setCreatedAt(rs.getLong("created_at"));
        infoFile.setUpdatedAt(rs.getLong("updated_at"));
        return infoFile;
    }
}
