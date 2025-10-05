import { invoke } from '@tauri-apps/api/core';
import type {
    ApiResponse,
    RootInfo,
    ExtractProgress,
    ArchiveParams,
    ScanParams,
    DirectoryInfo,
    FileInfo
} from './types';

/**
 * 扫描根目录
 */
export async function scanRoot(params: ScanParams): Promise<ApiResponse<void>> {
    try {
        await invoke('scan_root_with_progress', params as unknown as Record<string, unknown>);
        return { success: true };
    } catch (error) {
        return { success: false, error: error as string };
    }
}

/**
 * 归档文件
 */
export async function archiveFiles(params: ArchiveParams): Promise<ApiResponse<string>> {
    try {
        const result = await invoke<string>('archive_files', params as unknown as Record<string, unknown>);
        return { success: true, data: result };
    } catch (error) {
        return { success: false, error: error as string };
    }
}

/**
 * 解档文件
 */
export async function extractArchive(
    rootId: number,
    targetPath: string,
    dbPath: string
): Promise<ApiResponse<ExtractProgress>> {
    try {
        const result = await invoke<ExtractProgress>('extract_archive', {
            rootId,
            targetPath,
            dbPath
        });
        return { success: true, data: result };
    } catch (error) {
        return { success: false, error: error as string };
    }
}

/**
 * 获取所有根目录
 */
export async function getAllRoots(dbPath: string): Promise<ApiResponse<RootInfo[]>> {
    try {
        const result = await invoke<RootInfo[]>('get_all_roots', { dbPath } as unknown as Record<string, unknown>);
        return { success: true, data: result };
    } catch (error) {
        return { success: false, error: error as string };
    }
}

/**
 * 获取子目录列表
 */
export async function getChildDirectories(dbPath: string, rootId: number, parentPath: string): Promise<ApiResponse<DirectoryInfo[]>> {
    try {
        const result = await invoke<DirectoryInfo[]>('get_child_directories', {
            dbPath,
            rootId,
            parentPath
        } as unknown as Record<string, unknown>);
        return { success: true, data: result };
    } catch (error) {
        return { success: false, error: error as string };
    }
}

/**
 * 根据根目录ID获取所有目录
 */
export async function getDirectoriesByRootId(dbPath: string, rootId: number): Promise<ApiResponse<DirectoryInfo[]>> {
    try {
        const result = await invoke<DirectoryInfo[]>('get_directories_by_root_id', {
            dbPath,
            rootId
        } as unknown as Record<string, unknown>);
        return { success: true, data: result };
    } catch (error) {
        return { success: false, error: error as string };
    }
}

/**
 * 根据目录ID获取目录信息
 */
export async function getDirectoryById(dbPath: string, directoryId: number): Promise<ApiResponse<DirectoryInfo>> {
    try {
        const result = await invoke<DirectoryInfo>('get_directory_by_id', {
            dbPath,
            directoryId
        } as unknown as Record<string, unknown>);
        return { success: true, data: result };
    } catch (error) {
        return { success: false, error: error as string };
    }
}

/**
 * 根据目录ID获取文件列表
 */
export async function getFilesByDirectoryId(dbPath: string, directoryId: number): Promise<ApiResponse<FileInfo[]>> {
    try {
        const result = await invoke<FileInfo[]>('get_files_by_directory_id', {
            dbPath,
            directoryId
        } as unknown as Record<string, unknown>);
        return { success: true, data: result };
    } catch (error) {
        return { success: false, error: error as string };
    }
}
