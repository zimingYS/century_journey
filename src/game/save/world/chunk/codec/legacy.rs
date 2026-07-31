//! 冻结区块历史 bincode 布局；Section 上线后本模块不再扩展字段。

use super::super::model::SavedChunk;
use super::super::region::SaveError;
use crate::game::world::chunk::ChunkData;
use crate::game::world::{TreeGrowthStage, TreeInstance};
use crate::shared::identifier::Identifier;
use bevy::math::IVec3;
use bincode::Options;
use serde::{Deserialize, Serialize};

const MATURE_STAGE_CODE: u8 = 0;

/// 最后一个按字段顺序编码的区块布局。
#[derive(Debug, Serialize, Deserialize)]
pub(super) struct PositionalChunkPayload {
    pub(super) position: IVec3,
    pub(super) data: ChunkData,
    pub(super) tree_instances: Vec<PositionalTreeRecord>,
    pub(super) modified_time: f64,
}

/// 顺序式树实例布局，只供读取已存在的区块记录。
#[derive(Debug, Serialize, Deserialize)]
pub(super) struct PositionalTreeRecord {
    pub(super) root: IVec3,
    pub(super) species: Identifier,
    pub(super) shape_seed: u32,
    pub(super) stage_code: u8,
    pub(super) born_at_game_minute: u64,
    pub(super) stage_started_at_game_minute: u64,
    pub(super) health: u16,
    pub(super) last_simulated_game_minute: u64,
    pub(super) next_update_game_minute: Option<u64>,
}

/// 没有格式头和树实例字段的最早区块布局。
#[derive(Debug, Serialize, Deserialize)]
pub(super) struct BareChunkPayload {
    pub(super) position: IVec3,
    pub(super) data: ChunkData,
    pub(super) modified_time: f64,
}

/// 解码最后一个带格式头的顺序式区块布局。
pub(super) fn decode_positional(payload: &[u8]) -> Result<SavedChunk, SaveError> {
    let saved = strict_options().deserialize::<PositionalChunkPayload>(payload)?;
    let tree_instances = saved
        .tree_instances
        .into_iter()
        .map(TreeInstance::try_from)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(SavedChunk {
        position: saved.position,
        data: saved.data,
        tree_instances,
        modified_time: saved.modified_time,
    })
}

/// 解码没有格式头和树实例的最早区块布局。
pub(super) fn decode_bare(payload: &[u8]) -> Result<SavedChunk, SaveError> {
    let saved = strict_options().deserialize::<BareChunkPayload>(payload)?;
    Ok(SavedChunk {
        position: saved.position,
        data: saved.data,
        tree_instances: Vec::new(),
        modified_time: saved.modified_time,
    })
}

fn strict_options() -> impl Options {
    bincode::DefaultOptions::new()
        .with_varint_encoding()
        .reject_trailing_bytes()
}

impl From<&TreeInstance> for PositionalTreeRecord {
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

impl TryFrom<PositionalTreeRecord> for TreeInstance {
    type Error = SaveError;

    fn try_from(saved: PositionalTreeRecord) -> Result<Self, Self::Error> {
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
