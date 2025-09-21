<script setup lang="ts">
import { ref, computed, markRaw } from 'vue'
import { useRoute } from 'vue-router'
import Splitter from 'primevue/splitter'
import SplitterPanel from 'primevue/splitterpanel'
import TaskList from './task-center/TaskList.vue'
import ArchiveTask from './task-center/ArchiveTask.vue'
import ExtractTask from './task-center/ExtractTask.vue'
import ScanTask from './task-center/ScanTask.vue'

const route = useRoute()

// 任务类型定义
interface TaskType {
    id: string
    name: string
    icon: string
    component: any
}

// 任务列表 - 可以轻松扩展
const taskTypes = ref<TaskType[]>([
    {
        id: 'scan',
        name: '扫描',
        icon: 'pi pi-search',
        component: markRaw(ScanTask)
    },
    {
        id: 'archive',
        name: '归档任务',
        icon: 'pi pi-box',
        component: markRaw(ArchiveTask)
    },
    {
        id: 'extract',
        name: '解档任务',
        icon: 'pi pi-folder-open',
        component: markRaw(ExtractTask)
    },

    // 后续可以添加更多任务类型，例如：
    // { 
    //   id: 'sync', 
    //   name: '同步任务', 
    //   icon: 'pi pi-refresh',
    //   component: markRaw(SyncTask)
    // },
    // { 
    //   id: 'verify', 
    //   name: '校验任务', 
    //   icon: 'pi pi-check-circle',
    //   component: markRaw(VerifyTask) 
    // }
])

// 当前选中的任务
const currentTask = computed(() => {
    const taskId = route.query.task || 'archive'
    return taskTypes.value.find(task => task.id === taskId) || taskTypes.value[0]
})
</script>


<template>
    <div class="task-center-page">
        <h1 class="text-3xl font-bold mb-6 text-center text-primary-700">任务中心</h1>

        <Splitter class="mb-6" style="height: calc(100vh - 200px)">
            <!-- 左侧任务列表 -->
            <SplitterPanel :size="30" :minSize="20" class="flex flex-col">
                <TaskList :task-types="taskTypes" />
            </SplitterPanel>

            <!-- 右侧任务详情 -->
            <SplitterPanel :size="70" :minSize="50" class="flex flex-col">
                <component :is="currentTask.component" />
            </SplitterPanel>
        </Splitter>
    </div>
</template>

<style scoped>
.task-center-page {
    height: 100%;
}
</style>