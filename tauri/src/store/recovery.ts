import { defineStore } from 'pinia'
import type { RecoveryConfig, ArchiveFile } from '../types/recovery'
import { useGlobalStore } from './global'

export const useRecoveryStore = defineStore('recovery', {
    state: () => ({
        // 灾备配置
        config: {
            dataShards: 3,
            parityShards: 2,
            storageDir: "",
            prefix: "recovery",
            storeDataShards: false,
            dbPath: "",
            rootId: null,
        } as RecoveryConfig,

        // 归档文件分组
        archiveGroups: [[]] as ArchiveFile[][],

        // 可用的归档文件
        availableArchives: [] as ArchiveFile[],

        // 状态变量
        isGenerating: false,
        progress: 0,
        statusMessage: "准备就绪",
        errors: {} as Record<string, string>,
        resultMessage: "",
        groupId: "",
    }),

    getters: {
        // 获取配置
        getConfig: (state) => state.config,

        // 获取归档文件分组
        getArchiveGroups: (state) => state.archiveGroups,

        // 获取状态变量
        isProcessing: (state) => state.isGenerating || state.progress > 0,
    },

    actions: {
        // 初始化配置
        initializeConfig() {
            const globalStore = useGlobalStore()
            this.config.dbPath = globalStore.dbPath || ""
            this.config.rootId = globalStore.lastRoot || null
            this.config.storageDir = globalStore.currentWorkspace || ""
        },

        // 更新配置
        updateConfig(newConfig: Partial<RecoveryConfig>) {
            this.config = { ...this.config, ...newConfig }
        },

        // 更新特定配置字段
        updateConfigField<K extends keyof RecoveryConfig>(field: K, value: RecoveryConfig[K]) {
            this.config[field] = value
        },

        // 更新归档文件分组
        updateArchiveGroups(groups: ArchiveFile[][]) {
            this.archiveGroups = groups
        },

        // 添加分组
        addGroup() {
            this.archiveGroups.push([])
        },

        // 删除分组
        removeGroup(index: number) {
            this.archiveGroups.splice(index, 1)
        },

        // 更新特定分组
        updateGroup(index: number, group: ArchiveFile[]) {
            this.archiveGroups[index] = group
        },

        // 重置表单
        resetForm() {
            const globalStore = useGlobalStore()
            this.config = {
                dataShards: 3,
                parityShards: 2,
                storageDir: "",
                prefix: "recovery",
                storeDataShards: false,
                dbPath: globalStore.dbPath || "",
                rootId: globalStore.lastRoot || null,
            }
            this.archiveGroups = [[]]
            this.errors = {}
            this.progress = 0
            this.statusMessage = "准备就绪"
            this.resultMessage = ""
            this.groupId = ""
            this.availableArchives = []
        },

        // 设置状态
        setGeneratingStatus(status: boolean) {
            this.isGenerating = status
        },

        // 更新进度
        updateProgress(progress: number, message: string) {
            this.progress = progress
            this.statusMessage = message
        },

        // 设置结果
        setResult(message: string, groupId: string = "") {
            this.resultMessage = message
            this.groupId = groupId
        },

        // 设置错误
        setErrors(errors: Record<string, string>) {
            this.errors = errors
        },

        // 设置可用归档文件
        setAvailableArchives(archives: ArchiveFile[]) {
            this.availableArchives = archives
        }
    }
})