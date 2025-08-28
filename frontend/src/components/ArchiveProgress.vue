<script setup lang="ts">

const props = defineProps<{
  taskInfo: {
    taskId: string | null
    status: string
    progress: number
    message: string
    duration: number | null
  }
}>()

const getStatusSeverity = (status: string) => {
  switch (status) {
    case 'RUNNING': return 'info'
    case 'COMPLETED': return 'success'
    case 'FAILED': return 'danger'
    default: return 'secondary'
  }
}

const getStatusText = (status: string) => {
  switch (status) {
    case 'PENDING': return '等待中'
    case 'RUNNING': return '进行中'
    case 'COMPLETED': return '已完成'
    case 'FAILED': return '失败'
    default: return status
  }
}

const formatDuration = (milliseconds: number) => {
  const seconds = Math.floor(milliseconds / 1000)
  const mins = Math.floor(seconds / 60)
  const secs = seconds % 60
  return `${mins}分${secs}秒`
}
</script>

<template>
  <Card v-if="taskInfo.taskId" class="shadow-lg rounded-lg animate-fade-in">
    <template #header>
      <h2 class="text-xl font-semibold p-4 bg-secondary-50 text-secondary-800 rounded-t-lg">存档进度</h2>
    </template>
    <template #content>
      <div class="space-y-4">
        <div class="flex justify-between items-center p-2 bg-gray-50 rounded">
          <span class="font-medium">任务ID:</span>
          <span class="font-mono bg-gray-200 px-2 py-1 rounded">{{ taskInfo.taskId }}</span>
        </div>

        <div class="flex justify-between items-center p-2 bg-gray-50 rounded">
          <span class="font-medium">状态:</span>
          <Badge :severity="getStatusSeverity(taskInfo.status)" class="text-sm">
            {{ getStatusText(taskInfo.status) }}
          </Badge>
        </div>

        <div v-if="taskInfo.status !== 'PENDING'" class="p-2 bg-gray-50 rounded">
          <div class="flex justify-between mb-1">
            <span class="font-medium">进度:</span>
            <span>{{ taskInfo.progress }}%</span>
          </div>
          <ProgressBar :value="taskInfo.progress" class="rounded-full h-3">
            <template>
              <span class="text-xs text-white font-semibold">{{ taskInfo.progress }}%</span>
            </template>
          </ProgressBar>
        </div>

        <div class="p-2 bg-gray-50 rounded">
          <span class="font-medium block mb-1">消息:</span>
          <p class="text-gray-700 bg-white p-3 rounded border border-gray-200">{{ taskInfo.message }}</p>
        </div>

        <div v-if="taskInfo.duration" class="flex justify-between items-center p-2 bg-gray-50 rounded">
          <span class="font-medium">耗时:</span>
          <span class="font-mono">{{ formatDuration(taskInfo.duration) }}</span>
        </div>
      </div>

      <div v-if="taskInfo.status === 'COMPLETED'"
        class="mt-4 p-4 bg-green-100 rounded border border-green-300 animate-pulse-once">
        <div class="flex items-center">
          <i class="pi pi-check-circle text-green-600 mr-2 text-xl"></i>
          <p class="text-green-700 font-medium">存档已完成！</p>
        </div>
      </div>

      <div v-if="taskInfo.status === 'FAILED'" class="mt-4 p-4 bg-red-100 rounded border border-red-300">
        <div class="flex items-center">
          <i class="pi pi-times-circle text-red-600 mr-2 text-xl"></i>
          <div>
            <p class="text-red-700 font-medium">存档失败</p>
            <p class="text-red-600 text-sm mt-1">{{ taskInfo.message }}</p>
          </div>
        </div>
      </div>
    </template>
  </Card>
</template>