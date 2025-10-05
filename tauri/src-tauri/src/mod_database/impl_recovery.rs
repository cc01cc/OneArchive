//! 灾备相关数据库操作实现

use rusqlite::{Connection, Result as SqliteResult, params};

use crate::mod_database::schema_recovery::{RecoveryDataShard, RecoveryParityShard};
use crate::mod_database::trait_recovery::{
    RecoveryDataShardOperations, RecoveryParityShardOperations,
};

impl RecoveryDataShardOperations for Connection {
    /// 创建数据分片记录
    fn insert_recovery_data_shard(
        &self, group_id: i64, shard_index: i64, shard_path: Option<&str>, shard_hash: &str,
    ) -> SqliteResult<i64> {
        let mut stmt = self.prepare(
            "INSERT INTO disaster_recovery_data_shard 
            (group_id, shard_index, shard_path, shard_hash, status)
            VALUES (?1, ?2, ?3, ?4, 'HEALTH')
            RETURNING id",
        )?;

        let id: i64 = stmt
            .query_row(params![group_id, shard_index, shard_path, shard_hash], |row| row.get(0))?;

        Ok(id)
    }

    /// 根据组 ID 查找所有数据分片
    fn find_recovery_data_shards_by_group_id(
        &self, group_id: i64,
    ) -> SqliteResult<Vec<RecoveryDataShard>> {
        let mut stmt = self.prepare(
            "SELECT id, group_id, shard_index, shard_path, shard_hash, 
                    status, last_verified, created_at, updated_at
             FROM disaster_recovery_data_shard 
             WHERE group_id = ?1
             ORDER BY shard_index",
        )?;

        let mut rows = stmt.query(params![group_id])?;
        let mut shards = Vec::new();

        while let Some(row) = rows.next()? {
            shards.push(RecoveryDataShard::try_from(row)?);
        }

        Ok(shards)
    }

    /// 根据 ID 查找数据分片
    fn find_recovery_data_shard_by_id(&self, id: i64) -> SqliteResult<Option<RecoveryDataShard>> {
        let mut stmt = self.prepare(
            "SELECT id, group_id, shard_index, shard_path, shard_hash, 
                    status, last_verified, created_at, updated_at
             FROM disaster_recovery_data_shard 
             WHERE id = ?1",
        )?;

        let result = stmt.query_row(params![id], |row| RecoveryDataShard::try_from(row));

        match result {
            Ok(shard) => Ok(Some(shard)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// 更新数据分片状态
    fn update_recovery_data_shard_status(&self, id: i64, status: &str) -> SqliteResult<()> {
        let mut stmt = self.prepare(
            "UPDATE disaster_recovery_data_shard 
             SET status = ?1, updated_at = strftime('%s', 'now')
             WHERE id = ?2",
        )?;

        stmt.execute(params![status, id])?;
        Ok(())
    }

    /// 删除数据分片
    fn delete_recovery_data_shard(&self, id: i64) -> SqliteResult<()> {
        let mut stmt = self.prepare("DELETE FROM disaster_recovery_data_shard WHERE id = ?1")?;
        stmt.execute(params![id])?;
        Ok(())
    }
}

impl RecoveryParityShardOperations for Connection {
    /// 创建校验分片记录
    fn insert_recovery_parity_shard(
        &self, group_id: i64, shard_index: i64, shard_path: &str, shard_hash: &str,
    ) -> SqliteResult<i64> {
        let mut stmt = self.prepare(
            "INSERT INTO disaster_recovery_parity_shard 
            (group_id, shard_index, shard_path, shard_hash, status)
            VALUES (?1, ?2, ?3, ?4, 'HEALTH')
            RETURNING id",
        )?;

        let id: i64 = stmt
            .query_row(params![group_id, shard_index, shard_path, shard_hash], |row| row.get(0))?;

        Ok(id)
    }

    /// 根据组 ID 查找所有校验分片
    fn find_recovery_parity_shards_by_group_id(
        &self, group_id: i64,
    ) -> SqliteResult<Vec<RecoveryParityShard>> {
        let mut stmt = self.prepare(
            "SELECT id, group_id, shard_index, shard_path, shard_hash, 
                    status, last_verified, created_at, updated_at
             FROM disaster_recovery_parity_shard 
             WHERE group_id = ?1
             ORDER BY shard_index",
        )?;

        let mut rows = stmt.query(params![group_id])?;
        let mut shards = Vec::new();

        while let Some(row) = rows.next()? {
            shards.push(RecoveryParityShard::try_from(row)?);
        }

        Ok(shards)
    }

    /// 根据 ID 查找校验分片
    fn find_recovery_parity_shard_by_id(
        &self, id: i64,
    ) -> SqliteResult<Option<RecoveryParityShard>> {
        let mut stmt = self.prepare(
            "SELECT id, group_id, shard_index, shard_path, shard_hash, 
                    status, last_verified, created_at, updated_at
             FROM disaster_recovery_parity_shard 
             WHERE id = ?1",
        )?;

        let result = stmt.query_row(params![id], |row| RecoveryParityShard::try_from(row));

        match result {
            Ok(shard) => Ok(Some(shard)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// 更新校验分片状态
    fn update_recovery_parity_shard_status(&self, id: i64, status: &str) -> SqliteResult<()> {
        let mut stmt = self.prepare(
            "UPDATE disaster_recovery_parity_shard 
             SET status = ?1, updated_at = strftime('%s', 'now')
             WHERE id = ?2",
        )?;

        stmt.execute(params![status, id])?;
        Ok(())
    }

    /// 删除校验分片
    fn delete_recovery_parity_shard(&self, id: i64) -> SqliteResult<()> {
        let mut stmt = self.prepare("DELETE FROM disaster_recovery_parity_shard WHERE id = ?1")?;
        stmt.execute(params![id])?;
        Ok(())
    }
}
