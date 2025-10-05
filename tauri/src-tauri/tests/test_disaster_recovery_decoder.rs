use log::info;

use one_archive_lib::mod_disaster_recovery::service_disaster_recovery::service_decoder::GroupDecoder;
use one_archive_lib::mod_disaster_recovery::service_disaster_recovery::service_recovery_planner::{RecoveryPlanner, DefaultRecoveryPlanner};
use one_archive_lib::mod_disaster_recovery::core_disaster_recovery::model_disaster_recovery::RecoveryConfig;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Write;
mod common;
use common::*;
mod disaster_recovery_common;
use disaster_recovery_common::*;

/// 测试灾备恢复解码器
#[test]
fn test_disaster_recovery_decoder() -> Result<(), Box<dyn std::error::Error>> {
    init_test_env_in_file();
    info!("启动灾备恢复解码器测试...");

    let env = TestEnvironment::new()?;
    let database = &env.database;

    // 创建测试归档文件并插入数据库
    let archive_ids = create_test_archives_and_insert_to_db(&env.source_dir, &database)?;

    // 创建灾备组
    let config = RecoveryConfig {
        data_shards: 3,
        parity_shards: 2,
        storage_dir: env.archive_dir.to_string_lossy().into_owned(),
        prefix: "decode_test".to_string(),
        store_data_shards: false,
    };
    let encoder = one_archive_lib::mod_disaster_recovery::service_disaster_recovery::service_encoder::GroupEncoder::new(config.clone());
    let group_id =
        encoder.create_recovery_group_and_save_with_archives(&database, archive_ids.clone())?;

    // 初始化解码器
    let decoder = GroupDecoder::new(config);

    // 测试获取灾备组信息
    let recovery_planner = DefaultRecoveryPlanner::new();
    let group_info = recovery_planner.get_recovery_group_by_archive_id(archive_ids[0], &database)?;
    assert_eq!(group_info.group_info.group_id, group_id);
    assert_eq!(group_info.data_shards.len(), 3);
    assert_eq!(group_info.parity_shards.len(), 2);

    info!("灾备恢复解码器测试通过！");
    Ok(())
}

/// 测试灾备恢复解码器的恢复功能
#[test]
fn test_disaster_recovery_decoder_recover() -> Result<(), Box<dyn std::error::Error>> {
    init_test_env_in_file();
    info!("启动灾备恢复解码器恢复功能测试...");

    let env = TestEnvironment::new()?;
    let database = &env.database;

    // 创建测试归档文件并插入数据库
    let archive_ids = create_test_archives_and_insert_to_db(&env.source_dir, &database)?;
    assert_eq!(archive_ids.len(), 3);

    // 创建灾备组
    let config = RecoveryConfig {
        data_shards: 3,
        parity_shards: 2,
        storage_dir: env.archive_dir.to_string_lossy().into_owned(),
        prefix: "decode_recover_test".to_string(),
        store_data_shards: false,
    };
    let encoder = one_archive_lib::mod_disaster_recovery::service_disaster_recovery::service_encoder::GroupEncoder::new(config.clone());
    let group_id =
        encoder.create_recovery_group_and_save_with_archives(&database, archive_ids.clone())?;

    // 初始化解码器
    let decoder = GroupDecoder::new(config);

    // 验证灾备组信息
    let recovery_planner = DefaultRecoveryPlanner::new();
    let group_info = recovery_planner.get_recovery_group_by_archive_id(archive_ids[0], &database)?;
    assert_eq!(group_info.group_info.group_id, group_id);
    assert_eq!(group_info.data_shards.len(), 3);
    assert_eq!(group_info.parity_shards.len(), 2);

    // 选择一个归档文件进行恢复测试
    let test_archive_id = archive_ids[0];
    let work_dir = &env.temp_dir.path();

    // 首先确认文件是健康的，可以直接返回路径
    let result = decoder.recover_archive(&database, test_archive_id, work_dir)?;
    assert!(result.is_some(), "健康文件应该返回其路径");

    info!("灾备恢复解码器恢复功能测试通过！");
    Ok(())
}

/// 测试灾备恢复解码器在数据丢失情况下的恢复能力
#[test]
fn test_disaster_recovery_decoder_with_data_loss() -> Result<(), Box<dyn std::error::Error>> {
    init_test_env_in_file();
    info!("启动灾备恢复解码器数据丢失恢复能力测试...");

    let env = TestEnvironment::new()?;
    let database = &env.database;

    // 创建测试归档文件并插入数据库
    let archive_ids = create_test_archives_and_insert_to_db(&env.source_dir, &database)?;
    assert_eq!(archive_ids.len(), 3);

    // 创建灾备组
    let config = RecoveryConfig {
        data_shards: 3,
        parity_shards: 2,
        storage_dir: env.archive_dir.to_string_lossy().into_owned(),
        prefix: "decode_loss_test".to_string(),
        store_data_shards: false,
    };
    let encoder = one_archive_lib::mod_disaster_recovery::service_disaster_recovery::service_encoder::GroupEncoder::new(config.clone());
    let group_id =
        encoder.create_recovery_group_and_save_with_archives(&database, archive_ids.clone())?;

    // 初始化解码器
    let decoder = GroupDecoder::new(config);

    // 验证灾备组信息
    let recovery_planner = DefaultRecoveryPlanner::new();
    let group_info = recovery_planner.get_recovery_group_by_archive_id(archive_ids[0], &database)?;
    assert_eq!(group_info.group_info.group_id, group_id);
    assert_eq!(group_info.data_shards.len(), 3);
    assert_eq!(group_info.parity_shards.len(), 2);

    // 模拟损坏一个归档文件，使其无法访问
    let corrupted_archive_id = archive_ids[1];
    let corrupted_archive_path = env.source_dir.join("archive_1.tar");

    // 将原文件重命名
    let backup_path = env.source_dir.join("archive_1.tar.backup");
    fs::rename(&corrupted_archive_path, &backup_path)?;

    // 创建一个损坏的文件
    fs::write(&corrupted_archive_path, b"corrupted data")?;

    // 尝试恢复损坏的文件
    let work_dir = &env.temp_dir.path();
    let result = decoder.recover_archive(&database, corrupted_archive_id, work_dir)?;

    // 恢复应该成功
    assert!(result.is_some(), "应该能够恢复损坏的文件");
    info!("恢复的文件路径：{:?}", result);

    // 恢复完成后，将备份文件还原
    fs::rename(&backup_path, &corrupted_archive_path)?;

    info!("灾备恢复解码器数据丢失恢复能力测试通过！");
    Ok(())
}

/// 测试灾备恢复解码器在不同损坏情况下的恢复能力
#[test]
fn test_disaster_recovery_decoder_with_multiple_failures() -> Result<(), Box<dyn std::error::Error>> {
    init_test_env_in_file();
    info!("启动灾备恢复解码器多种损坏情况恢复能力测试...");

    let env = TestEnvironment::new()?;
    let database = &env.database;

    // 创建测试归档文件并插入数据库 - 使用 4 个文件，2 个校验，这样可以测试丢失 2 个的情况
    let mut archive_ids = Vec::new();
    let archive_dao = one_archive_lib::mod_database::dao_database::dao_archive_metadata::ArchiveMetadataDao::new(database.conn.clone());

    for i in 0..4 {
        let path = env.source_dir.join(format!("archive_multi_{}.tar", i));
        let data = vec![i as u8; 1024 * (i + 1)]; // 1KB, 2KB, 3KB, 4KB
        let mut file = fs::File::create(&path)?;
        file.write_all(&data)?;

        // 计算文件哈希
        let mut hasher = Sha256::new();
        hasher.update(&data);
        let sha256 = format!("{:x}", hasher.finalize());

        // 插入归档元数据到数据库
        let params = one_archive_lib::mod_database::schema::CreateArchiveMetadataParams {
            archive_uri: path.to_string_lossy().into_owned(),
            archive_name: format!("archive_multi_{}.tar", i),
            archive_limit_size: 10 * 1024 * 1024, // 10MB
            is_compressed: 0,
            is_encrypted: 0,
            compression_algorithm: None,
            encryption_algorithm: None,
            status: one_archive_lib::mod_database::constants::ArchiveStatus::Health,
        };

        let archive_id = archive_dao.create(params).unwrap();

        // 更新记录，添加哈希值和大小
        if let Some(mut metadata) = archive_dao.find_by_id(archive_id).unwrap() {
            metadata.archive_hash = Some(sha256);
            metadata.archive_size = Some(data.len() as i64);
            archive_dao.update(archive_id, metadata).unwrap();
        }

        archive_ids.push(archive_id);
    }

    // 创建灾备组，使用 4 个数据分片，2 个校验分片
    let config = RecoveryConfig {
        data_shards: 4,
        parity_shards: 2,
        storage_dir: env.archive_dir.to_string_lossy().into_owned(),
        prefix: "decode_multi_test".to_string(),
        store_data_shards: false,
    };
    let encoder = one_archive_lib::mod_disaster_recovery::service_disaster_recovery::service_encoder::GroupEncoder::new(config.clone());
    let group_id =
        encoder.create_recovery_group_and_save_with_archives(&database, archive_ids.clone())?;

    // 初始化解码器
    let decoder = GroupDecoder::new(config);

    // 验证灾备组信息
    let recovery_planner = DefaultRecoveryPlanner::new();
    let group_info = recovery_planner.get_recovery_group_by_archive_id(archive_ids[0], &database)?;
    assert_eq!(group_info.group_info.group_id, group_id);
    assert_eq!(group_info.data_shards.len(), 4);
    assert_eq!(group_info.parity_shards.len(), 2);

    // 尝试恢复一个健康的文件，验证恢复功能
    let test_archive_id = archive_ids[0];
    let work_dir = &env.temp_dir.path();
    let result = decoder.recover_archive(&database, test_archive_id, work_dir)?;

    // 对于健康文件，应该返回其原始路径
    assert!(result.is_some(), "健康文件应该返回其路径");
    info!("健康文件路径：{:?}", result);

    // 模拟损坏两个归档文件，这应该仍然可以恢复（因为有 2 个校验分片）
    let corrupted_archive_id1 = archive_ids[2];
    let corrupted_archive_id2 = archive_ids[3];
    let corrupted_path1 = env.source_dir.join("archive_multi_2.tar");
    let corrupted_path2 = env.source_dir.join("archive_multi_3.tar");

    // 将原文件重命名作为备份
    let backup_path1 = env.source_dir.join("archive_multi_2.tar.backup");
    let backup_path2 = env.source_dir.join("archive_multi_3.tar.backup");
    fs::rename(&corrupted_path1, &backup_path1)?;
    fs::rename(&corrupted_path2, &backup_path2)?;

    // 创建损坏的文件
    fs::write(&corrupted_path1, b"corrupted data 1")?;
    fs::write(&corrupted_path2, b"corrupted data 2")?;

    // 尝试恢复第一个损坏的文件
    let result1 = decoder.recover_archive(&database, corrupted_archive_id1, work_dir)?;
    assert!(result1.is_some(), "应该能够恢复第一个损坏的文件");
    info!("恢复的第一个文件路径：{:?}", result1);

    // 尝试恢复第二个损坏的文件
    let result2 = decoder.recover_archive(&database, corrupted_archive_id2, work_dir)?;
    assert!(result2.is_some(), "应该能够恢复第二个损坏的文件");
    info!("恢复的第二个文件路径：{:?}", result2);

    // 恢复完成后，将备份文件还原
    fs::rename(&backup_path1, &corrupted_path1)?;
    fs::rename(&backup_path2, &corrupted_path2)?;

    info!("灾备恢复解码器多种损坏情况恢复能力测试通过！");
    Ok(())
}