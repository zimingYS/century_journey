//! 编解码带独立格式头的区块载荷，并迁移没有树实例的旧区块记录。

use super::model::SavedChunk;
use super::region::SaveError;
use crate::game::world::TreeGrowthStage;
use crate::game::world::TreeInstance;
use crate::game::world::chunk::ChunkData;
use crate::shared::identifier::Identifier;
use bevy::math::IVec3;
use bincode::Options;
use serde::{Deserialize, Serialize};

/// 区分带版本区块载荷与旧版裸 bincode 记录的固定魔数。
const CHUNK_RECORD_MAGIC: [u8; 4] = *b"CJCH";
/// 当前区块载荷协议编号；它只描述磁盘布局，不作为功能阶段名称。
const CURRENT_CHUNK_FORMAT_VERSION: u32 = 1;
/// 魔数与小端版本号合计长度。
const CHUNK_RECORD_HEADER_LEN: usize = CHUNK_RECORD_MAGIC.len() + size_of::<u32>();
/// 成熟阶段的稳定磁盘代码。
const MATURE_STAGE_CODE: u8 = 0;

/// 当前区块载荷的持久化 DTO，不直接暴露给世界规则。
#[derive(Debug, Serialize, Deserialize)]
struct CurrentChunkPayload {
    position: IVec3,
    data: ChunkData,
    tree_instances: Vec<SavedTreeInstance>,
    modified_time: f64,
}

/// 使用显式阶段代码保存树实例，避免枚举声明顺序成为 bincode 契约。
#[derive(Debug, Serialize, Deserialize)]
struct SavedTreeInstance {
    root: IVec3,
    species: Identifier,
    shape_seed: u32,
    stage_code: u8,
    born_at_game_minute: u64,
    stage_started_at_game_minute: u64,
    health: u16,
    last_simulated_game_minute: u64,
    next_update_game_minute: Option<u64>,
}

/// 没有独立格式头和树实例字段的历史区块布局。
#[derive(Debug, Serialize, Deserialize)]
struct LegacySavedChunk {
    position: IVec3,
    data: ChunkData,
    modified_time: f64,
}

/// 把运行时区块快照编码为带 magic 与小端格式号的载荷。
pub(super) fn encode_chunk_record(chunk: &SavedChunk) -> Result<Vec<u8>, SaveError> {
    chunk.validate().map_err(SaveError::Serialize)?;
    let payload = CurrentChunkPayload {
        position: chunk.position,
        data: chunk.data.clone(),
        tree_instances: chunk
            .tree_instances
            .iter()
            .map(SavedTreeInstance::from)
            .collect(),
        modified_time: chunk.modified_time,
    };
    let encoded = bincode::DefaultOptions::new()
        .with_varint_encoding()
        .serialize(&payload)?;

    let mut record = Vec::with_capacity(CHUNK_RECORD_HEADER_LEN + encoded.len());
    record.extend_from_slice(&CHUNK_RECORD_MAGIC);
    record.extend_from_slice(&CURRENT_CHUNK_FORMAT_VERSION.to_le_bytes());
    record.extend_from_slice(&encoded);
    Ok(record)
}

/// 解码当前区块载荷；缺少 magic 时按旧三字段布局迁移为空树实例集合。
pub(super) fn decode_chunk_record(record: &[u8]) -> Result<SavedChunk, SaveError> {
    let saved = if record.starts_with(&CHUNK_RECORD_MAGIC) {
        decode_current_record(record)?
    } else {
        let legacy = strict_options().deserialize::<LegacySavedChunk>(record)?;
        SavedChunk {
            position: legacy.position,
            data: legacy.data,
            tree_instances: Vec::new(),
            modified_time: legacy.modified_time,
        }
    };
    saved.validate().map_err(SaveError::Serialize)?;
    Ok(saved)
}

fn decode_current_record(record: &[u8]) -> Result<SavedChunk, SaveError> {
    if record.len() < CHUNK_RECORD_HEADER_LEN {
        return Err(SaveError::Serialize("区块载荷头不完整".into()));
    }
    let version = u32::from_le_bytes(
        record[CHUNK_RECORD_MAGIC.len()..CHUNK_RECORD_HEADER_LEN]
            .try_into()
            .expect("区块载荷头长度已经完成预检"),
    );
    if version != CURRENT_CHUNK_FORMAT_VERSION {
        return Err(SaveError::UnsupportedVersion {
            found: version,
            supported: CURRENT_CHUNK_FORMAT_VERSION,
        });
    }

    let payload =
        strict_options().deserialize::<CurrentChunkPayload>(&record[CHUNK_RECORD_HEADER_LEN..])?;
    let tree_instances = payload
        .tree_instances
        .into_iter()
        .map(TreeInstance::try_from)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(SavedChunk {
        position: payload.position,
        data: payload.data,
        tree_instances,
        modified_time: payload.modified_time,
    })
}

fn strict_options() -> impl Options {
    bincode::DefaultOptions::new()
        .with_varint_encoding()
        .reject_trailing_bytes()
}

impl From<&TreeInstance> for SavedTreeInstance {
    fn from(instance: &TreeInstance) -> Self {
        let stage_code = match instance.stage() {
            TreeGrowthStage::Mature => MATURE_STAGE_CODE,
        };
        Self {
            root: instance.root(),
            species: instance.species().clone(),
            shape_seed: instance.shape_seed(),
            stage_code,
            born_at_game_minute: instance.born_at_game_minute(),
            stage_started_at_game_minute: instance.stage_started_at_game_minute(),
            health: instance.health(),
            last_simulated_game_minute: instance.last_simulated_game_minute(),
            next_update_game_minute: instance.next_update_game_minute(),
        }
    }
}

impl TryFrom<SavedTreeInstance> for TreeInstance {
    type Error = SaveError;

    fn try_from(saved: SavedTreeInstance) -> Result<Self, Self::Error> {
        let stage = match saved.stage_code {
            MATURE_STAGE_CODE => TreeGrowthStage::Mature,
            code => {
                return Err(SaveError::Serialize(format!(
                    "未知树木生命周期阶段代码 {code}"
                )));
            }
        };
        TreeInstance::from_persisted(
            saved.root,
            saved.species,
            saved.shape_seed,
            stage,
            saved.born_at_game_minute,
            saved.stage_started_at_game_minute,
            saved.health,
            saved.last_simulated_game_minute,
            saved.next_update_game_minute,
        )
        .map_err(SaveError::Serialize)
    }
}

#[cfg(test)]
#[path = "../../../../../tests/unit/game/save/world/chunk/codec.rs"]
mod tests;
