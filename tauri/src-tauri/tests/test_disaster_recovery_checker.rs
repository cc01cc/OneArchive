use log::info;

use one_archive_lib::mod_disaster_recovery::service_disaster_recovery::service_recovery_planner::{RecoveryPlanner, DefaultRecoveryPlanner};
use one_archive_lib::mod_disaster_recovery::core_disaster_recovery::model_disaster_recovery::RecoveryConfig;
mod common;
use common::*;
mod disaster_recovery_common;
use disaster_recovery_common::*;

/// 测试灾备恢复分析器 - 检查归档文件是否满足恢复条件
#[test]
fn test_recovery_analyzer_check_recovery_conditions() -> Result<(), Box<dyn std::error::Error>> {
    init_test_env_in_file();
    info!("启动灾备恢复分析器检查测试...");

    let env = TestEnvironment::new()?;
    let database = &env.database;

    // 创建测试归档文件并插入数据库
    let archive_ids = create_test_archives_and_insert_to_db(&env.source_dir, &database)?;

    // 创建灾备组
    let config = RecoveryConfig {
        data_shards: 3,
        parity_shards: 2,
        storage_dir: env.archive_dir.to_string_lossy().into_owned(),
        prefix: "check_test".to_string(),
        store_data_shards: false,
    };
    let encoder = one_archive_lib::mod_disaster_recovery::service_disaster_recovery::service_encoder::GroupEncoder::new(config);
    let group_id =
        encoder.create_recovery_group_and_save_with_archives(&database, archive_ids.clone())?;

    // 测试获取灾备组信息
    let recovery_planner = DefaultRecoveryPlanner::new();
    let group_info = recovery_planner.get_recovery_group_by_archive_id(archive_ids[0], &database)?;
    assert_eq!(group_info.group_info.group_id, group_id);
    assert_eq!(group_info.data_shards.len(), 3);
    assert_eq!(group_info.parity_shards.len(), 2);

    // 验证灾备组信息正确性
    assert_eq!(group_info.group_info.num_data_shards, 3);
    assert_eq!(group_info.group_info.num_parity_shards, 2);
    assert!(!group_info.group_info.data_shards_stored);

    info!("灾备恢复分析器检查测试通过！");
    Ok(())
}

/// 测试灾备恢复分析器 - 验证数据分片完整性
#[test]
fn test_recovery_analyzer_verify_data_shards_integrity() -> Result<(), Box<dyn std::error::Error>> {
    init_test_env_in_file();
    info!("启动灾备恢复分析器数据分片完整性验证测试...");

    let env = TestEnvironment::new()?;
    let database = &env.database;

    // 创建测试归档文件并插入数据库
    let archive_ids = create_test_archives_and_insert_to_db(&env.source_dir, &database)?;

    // 创建灾备组（启用数据分片存储）
    let config = RecoveryConfig {
        data_shards: 3,
        parity_shards: 2,
        storage_dir: env.archive_dir.to_string_lossy().into_owned(),
        prefix: "integrity_test".to_string(),
        store_data_shards: true,
    };
    let encoder = one_archive_lib::mod_disaster_recovery::service_disaster_recovery::service_encoder::GroupEncoder::new(config);
    let group_id =
        encoder.create_recovery_group_and_save_with_archives(&database, archive_ids.clone())?;

    // 获取灾备组信息
    let recovery_planner = DefaultRecoveryPlanner::new();
    let group_info = recovery_planner.get_recovery_group_by_archive_id(archive_ids[0], &database)?;
    assert_eq!(group_info.group_info.group_id, group_id);
    assert_eq!(group_info.data_shards.len(), 3);
    assert_eq!(group_info.parity_shards.len(), 2);
    assert!(group_info.group_info.data_shards_stored);

    // 验证数据分片文件存在且哈希正确（通过 assert_recovery_group_and_shards 函数）
    assert_recovery_group_and_shards(&database, group_id, 3, 2)?;

    info!("灾备恢复分析器数据分片完整性验证测试通过！");
    Ok(())
}

/// 测试灾备恢复分析器 - 检查归档文件状态
#[test]
fn test_recovery_analyzer_check_archive_status() -> Result<(), Box<dyn std::error::Error>> {
    init_test_env_in_file();
    info!("启动灾备恢复分析器归档文件状态检查测试...");

    let env = TestEnvironment::new()?;
    let database = &env.database;

    // 创建测试归档文件并插入数据库
    let archive_ids = create_test_archives_and_insert_to_db(&env.source_dir, &database)?;

    // 创建灾备组
    let config = RecoveryConfig {
        data_shards: 3,
        parity_shards: 2,
        storage_dir: env.archive_dir.to_string_lossy().into_owned(),
        prefix: "status_test".to_string(),
        store_data_shards: false,
    };
    let encoder = one_archive_lib::mod_disaster_recovery::service_disaster_recovery::service_encoder::GroupEncoder::new(config);
    let group_id =
        encoder.create_recovery_group_and_save_with_archives(&database, archive_ids.clone())?;

    // 获取灾备组信息
    let recovery_planner = DefaultRecoveryPlanner::new();
    let group_info = recovery_planner.get_recovery_group_by_archive_id(archive_ids[0], &database)?;

    // 验证所有归档文件都在灾备组中
    for archive_id in &archive_ids {
        let found_group = DefaultRecoveryPlanner::new().get_recovery_group_by_archive_id(*archive_id, &database)?;
        assert_eq!(found_group.group_info.group_id, group_id);
    }

    // 验证分片数量匹配
    assert_eq!(group_info.data_shards.len(), archive_ids.len());
    assert_eq!(group_info.parity_shards.len(), 2);

    info!("灾备恢复分析器归档文件状态检查测试通过！");
    Ok(())
}

/// 测试灾备恢复分析器 - 验证校验分片完整性
#[test]
fn test_recovery_analyzer_verify_parity_shards_integrity() -> Result<(), Box<dyn std::error::Error>> {
    init_test_env_in_file();
    info!("启动灾备恢复分析器校验分片完整性验证测试...");

    let env = TestEnvironment::new()?;
    let database = &env.database;

    // 创建测试归档文件并插入数据库
    let archive_ids = create_test_archives_and_insert_to_db(&env.source_dir, &database)?;

    // 创建灾备组
    let config = RecoveryConfig {
        data_shards: 3,
        parity_shards: 2,
        storage_dir: env.archive_dir.to_string_lossy().into_owned(),
        prefix: "parity_test".to_string(),
        store_data_shards: false,
    };
    let encoder = one_archive_lib::mod_disaster_recovery::service_disaster_recovery::service_encoder::GroupEncoder::new(config);
    let group_id =
        encoder.create_recovery_group_and_save_with_archives(&database, archive_ids.clone())?;

    // 获取灾备组信息
    let recovery_planner = DefaultRecoveryPlanner::new();
    let group_info = recovery_planner.get_recovery_group_by_archive_id(archive_ids[0], &database)?;
    assert_eq!(group_info.group_info.group_id, group_id);

    // 验证校验分片数量和完整性（通过 assert_recovery_group_and_shards 函数）
    assert_recovery_group_and_shards(&database, group_id, 3, 2)?;

    // 额外验证校验分片信息
    for parity_shard in &group_info.parity_shards {
        assert!(!parity_shard.shard_path.is_empty(), "校验分片路径不应为空");
        assert!(!parity_shard.shard_hash.is_empty(), "校验分片哈希不应为空");
    }

    info!("灾备恢复分析器校验分片完整性验证测试通过！");
    Ok(())
}