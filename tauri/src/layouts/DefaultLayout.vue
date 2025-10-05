<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import type { MenuItem } from 'primevue/menuitem'
import debounce from 'lodash-es/debounce';
import Menubar from 'primevue/menubar';
import ProgressSpinner from 'primevue/progressspinner';
import { WebviewWindow } from '@tauri-apps/api/webviewWindow';
import { onMounted } from 'vue';
import { useGlobalStore } from '../store/global';
import { useAppConfig } from '../composables/useAppConfig';
import { useToast } from 'primevue/usetoast';

let isSettingOpened = ref(false);
const globalStore = useGlobalStore();
const { isLoading, loadConfig } = useAppConfig();
const toast = useToast();

const currentWorkspace = computed(() => globalStore.currentWorkspace);

onMounted(async () => {
    const result = await loadConfig();
    if (!result.success) {
        toast.add({
            severity: 'error',
            summary: '配置加载失败',
            detail: '无法加载应用配置，请检查设置',
            life: 5000
        });
        // 延迟打开设置窗口，给用户时间看到 toast
        setTimeout(() => {
            openSettings();
        }, 1000);
    } else if (result.needsSettings) {
        openSettings();
    }
});

const openSettings = async () => {

    if (isSettingOpened.value) {
        console.warn('设置窗口已经打开或正在打开中')
        return;
    }

    isSettingOpened.value = true;

    try {
        // 检查窗口是否已存在
        const existingWindow = await WebviewWindow.getByLabel('settings');

        if (existingWindow) {
            try {
                // 检查窗口是否可见
                const isVisible = await existingWindow.isVisible();

                if (isVisible) {
                    // 如果窗口已存在且可见，将其聚焦
                    await existingWindow.setFocus();
                    await existingWindow.show();
                    return;
                } else {
                    // 如果窗口存在但不可见，可能是已关闭但引用还在
                    try {
                        await existingWindow.close();
                    } catch (closeError) {
                        console.warn('关闭已存在窗口失败：', closeError);
                    }
                }
            } catch (error) {
                console.warn('检查现有窗口状态失败：', error);
            }
        }

        // 创建新窗口
        const newSettingsWindow = new WebviewWindow('settings', {
            url: '/config',
            title: '设置 - one-archive',
            width: 500,
            height: 400,
            resizable: false,
        });

        // 创建后手动聚焦
        newSettingsWindow.once('tauri://created', async () => {
            try {
                await newSettingsWindow.setFocus();
                console.log('设置窗口创建成功');
            } catch (error) {
                console.error('窗口初始化失败：', error);
            }
        });

        // 监听窗口关闭事件
        newSettingsWindow.once('tauri://close-requested', async () => {
            try {
                await newSettingsWindow.close();
            } catch (closeError) {
                console.error('关闭窗口失败：', closeError);
            } finally {
                isSettingOpened.value = false;
            }
        });

        // 监听窗口创建错误
        newSettingsWindow.once('tauri://error', (e: any) => {
            console.error('创建设置窗口失败：', e);

            const errorMessage = typeof e.payload === 'string' ? e.payload : String(e.payload);

            // 如果是因为窗口已存在，尝试获取并聚焦现有窗口
            if (errorMessage.includes('already exists')) {
                WebviewWindow.getByLabel('settings').then((win) => {
                    if (win) {
                        win.setFocus().catch(console.error);
                        win.show().catch(console.error);
                    }
                }).catch(console.error);
            }
            isSettingOpened.value = false;
        });
    } catch (error) {
        console.error('打开设置窗口失败：', error);
        isSettingOpened.value = false;
    }
};

// 将 mainWindow 改为一个 Promise，避免在 setup 阶段使用 await
const mainWindowPromise = WebviewWindow.getByLabel('main');
// TODO 禁用主窗口后，需要更加清晰的提示用户窗口被禁用，例如添加遮罩层，提示音，以及设置窗口聚焦和闪烁
watch(isSettingOpened, async () => {
    if (isSettingOpened.value) {
        console.log("设置窗口已打开，禁用主窗口");
        const mainWindow = await mainWindowPromise;
        mainWindow?.setEnabled(false)
    } else {
        console.log("设置窗口已关闭，启用主窗口");
        const mainWindow = await mainWindowPromise;
        mainWindow?.setEnabled(true);
        await mainWindow?.setFocus();
        await mainWindow?.show();
    }
})


const debouncedOpenSettings = debounce(openSettings, 300); // 防抖函数，延迟 300ms 执行

const router = useRouter()
const route = useRoute()

// 扩展 MenuItem 类型以包含 routeName 属性
interface ExtendedMenuItem extends MenuItem {
    routeName?: string
    root?: boolean
}

// 导航菜单项列表
const navItems = ref<ExtendedMenuItem[]>([
    {
        label: '工作区',
        icon: 'pi pi-home',
        routeName: 'Workspace',
        root: true,
        command: () => {
            router.push({ name: 'Workspace' })
        }
    },

    {
        label: '资源管理',
        icon: 'pi pi-database',
        routeName: 'ResourceManager',
        root: true,
        command: () => {
            router.push({ name: 'ResourceManager' })
        }
    },
    {
        label: '任务中心',
        icon: 'pi pi-server',
        routeName: 'TaskCenter',
        root: true,
        command: () => {
            router.push({ name: 'TaskCenter' })
        }
    }
])

// 根据当前路由确定活动按钮
const isActive = (routeName: string) => {
    console.log("route.name " + String(route.name) + "; routeName " + routeName)
    return route.name === routeName
}

// 包装 item.command 以解决类型不匹配问题
const handleItemClick = (item: ExtendedMenuItem) => {
    if (item.command) {
        item.command({ originalEvent: new Event('click'), item } as any)
    }
}

</script>

<template>
    <div v-if="isLoading" class="h-screen flex items-center justify-center">
        <ProgressSpinner />
    </div>
    <div v-else class="h-screen flex flex-col">

        <header class="flex-shrink-0">
            <!-- // TODO 优化导航栏折叠后，宽屏显示效果 600px-900px -->
            <Menubar :model="navItems" class="p-4 mt-2" style="border-radius: 3rem">
                <template #start>
                    <span>One Archive</span>
                </template>
                <template #item="{ item }">
                    <a v-ripple :class="[
                        'flex items-center cursor-pointer px-4 py-2 overflow-hidden relative font-semibold rounded-md transition-colors duration-200 ',
                        {
                            'bg-primary text-primary-contrast': isActive(item.routeName || ''),
                            'text-surface-900 hover:bg-surface-200': !isActive(item.routeName || '')
                        }
                    ]" @click="handleItemClick(item)">
                        <i v-if="item.icon" :class="item.icon" class="mr-2"></i>
                        <span>{{ item.label }}</span>
                        <i v-if="item.items" class="pi pi-angle-down ml-2"></i>
                    </a>
                </template>
                <template #end>
                    <div class="flex items-center">
                        <span v-if="currentWorkspace"
                            class="workspace-path text-sm text-surface-600 mr-3 max-w-40 sm:max-w-60 md:max-w-100 truncate"
                            :title="currentWorkspace">
                            {{ currentWorkspace }}
                        </span>
                        <Button icon="pi pi-cog" severity="secondary" @click="debouncedOpenSettings" rounded
                            aria-label="设置" />
                    </div>
                </template>
            </Menubar>
        </header>

        <main class="flex-1 overflow-hidden p-4">
            <div class="debug-info text-xs p-2">
                当前路由：{{ route.name }} | 路径：{{ route.path }}
            </div>
            <div class="h-full overflow-y-auto">
                <router-view />
            </div>
        </main>

        <footer class="flex-shrink-0 p-4 text-center text-sm text-gray-600 border-t">
            One Archive &copy; {{ new Date().getFullYear() }}
        </footer>
    </div>
</template>
<style scoped></style>
