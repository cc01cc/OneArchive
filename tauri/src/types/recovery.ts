export interface RecoveryConfig {
    dataShards: number;
    parityShards: number;
    storageDir: string;
    prefix: string;
    storeDataShards: boolean;
    dbPath: string;
    rootId: number | null;
}

export interface ArchiveFile {
    id: number;
    archive_name: string;
    archive_size: number;
}

export interface RootInfo {
    id: number;
    root_path: string;
    root_name: string;
}

export interface ArchiveGroup {
    source: ArchiveFile[];
    target: ArchiveFile[];
}

export interface RecoveryGenerationResult {
    group_id: string;
    data_shards_count: number;
    parity_shards_count: number;
    message: string;
}