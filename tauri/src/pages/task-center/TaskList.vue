<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import Menu from 'primevue/menu'

interface TaskType {
    id: string
    name: string
    icon: string
    component: any
}

const props = defineProps<{
    taskTypes: TaskType[]
}>()

const router = useRouter()
const route = useRoute()

// 构建菜单项
const menuItems = computed(() => props.taskTypes.map(task => ({
    label: task.name,
    id: task.id,
    icon: task.icon,
    command: () => {
        router.push({ query: { task: task.id } })
    }
})))
</script>

<template>
    <div class="p-4 border-b border-surface-200">
        <h2 class="text-xl font-semibold mb-4">任务类型</h2>
    </div>

    <div class="flex-1 overflow-y-auto">
        <Menu :model="menuItems" class="w-full border-0 p-0">
            <template #item="{ item, props }">
                <a v-ripple
                    class="flex align-items-center px-3 py-2 cursor-pointer transition-colors transition-duration-200"
                    v-bind="props.action" :class="route.query.task === item.id || (!route.query.task && item.id === 'archive') ?
                        'bg-primary text-primary-contrast font-medium rounded-md m-1' :
                        'hover:bg-surface-200 text-surface-700 rounded-md m-1'">
                    <i :class="[item.icon, 'mr-2', route.query.task === item.id || (!route.query.task && item.id === 'archive') ? 'text-primary-contrast' : '']"
                        :style="{
                            color: route.query.task === item.id || (!route.query.task && item.id === 'archive') ?
                                'inherit' : (item.id === 'archive' ? '#3b82f6' : '#10b981')
                        }"></i>
                    <span class="whitespace-nowrap overflow-hidden text-ellipsis" :class="route.query.task === item.id || (!route.query.task && item.id === 'archive') ?
                        'text-primary-contrast' : ''">
                        {{ item.label }}
                    </span>
                </a>
            </template>
        </Menu>
    </div>
</template>