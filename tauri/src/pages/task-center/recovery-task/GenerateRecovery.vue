<template>
    <div class="container mx-auto p-4">
        <h1 class="text-3xl font-bold mb-6 text-center text-primary-700">生成灾备文件</h1>

        <RecoveryConfigForm />

        <!-- 归档文件分组 -->
        <ArchiveGroupManager />

        <!-- 操作按钮 -->
        <div class="flex justify-center gap-4 mt-8">
            <Button label="重置" icon="pi pi-refresh" @click="resetForm" :disabled="recoveryStore.isGenerating"
                class="p-button-outlined" />
            <Button label="生成灾备文件" icon="pi pi-database" @click="generateRecoveryFiles"
                :disabled="recoveryStore.isGenerating" class="p-button-success" />
        </div>

        <!-- 进度显示 -->
        <GenerationProgress :visible="recoveryStore.isProcessing" :progress="recoveryStore.progress"
            :status-message="recoveryStore.statusMessage" :group-id="recoveryStore.groupId" />

        <!-- 结果显示 -->
        <GenerationResult :visible="!!recoveryStore.resultMessage" :result-message="recoveryStore.resultMessage" />
    </div>
</template>

<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core';
import Button from "primevue/button";
import { useToast } from 'primevue/usetoast';
import { onMounted } from 'vue';
import ArchiveGroupManager from '../../../components/recovery/ArchiveGroupManager.vue';
import GenerationProgress from '../../../components/recovery/GenerationProgress.vue';
import GenerationResult from '../../../components/recovery/GenerationResult.vue';
import RecoveryConfigForm from '../../../components/recovery/RecoveryConfigForm.vue';
import { useRecoveryStore } from '../../../store/recovery';
import type { RecoveryGenerationResult } from '../../../types/recovery';

const toast = useToast();
const recoveryStore = useRecoveryStore();

// 组件挂载时设置默认数据库路径
onMounted(() => {
    recoveryStore.initializeConfig();
});

// 验证表单
const validateForm = () => {
    const errors: Record<string, string> = {};

    if (!recoveryStore.config.storageDir) {
        errors.storageDir = "存储目录是必填项";
    }

    if (!recoveryStore.config.dbPath) {
        errors.dbPath = "数据库路径是必填项";
    }

    // 验证归档文件分组
    if (recoveryStore.archiveGroups.length === 0) {
        errors.archiveGroups = "至少需要一个归档文件分组";
    }

    for (let i = 0; i < recoveryStore.archiveGroups.length; i++) {
        const group = recoveryStore.archiveGroups[i];
        if (group.length !== recoveryStore.config.dataShards) {
            errors[`group${i}`] = `分组 ${i + 1} 必须包含 exactly ${recoveryStore.config.dataShards} 个归档文件`;
        }
    }

    recoveryStore.setErrors(errors);
    return Object.keys(errors).length === 0;
};

// 生成灾备文件
const generateRecoveryFiles = async () => {
    if (!validateForm()) {
        console.error("Form validation error:", recoveryStore.errors);
        toast.add({ severity: 'error', summary: '错误', detail: '请填写所有必填项并确保分组正确', life: 3000 });
        return;
    }

    recoveryStore.setGeneratingStatus(true);
    recoveryStore.updateProgress(0, "正在初始化...");
    recoveryStore.setResult("");

    try {
        // 将分组中的目标项转换为数字 ID
        const archiveGroupsIds: number[][] = recoveryStore.archiveGroups.map(group =>
            group.map(archive => archive.id)
        );
        console.log("Archive Groups IDs:", archiveGroupsIds);

        // 调用 Tauri 后端的灾备生成功能
        // TODO 这里 前后端 参数命名 风格需要统一
        const results = await invoke('generate_recovery_files', {
            config: {
                data_shards: recoveryStore.config.dataShards,
                parity_shards: recoveryStore.config.parityShards,
                storage_dir: recoveryStore.config.storageDir,
                prefix: recoveryStore.config.prefix,
                store_data_shards: recoveryStore.config.storeDataShards
            },
            archiveGroups: archiveGroupsIds,
            dbPath: recoveryStore.config.dbPath
        });

        recoveryStore.updateProgress(100, "灾备文件生成完成");

        // 为 results 添加类型断言
        const typedResults = results as RecoveryGenerationResult[];

        // 显示第一个组的 ID 作为示例
        let groupId = "";
        if (typedResults.length > 0) {
            groupId = typedResults[0].group_id;
        }

        recoveryStore.setResult(JSON.stringify(typedResults, null, 2), groupId);

        toast.add({ severity: 'success', summary: '成功', detail: `成功生成 ${typedResults.length} 个灾备组`, life: 3000 });

    } catch (error: any) {
        recoveryStore.updateProgress(0, `生成失败：${error.message || error}`);
        console.error("Generate recovery error:", error);
        toast.add({ severity: 'error', summary: '错误', detail: '灾备文件生成失败：' + (error.message || error), life: 5000 });
    } finally {
        recoveryStore.setGeneratingStatus(false);
    }
};

// 重置表单
const resetForm = () => {
    recoveryStore.resetForm();
};
</script>