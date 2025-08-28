<!-- src/pages/archive.vue -->
<template>
  <div class="container mx-auto p-4">
    <h1 class="text-3xl font-bold mb-6 text-center text-primary-700">执行存档</h1>

    <!-- 配置表单 -->
    <ArchiveConfigForm v-model="archiveConfig" @start-archive="startArchive" @reset-form="resetForm"
      @[ArchiveFormEvents.OPEN_DIRECTORY_DIALOG]="openDirectoryDialog" @open-file-dialog="openFileDialog" />

    <!-- 进度显示区域 -->
    <ArchiveProgress :task-info="taskInfo" />

    <!-- 目录选择对话框 -->
    <DirectoryDialog :visible="showDirectoryDialog" :model-value="directoryPath"
      @update:visible="showDirectoryDialog = $event" @update:model-value="directoryPath = $event"
      @confirm="confirmDirectory" />

    <!-- 文件选择对话框 -->
    <FileDialog :visible="showFileDialog" :model-value="filePath" @update:visible="showFileDialog = $event"
      @update:model-value="filePath = $event" @confirm="confirmFile" />
  </div>
</template>

<script setup lang="ts">
import { useToast } from 'primevue/usetoast'
import { ref } from 'vue'
import ArchiveConfigForm from '../components/ArchiveConfigForm.vue'
import ArchiveProgress from '../components/ArchiveProgress.vue'
import DirectoryDialog from '../components/DirectoryDialog.vue'
import FileDialog from '../components/FileDialog.vue'
import { ArchiveFormEvents } from '../event'

const toast = useToast()

const archiveConfig = ref({
  rootDir: '',
  archiveDir: '',
  archivePrefix: 'archive',
  dbPath: '',
  archiveLimitSize: null as number | null
})

const isArchiving = ref(false)
const taskInfo = ref({
  taskId: null as string | null,
  status: 'PENDING', // PENDING, RUNNING, COMPLETED, FAILED
  progress: 0,
  message: '等待开始',
  duration: null as number | null
})

// 对话框相关
const showDirectoryDialog = ref(false)
const showFileDialog = ref(false)
const currentField = ref('')
const directoryPath = ref('')
const filePath = ref('')

// 重置表单
const resetForm = () => {
  archiveConfig.value = {
    rootDir: '',
    archiveDir: '',
    archivePrefix: 'archive',
    dbPath: '',
    archiveLimitSize: null
  }
}

// 打开目录选择对话框
const openDirectoryDialog = (field: string) => {
  currentField.value = field
  directoryPath.value = (archiveConfig.value as any)[field] || ''
  showDirectoryDialog.value = true
}

// 打开文件选择对话框
const openFileDialog = (field: string) => {
  currentField.value = field
  filePath.value = (archiveConfig.value as any)[field] || ''
  showFileDialog.value = true
}

// 确认目录选择
const confirmDirectory = () => {
  (archiveConfig.value as any)[currentField.value] = directoryPath.value
  showDirectoryDialog.value = false
}

// 确认文件选择
const confirmFile = () => {
  (archiveConfig.value as any)[currentField.value] = filePath.value
  showFileDialog.value = false
}

// 开始存档
const startArchive = async () => {
  if (isArchiving.value) return

  isArchiving.value = true
  taskInfo.value = {
    taskId: null,
    status: 'PENDING',
    progress: 0,
    message: '正在初始化任务...',
    duration: null
  }

  try {
    // 创建一个 AbortController 用于取消请求
    const controller = new AbortController()

    // 发起存档请求
    const response = await fetch('/api/v1/archive', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json'
      },
      body: JSON.stringify(archiveConfig.value),
      signal: controller.signal
    })

    if (!response.body) {
      throw new Error('ReadableStream not supported')
    }

    const reader = response.body.getReader()
    const decoder = new TextDecoder()

    // 读取 SSE 流
    while (true) {
      const { done, value } = await reader.read()

      if (done) {
        break
      }

      const chunk = decoder.decode(value, { stream: true })

      // 解析 SSE 数据
      const lines = chunk.split('\n')

      for (const line of lines) {
        if (line.startsWith('data: ')) {
          try {
            const data = JSON.parse(line.slice(6))

            // 根据事件类型更新任务信息
            if (data.taskId) {
              taskInfo.value.taskId = data.taskId
            }

            if (data.status) {
              taskInfo.value.status = data.status
            }

            if (data.progress !== undefined) {
              taskInfo.value.progress = data.progress
            }

            if (data.message) {
              taskInfo.value.message = data.message
            }

            if (data.duration) {
              taskInfo.value.duration = data.duration
            }

            // 如果任务完成或失败，停止轮询
            if (data.status === 'COMPLETED' || data.status === 'FAILED') {
              isArchiving.value = false
              break
            }
          } catch (e) {
            console.error('Error parsing SSE data:', e)
          }
        }
      }
    }
  } catch (error) {
    taskInfo.value.status = 'FAILED'
    taskInfo.value.message = '启动存档任务失败: ' + (error instanceof Error ? error.message : String(error))
    isArchiving.value = false

    // 显示错误提示
    toast.add({
      severity: 'error',
      summary: '错误',
      detail: taskInfo.value.message,
      life: 5000
    })
  }
}
</script>

<style scoped></style>