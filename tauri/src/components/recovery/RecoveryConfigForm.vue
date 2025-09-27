<!-- TODO
  - 用户可以以 根目录为单位，也可以以一个工作区，即以 sqlite 为单位，选择存档文件生成校验文件
-->
<template>
    <Card>
        <template #title>
            <h2 class="text-xl font-semibold">灾备配置</h2>
        </template>
        <template #content>
            <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                <!-- 数据分片数 -->
                <div class="field">
                    <label for="dataShards" class="block text-sm font-medium mb-2">数据分片数</label>
                    <InputNumber id="dataShards" v-model="recoveryStore.config.dataShards" :min="1" :max="100"
                        class="w-full" />
                    <small class="block mt-1 text-gray-500">需要保护的归档文件数量</small>
                </div>

                <!-- 校验分片数 -->
                <div class="field">
                    <label for="parityShards" class="block text-sm font-medium mb-2">校验分片数</label>
                    <InputNumber id="parityShards" v-model="recoveryStore.config.parityShards" :min="1" :max="10"
                        class="w-full" />
                    <small class="block mt-1 text-gray-500">用于恢复的校验文件数量</small>
                </div>

                <!-- 存储目录 -->
                <div class="field md:col-span-2">
                    <label for="storageDir" class="block text-sm font-medium mb-2">灾备文件存储目录 *</label>
                    <div class="flex gap-2">
                        <InputText id="storageDir" v-model="recoveryStore.config.storageDir" placeholder="选择灾备文件存储目录"
                            class="flex-1" />
                        <Button label="浏览" @click="selectDirectory('storageDir')" />
                    </div>
                </div>

                <!-- 文件前缀 -->
                <div class="field md:col-span-2">
                    <label for="prefix" class="block text-sm font-medium mb-2">灾备文件前缀</label>
                    <InputText id="prefix" v-model="recoveryStore.config.prefix" placeholder="灾备文件前缀" class="w-full" />
                    <small class="block mt-1 text-gray-500">生成的灾备文件名前缀</small>
                </div>

                <!-- 数据分片存储选项 -->
                <div class="field md:col-span-2">
                    <div class="flex items-center">
                        <Checkbox id="storeDataShards" v-model="recoveryStore.config.storeDataShards" :binary="true" />
                        <label for="storeDataShards" class="ml-2">存储数据分片副本</label>
                    </div>
                    <small class="block mt-1 text-gray-500">
                        如果启用，将创建数据分片的副本；否则只创建校验分片
                    </small>
                </div>

                <!-- 数据库路径 -->
                <div class="field md:col-span-2">
                    <label for="dbPath" class="block text-sm font-medium mb-2">数据库路径 *</label>
                    <div class="flex gap-2">
                        <InputText id="dbPath" v-model="recoveryStore.config.dbPath" placeholder="选择数据库文件"
                            class="flex-1" />
                        <Button label="浏览" @click="selectFile('dbPath')" />
                    </div>
                </div>

                <!-- 根目录选择 -->
                <div class="field md:col-span-2">
                    <label for="rootId" class="block text-sm font-medium mb-2">选择根目录 *</label>
                    <div class="flex gap-2">
                        <Select id="rootId" v-model="recoveryStore.config.rootId" :options="rootList"
                            option-label="root_path" option-value="id" placeholder="选择根目录" :disabled="loadingRoots"
                            class="flex-1" />
                        <Button label="刷新" @click="loadRootList" :disabled="loadingRoots" :loading="loadingRoots" />
                    </div>
                </div>
            </div>
        </template>
    </Card>
</template>
<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import Button from "primevue/button";
import Card from "primevue/card";
import Checkbox from "primevue/checkbox";

import InputNumber from "primevue/inputnumber";
import InputText from "primevue/inputtext";
import { useToast } from 'primevue/usetoast';
import { onMounted, ref } from 'vue';
import { useRecoveryStore } from '../../store/recovery';
import { RootInfo } from '../../types/recovery';

const toast = useToast();
const recoveryStore = useRecoveryStore();

// 根目录相关
const rootList = ref<RootInfo[]>([]);
const loadingRoots = ref(false);

// 选择目录
const selectDirectory = async (field: 'storageDir') => {
    const selected = await open({
        directory: true,
        multiple: false,
        title: '选择灾备文件存储目录'
    });

    if (selected) {
        recoveryStore.updateConfigField(field, selected as string);
    }
};

// 选择数据库文件
const selectFile = async (field: 'dbPath') => {
    const selected = await open({
        directory: false,
        multiple: false,
        filters: [{
            name: "Database Files",
            extensions: ["db", "sqlite", "sqlite3"]
        }],
        title: '选择数据库文件'
    });

    if (selected) {
        recoveryStore.updateConfigField(field, selected as string);
        if (recoveryStore.config.dbPath) {
            loadRootList();
        }
    }
};

// 加载根目录列表
const loadRootList = async () => {
    if (!recoveryStore.config.dbPath) {
        return;
    }

    loadingRoots.value = true;
    try {
        const roots: RootInfo[] = await invoke("get_all_roots", {
            dbPath: recoveryStore.config.dbPath,
        });

        rootList.value = roots;

    } catch (error: any) {
        console.error("Load roots error:", error);
        toast.add({ severity: 'error', summary: '错误', detail: '加载根目录失败：' + (error.message || error), life: 5000 });
    } finally {
        loadingRoots.value = false;
    }
};

onMounted(() => {
    if (recoveryStore.config.dbPath) {
        loadRootList();
    }
});
</script>