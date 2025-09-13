<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import Card from "primevue/card";
import InputText from "primevue/inputtext";
import Button from "primevue/button";
import Message from "primevue/message";
import ProgressBar from "primevue/progressbar";
import Checkbox from "primevue/checkbox";

// 表单数据
const scanConfig = ref({
  rootDir: "",
  workspaceDir: "",
  dbPath: "",
  overwrite: false
});

// 表单验证错误
const errors = ref<Record<string, string>>({});

// 扫描状态
const isScanning = ref(false);
const progress = ref(0);
const statusMessage = ref("准备就绪");

// 事件监听器引用
let unlisten: (() => void) | null = null;

// 验证表单
const validateForm = () => {
  const newErrors: Record<string, string> = {};

  if (!scanConfig.value.rootDir) {
    newErrors.rootDir = "根目录不能为空";
  }

  if (!scanConfig.value.workspaceDir) {
    newErrors.workspaceDir = "工作目录不能为空";
  }

  if (!scanConfig.value.dbPath) {
    newErrors.dbPath = "数据库路径不能为空";
  }

  errors.value = newErrors;
  return Object.keys(newErrors).length === 0;
};

// 选择根目录
const selectRootDirectory = async () => {
  const selected = await open({
    directory: true,
  });

  if (selected) {
    scanConfig.value.rootDir = selected;
  }
};

// 选择工作目录
const selectWorkspaceDirectory = async () => {
  const selected = await open({
    directory: true,
  });

  if (selected) {
    scanConfig.value.workspaceDir = selected;
    // 自动生成默认数据库路径
    scanConfig.value.dbPath = selected + "/archive.db";
  }
};

// 选择数据库文件
const selectDatabaseFile = async () => {
  const selected = await open({
    filters: [{
      name: "SQLite Database",
      extensions: ["db", "sqlite", "sqlite3"]
    }]
  });

  if (selected) {
    scanConfig.value.dbPath = selected;
  }
};

// 开始扫描
const startScan = async () => {
  if (!validateForm() || isScanning.value) return;

  isScanning.value = true;
  progress.value = 0;
  statusMessage.value = "正在初始化...";

  try {
    // 调用 Rust 命令执行扫描（带进度回调）
    statusMessage.value = "正在扫描目录...";
    await invoke("scan_root_with_progress", {
      rootPath: scanConfig.value.rootDir,
      dbPath: scanConfig.value.dbPath
    });

    statusMessage.value = "扫描完成";
    progress.value = 100;
  } catch (error: any) {
    statusMessage.value = `扫描失败：${error}`;
    console.error("Scan error:", error);
  } finally {
    isScanning.value = false;
  }
};

// 设置进度监听器
const setupProgressListener = async () => {
  // 监听scan-progress事件
  unlisten = await listen('scan-progress', (event) => {
    const progressData = event.payload as {
      processed: number;
      total: number;
      message: string;
      progress: number;
    };

    // 更新进度条和状态信息
    progress.value = progressData.progress;
    statusMessage.value = progressData.message;
  });
};

// 清理事件监听器
const cleanupProgressListener = () => {
  if (unlisten) {
    unlisten();
    unlisten = null;
  }
};

// 重置表单
const resetForm = () => {
  scanConfig.value = {
    rootDir: "",
    workspaceDir: "",
    dbPath: "",
    overwrite: false
  };
  errors.value = {};
  progress.value = 0;
  statusMessage.value = "准备就绪";
};

// 组件挂载和卸载时的处理
onMounted(async () => {
  await setupProgressListener();
});

onUnmounted(() => {
  cleanupProgressListener();
});
</script>

<template>
  <div class="container mx-auto p-4">
    <h1 class="text-3xl font-bold mb-6 text-center text-primary-700">扫描根目录</h1>

    <Card>
      <template #title>
        <h2 class="text-xl font-semibold">扫描配置</h2>
      </template>
      <template #content>
        <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
          <!-- 根目录 -->
          <div class="field">
            <label for="rootDir" class="block text-sm font-medium mb-2">根目录 *</label>
            <div class="flex gap-2">
              <InputText id="rootDir" v-model="scanConfig.rootDir" placeholder="选择要扫描的根目录" :disabled="isScanning"
                class="flex-1" :class="{ 'p-invalid': errors.rootDir }" />
              <Button label="浏览" @click="selectRootDirectory" :disabled="isScanning" />
            </div>
            <Message v-if="errors.rootDir" severity="error" size="small">{{ errors.rootDir }}</Message>
          </div>

          <!-- 工作目录 -->
          <div class="field">
            <label for="workspaceDir" class="block text-sm font-medium mb-2">工作目录 *</label>
            <div class="flex gap-2">
              <InputText id="workspaceDir" v-model="scanConfig.workspaceDir" placeholder="选择工作目录" :disabled="isScanning"
                class="flex-1" :class="{ 'p-invalid': errors.workspaceDir }" />
              <Button label="浏览" @click="selectWorkspaceDirectory" :disabled="isScanning" />
            </div>
            <Message v-if="errors.workspaceDir" severity="error" size="small">{{ errors.workspaceDir }}</Message>
            <small class="block text-gray-500 mt-1">用于存放数据库文件的工作目录</small>
          </div>

          <!-- 数据库路径 -->
          <div class="field md:col-span-2">
            <label for="dbPath" class="block text-sm font-medium mb-2">数据库路径</label>
            <div class="flex gap-2">
              <InputText id="dbPath" v-model="scanConfig.dbPath" placeholder="数据库文件路径" :disabled="isScanning"
                class="flex-1" :class="{ 'p-invalid': errors.dbPath }" />
              <Button label="浏览" @click="selectDatabaseFile" :disabled="isScanning" />
            </div>
            <Message v-if="errors.dbPath" severity="error" size="small">{{ errors.dbPath }}</Message>
            <small class="block text-gray-500 mt-1">系统将根据工作目录自动生成，也可手动指定</small>
          </div>

          <!-- 覆盖选项 -->
          <div class="field md:col-span-2">
            <div class="flex items-center gap-2">
              <Checkbox id="overwrite" v-model="scanConfig.overwrite" :disabled="isScanning" binary />
              <label for="overwrite" class="text-sm font-medium">覆盖已存在的数据</label>
            </div>
            <small class="block text-gray-500 mt-1">勾选此项将覆盖数据库中已存在的相同路径的数据</small>
          </div>
        </div>

        <!-- 操作按钮 -->
        <div class="flex justify-center gap-4 mt-8">
          <Button label="开始扫描" icon="pi pi-search" @click="startScan" :disabled="isScanning" severity="primary" />
          <Button label="重置" icon="pi pi-refresh" @click="resetForm" :disabled="isScanning" severity="secondary" />
        </div>
      </template>
    </Card>

    <!-- 进度显示 -->
    <Card v-if="isScanning || progress > 0" class="mt-6">
      <template #title>
        <h2 class="text-xl font-semibold">扫描进度</h2>
      </template>
      <template #content>
        <div class="space-y-4">
          <ProgressBar :value="progress" />
          <p class="text-center">{{ statusMessage }}</p>
        </div>
      </template>
    </Card>
  </div>
</template>

<style scoped>
.field {
  margin-bottom: 1rem;
}

.p-card {
  border: 1px solid #dee2e6;
  border-radius: 6px;
}

.p-input,
.p-inputtext {
  width: 100%;
}
</style>