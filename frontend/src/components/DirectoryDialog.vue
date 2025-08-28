<script setup lang="ts">
import { ref, watch } from 'vue'

const props = defineProps<{
  visible: boolean
  modelValue: string
}>()

const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void
  (e: 'update:modelValue', value: string): void
  (e: 'confirm'): void
}>()

const filePath = ref('')

watch(() => props.visible, (newVal) => {
  if (newVal) {
    filePath.value = props.modelValue
  }
})

watch(() => props.modelValue, (newVal) => {
  filePath.value = newVal
})

const handleConfirm = () => {
  emit('update:modelValue', filePath.value)
  emit('confirm')
  emit('update:visible', false)
}

const handleCancel = () => {
  emit('update:visible', false)
}
</script>

<template>
  <Dialog :visible="visible" :modal="true" header="选择数据库文件" :style="{ width: '50vw' }" class="rounded-lg"
    @update:visible="$emit('update:visible', $event)">
    <div class="flex flex-col space-y-4">
      <div class="flex flex-col">
        <FloatLabel variant="on">
          <InputText v-model="filePath" placeholder="输入文件路径"
            class="w-full focus:ring-2 focus:ring-primary-500 rounded-md" />
          <label>文件路径</label>
        </FloatLabel>
      </div>
      <div class="flex justify-end space-x-2 pt-2">
        <Button label="取消" icon="pi pi-times" @click="handleCancel" severity="secondary" class="px-4 py-2" />
        <Button label="确认" icon="pi pi-check" @click="handleConfirm" class="px-4 py-2" />
      </div>
    </div>
  </Dialog>
</template>