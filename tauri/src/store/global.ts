import { defineStore } from 'pinia'

export const useGlobalStore = defineStore('global', {
  state: () => ({
    isConfigDialogVisible: false,
    currentWorkspace: null as string | null,
    dbPath: '' as string
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
    }
  }
})