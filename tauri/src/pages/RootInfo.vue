<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import Card from "primevue/card";
import Button from "primevue/button";
import DataTable from "primevue/datatable";
import Column from "primevue/column";
import Message from "primevue/message";
import { useGlobalStore } from '../store/global';

const globalStore = useGlobalStore();

// 状态变量
const roots = ref<any[]>([]);
const loading = ref(false);
const error = ref("");

// 选择数据库文件
const selectDatabaseFile = async () => {
  const selected = await open({
    filters: [{
      name: "SQLite Database",
      extensions: ["db", "sqlite", "sqlite3"]
    }]
  });

  if (selected) {
    globalStore.setDbPath(selected);
    loadRoots();
  }
};

// 加载所有根目录
const loadRoots = async () => {
  if (!globalStore.dbPath) {
    error.value = "请先选择数据库文件";
    return;
  }

  loading.value = true;
  error.value = "";

  try {
    const result = await invoke("get_all_roots", {
      dbPath: globalStore.dbPath
    });

    roots.value = result as any[];
  } catch (err: any) {
    error.value = `加载根目录失败：${err}`;
    console.error("Load roots error:", err);
  } finally {
    loading.value = false;
  }
};

// 格式化时间戳
const formatTimestamp = (timestamp: number | null) => {
  if (!timestamp) return "-";
  const date = new Date(timestamp * 1000);
  return date.toLocaleString();
};

// 页面加载时尝试使用默认数据库路径
onMounted(() => {
  // 从全局状态获取数据库路径
  if (globalStore.dbPath) {
    loadRoots();
  }
});
</script>

<template>
  <div class="container mx-auto p-4">
    <h1 class="text-3xl font-bold mb-6 text-center text-primary-700">根目录管理</h1>

    <Card>
      <template #title>
        <h2 class="text-xl font-semibold">选择数据库</h2>
      </template>
      <template #content>
        <div class="flex flex-col md:flex-row gap-4 items-center">
          <div class="flex-1">
            <label for="dbPath" class="block text-sm font-medium mb-2">数据库文件</label>
            <div class="flex gap-2">
              <input id="dbPath" :value="globalStore.dbPath" type="text" placeholder="请选择数据库文件"
                class="flex-1 p-2 border rounded" readonly />
              <Button @click="selectDatabaseFile" label="浏览" />
            </div>
          </div>

          <div class="pt-6">
            <Button @click="loadRoots" :disabled="loading || !globalStore.dbPath" :loading="loading" label="加载根目录" />
          </div>
        </div>

        <div v-if="error" class="mt-4">
          <Message severity="error">{{ error }}</Message>
        </div>
      </template>
    </Card>

    <Card class="mt-6">
      <template #title>
        <h2 class="text-xl font-semibold">根目录列表</h2>
      </template>
      <template #content>
        <DataTable :value="roots" :loading="loading" stripedRows responsiveLayout="scroll" class="p-datatable-sm">
          <Column field="id" header="ID" :sortable="true"></Column>
          <Column field="root_name" header="名称" :sortable="true"></Column>
          <Column field="root_path" header="路径" :sortable="true">
            <template #body="slotProps">
              <span class="font-mono text-sm">{{ slotProps.data.root_path }}</span>
            </template>
          </Column>
          <Column field="status" header="状态" :sortable="true">
            <template #body="slotProps">
              <span :class="{
                'px-2 py-1 rounded text-xs': true,
                'bg-green-100 text-green-800': slotProps.data.status === 'HEALTH',
                'bg-yellow-100 text-yellow-800': slotProps.data.status === 'RECYCLED'
              }">
                {{ slotProps.data.status }}
              </span>
            </template>
          </Column>
          <Column field="created_at" header="创建时间" :sortable="true">
            <template #body="slotProps">
              {{ formatTimestamp(slotProps.data.created_at) }}
            </template>
          </Column>
          <Column field="updated_at" header="更新时间" :sortable="true">
            <template #body="slotProps">
              {{ formatTimestamp(slotProps.data.updated_at) }}
            </template>
          </Column>
        </DataTable>

        <div v-if="!loading && roots.length === 0" class="text-center py-8 text-gray-500">
          <p>暂无根目录数据</p>
          <p class="text-sm mt-2">请选择数据库文件并点击"加载根目录"按钮</p>
        </div>
      </template>
    </Card>
  </div>
</template>

<style scoped>
.font-mono {
  font-family: ui-monospace, SFMono-Regular, "SF Mono", Menlo, Consolas, "Liberation Mono", monospace;
}
</style>