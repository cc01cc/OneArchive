import { defineStore } from 'pinia'

export const useGlobalStore = defineStore('global', {
  state: () => ({
    isConfigDialogVisible: false,
    currentWorkspace: null as string | null,
    dbPath: '' as string,
    lastRoot: null as number | null  // 添加对最后选择根目录的跟踪
  }),
  actions: {
    toggleConfigDialog() {
      this.isConfigDialogVisible = !this.isConfigDialogVisible
    },
    setWorkspace(workspace: string) {
      this.currentWorkspace = workspace
    },
    setDbPath(path: string) {
      this.dbPath = path
    },
    setLastRoot(rootId: number) {  // 添加设置最后根目录的方法
      this.lastRoot = rootId
    }
  }
})