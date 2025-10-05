<!-- TODO
    - 归档，解档都需要让用户二次确认，配置
    - 把 归档，解档的 调用都调整为 异步执行
    - 进度条信息需要优化
    - 提供解档历史记录
    - 用户选择根目录后，呈现涉及到的 归档文件列表
-->

<script setup lang="ts">
import { ref, onMounted, watch } from "vue";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import { useGlobalStore } from "../../store/global";
import { getAllRoots, extractArchive } from "../../api";
import Card from "primevue/card";
import InputText from "primevue/inputtext";
import Button from "primevue/button";
import Message from "primevue/message";
import ProgressBar from "primevue/progressbar";

import Checkbox from "primevue/checkbox";
import { useToast } from "primevue/usetoast";

const toast = useToast();
const globalStore = useGlobalStore();

// 表单数据
const extractConfig = ref({
    targetPath: "",
    dbPath: globalStore.dbPath || "",
    rootId: null as number | null,
    overwrite: false,
});

// 表单验证错误
const errors = ref<Record<string, string>>({});

// 解档状态
const isExtracting = ref(false);
const progress = ref(0);
const statusMessage = ref("准备就绪");
const extractResult = ref<any>(null);
const totalFiles = ref(0);
const processedFiles = ref(0);

// 根目录列表
const rootList = ref<Array<{ id: number; path: string; name: string }>>([]);

// 选中的根目录信息
const selectedRoot = ref<{ id: number; path: string; name: string } | null>(null);

// 验证表单
const validateForm = () => {
    const newErrors: Record<string, string> = {};

    if (!extractConfig.value.targetPath) {
        newErrors.targetPath = "目标路径不能为空";
    }

    if (!extractConfig.value.dbPath) {
        newErrors.dbPath = "数据库路径不能为空";
    }

    if (!extractConfig.value.rootId) {
        newErrors.rootId = "请选择要解档的根目录";
    }

    errors.value = newErrors;
    return Object.keys(newErrors).length === 0;
};

// 选择目标路径
const selectTargetPath = async () => {
    const selected = await open({
        directory: true,
        title: "选择解档目标路径",
    });

    if (selected) {
        extractConfig.value.targetPath = selected;
    }
};

// 选择数据库文件
const selectDatabaseFile = async () => {
    const selected = await open({
        filters: [
            {
                name: "SQLite Database",
                extensions: ["db", "sqlite", "sqlite3"],
            },
        ],
        title: "选择数据库文件",
    });

    if (selected) {
        extractConfig.value.dbPath = selected;
        globalStore.setDbPath(selected);

        // 加载根目录列表
        loadRootList();
    }
};

// 加载根目录列表
const loadRootList = async () => {
    if (!extractConfig.value.dbPath) {
        return;
    }

    try {
        const result = await getAllRoots(extractConfig.value.dbPath);
        if (result.success && result.data) {
            rootList.value = result.data.map(root => ({
                id: root.id,
                path: root.rootPath,
                name: root.rootName || `根目录 ${root.id}`,
            }));

            // 如果之前已选择根目录，但不在新的列表中，则重置选择
            if (extractConfig.value.rootId &&
                !rootList.value.some(root => root.id === extractConfig.value.rootId)) {
                extractConfig.value.rootId = null;
                selectedRoot.value = null;
            }
        } else {
            throw new Error(result.error || '获取根目录失败');
        }
    } catch (error: any) {
        console.error("加载根目录失败：", error);
        toast.add({
            severity: "error",
            summary: "加载失败",
            detail: `加载根目录失败：${error.message || error.toString()}`,
            life: 5000,
        });
    }
};

// 监听根目录选择变化
watch(() => extractConfig.value.rootId, (newRootId) => {
    if (newRootId) {
        const root = rootList.value.find(r => r.id === newRootId);
        if (root) {
            selectedRoot.value = root;
        }
    } else {
        selectedRoot.value = null;
    }
});

// 开始解档
const startExtract = async () => {
    if (!validateForm() || isExtracting.value) return;

    if (!selectedRoot.value) {
        toast.add({
            severity: "error",
            summary: "参数错误",
            detail: "未找到选中的根目录信息",
            life: 5000,
        });
        return;
    }

    isExtracting.value = true;
    progress.value = 0;
    statusMessage.value = "正在初始化...";
    extractResult.value = null;
    totalFiles.value = 0;
    processedFiles.value = 0;

    // 监听解档进度事件
    const unsubscribe = await listen('extract-progress', (event) => {
        const progressData = event.payload as {
            total_files: number;
            processed_files: number;
            current_file: string | null;
            completed: boolean;
            error: string | null;
        };

        totalFiles.value = progressData.total_files;
        processedFiles.value = progressData.processed_files;
        progress.value = progressData.total_files > 0
            ? Math.round((progressData.processed_files / progressData.total_files) * 100)
            : 0;

        if (progressData.current_file) {
            statusMessage.value = `正在处理：${progressData.current_file}`;
        } else {
            statusMessage.value = "正在解档...";
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

    try {
        // 调用新的API执行解档
        const result = await extractArchive(
            extractConfig.value.rootId!,
            extractConfig.value.targetPath,
            extractConfig.value.dbPath
        );

        if (result.success && result.data) {
            extractResult.value = result.data;
            statusMessage.value = "解档完成";

            toast.add({
                severity: "success",
                summary: "解档完成",
                detail: "文件解档成功",
                life: 3000,
            });
        } else {
            throw new Error(result.error || '解档失败');
        }
    } catch (error: any) {
        statusMessage.value = `解档失败：${error.message || error}`;
        console.error("Extract error:", error);
        toast.add({
            severity: "error",
            summary: "解档失败",
            detail: error.message || error.toString(),
            life: 5000,
        });
    } finally {
        // 取消监听进度事件
        unsubscribe();
        isExtracting.value = false;
    }
};

// 重置表单
const resetForm = () => {
    extractConfig.value = {
        targetPath: "",
        dbPath: globalStore.dbPath || "",
        rootId: null,
        overwrite: false,
    };
    errors.value = {};
    progress.value = 0;
    statusMessage.value = "准备就绪";
    extractResult.value = null;
    rootList.value = [];
    selectedRoot.value = null;
    totalFiles.value = 0;
    processedFiles.value = 0;
};

// 组件挂载时加载数据
onMounted(() => {
    if (globalStore.dbPath) {
        loadRootList();
    }
});
</script>

<template>
    <div class="container mx-auto p-4">
        <h1 class="text-3xl font-bold mb-6 text-center text-primary-700">执行解档</h1>

        <Card>
            <template #title>
                <h2 class="text-xl font-semibold">解档配置</h2>
            </template>
            <template #content>
                <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                    <!-- 目标路径 -->
                    <div class="field">
                        <label for="targetPath" class="block text-sm font-medium mb-2">解档目标路径 *</label>
                        <div class="flex gap-2">
                            <InputText id="targetPath" v-model="extractConfig.targetPath" placeholder="选择解档目标路径"
                                :disabled="isExtracting" class="flex-1" :class="{ 'p-invalid': errors.targetPath }" />
                            <Button label="浏览" @click="selectTargetPath" :disabled="isExtracting" />
                        </div>
                        <Message v-if="errors.targetPath" severity="error" size="small">{{
                            errors.targetPath
                            }}</Message>
                    </div>

                    <!-- 数据库文件 -->
                    <div class="field">
                        <label for="dbPath" class="block text-sm font-medium mb-2">数据库文件 *</label>
                        <div class="flex gap-2">
                            <InputText id="dbPath" v-model="extractConfig.dbPath" placeholder="选择数据库文件"
                                :disabled="isExtracting" class="flex-1" :class="{ 'p-invalid': errors.dbPath }" />
                            <Button label="浏览" @click="selectDatabaseFile" :disabled="isExtracting" />
                        </div>
                        <Message v-if="errors.dbPath" severity="error" size="small">{{
                            errors.dbPath
                            }}</Message>
                    </div>

                    <!-- 根目录选择 -->
                    <div class="field">
                        <label for="rootId" class="block text-sm font-medium mb-2">根目录 *</label>
                        <Select id="rootId" v-model="extractConfig.rootId" :options="rootList" option-label="name"
                            option-value="id" :placeholder="rootList.length > 0 ? '请选择根目录' : '请先选择数据库文件'"
                            :disabled="isExtracting || rootList.length === 0" class="w-full"
                            :class="{ 'p-invalid': errors.rootId }" />
                        <Message v-if="errors.rootId" severity="error" size="small">{{
                            errors.rootId
                            }}</Message>
                    </div>

                    <!-- 覆盖选项 -->
                    <div class="field flex items-center">
                        <Checkbox id="overwrite" v-model="extractConfig.overwrite" :binary="true"
                            :disabled="isExtracting" />
                        <label for="overwrite" class="ml-2 block text-sm font-medium">覆盖已存在的文件</label>
                    </div>

                    <!-- 归档目录（从数据库获取） -->
                    <div class="field">
                        <label class="block text-sm font-medium mb-2">归档根目录</label>
                        <InputText :value="selectedRoot ? selectedRoot.path : '请先选择根目录'" readonly class="w-full"
                            :disabled="!selectedRoot" />
                        <small class="text-gray-500">该路径将从数据库中获取</small>
                    </div>
                </div>

                <!-- 进度条 -->
                <div class="mt-6" v-if="isExtracting || progress > 0">
                    <ProgressBar :value="progress" :show-value="true" />
                    <p class="text-center mt-2">{{ statusMessage }}</p>
                    <p class="text-center text-sm text-gray-500 mt-1" v-if="totalFiles > 0">
                        已处理：{{ processedFiles }} / {{ totalFiles }} 文件
                    </p>
                </div>

                <!-- 结果显示 -->
                <div class="mt-4" v-if="extractResult">
                    <Card>
                        <template #title>解档结果</template>
                        <template #content>
                            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                                <div>
                                    <p><strong>总文件数：</strong> {{ extractResult.total_files }}</p>
                                    <p><strong>已处理文件数：</strong> {{ extractResult.processed_files }}</p>
                                </div>
                                <div>
                                    <p><strong>完成状态：</strong> {{ extractResult.completed ? '已完成' : '未完成' }}</p>
                                    <p v-if="extractResult.error"><strong>错误：</strong> {{ extractResult.error }}</p>
                                </div>
                            </div>
                        </template>
                    </Card>
                </div>

                <!-- 操作按钮 -->
                <div class="flex justify-center gap-4 mt-8">
                    <Button label="开始解档" icon="pi pi-download" @click="startExtract"
                        :disabled="isExtracting || !selectedRoot" class="px-6" />
                    <Button label="重置" icon="pi pi-refresh" @click="resetForm" :disabled="isExtracting"
                        class="p-button-secondary px-6" />
                </div>
            </template>
        </Card>
    </div>
</template>

<style scoped>
.p-button {
    min-width: 100px;
}
</style>
