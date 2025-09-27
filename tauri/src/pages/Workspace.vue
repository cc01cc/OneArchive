<template>
    <Splitter class="mb-6">
        <!-- 左侧工作区列表 -->
        <SplitterPanel :size="30" :minSize="20" class="flex flex-col">
            <div class="p-4 border-b border-surface-200 ">
                <div class="flex justify-between items-center mb-4">
                    <h2 class="text-xl font-semibold">工作区列表</h2>
                    <div class="flex gap-1">
                        <Button icon="pi pi-plus" rounded text @click="addWorkspace" />
                        <Button icon="pi pi-file-import" rounded text @click="importWorkspace" />
                    </div>
                </div>
                <div class="relative">
                    <InputText v-model="searchTerm" placeholder="搜索工作区..." class="w-full p-inputtext-sm" />
                    <i class="pi pi-search absolute top-1/2 right-3 -mt-2 text-surface-500"></i>
                </div>
            </div>

            <div class="flex-1 overflow-y-auto">
                <Menu :model="workspaceMenuItems" class="w-full border-0 p-0">
                    <template #item="{ item, props }">
                        <a v-ripple
                            class="flex align-items-center px-3 py-2 cursor-pointer transition-colors transition-duration-200"
                            v-bind="props.action" @click="selectWorkspace(item.workspace)" :class="selectedWorkspace?.key === item.workspace.key ?
                                'bg-primary text-primary-contrast font-medium rounded-md m-1' :
                                'hover:bg-surface-200 text-surface-700 rounded-md m-1'">
                            <i class="pi pi-folder mr-2" :class="selectedWorkspace?.key === item.workspace.key ?
                                'text-primary-contrast' : 'text-orange-400'"></i>
                            <span class="whitespace-nowrap overflow-hidden text-ellipsis" :class="selectedWorkspace?.key === item.workspace.key ?
                                'text-primary-contrast' : ''">{{ item.label }}</span>
                            <Button icon="pi pi-times" rounded text size="small" class="ml-auto"
                                @click.stop="removeWorkspace(item.workspace)" v-tooltip="'从配置中移除该工作区'" />
                        </a>
                    </template>
                </Menu>
            </div>
        </SplitterPanel>

        <!-- 右侧工作区详细配置 -->
        <SplitterPanel :size="70" :minSize="50" class="flex flex-col min-h-150">
            <Panel v-if="selectedWorkspace" header="工作区配置" class="h-full">
                <template #icons>
                    <Button label="激活" :disabled="isWorkspaceActive" @click="activateWorkspace(selectedWorkspace)" />
                    <Button icon="pi pi-save" text @click="saveWorkspace" />
                    <Button icon="pi pi-trash" text @click="removeWorkspace(selectedWorkspace)" v-tooltip="'移除'" />
                </template>

                <div class="p-4">
                    <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                        <div class="field md:col-span-2">
                            <label for="workspaceName" class="block text-sm font-medium mb-2">工作区名称 *</label>
                            <InputText id="workspaceName" v-model="selectedWorkspace.name" class="w-full"
                                :class="{ 'p-invalid': errors.name }" />
                            <small v-if="errors.name" class="p-error">{{ errors.name }}</small>
                        </div>

                        <div class="field md:col-span-2">
                            <label for="workspacePath" class="block text-sm font-medium mb-2">工作区路径 *</label>
                            <div class="flex gap-2">
                                <InputText id="workspacePath" v-model="selectedWorkspace.path" class="flex-1"
                                    :class="{ 'p-invalid': errors.path }" readonly />
                                <Button icon="pi pi-folder" @click="selectWorkspacePath" />
                            </div>
                            <small v-if="errors.path" class="p-error">{{ errors.path }}</small>
                        </div>

                        <div class="field md:col-span-2">
                            <label class="block text-sm font-medium mb-2">数据库配置</label>
                            <div class="border rounded-lg p-4">
                                <div class="field">
                                    <label for="dbPath" class="block text-sm font-medium mb-2">数据库路径</label>
                                    <div class="flex gap-2">
                                        <InputText id="dbPath" v-model="selectedWorkspace.dbPath" class="flex-1"
                                            readonly />
                                        <Button icon="pi pi-database" @click="selectDbPath" />
                                    </div>
                                </div>

                                <div class="field mt-4">
                                    <div class="flex items-center">
                                        <Checkbox id="autoCreateDb" v-model="selectedWorkspace.autoCreateDb"
                                            :binary="true" class="mr-2" />
                                        <label for="autoCreateDb" class="text-sm">自动创建数据库</label>
                                    </div>
                                </div>
                            </div>
                        </div>

                        <div class="field md:col-span-2">
                            <label class="block text-sm font-medium mb-2">扫描配置</label>
                            <div class="border rounded-lg p-4">
                                <div class="field">
                                    <div class="flex items-center">
                                        <Checkbox id="recursiveScan" v-model="selectedWorkspace.recursiveScan"
                                            :binary="true" class="mr-2" />
                                        <label for="recursiveScan" class="text-sm">递归扫描子目录</label>
                                    </div>
                                </div>

                                <div class="field mt-4">
                                    <div class="flex items-center">
                                        <Checkbox id="followSymlinks" v-model="selectedWorkspace.followSymlinks"
                                            :binary="true" class="mr-2" />
                                        <label for="followSymlinks" class="text-sm">跟随符号链接</label>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>

                <template #footer>
                    <div class="flex justify-end gap-2">
                        <Button label="重置" severity="secondary" @click="resetForm" />
                        <Button label="保存" @click="saveWorkspace" />
                    </div>
                </template>
            </Panel>

            <div v-else class="flex items-center justify-center h-full">
                <div class="text-center p-8">
                    <i class="pi pi-info-circle text-5xl text-surface-400 mb-4"></i>
                    <p class="text-lg text-surface-500">请选择一个工作区或创建新的工作区</p>
                </div>
            </div>
        </SplitterPanel>
    </Splitter>
    <ConfirmDialog></ConfirmDialog>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import Splitter from 'primevue/splitter'
import SplitterPanel from 'primevue/splitterpanel'
import Menu from 'primevue/menu'
import Panel from 'primevue/panel'
import InputText from 'primevue/inputtext'
import Button from 'primevue/button'
import Checkbox from 'primevue/checkbox'
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { useToast } from 'primevue/usetoast';
import ConfirmDialog from 'primevue/confirmdialog';
import { useGlobalStore } from '../store/global';
import { useConfirm } from 'primevue/useconfirm';

// 定义应用设置接口
interface AppSettings {
    last_workspace: string | null;
    last_root: number | null;
    last_db_path: string | null;
    window_width: number | null;
    window_height: number | null;
    window_x: number | null;
    window_y: number | null;
}

// 定义工作区配置接口
interface WorkspaceConfig {
    db_path?: string;
    auto_create_db?: boolean;
    recursive_scan?: boolean;
    follow_symlinks?: boolean;
}

// 定义工作区接口
interface WorkspaceDetail {
    key: string
    name: string
    path: string
    dbPath: string
    autoCreateDb: boolean
    recursiveScan: boolean
    followSymlinks: boolean
}
// 获取全局状态存储实例
const globalStore = useGlobalStore();

// 搜索词
const searchTerm = ref('')

// 所有工作区（基础信息）
const basicWorkspaces = ref<Array<{ key: string, name: string, path: string }>>([])

// 所有工作区（详细信息）
const workspaces = ref<WorkspaceDetail[]>([])

// 选中的工作区
const selectedWorkspace = ref<WorkspaceDetail | null>(null)
// 获取 Toast 实例
const toast = useToast();
// 获取 Confirm 实例
const confirm = useConfirm();
// 检查当前工作区是否已激活
const isWorkspaceActive = computed(() => {
    return globalStore.currentWorkspace === selectedWorkspace.value?.path;
});

// 激活工作区
const activateWorkspace = async (workspace: WorkspaceDetail) => {
    if (workspace) {
        globalStore.setWorkspace(workspace.path);
        globalStore.setDbPath(workspace.dbPath);

        try {
            // 保存当前工作区到应用设置
            await invoke('save_app_settings', {
                settings: {
                    last_workspace: workspace.path,
                    last_root: null, // 根目录将在使用时设置
                    last_db_path: workspace.dbPath,
                    window_width: null,
                    window_height: null,
                    window_x: null,
                    window_y: null
                }
            });

            toast.add({ severity: 'success', summary: '成功', detail: '工作区已激活并已保存设置', life: 3000 });
        } catch (error) {
            console.error('保存应用设置失败：', error);
            toast.add({ severity: 'error', summary: '错误', detail: '工作区已激活但保存设置失败', life: 3000 });
        }
    }
};

// 工作区菜单项
const workspaceMenuItems = computed(() => {
    return basicWorkspaces.value
        .filter(workspace =>
            workspace.name.toLowerCase().includes(searchTerm.value.toLowerCase())
        )
        .map(workspace => ({
            label: workspace.name,
            icon: 'pi pi-folder',
            workspace: workspace
        }))
})

// 错误信息
const errors = ref<{ [key: string]: string }>({})

// 选择工作区
const selectWorkspace = async (workspace: { key: string, name: string, path: string }) => {
    // 查找是否已加载详细信息
    let detailedWorkspace = workspaces.value.find(w => w.key === workspace.key);

    // 如果没有加载详细信息，则从工作区配置文件中加载
    if (!detailedWorkspace) {
        try {
            // 调用后端接口加载工作区详细配置
            const config = await invoke<WorkspaceConfig>('load_workspace_config', { workspacePath: workspace.path });

            detailedWorkspace = {
                key: workspace.key,
                name: workspace.name,
                path: workspace.path,
                dbPath: config.db_path || `${workspace.path}/default.sqlite`,
                autoCreateDb: config.auto_create_db !== undefined ? config.auto_create_db : true,
                recursiveScan: config.recursive_scan !== undefined ? config.recursive_scan : true,
                followSymlinks: config.follow_symlinks !== undefined ? config.follow_symlinks : false
            };

            // 将详细信息添加到工作区列表中
            workspaces.value.push(detailedWorkspace);
        } catch (error) {
            console.error('加载工作区配置失败：', error);
            // 使用默认配置
            detailedWorkspace = {
                key: workspace.key,
                name: workspace.name,
                path: workspace.path,
                dbPath: `${workspace.path}/default.sqlite`,
                autoCreateDb: true,
                recursiveScan: true,
                followSymlinks: false
            };
            workspaces.value.push(detailedWorkspace);
        }
    }

    selectedWorkspace.value = { ...detailedWorkspace };
}

// 添加工作区
const addWorkspace = async () => {
    try {
        // 使用 Tauri 对话框插件选择目录
        const selected = await open({
            directory: true,
            multiple: false,
            title: "选择工作区目录"
        });

        if (selected) {
            // 弹出输入工作区名称的提示框
            // 这里可以使用更复杂的表单，但为了简化，我们使用工作区目录名作为名称
            const workspaceName = prompt("请输入工作区名称：", selected.split(/[\\/]/).pop() || "新工作区");

            if (workspaceName) {
                // 调用后端创建工作区功能
                await invoke('create_workspace', {
                    params: {
                        name: workspaceName,
                        path: selected
                    }
                });

                // 重新加载工作区列表
                await loadWorkspaces();

                // 显示成功消息
                toast.add({ severity: 'success', summary: '成功', detail: '工作区创建成功', life: 3000 });
            }
        }
    } catch (error: any) {
        console.error('创建工作区失败：', error);
        // 使用 Toast 显示错误消息
        toast.add({ severity: 'error', summary: '错误', detail: '创建工作区失败：' + (error.message || error.toString()), life: 5000 });
    }
}

// 导入工作区
const importWorkspace = async () => {
    try {
        // 使用 Tauri 对话框插件选择目录
        const selected = await open({
            directory: true,
            multiple: false,
            title: "选择要导入的工作区目录"
        });

        if (selected) {
            // 调用后端导入工作区功能
            const importedWorkspace = await invoke<{ name: string, path: string }>('import_workspace', {
                params: { path: selected }
            });

            // 更新工作区列表
            await loadWorkspaces();

            // 显示成功消息
            toast.add({ severity: 'success', summary: '成功', detail: '工作区导入成功', life: 3000 });
        }
    } catch (error: any) {
        console.error('导入工作区失败：', error);
        // 使用 Toast 显示错误消息
        toast.add({ severity: 'error', summary: '错误', detail: '导入工作区失败：' + (error.message || error.toString()), life: 5000 });
    }
}

// 移除工作区（从配置中移除）
const removeWorkspace = (workspace: { key: string, name: string, path: string }) => {
    confirm.require({
        message: `确定要从配置中移除工作区 "${workspace.name}" 吗？此操作不会删除实际的工作区目录，但会将其从列表中移除。`,
        header: '确认移除',
        icon: 'pi pi-exclamation-triangle',
        accept: async () => {
            try {
                // 调用后端移除工作区功能
                await invoke('remove_workspace', {
                    params: { path: workspace.path }
                });

                // 如果移除的是当前选中的工作区，则取消选中
                if (selectedWorkspace.value && selectedWorkspace.value.path === workspace.path) {
                    selectedWorkspace.value = null;
                }

                // 重新加载工作区列表
                await loadWorkspaces();

                // 显示成功消息
                toast.add({ severity: 'success', summary: '成功', detail: '工作区移除成功', life: 3000 });
            } catch (error: any) {
                console.error('移除工作区失败：', error);
                // 使用 Toast 显示错误消息
                toast.add({ severity: 'error', summary: '错误', detail: '移除工作区失败：' + (error.message || error.toString()), life: 5000 });
            }
        }
    });
}

// 保存工作区
const saveWorkspace = () => {
    // 验证表单
    errors.value = {}

    if (!selectedWorkspace.value?.name) {
        errors.value.name = '工作区名称不能为空'
    }

    if (!selectedWorkspace.value?.path) {
        errors.value.path = '工作区路径不能为空'
    }

    if (Object.keys(errors.value).length > 0) {
        return
    }

    // 更新工作区列表中的对应项
    const index = workspaces.value.findIndex(w => w.key === selectedWorkspace.value?.key)
    if (index !== -1 && selectedWorkspace.value) {
        workspaces.value[index] = { ...selectedWorkspace.value }
    }

    // 保存逻辑
    console.log('保存工作区：', selectedWorkspace.value)
    // 这里应该调用后端 API 保存工作区配置
}

// 删除工作区
// TODO 需要二次确认
const deleteWorkspace = () => {
    if (!selectedWorkspace.value) return

    const index = workspaces.value.findIndex(w => w.key === selectedWorkspace.value?.key)
    if (index !== -1) {
        workspaces.value.splice(index, 1)
        selectedWorkspace.value = null
    }
}

// 重置表单
const resetForm = () => {
    if (selectedWorkspace.value) {
        const original = workspaces.value.find(w => w.key === selectedWorkspace.value?.key)
        if (original) {
            selectedWorkspace.value = { ...original }
        }
    }
}

// 选择工作区路径
const selectWorkspacePath = async () => {
    // 这里应该调用 Tauri 的文件选择对话框
    console.log('选择工作区路径')
}

// 选择数据库路径
const selectDbPath = async () => {
    // 这里应该调用 Tauri 的文件选择对话框
    console.log('选择数据库路径')
}

// 获取工作区列表
const loadWorkspaces = async () => {
    try {
        // 调用后端 API 获取工作区列表
        const result = await invoke<{ name: string, path: string }[]>('get_config_workspace')
        basicWorkspaces.value = result.map((w: { name: string, path: string }, index: number) => ({
            key: `${index}`,
            ...w
        }));

        console.log('加载工作区列表')
    } catch (error) {
        console.error('加载工作区列表失败：', error)
    }
}

onMounted(() => {
    loadWorkspaces()
})
</script>