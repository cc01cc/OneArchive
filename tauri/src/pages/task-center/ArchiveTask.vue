<!-- TODO 存在的问题
- 归档目录应该直接从当前工作区中获取
- 归档进度显示不正确
- 根目录提供两个选项，一个是从扫描的结果，即数据库中获取，另一个是可以选择新的根目录
- 不支持非空的归档目录 (工作区新建的时候，要求目录为空，但是归档目录不必一定要非空，如果中断续存，那肯定是要非空的)
-->


<!-- TODO 存在的问题
- 归档目录应该直接从当前工作区中获取
- 归档进度显示不正确
- 根目录提供两个选项，一个是从扫描的结果，即数据库中获取，另一个是可以选择新的根目录
- 不支持非空的归档目录 (工作区新建的时候，要求目录为空，但是归档目录不必一定要非空，如果中断续存，那肯定是要非空的)
-->


<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { open } from '@tauri-apps/plugin-dialog';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { useGlobalStore } from '../../store/global';
import { useToast } from 'primevue/usetoast';
import Card from "primevue/card";
import InputText from "primevue/inputtext";
import Button from "primevue/button";
import Message from "primevue/message";
import ProgressBar from "primevue/progressbar";
import InputNumber from "primevue/inputnumber";

const toast = useToast();
// 使用全局状态存储
const globalStore = useGlobalStore();

// 定义归档配置
const archiveConfig = reactive({
    rootDir: "",
    archiveDir: "",
    archivePrefix: "archive",
    dbPath: "",
    archiveLimitSize: 104857600, // 100MB
});

// 定义状态变量
const isArchiving = ref(false);
const progress = ref(0);
const statusMessage = ref("准备就绪");
const errors = ref<Record<string, string>>({});
const archiveResult = ref("");
const totalFiles = ref(0);
const processedFiles = ref(0);
const processedBytes = ref(0);
const totalBytes = ref(0);

// 组件挂载时设置默认数据库路径
onMounted(() => {
    if (globalStore.dbPath) {
        archiveConfig.dbPath = globalStore.dbPath;
    }
});

// 验证表单
const validateForm = () => {
    errors.value = {};

    if (!archiveConfig.rootDir) {
        errors.value.rootDir = "根目录是必填项";
    }

    if (!archiveConfig.archiveDir) {
        errors.value.archiveDir = "归档目录是必填项";
    }

    if (!archiveConfig.dbPath) {
        errors.value.dbPath = "数据库路径是必填项";
    }

    return Object.keys(errors.value).length === 0;
};

// 选择目录
const selectDirectory = async (field: 'rootDir' | 'archiveDir') => {
    const selected = await open({
        directory: true,
        multiple: false,
        title: field === 'rootDir' ? '选择要归档的根目录' : '选择归档目录'
    });

    if (selected) {
        archiveConfig[field] = selected as string;
    }
};

// 选择文件
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
        archiveConfig[field] = selected as string;
    }
};

// 格式化文件大小显示
const formatFileSize = (bytes: number) => {
    if (bytes === 0) return '0 Bytes';

    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));

    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
};

// 开始归档
const startArchive = async () => {
    if (!validateForm()) {
        console.error("Form validation error:", errors.value);
        toast.add({ severity: 'error', summary: '错误', detail: '请填写所有必填项', life: 3000 });
        return;
    }

    isArchiving.value = true;
    progress.value = 0;
    statusMessage.value = "正在初始化...";
    archiveResult.value = "";
    totalFiles.value = 0;
    processedFiles.value = 0;
    processedBytes.value = 0;
    totalBytes.value = 0;

    // 监听归档进度事件
    const unsubscribe = await listen('archive-progress', (event) => {
        const progressData = event.payload as {
            total_files: number;
            processed_files: number;
            processed_bytes: number;
            total_bytes: number;
            current_file: string | null;
            completed: boolean;
            error: string | null;
        };

        totalFiles.value = progressData.total_files;
        processedFiles.value = progressData.processed_files;
        processedBytes.value = progressData.processed_bytes;
        totalBytes.value = progressData.total_bytes;

        progress.value = progressData.total_bytes > 0
            ? Math.round((progressData.processed_bytes / progressData.total_bytes) * 100)
            : 0;

        if (progressData.current_file) {
            statusMessage.value = `正在处理：${progressData.current_file}`;
        } else {
            statusMessage.value = "正在归档...";
        }

        if (progressData.error) {
            toast.add({
                severity: "warn",
                summary: "处理警告",
                detail: progressData.error,
                life: 5000,
            });
        }
    });

    // 监听扫描进度事件
    const unsubscribeScan = await listen('scan-progress', (event) => {
        const progressData = event.payload as {
            processed: number;
            total: number;
            message: string;
            progress: number;
        };

        progress.value = Math.round(progressData.progress);
        statusMessage.value = progressData.message;
    });

    try {
        // 调用 Tauri 后端的归档功能
        const result = await invoke('archive_files', {
            rootDir: archiveConfig.rootDir,
            archiveDir: archiveConfig.archiveDir,
            archivePrefix: archiveConfig.archivePrefix,
            dbPath: archiveConfig.dbPath,
            archiveLimitSize: archiveConfig.archiveLimitSize
        });

        statusMessage.value = "归档完成";
        progress.value = 100;
        archiveResult.value = JSON.stringify(result, null, 2);
    } catch (error: any) {
        statusMessage.value = `归档失败：${error.message || error}`;
        console.error("Archive error:", error);
    } finally {
        // 取消监听进度事件
        unsubscribe();
        unsubscribeScan();
        isArchiving.value = false;
    }
};

// 重置表单
const resetForm = () => {
    archiveConfig.rootDir = "";
    archiveConfig.archiveDir = "";
    archiveConfig.archivePrefix = "archive";
    // 保持数据库路径与全局状态同步
    if (globalStore.dbPath) {
        archiveConfig.dbPath = globalStore.dbPath;
    }
    errors.value = {};
    progress.value = 0;
    statusMessage.value = "准备就绪";
    archiveResult.value = "";
    totalFiles.value = 0;
    processedFiles.value = 0;
    processedBytes.value = 0;
    totalBytes.value = 0;
};
</script>


<template>
    <div class="container mx-auto p-4">
        <h1 class="text-3xl font-bold mb-6 text-center text-primary-700">执行归档</h1>

        <Card>
            <template #title>
                <h2 class="text-xl font-semibold">归档配置</h2>
            </template>
            <template #content>
                <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                    <!-- 根目录 -->
                    <div class="field">
                        <label for="rootDir" class="block text-sm font-medium mb-2">根目录 *</label>
                        <div class="flex gap-2">
                            <InputText id="rootDir" v-model="archiveConfig.rootDir" placeholder="选择要归档的根目录"
                                :disabled="isArchiving" class="flex-1" :class="{ 'p-invalid': errors.rootDir }" />
                            <Button label="浏览" @click="selectDirectory('rootDir')" :disabled="isArchiving" />
                        </div>
                        <Message v-if="errors.rootDir" severity="error" size="small">{{ errors.rootDir }}</Message>
                    </div>

                    <!-- 归档目录 -->
                    <div class="field">
                        <label for="archiveDir" class="block text-sm font-medium mb-2">归档目录 *</label>
                        <div class="flex gap-2">
                            <InputText id="archiveDir" v-model="archiveConfig.archiveDir" placeholder="选择归档目录"
                                :disabled="isArchiving" class="flex-1" :class="{ 'p-invalid': errors.archiveDir }" />
                            <Button label="浏览" @click="selectDirectory('archiveDir')" :disabled="isArchiving" />
                        </div>
                        <Message v-if="errors.archiveDir" severity="error" size="small">{{ errors.archiveDir }}
                        </Message>
                    </div>

                    <!-- 归档文件前缀 -->
                    <div class="field">
                        <label for="archivePrefix" class="block text-sm font-medium mb-2">归档文件前缀</label>
                        <InputText id="archivePrefix" v-model="archiveConfig.archivePrefix" placeholder="归档文件前缀"
                            :disabled="isArchiving" class="w-full" />
                    </div>

                    <!-- 归档大小限制 -->
                    <div class="field">
                        <label for="archiveLimitSize" class="block text-sm font-medium mb-2">归档大小限制 (字节)</label>
                        <InputNumber id="archiveLimitSize" v-model="archiveConfig.archiveLimitSize" mode="decimal"
                            :min="1024" :max="10737418240" :step="1024" :disabled="isArchiving" class="w-full" />
                        <small class="block mt-1 text-gray-500">当前值：{{ formatFileSize(archiveConfig.archiveLimitSize)
                            }}</small>
                    </div>

                    <!-- 数据库路径 -->
                    <div class="field md:col-span-2">
                        <label for="dbPath" class="block text-sm font-medium mb-2">数据库路径 *</label>
                        <div class="flex gap-2">
                            <InputText id="dbPath" v-model="archiveConfig.dbPath" placeholder="选择数据库文件"
                                :disabled="isArchiving" class="flex-1" :class="{ 'p-invalid': errors.dbPath }" />
                            <Button label="浏览" @click="selectFile('dbPath')" :disabled="isArchiving" />
                        </div>
                        <Message v-if="errors.dbPath" severity="error" size="small">{{ errors.dbPath }}</Message>
                    </div>
                </div>

                <!-- 操作按钮 -->
                <div class="flex justify-center gap-4 mt-8">
                    <Button label="重置" icon="pi pi-refresh" @click="resetForm" :disabled="isArchiving"
                        class="p-button-outlined" />
                    <Button label="开始归档" icon="pi pi-box" @click="startArchive" :disabled="isArchiving"
                        class="p-button-success" />
                </div>
            </template>
        </Card>

        <!-- 进度显示 -->
        <div v-if="isArchiving || progress > 0" class="mt-6">
            <Card>
                <template #title>
                    <h2 class="text-xl font-semibold">归档进度</h2>
                </template>
                <template #content>
                    <div class="space-y-4">
                        <div class="flex justify-between mb-2">
                            <span class="font-medium">{{ statusMessage }}</span>
                            <span class="font-medium">{{ progress }}%</span>
                        </div>
                        <ProgressBar :value="progress" :showValue="false" />

                        <div class="grid grid-cols-1 md:grid-cols-2 gap-4 mt-4">
                            <div class="p-3 bg-gray-50 rounded">
                                <p class="text-sm text-gray-600">文件进度</p>
                                <p class="font-medium">{{ processedFiles }} / {{ totalFiles }} 文件</p>
                            </div>
                            <div class="p-3 bg-gray-50 rounded">
                                <p class="text-sm text-gray-600">字节进度</p>
                                <p class="font-medium">{{ formatFileSize(processedBytes) }} / {{
                                    formatFileSize(totalBytes) }}</p>
                            </div>
                        </div>
                    </div>
                </template>
            </Card>
        </div>

        <!-- 结果显示 -->
        <div v-if="archiveResult" class="mt-6">
            <Card>
                <template #title>
                    <h2 class="text-xl font-semibold">归档结果</h2>
                </template>
                <template #content>
                    <pre class="bg-gray-100 p-4 rounded">{{ archiveResult }}</pre>
                </template>
            </Card>
        </div>
    </div>
</template>

<style scoped>
.p-button {
    min-width: 100px;
}
</style>