<template>
    <Card class="mt-6">
        <template #title>
            <h2 class="text-xl font-semibold">归档文件分组</h2>
        </template>
        <template #content>
            <div class="mb-4 flex justify-between items-center">
                <div>
                    <Button label="添加分组" icon="pi pi-plus" @click="addGroup" />
                </div>
                <div v-if="recoveryStore.availableArchives.length > 0" class="text-sm text-gray-500">
                    共 {{ recoveryStore.availableArchives.length }} 个可用归档文件
                </div>
            </div>

            <div v-for="(group, groupIndex) in recoveryStore.archiveGroups" :key="groupIndex"
                class="border rounded-lg p-4 mb-4">
                <div class="flex justify-between items-center mb-3">
                    <h3 class="text-lg font-medium">分组 {{ groupIndex + 1 }}</h3>
                    <Button icon="pi pi-trash" @click="removeGroup(groupIndex)" class="p-button-text p-button-danger" />
                </div>

                <div class="grid grid-cols-1 gap-4">
                    <PickList :modelValue="[recoveryStore.availableArchives, group]"
                        @update:modelValue="val => onGroupUpdate(val, groupIndex)"
                        :options="recoveryStore.availableArchives" option-label="archive_name" option-value="id"
                        :showSourceControls="false" :showTargetControls="false" breakpoint="1024px" dragdrop="true">
                        <template #sourceheader>
                            <div class="font-bold">可用归档文件</div>
                        </template>
                        <template #targetheader>
                            <div class="font-bold">已选归档文件 ({{ group.length }}/{{ recoveryStore.config.dataShards }})
                            </div>
                        </template>
                        <template #option="slotProps">
                            <div class="flex flex-wrap p-2 items-center gap-2 w-full">
                                <div class="flex-1 flex flex-col">
                                    <span class="font-medium">{{ slotProps.option.archive_name }}</span>
                                    <span class="text-sm text-surface-500 dark:text-surface-400">
                                        ID: {{ slotProps.option.id }} | 大小：{{
                                            formatFileSize(slotProps.option.archive_size)
                                        }}
                                    </span>
                                </div>
                            </div>
                        </template>
                    </PickList>

                    <div v-if="group.length > recoveryStore.config.dataShards" class="mt-2 text-red-500 text-sm">
                        警告：当前分组包含 {{ group.length }} 个归档文件，超过数据分片数 {{ recoveryStore.config.dataShards }}
                    </div>

                    <div v-else-if="group.length < recoveryStore.config.dataShards"
                        class="mt-2 text-yellow-500 text-sm">
                        提示：还需添加 {{ recoveryStore.config.dataShards - group.length }} 个归档文件
                    </div>

                    <div v-else class="mt-2 text-green-500 text-sm">
                        分组已完成，包含 {{ group.length }} 个归档文件
                    </div>
                </div>
            </div>
        </template>
    </Card>
</template>

<script setup lang="ts">
import { ref, watch, onMounted } from 'vue'
import Card from "primevue/card";
import Button from "primevue/button";
import PickList from "primevue/picklist";
import { useRecoveryStore } from '../../store/recovery';
import { ArchiveFile } from '../../types/recovery';
import { invoke } from '@tauri-apps/api/core';
import { useToast } from 'primevue/usetoast';

const toast = useToast();
const recoveryStore = useRecoveryStore();

// 当 PickList 更新时的处理函数
const onGroupUpdate = (val: any[][], index: number) => {
    recoveryStore.updateGroup(index, val[1] as ArchiveFile[]);
};

// 添加分组
const addGroup = () => {
    recoveryStore.addGroup();
};

// 删除分组
const removeGroup = (index: number) => {
    recoveryStore.removeGroup(index);
};

// 格式化文件大小
const formatFileSize = (bytes: number) => {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
};

watch(() => recoveryStore.config.rootId, async (newRootId) => {
    if (newRootId !== null) {
        await onRootChange(newRootId);
    }
});

// 根目录变化时的处理
const onRootChange = async (rootId: number) => {
    try {
        console.log("dbPath:", recoveryStore.config.dbPath);
        console.log("Loading archives for root:", rootId);
        // 使用正确的函数获取归档文件列表
        const archives = await invoke("get_archives_by_root_id", {
            dbPath: recoveryStore.config.dbPath,
            rootId: rootId
        });

        console.log("Archives loaded:", archives);
        recoveryStore.setAvailableArchives(archives as ArchiveFile[]);

    } catch (error: any) {
        console.error("Load archives error:", error);
        toast.add({ severity: 'error', summary: '错误', detail: '加载归档文件失败：' + (error.message || error), life: 5000 });
    }
};

onMounted(async () => {
    if (recoveryStore.config.rootId !== null) {
        await onRootChange(recoveryStore.config.rootId);
    }
});
</script>