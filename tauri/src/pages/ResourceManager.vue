<script setup lang="ts">
import { ref, onMounted, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import Splitter from 'primevue/splitter'
import SplitterPanel from 'primevue/splitterpanel'
import Tree from 'primevue/tree'
import Card from 'primevue/card'
import Button from 'primevue/button'
import { useGlobalStore } from '../store/global'
import { open } from '@tauri-apps/plugin-dialog'
import type { TreeNode } from 'primevue/treenode'

// 定义文件/目录节点类型
interface CustomTreeNode extends TreeNode {
    key: string
    label: string
    data: string
    icon?: string
    children?: CustomTreeNode[]
    leaf?: boolean
}

const globalStore = useGlobalStore()

// 状态变量
const roots = ref<Array<{ id: number, root_name: string, root_path: string }>>([])
const selectedRoot = ref<{ id: number, root_name: string, root_path: string } | null>(null)
const fileTree = ref<TreeNode[]>([])
const loadingRoots = ref(false)
const loadingTree = ref(false)
const error = ref('')

// 选择数据库文件
const selectDatabaseFile = async () => {
    const selected = await open({
        filters: [{
            name: "SQLite Database",
            extensions: ["db", "sqlite", "sqlite3"]
        }]
    })

    if (selected) {
        globalStore.setDbPath(selected)
        loadRoots()
    }
}

// 加载所有根目录
const loadRoots = async () => {
    if (!globalStore.dbPath) {
        error.value = "请先选择数据库文件"
        return
    }

    loadingRoots.value = true
    error.value = ""

    try {
        const result = await invoke("get_all_roots", {
            dbPath: globalStore.dbPath
        })

        roots.value = result as Array<{ id: number, root_name: string, root_path: string }>

        // 如果有根目录，默认选择第一个
        if (roots.value.length > 0 && !selectedRoot.value) {
            selectedRoot.value = roots.value[0]
        }
    } catch (err: any) {
        error.value = `加载根目录失败：${err}`
        console.error("Load roots error:", err)
    } finally {
        loadingRoots.value = false
    }
}

// 当选中的根目录改变时，加载文件树
watch(() => selectedRoot.value, (newRoot) => {
    if (newRoot) {
        loadFileTree(newRoot.id)
    }
})

// 加载文件树结构
const loadFileTree = async (rootId: number) => {
    if (!globalStore.dbPath) return

    loadingTree.value = true
    try {
        // 获取根目录下的直接子目录
        const rootDirs = await invoke("get_child_directories", {
            dbPath: globalStore.dbPath,
            rootId: rootId,
            parentPath: ""
        }) as Array<{ id: number, directory_name: string, directory_path: string | null }>

        console.log("Root directories:", rootDirs);
        // 根据路径为空来获取根目录的目录 ID
        let rootDirectoryId = 0;
        try {
            // 获取根目录信息，查找路径为空的目录记录
            const rootDirInfo = await invoke("get_directories_by_root_id", {
                dbPath: globalStore.dbPath,
                rootId: rootId
            }) as Array<{ id: number, root_id: number, directory_name: string, directory_path: string | null }>

            // 查找路径为空的根目录记录
            const rootDir = rootDirInfo.find(dir =>
                dir.directory_path === null ||
                dir.directory_path === ""
            );

            if (rootDir) {
                rootDirectoryId = rootDir.id;
            }
            console.log("Root directory ID:", rootDirectoryId);
        } catch (err) {
            console.warn("无法获取根目录 ID，使用默认值 0:", err);
        }

        // 获取根目录下的文件 (使用根目录的目录 ID)
        const rootFiles = await invoke("get_files_by_directory_id", {
            dbPath: globalStore.dbPath,
            directoryId: rootDirectoryId
        }) as Array<{ id: number, file_name: string, directory_id: number }>

        console.log("Root directory files:", rootFiles);
        // 构建根节点树结构
        fileTree.value = buildInitialTree(rootDirs, rootFiles, rootId)
    } catch (err: any) {
        error.value = `加载文件树失败：${err}`
        console.error("Load file tree error:", err)
    } finally {
        loadingTree.value = false
    }
}
// 构建初始树结构
const buildInitialTree = (
    directories: Array<{ id: number, directory_name: string, directory_path: string | null }>,
    files: Array<{ id: number, file_name: string, directory_id: number }>,
    rootId: number
): TreeNode[] => {
    // 创建一个映射来存储所有节点，便于查找父节点
    const nodeMap = new Map<string, TreeNode>();
    const rootNodes: TreeNode[] = [];

    // 先处理所有目录节点
    directories.forEach(dir => {
        const node: TreeNode = {
            key: `dir_${dir.id}`,
            label: dir.directory_name,
            data: 'directory',
            icon: 'pi pi-folder',
            children: [],
            leaf: false
        };

        nodeMap.set(dir.directory_path || '', node);

        // 如果是根目录下的直接子目录（路径中不包含'/'或者为 null）
        if (!dir.directory_path || dir.directory_path.split('/').length === 1) {
            rootNodes.push(node);
        }
    });

    // // 建立目录节点之间的父子关系
    // directories.forEach(dir => {
    //     if (dir.directory_path) {
    //         // 查找父目录路径
    //         const pathParts = dir.directory_path.split('/');
    //         if (pathParts.length > 1) {
    //             pathParts.pop(); // 移除当前目录名
    //             const parentPath = pathParts.join('/');

    //             // 查找父节点
    //             const parentNode = nodeMap.get(parentPath);
    //             const currentNode = nodeMap.get(dir.directory_path);

    //             if (parentNode && currentNode) {
    //                 parentNode.children = parentNode.children || [];
    //                 parentNode.children.push(currentNode);
    //             }
    //         }
    //     }
    // });

    // 处理根目录下的文件节点
    files.forEach(file => {
        rootNodes.push({
            key: `file_${file.id}`,
            label: file.file_name,
            data: 'file',
            icon: 'pi pi-file',
            leaf: true
        });
    });

    return rootNodes;
}

// 懒加载节点的子节点
const onNodeExpand = async (node: TreeNode) => {
    if (!globalStore.dbPath || !selectedRoot.value) return

    // 从节点 key 中提取目录 ID
    const dirId = parseInt((node.key as string).split('_')[1])

    try {
        // 获取目录信息以获取路径
        const dirInfo = await invoke("get_directory_by_id", {
            dbPath: globalStore.dbPath,
            directoryId: dirId
        }) as { id: number, directory_name: string, directory_path: string | null }

        // 获取该目录的直接子目录
        const childDirs = await invoke("get_child_directories", {
            dbPath: globalStore.dbPath,
            rootId: selectedRoot.value.id,
            parentPath: dirInfo.directory_path || ''
        }) as Array<{ id: number, directory_name: string, directory_path: string | null }>
        console.log("Directory children:", childDirs)

        // 获取该目录下的文件
        const dirFiles = await invoke("get_files_by_directory_id", {
            dbPath: globalStore.dbPath,
            directoryId: dirId
        }) as Array<{ id: number, file_name: string, directory_id: number }>
        console.log("Directory files:", dirFiles)

        // 构建子节点
        const childNodes: TreeNode[] = []

        // 添加子目录
        childDirs.forEach(dir => {
            childNodes.push({
                key: `dir_${dir.id}`,
                label: dir.directory_name,
                data: 'directory',
                icon: 'pi pi-folder',
                children: [], // 空数组表示可以展开，实际子节点将按需加载
                leaf: false
            })
        })

        // 添加文件
        dirFiles.forEach(file => {
            childNodes.push({
                key: `file_${file.id}`,
                label: file.file_name,
                data: 'file',
                icon: 'pi pi-file',
                leaf: true
            })
        })

        // 更新节点的子节点
        node.children = childNodes
    } catch (err: any) {
        error.value = `加载子节点失败：${err}`
        console.error("Load child nodes error:", err)
    }
}
// 页面加载时尝试使用默认数据库路径
onMounted(() => {
    if (globalStore.dbPath) {
        loadRoots()
    }
})
</script>

<template>
    <div class="resource-manager">
        <Card>
            <template #title>
                <h2 class="text-xl font-semibold">资源管理</h2>
            </template>
            <template #content>
                <div class="flex flex-col md:flex-row gap-4 items-center mb-4">
                    <div class="flex-1">
                        <label for="dbPath" class="block text-sm font-medium mb-2">数据库文件</label>
                        <div class="flex gap-2">
                            <input id="dbPath" :value="globalStore.dbPath" type="text" placeholder="请选择数据库文件"
                                class="flex-1 p-2 border rounded" readonly />
                            <Button @click="selectDatabaseFile" label="浏览" />
                        </div>
                    </div>

                    <div class="pt-6">
                        <Button @click="loadRoots" :disabled="loadingRoots || !globalStore.dbPath"
                            :loading="loadingRoots" label="加载根目录" />
                    </div>
                </div>

                <div v-if="error" class="mb-4">
                    <Message severity="error">{{ error }}</Message>
                </div>
            </template>
        </Card>

        <Splitter style="height: calc(100vh - 250px)" class="mt-4">
            <!-- 左侧根目录列表 -->
            <SplitterPanel :size="30" :minSize="20" class="flex flex-col">
                <Card class="flex-1">
                    <template #title>
                        <h3 class="text-lg font-medium">根目录</h3>
                    </template>
                    <template #content>
                        <div class="root-list">
                            <div v-for="root in roots" :key="root.id" @click="selectedRoot = root"
                                class="p-3 border rounded mb-2 cursor-pointer transition-colors"
                                :class="selectedRoot?.id === root.id ? 'bg-primary text-white' : 'hover:bg-surface-200'">
                                <div class="font-medium">{{ root.root_name }}</div>
                                <div class="text-sm opacity-75 truncate">{{ root.root_path }}</div>
                            </div>

                            <div v-if="roots.length === 0 && !loadingRoots" class="text-center py-8 text-gray-500">
                                <p>暂无根目录数据</p>
                                <p class="text-sm mt-2">请选择数据库文件并点击"加载根目录"按钮</p>
                            </div>
                        </div>
                    </template>
                </Card>
            </SplitterPanel>

            <!-- 右侧文件树 -->
            <SplitterPanel :size="70" :minSize="50" class="flex flex-col">
                <Card class="flex-1">
                    <template #title>
                        <h3 class="text-lg font-medium">文件结构</h3>
                    </template>
                    <template #content>
                        <div v-if="selectedRoot" class="file-tree-container">
                            <Tree :value="fileTree" :loading="loadingTree" class="w-full" :expandedKeys="{}"
                                @node-expand="onNodeExpand" />
                        </div>

                        <div v-else class="text-center py-8 text-gray-500">
                            <p>请选择一个根目录查看文件结构</p>
                        </div>
                    </template>
                </Card>
            </SplitterPanel>
        </Splitter>
    </div>
</template>

<style scoped>
.root-list {
    max-height: calc(100vh - 300px);
    overflow-y: auto;
}

.file-tree-container {
    height: calc(100vh - 300px);
    overflow-y: auto;
}
</style>