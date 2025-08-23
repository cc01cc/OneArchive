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

import com.cc01cc.onearchive.core.entity.ViewFile;
import org.jdbi.v3.core.mapper.RowMapper;
import org.jdbi.v3.core.statement.StatementContext;

import java.sql.ResultSet;
import java.sql.SQLException;

public class ViewFileRowMapper implements RowMapper<ViewFile> {
    @Override
    public ViewFile map(ResultSet rs, StatementContext ctx) throws SQLException {
        ViewFile viewFile = new ViewFile();
        viewFile.setRootId(rs.getLong("root_id"));
        viewFile.setRootPath(rs.getString("root_path"));
        viewFile.setRootStatus(rs.getString("root_status"));
        viewFile.setDirectoryId(rs.getLong("directory_id"));
        viewFile.setDirectoryPath(rs.getString("directory_path"));
        viewFile.setDirectoryMtime(rs.getLong("directory_mtime"));
        viewFile.setDirectoryStatus(rs.getString("directory_status"));
        viewFile.setFileId(rs.getLong("file_id"));
        viewFile.setFileName(rs.getString("file_name"));
        viewFile.setFileSize(rs.getLong("file_size"));
        viewFile.setFileMtime(rs.getLong("file_mtime"));
        viewFile.setFileHash(rs.getString("file_hash"));
        viewFile.setFileStatus(rs.getString("file_status"));
        return viewFile;
    }
}
