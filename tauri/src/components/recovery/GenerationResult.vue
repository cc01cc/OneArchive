<template>
    <div v-if="visible" class="mt-6">
        <Card>
            <template #title>
                <h2 class="text-xl font-semibold">生成结果</h2>
            </template>
            <template #content>
                <pre class="bg-gray-100 p-4 rounded">{{ resultMessage }}</pre>
            </template>
        </Card>
    </div>
</template>

<script setup lang="ts">
import Card from "primevue/card";

defineProps<{
    visible: boolean;
    resultMessage: string;
}>();
</script>