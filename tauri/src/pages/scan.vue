<!-- TODO 存在的问题
- 扫描页面的功能, 需要调整, 纯扫描对于软件并无用处, 因为归档时为了保证数据最新, 会重新扫描
-->

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import Card from "primevue/card";
import InputText from "primevue/inputtext";
import Button from "primevue/button";
import Message from "primevue/message";
import ProgressBar from "primevue/progressbar";
import Checkbox from "primevue/checkbox";
import { useGlobalStore } from "../store/global";
import { useToast } from 'primevue/usetoast';
import { scanRoot } from '../api';

const toast = useToast();
const globalStore = useGlobalStore();

// 表单数据
const scanConfig = ref({
  rootDir: "",
  dbPath: globalStore.dbPath || "",
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
    globalStore.setDbPath(selected);
  }
};

// 开始扫描
const startScan = async () => {
  if (!validateForm() || isScanning.value) return;

  isScanning.value = true;
  progress.value = 0;
  statusMessage.value = "正在初始化...";

  try {
    // 调用新的API执行扫描
    statusMessage.value = "正在扫描目录...";
    const result = await scanRoot({
      rootPath: scanConfig.value.rootDir,
      dbPath: scanConfig.value.dbPath
    });

    if (result.success) {
      statusMessage.value = "扫描完成";
      progress.value = 100;
    } else {
      throw new Error(result.error || '扫描失败');
    }
  } catch (error: any) {
    statusMessage.value = `扫描失败：${error}`;
    console.error("Scan error:", error);
    toast.add({ severity: 'error', summary: '扫描失败', detail: error.message || error.toString(), life: 5000 });
  } finally {
    isScanning.value = false;
  }
};

// 设置进度监听器
const setupProgressListener = async () => {
  // 监听 scan-progress 事件
  unlisten = await listen('scan-progress', (event) => {
    const progressData = event.payload as {
      processed: number;
      total: number;
      message: string;
      progress: number;
    };

    // 更新进度条和状态信息
    progress.value = Math.round((progressData.processed / progressData.total) * 100);
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

// 组件挂载时设置监听器
onMounted(() => {
  setupProgressListener();
});

// 组件卸载时清理监听器
onUnmounted(() => {
  cleanupProgressListener();
});

// 重置表单
const resetForm = () => {
  scanConfig.value = {
    rootDir: "",
    dbPath: globalStore.dbPath || "",
    overwrite: false
  };
  errors.value = {};
  progress.value = 0;
  statusMessage.value = "准备就绪";
};
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
              <InputText id="rootDir" v-model="scanConfig.rootDir" placeholder="选择要扫描的根目录"
                :class="{ 'p-invalid': errors.rootDir }" class="flex-1" />
              <Button @click="selectRootDirectory" label="浏览" />
            </div>
            <Message v-if="errors.rootDir" severity="error" size="small" variant="simple">{{ errors.rootDir }}</Message>
          </div>

          <!-- 数据库路径 -->
          <div class="field md:col-span-2">
            <label for="dbPath" class="block text-sm font-medium mb-2">数据库路径 *</label>
            <div class="flex gap-2">
              <InputText id="dbPath" v-model="scanConfig.dbPath" placeholder="选择或输入数据库路径"
                :class="{ 'p-invalid': errors.dbPath }" class="flex-1" />
              <Button @click="selectDatabaseFile" label="浏览" />
            </div>
            <Message v-if="errors.dbPath" severity="error" size="small" variant="simple">{{ errors.dbPath }}</Message>
          </div>

          <!-- 覆盖选项 -->
          <div class="field md:col-span-2">
            <div class="flex items-center">
              <Checkbox id="overwrite" v-model="scanConfig.overwrite" :binary="true" class="mr-2" />
              <label for="overwrite" class="text-sm">覆盖已有数据</label>
            </div>
          </div>
        </div>
      </template>
      <template #footer>
        <div class="flex justify-end gap-2">
          <Button label="重置" severity="secondary" @click="resetForm" :disabled="isScanning" />
          <Button label="开始扫描" @click="startScan" :loading="isScanning" />
        </div>
      </template>
    </Card>

    <!-- 进度和状态显示 -->
    <Card class="mt-6" v-if="isScanning || progress > 0">
      <template #title>
        <h2 class="text-xl font-semibold">扫描进度</h2>
      </template>
      <template #content>
        <ProgressBar :value="progress" :showValue="true" />
        <p class="mt-2 text-center">{{ statusMessage }}</p>
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
