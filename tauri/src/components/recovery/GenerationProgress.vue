<template>
    <div v-if="visible" class="mt-6">
        <Card>
            <template #title>
                <h2 class="text-xl font-semibold">生成进度</h2>
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
                            <p class="text-sm text-gray-600">处理状态</p>
                            <p class="font-medium">{{ statusMessage }}</p>
                        </div>
                        <div class="p-3 bg-gray-50 rounded">
                            <p class="text-sm text-gray-600">灾备组 ID</p>
                            <p class="font-medium">{{ groupId || '未生成' }}</p>
                        </div>
                    </div>
                </div>
            </template>
        </Card>
    </div>
</template>

<script setup lang="ts">
import Card from "primevue/card";
import ProgressBar from "primevue/progressbar";

defineProps<{
    visible: boolean;
    progress: number;
    statusMessage: string;
    groupId: string;
}>();
</script>