import { ref, onMounted } from 'vue';
import { getCurrentWindow } from '@tauri-apps/api/window';
import debounce from 'lodash-es/debounce';
import { useGlobalStore } from '../store/global';
import { loadAppConfig, saveWindowPosition, setWindowPosition } from '../api';
import type { AppConfig } from '../api';

export function useAppConfig() {
    const isLoading = ref(true);
    const appConfig = ref<AppConfig | null>(null);
    const isLoaded = ref(false); // 确保幂等性
    const globalStore = useGlobalStore();

    // 窗口位置保存的防抖函数
    const debouncedSaveWindowPosition = debounce(async () => {
        try {
            await saveWindowPosition();
        } catch (error) {
            console.warn('保存窗口位置失败：', error);
        }
    }, 500);

    // 设置窗口事件监听
    const setupWindowListeners = async () => {
        try {
            const currentWindow = getCurrentWindow();

            // 监听窗口移动事件
            currentWindow.listen('moved', () => {
                debouncedSaveWindowPosition();
            });

            // 监听窗口关闭事件，确保最终位置被保存
            currentWindow.listen('close-requested', async () => {
                try {
                    await saveWindowPosition();
                } catch (error) {
                    console.warn('关闭时保存窗口位置失败：', error);
                }
            });
        } catch (error) {
            console.warn('设置窗口监听器失败：', error);
        }
    };

    const loadConfig = async (): Promise<{ success: boolean; needsSettings?: boolean }> => {
        if (isLoaded.value) {
            return { success: true };
        }

        try {
            isLoading.value = true;

            // 调用后端接口获取应用配置
            const result = await loadAppConfig();
            if (!result.success || !result.data) {
                throw new Error(result.error || '获取配置失败');
            }
            const config: AppConfig = result.data;
            appConfig.value = config;

            // 如果工作区列表为空，返回需要设置的标志
            if (!config.workspace || config.workspace.length === 0) {
                return { success: true, needsSettings: true };
            }

            // 恢复窗口位置
            if (config.lastSettings.windowX !== undefined && config.lastSettings.windowY !== undefined) {
                try {
                    await setWindowPosition({
                        x: config.lastSettings.windowX,
                        y: config.lastSettings.windowY
                    });
                } catch (error) {
                    console.warn('恢复窗口位置失败：', error);
                }
            }

            // 设置窗口事件监听
            await setupWindowListeners();

            // 加载应用设置到全局 store
            if (config.lastSettings.lastWorkspace) {
                globalStore.setWorkspace(config.lastSettings.lastWorkspace);
            }
            if (config.lastSettings.lastDbPath) {
                globalStore.setDbPath(config.lastSettings.lastDbPath);
            }

            isLoaded.value = true;
            return { success: true };
        } catch (error) {
            console.error('获取应用配置失败：', error);
            return { success: false };
        } finally {
            isLoading.value = false;
        }
    };

    onMounted(async () => {
        await loadConfig();
    });

    return {
        isLoading,
        appConfig,
        loadConfig
    };
}
