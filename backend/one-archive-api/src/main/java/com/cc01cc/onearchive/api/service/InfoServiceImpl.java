package com.cc01cc.onearchive.api.service;

import com.cc01cc.onearchive.core.dao.DatabaseAccessor;
import com.cc01cc.onearchive.core.entity.InfoRoot;
import jakarta.annotation.PostConstruct;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Service;

import java.util.List;

@Service
public class InfoServiceImpl implements InfoService {

    @Value("${database.url:onearchive.sqlite}")
    private String databaseUrl;

    private DatabaseAccessor databaseAccessor;

    @PostConstruct
    public void init() {
        String dbUrl = "jdbc:sqlite:" + databaseUrl;
        this.databaseAccessor = new DatabaseAccessor(dbUrl);
    }

    public List<InfoRoot> getAllInfoRoots() {
        return databaseAccessor.getHealthRootDirList();
    }
}