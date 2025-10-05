import { invoke } from '@tauri-apps/api/core';
import type { ApiResponse, AppConfig, WindowPosition } from './types';

/**
 * 加载应用配置
 */
export async function loadAppConfig(): Promise<ApiResponse<AppConfig>> {
    try {
        const result = await invoke<AppConfig>('load_app_config_command');
        return { success: true, data: result };
    } catch (error) {
        return { success: false, error: error as string };
    }
}

/**
 * 保存窗口位置
 */
export async function saveWindowPosition(): Promise<ApiResponse<void>> {
    try {
        await invoke('save_window_position');
        return { success: true };
    } catch (error) {
        return { success: false, error: error as string };
    }
}

/**
 * 设置窗口位置
 */
export async function setWindowPosition(params: WindowPosition): Promise<ApiResponse<void>> {
    try {
        await invoke('set_window_position', params as unknown as Record<string, unknown>);
        return { success: true };
    } catch (error) {
        return { success: false, error: error as string };
    }
}
