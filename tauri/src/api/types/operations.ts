/// 归档文件信息
export interface ArchiveFile {
    id: number;
    archiveName: string;
    archiveSize: number;
}

/// 根目录信息
export interface RootInfo {
    id: number;
    rootPath: string;
    rootName: string;
}

/// 扫描进度信息
export interface ScanProgress {
    processed: number;
    total: number;
    message: string;
    progress: number;
}

/// 归档进度信息
export interface ArchiveProgress {
    totalFiles: number;
    processedFiles: number;
    processedBytes: number;
    totalBytes: number;
    currentFile?: string;
    completed: boolean;
    error?: string;
}

/// 解档任务参数
export interface ExtractTask {
    rootId: number;
    targetPath: string;
    overwrite: boolean;
}

/// 解档进度信息
export interface ExtractProgress {
    totalFiles: number;
    processedFiles: number;
    currentFile?: string;
    completed: boolean;
    error?: string;
}

/// 归档参数
export interface ArchiveParams {
    rootDir: string;
    archiveDir: string;
    archivePrefix: string;
    dbPath: string;
    archiveLimitSize: number;
}

/// 扫描参数
export interface ScanParams {
    rootPath: string;
    dbPath: string;
}

/// 目录信息
export interface DirectoryInfo {
    id: number;
    directoryName: string;
    directoryPath: string | null;
}

/// 文件信息
export interface FileInfo {
    id: number;
    fileName: string;
    directoryId: number;
}
