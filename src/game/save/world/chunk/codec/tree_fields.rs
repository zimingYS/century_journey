//! 使用字段 TLV 编解码树实例，使单个树字段可独立增加、删除和跳过。

use super::super::region::SaveError;
use super::section::{SliceReader, push_i32, push_u16, push_u32};
use crate::game::world::{TreeGrowthStage, TreeInstance};
use crate::shared::identifier::Identifier;
use bevy::math::IVec3;
use std::collections::HashSet;

const MAX_TREE_COUNT: usize = 4_096;
const MAX_TREE_RECORD_BYTES: usize = 64 * 1024;
const FULL_TREE_HEALTH: u16 = 1_000;
// `Mature = 0` 已写入现有存档，后续阶段只能分配新代码。
const MATURE_STAGE_CODE: u8 = 0;
const SAPLING_STAGE_CODE: u8 = 1;
const YOUNG_STAGE_CODE: u8 = 2;

// 字段编号写入磁盘后永久保留，删除字段也不得复用其编号。
const ROOT_FIELD: u16 = 1;
const SPECIES_FIELD: u16 = 2;
const SHAPE_SEED_FIELD: u16 = 3;
const STAGE_FIELD: u16 = 4;
const BORN_AT_FIELD: u16 = 5;
const STAGE_STARTED_AT_FIELD: u16 = 6;
const HEALTH_FIELD: u16 = 7;
const LAST_SIMULATED_AT_FIELD: u16 = 8;
const NEXT_UPDATE_AT_FIELD: u16 = 9;

/// 将有序树实例集合编码为逐记录、逐字段 TLV 载荷。
pub(super) fn encode(instances: &[TreeInstance]) -> Result<Vec<u8>, SaveError> {
    if instances.len() > MAX_TREE_COUNT {
        return Err(SaveError::Serialize(format!(
            "树实例数量 {} 超过上限 {MAX_TREE_COUNT}",
            instances.len()
        )));
    }
    let mut payload = Vec::new();
    push_u32(&mut payload, instances.len() as u32);
    for instance in instances {
        let record = encode_tree(instance)?;
        push_u32(&mut payload, record.len() as u32);
        payload.extend_from_slice(&record);
    }
    Ok(payload)
}

/// 解码树实例 TLV；未知字段跳过，缺失的非身份字段使用语义默认值。
pub(super) fn decode(payload: &[u8]) -> Result<Vec<TreeInstance>, SaveError> {
    let mut reader = SliceReader::new(payload);
    let count = reader.read_u32()? as usize;
    if count > MAX_TREE_COUNT {
        return Err(SaveError::Serialize(format!(
            "树实例数量 {count} 超过上限 {MAX_TREE_COUNT}"
        )));
    }
    let mut instances = Vec::with_capacity(count);
    for _ in 0..count {
        let record_len = reader.read_u32()? as usize;
        if record_len > MAX_TREE_RECORD_BYTES {
            return Err(SaveError::Serialize(format!(
                "树实例记录长度 {record_len} 超过上限"
            )));
        }
        instances.push(decode_tree(reader.take(record_len)?)?);
    }
    reader.finish()?;
    Ok(instances)
}

fn encode_tree(instance: &TreeInstance) -> Result<Vec<u8>, SaveError> {
    let mut record = Vec::new();

    let mut root = Vec::with_capacity(12);
    push_i32(&mut root, instance.root().x);
    push_i32(&mut root, instance.root().y);
    push_i32(&mut root, instance.root().z);
    push_field(&mut record, ROOT_FIELD, &root)?;
    push_field(
        &mut record,
        SPECIES_FIELD,
        instance.species().to_string().as_bytes(),
    )?;
    push_field(
        &mut record,
        SHAPE_SEED_FIELD,
        &instance.shape_seed().to_le_bytes(),
    )?;
    let stage = match instance.stage() {
        TreeGrowthStage::Sapling => SAPLING_STAGE_CODE,
        TreeGrowthStage::Young => YOUNG_STAGE_CODE,
        TreeGrowthStage::Mature => MATURE_STAGE_CODE,
    };
    push_field(&mut record, STAGE_FIELD, &[stage])?;
    push_field(
        &mut record,
        BORN_AT_FIELD,
        &instance.born_at_game_minute().to_le_bytes(),
    )?;
    push_field(
        &mut record,
        STAGE_STARTED_AT_FIELD,
        &instance.stage_started_at_game_minute().to_le_bytes(),
    )?;
    push_field(&mut record, HEALTH_FIELD, &instance.health().to_le_bytes())?;
    push_field(
        &mut record,
        LAST_SIMULATED_AT_FIELD,
        &instance.last_simulated_game_minute().to_le_bytes(),
    )?;
    if let Some(next_update) = instance.next_update_game_minute() {
        push_field(
            &mut record,
            NEXT_UPDATE_AT_FIELD,
            &next_update.to_le_bytes(),
        )?;
    }
    Ok(record)
}

fn decode_tree(record: &[u8]) -> Result<TreeInstance, SaveError> {
    let mut reader = SliceReader::new(record);
    let mut seen = HashSet::new();
    let mut root = None;
    let mut species = None;
    let mut shape_seed = 0;
    let mut stage = TreeGrowthStage::Mature;
    let mut born_at = 0;
    let mut stage_started_at = None;
    let mut health = FULL_TREE_HEALTH;
    let mut last_simulated_at = None;
    let mut next_update_at = None;

    while reader.remaining() > 0 {
        let field_id = reader.read_u16()?;
        let field_len = reader.read_u32()? as usize;
        let field = reader.take(field_len)?;
        if is_known_field(field_id) && !seen.insert(field_id) {
            return Err(SaveError::Serialize(format!(
                "树实例包含重复字段 {field_id}"
            )));
        }
        match field_id {
            ROOT_FIELD => root = Some(decode_root(field)?),
            SPECIES_FIELD => {
                let raw = std::str::from_utf8(field).map_err(|error| {
                    SaveError::Serialize(format!("树种标识不是 UTF-8: {error}"))
                })?;
                species = Some(
                    Identifier::parse(raw)
                        .map_err(|error| SaveError::Serialize(format!("树种标识无效: {error}")))?,
                );
            }
            SHAPE_SEED_FIELD => shape_seed = decode_u32(field, "shape_seed")?,
            STAGE_FIELD => {
                stage = match decode_u8(field, "stage")? {
                    MATURE_STAGE_CODE => TreeGrowthStage::Mature,
                    SAPLING_STAGE_CODE => TreeGrowthStage::Sapling,
                    YOUNG_STAGE_CODE => TreeGrowthStage::Young,
                    code => {
                        return Err(SaveError::Serialize(format!(
                            "未知树木生命周期阶段代码 {code}"
                        )));
                    }
                };
            }
            BORN_AT_FIELD => born_at = decode_u64(field, "born_at")?,
            STAGE_STARTED_AT_FIELD => {
                stage_started_at = Some(decode_u64(field, "stage_started_at")?)
            }
            HEALTH_FIELD => health = decode_u16(field, "health")?,
            LAST_SIMULATED_AT_FIELD => {
                last_simulated_at = Some(decode_u64(field, "last_simulated_at")?)
            }
            NEXT_UPDATE_AT_FIELD => {
                next_update_at = match field.len() {
                    0 => None,
                    8 => Some(decode_u64(field, "next_update_at")?),
                    len => {
                        return Err(SaveError::Serialize(format!(
                            "树字段 next_update_at 长度 {len} 不合法"
                        )));
                    }
                }
            }
            _ => {
                // 未知字段由 field_len 提供边界，读取器可直接跳过。
            }
        }
    }

    let root = root.ok_or_else(|| SaveError::Serialize("树实例缺少 root 字段".into()))?;
    let species = species.ok_or_else(|| SaveError::Serialize("树实例缺少 species 字段".into()))?;
    let stage_started_at = stage_started_at.unwrap_or(born_at);
    let last_simulated_at = last_simulated_at.unwrap_or(stage_started_at);
    TreeInstance::from_persisted(
        root,
        species,
        shape_seed,
        stage,
        born_at,
        stage_started_at,
        health,
        last_simulated_at,
        next_update_at,
    )
    .map_err(SaveError::Serialize)
}

fn push_field(target: &mut Vec<u8>, id: u16, payload: &[u8]) -> Result<(), SaveError> {
    if payload.len() > MAX_TREE_RECORD_BYTES {
        return Err(SaveError::Serialize(format!("树字段 {id} 长度超过上限")));
    }
    push_u16(target, id);
    push_u32(target, payload.len() as u32);
    target.extend_from_slice(payload);
    if target.len() > MAX_TREE_RECORD_BYTES {
        return Err(SaveError::Serialize("树实例记录超过容量上限".into()));
    }
    Ok(())
}

fn is_known_field(field_id: u16) -> bool {
    matches!(
        field_id,
        ROOT_FIELD
            | SPECIES_FIELD
            | SHAPE_SEED_FIELD
            | STAGE_FIELD
            | BORN_AT_FIELD
            | STAGE_STARTED_AT_FIELD
            | HEALTH_FIELD
            | LAST_SIMULATED_AT_FIELD
            | NEXT_UPDATE_AT_FIELD
    )
}

fn decode_root(bytes: &[u8]) -> Result<IVec3, SaveError> {
    let mut reader = SliceReader::new(bytes);
    let root = IVec3::new(reader.read_i32()?, reader.read_i32()?, reader.read_i32()?);
    reader.finish()?;
    Ok(root)
}

fn decode_u8(bytes: &[u8], name: &str) -> Result<u8, SaveError> {
    if bytes.len() != 1 {
        return Err(invalid_length(name, bytes.len(), 1));
    }
    Ok(bytes[0])
}

fn decode_u16(bytes: &[u8], name: &str) -> Result<u16, SaveError> {
    if bytes.len() != 2 {
        return Err(invalid_length(name, bytes.len(), 2));
    }
    Ok(u16::from_le_bytes(bytes.try_into().expect("长度已经校验")))
}

fn decode_u32(bytes: &[u8], name: &str) -> Result<u32, SaveError> {
    if bytes.len() != 4 {
        return Err(invalid_length(name, bytes.len(), 4));
    }
    Ok(u32::from_le_bytes(bytes.try_into().expect("长度已经校验")))
}

fn decode_u64(bytes: &[u8], name: &str) -> Result<u64, SaveError> {
    if bytes.len() != 8 {
        return Err(invalid_length(name, bytes.len(), 8));
    }
    Ok(u64::from_le_bytes(bytes.try_into().expect("长度已经校验")))
}

fn invalid_length(name: &str, actual: usize, expected: usize) -> SaveError {
    SaveError::Serialize(format!("树字段 {name} 长度 {actual}，预期 {expected}"))
}
