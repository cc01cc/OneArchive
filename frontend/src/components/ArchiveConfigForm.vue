<script setup lang="ts">
import { reactive, computed, ref } from 'vue'
import InputText from 'primevue/inputtext'
import InputNumber from 'primevue/inputnumber'
import Button from 'primevue/button'
import Card from 'primevue/card'
import Message from 'primevue/message'
import { Form } from '@primevue/forms'
import { ArchiveFormEvents } from '../event'

const props = defineProps<{
  modelValue: {
    rootDir: string
    archiveDir: string
    archivePrefix: string
    dbPath: string
    archiveLimitSize: number | null
  }
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', value: {
    rootDir: string
    archiveDir: string
    archivePrefix: string
    archiveLimitSize: number | null
  }): void
  (e: 'startArchive'): void
  (e: 'resetForm'): void
  (e: ArchiveFormEvents.OPEN_DIRECTORY_DIALOG, field: string): void
  (e: 'openFileDialog', field: string): void
}>()

const form = reactive({ ...props.modelValue })
const touchedFields = ref<Record<string, boolean>>({})

const errors = computed(() => {
  const err: Record<string, string> = {}
  if (!form.rootDir) err.rootDir = '根目录不能为空'
  if (!form.archiveDir) err.archiveDir = '存档目录不能为空'
  if (!form.archivePrefix) err.archivePrefix = '存档文件前缀不能为空'
  if (!form.dbPath) err.dbPath = '数据库路径不能为空'
  if (form.archiveLimitSize !== null && form.archiveLimitSize < 0) err.archiveLimitSize = '必须为非负数'
  return err
})

const isValid = computed(() => Object.keys(errors.value).length === 0)

const shouldShowError = (field: string) => {
  return touchedFields.value[field] && errors.value[field]
}

const onSubmit = () => {
  // 标记所有字段为已触摸，以显示所有错误
  Object.keys(form).forEach(field => {
    touchedFields.value[field] = true
  })

  if (!isValid.value) return
  emit('update:modelValue', { ...form })
  emit('startArchive')
}

const onReset = () => {
  Object.assign(form, props.modelValue)
  // 重置所有字段的触摸状态
  touchedFields.value = {}
  emit('resetForm')
}

const markAsTouched = (field: string) => {
  touchedFields.value[field] = true
}
</script>
<template>
  <Card>
    <template #title>
      <h2 class="form-title">存档配置</h2>
    </template>
    <template #content>
      <Form @submit.prevent="onSubmit">
        <div class="form-grid">
          <div class="form-field">
            <label for="rootDir" class="form-label">根目录</label>
            <InputText id="rootDir" v-model="form.rootDir" placeholder="选择要存档的根目录" readonly
              @click="$emit(ArchiveFormEvents.OPEN_DIRECTORY_DIALOG, 'rootDir'); markAsTouched('rootDir')"
              :class="{ 'p-invalid': shouldShowError('rootDir') }" />
            <Message v-if="shouldShowError('rootDir')" severity="error" size="small" variant="simple">{{ errors.rootDir
              }}</Message>
          </div>

          <div class="form-field">
            <label for="archiveDir" class="form-label">存档目录</label>
            <InputText id="archiveDir" v-model="form.archiveDir" placeholder="选择存档存储目录" readonly
              @click="$emit(ArchiveFormEvents.OPEN_DIRECTORY_DIALOG, 'archiveDir'); markAsTouched('archiveDir')"
              :class="{ 'p-invalid': shouldShowError('archiveDir') }" />
            <Message v-if="shouldShowError('archiveDir')" severity="error" size="small" variant="simple">{{
              errors.archiveDir }}
            </Message>
          </div>

          <div class="form-field">
            <label for="archivePrefix" class="form-label">存档文件前缀</label>
            <InputText id="archivePrefix" v-model="form.archivePrefix" placeholder="存档文件前缀"
              @blur="markAsTouched('archivePrefix')" :class="{ 'p-invalid': shouldShowError('archivePrefix') }" />
            <Message v-if="shouldShowError('archivePrefix')" severity="error" size="small" variant="simple">{{
              errors.archivePrefix
              }}</Message>
          </div>

          <div class="form-field">
            <label for="archiveLimitSize" class="form-label">存档大小限制 (字节)</label>
            <InputNumber id="archiveLimitSize" v-model="form.archiveLimitSize" placeholder="默认无限制" :min="0"
              :useGrouping="false" @blur="markAsTouched('archiveLimitSize')"
              :class="{ 'p-invalid': shouldShowError('archiveLimitSize') }" />
            <Message v-if="shouldShowError('archiveLimitSize')" severity="error" size="small" variant="simple">{{
              errors.archiveLimitSize }}</Message>
          </div>
        </div>

        <div class="form-actions">
          <Button type="submit" severity="primary" icon="pi" :disabled="!isValid" label="开始存档" />
          <Button type="button" severity="secondary" icon="pi pi-refresh" @click="onReset" label="重置" />
        </div>
      </Form>
    </template>
  </Card>
</template>

<style scoped>
.p-card {
  border: 2px solid #d1d5db !important;
  box-shadow: 0 6px 32px 0 rgba(0, 0, 0, 0.16), 0 2px 8px 0 rgba(0, 0, 0, 0.12) !important;
  border-radius: 14px !important;
  transition: box-shadow 0.2s;
  max-width: 560px;
  margin: 32px auto;
  width: 100%;
  background: #fff;
}

.p-card:hover {
  box-shadow: 0 8px 32px 0 rgba(0, 0, 0, 0.18), 0 3px 8px 0 rgba(0, 0, 0, 0.14) !important;
  border-color: #bdbdbd !important;
}

.form-title {
  font-size: 1.25rem;
  font-weight: 600;
  padding: 1rem;
  background: #eef2ff;
  color: #3730a3;
  border-radius: 12px 12px 0 0;
  margin-bottom: 0;
}

.form-grid {
  display: grid;
  grid-template-columns: 1fr;
  gap: 0;
}

@media (min-width: 768px) {
  .form-grid {
    grid-template-columns: 1fr 1fr;
    gap: 2rem;
  }
}

.form-field {
  display: flex;
  flex-direction: column;
  margin-bottom: 2rem;
}

.form-label {
  font-size: 1rem;
  font-weight: 500;
  color: #374151;
  margin-bottom: 0.5rem;
}

.p-inputnumber {
  border: none !important;
  padding: 0 !important;
  background: transparent !important;
}

.p-inputnumber input {
  min-height: 44px;
  font-size: 1rem;
  border-radius: 8px;
  border: 1.5px solid #d1d5db;
  padding: 0 12px;
  transition: border-color 0.2s;
  width: 100%;
  box-sizing: border-box;
  background: #fff;
}

.p-inputnumber input:focus {
  border-color: #6366f1;
  box-shadow: 0 0 0 2px rgba(99, 102, 241, 0.15);
}

.p-inputnumber.p-invalid input {
  border-color: #ef4444 !important;
}

.p-inputtext {
  min-height: 44px;
  font-size: 1rem;
  border-radius: 8px;
  border: 1.5px solid #d1d5db;
  padding: 0 12px;
  transition: border-color 0.2s;
  width: 100%;
  box-sizing: border-box;
  background: #fff;
}

.p-inputtext:focus {
  border-color: #6366f1;
  box-shadow: 0 0 0 2px rgba(99, 102, 241, 0.15);
}

.p-inputtext.p-invalid {
  border-color: #ef4444 !important;
}

.p-message {
  margin-top: 0.25rem;
  font-size: 0.95rem;
}

.form-actions {
  display: flex;
  gap: 1rem;
  margin-top: 1rem;
}
</style>