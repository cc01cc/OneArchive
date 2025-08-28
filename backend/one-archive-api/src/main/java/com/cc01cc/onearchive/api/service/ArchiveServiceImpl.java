package com.cc01cc.onearchive.api.service;

import com.cc01cc.onearchive.core.OneArchive;
import com.cc01cc.onearchive.core.archive.ArchiveIn;
import com.cc01cc.onearchive.core.dao.DatabaseAccessor;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Service;

import java.io.IOException;

@Service
public class ArchiveServiceImpl implements ArchiveService {

    @Override
    public void archive(
            String rootDir,
            String archiveDir,
            String archivePrefix,
            String dbPath,
            Long archiveLimitSize
    ) {
        try {
            OneArchive.initializeDatabase(dbPath);
            OneArchive.performArchive(rootDir, archiveDir, archivePrefix, dbPath, archiveLimitSize);
        } catch (IOException e) {
            throw new RuntimeException(e);
        }
    }
}