/// 工作区信息
export interface Workspace {
    name: string;
    path: string;
}

/// 最后使用设置
export interface LastSettings {
    lastWorkspace: string;
    lastRoot: number | null;
    lastDbPath: string | null;
    windowWidth: number;
    windowHeight: number;
    windowX: number;
    windowY: number;
}

/// 应用配置
export interface AppConfig {
    workspace: Workspace[];
    lastSettings: LastSettings;
}

/// 窗口位置参数
export interface WindowPosition {
    x: number;
    y: number;
}
