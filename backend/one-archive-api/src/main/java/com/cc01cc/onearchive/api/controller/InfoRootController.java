package com.cc01cc.onearchive.api.controller;

import com.cc01cc.onearchive.api.service.InfoService;
import com.cc01cc.onearchive.core.entity.InfoRoot;
import lombok.AllArgsConstructor;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.RestController;

import java.util.List;

@RestController
@RequestMapping("/api/v1/info-roots")
@AllArgsConstructor
public class InfoRootController {

    private final InfoService infoService;

    @GetMapping
    public List<InfoRoot> getAllInfoRoots() {
        return infoService.getAllInfoRoots();
    }
}