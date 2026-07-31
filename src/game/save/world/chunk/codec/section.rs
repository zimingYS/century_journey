//! 编解码区块具名 Section 容器及固定布局的元数据、方块段。

use super::super::model::SavedChunk;
use super::super::region::SaveError;
use super::{CHUNK_RECORD_MAGIC, CURRENT_CHUNK_FORMAT};
use crate::game::world::chunk::ChunkData;
use crate::shared::voxel::{CHUNK_SIZE, CHUNK_VOLUME};
use bevy::math::IVec3;

const METADATA_SECTION: &str = "metadata";
const BLOCKS_SECTION: &str = "blocks";
const TREES_SECTION: &str = "tree_instances";
const RAW_BLOCK_ENCODING: u8 = 0;
const MAX_SECTION_COUNT: usize = 64;
const MAX_SECTION_NAME_BYTES: usize = 64;
const MAX_SECTION_PAYLOAD_BYTES: usize = 16 * 1024 * 1024;
const MAX_CHUNK_RECORD_BYTES: usize = 20 * 1024 * 1024;

/// 已完成边界检查的具名段，供格式测试组合未知段与不同顺序。
#[derive(Debug, Clone)]
pub(super) struct Section {
    pub(super) name: String,
    pub(super) payload: Vec<u8>,
}

/// 将当前区块快照编码为完整 Section 记录。
pub(super) fn encode(chunk: &SavedChunk) -> Result<Vec<u8>, SaveError> {
    encode_container(&build_sections(chunk)?)
}

/// 将已经去除 magic 和格式号的 Section 载荷恢复为区块快照。
pub(super) fn decode(payload: &[u8]) -> Result<SavedChunk, SaveError> {
    let sections = decode_container(payload)?;
    let mut metadata = None;
    let mut blocks = None;
    let mut trees = None;

    for section in sections {
        match section.name.as_str() {
            METADATA_SECTION => {
                reject_duplicate(&metadata, METADATA_SECTION)?;
                metadata = Some(decode_metadata(&section.payload)?);
            }
            BLOCKS_SECTION => {
                reject_duplicate(&blocks, BLOCKS_SECTION)?;
                blocks = Some(decode_blocks(&section.payload)?);
            }
            TREES_SECTION => {
                reject_duplicate(&trees, TREES_SECTION)?;
                trees = Some(super::tree_fields::decode(&section.payload)?);
            }
            _ => {
                // 未知段已经由长度边界完整读取；旧程序无需理解即可安全跳过。
            }
        }
    }

    let (position, modified_time) =
        metadata.ok_or_else(|| SaveError::Serialize("区块缺少 metadata Section".into()))?;
    let data = blocks.ok_or_else(|| SaveError::Serialize("区块缺少 blocks Section".into()))?;
    Ok(SavedChunk {
        position,
        data,
        tree_instances: trees.unwrap_or_default(),
        modified_time,
    })
}

/// 从当前区块字段构造标准的三个具名 Section。
pub(super) fn build_sections(chunk: &SavedChunk) -> Result<Vec<Section>, SaveError> {
    Ok(vec![
        Section {
            name: METADATA_SECTION.into(),
            payload: encode_metadata(chunk),
        },
        Section {
            name: BLOCKS_SECTION.into(),
            payload: encode_blocks(&chunk.data),
        },
        Section {
            name: TREES_SECTION.into(),
            payload: super::tree_fields::encode(&chunk.tree_instances)?,
        },
    ])
}

/// 为已经完成业务编码的 Section 写入名称与长度边界。
pub(super) fn encode_container(sections: &[Section]) -> Result<Vec<u8>, SaveError> {
    if sections.len() > MAX_SECTION_COUNT {
        return Err(SaveError::Serialize(format!(
            "区块 Section 数量 {} 超过上限 {MAX_SECTION_COUNT}",
            sections.len()
        )));
    }

    let mut record = Vec::new();
    record.extend_from_slice(&CHUNK_RECORD_MAGIC);
    record.extend_from_slice(&CURRENT_CHUNK_FORMAT.to_le_bytes());
    push_u32(&mut record, sections.len() as u32);
    for section in sections {
        let name = section.name.as_bytes();
        if name.is_empty() || name.len() > MAX_SECTION_NAME_BYTES {
            return Err(SaveError::Serialize(format!(
                "Section 名称长度 {} 不在 1..={MAX_SECTION_NAME_BYTES}",
                name.len()
            )));
        }
        if section.payload.len() > MAX_SECTION_PAYLOAD_BYTES {
            return Err(SaveError::Serialize(format!(
                "Section {} 长度 {} 超过上限",
                section.name,
                section.payload.len()
            )));
        }
        push_u16(&mut record, name.len() as u16);
        record.extend_from_slice(name);
        push_u32(&mut record, section.payload.len() as u32);
        record.extend_from_slice(&section.payload);
        if record.len() > MAX_CHUNK_RECORD_BYTES {
            return Err(SaveError::Serialize("区块记录超过容量上限".into()));
        }
    }
    Ok(record)
}

/// 读取 Section 容器并在复制载荷前完成数量和长度校验。
pub(super) fn decode_container(payload: &[u8]) -> Result<Vec<Section>, SaveError> {
    if payload.len() > MAX_CHUNK_RECORD_BYTES {
        return Err(SaveError::Serialize("区块记录超过容量上限".into()));
    }
    let mut reader = SliceReader::new(payload);
    let count = reader.read_u32()? as usize;
    if count > MAX_SECTION_COUNT {
        return Err(SaveError::Serialize(format!(
            "区块 Section 数量 {count} 超过上限 {MAX_SECTION_COUNT}"
        )));
    }
    let mut sections = Vec::with_capacity(count);
    for _ in 0..count {
        let name_len = reader.read_u16()? as usize;
        if name_len == 0 || name_len > MAX_SECTION_NAME_BYTES {
            return Err(SaveError::Serialize(format!(
                "Section 名称长度 {name_len} 不合法"
            )));
        }
        let name = std::str::from_utf8(reader.take(name_len)?)
            .map_err(|error| SaveError::Serialize(format!("Section 名称不是 UTF-8: {error}")))?
            .to_string();
        let payload_len = reader.read_u32()? as usize;
        if payload_len > MAX_SECTION_PAYLOAD_BYTES {
            return Err(SaveError::Serialize(format!(
                "Section {name} 长度 {payload_len} 超过上限"
            )));
        }
        sections.push(Section {
            name,
            payload: reader.take(payload_len)?.to_vec(),
        });
    }
    reader.finish()?;
    Ok(sections)
}

fn encode_metadata(chunk: &SavedChunk) -> Vec<u8> {
    let mut payload = Vec::with_capacity(20);
    push_i32(&mut payload, chunk.position.x);
    push_i32(&mut payload, chunk.position.y);
    push_i32(&mut payload, chunk.position.z);
    push_f64(&mut payload, chunk.modified_time);
    payload
}

fn decode_metadata(payload: &[u8]) -> Result<(IVec3, f64), SaveError> {
    let mut reader = SliceReader::new(payload);
    let position = IVec3::new(reader.read_i32()?, reader.read_i32()?, reader.read_i32()?);
    let modified_time = reader.read_f64()?;
    reader.finish()?;
    Ok((position, modified_time))
}

fn encode_blocks(data: &ChunkData) -> Vec<u8> {
    let mut payload = Vec::with_capacity(11 + CHUNK_VOLUME * size_of::<u16>());
    payload.push(RAW_BLOCK_ENCODING);
    push_u16(&mut payload, CHUNK_SIZE as u16);
    push_u16(&mut payload, CHUNK_SIZE as u16);
    push_u16(&mut payload, CHUNK_SIZE as u16);
    push_u32(&mut payload, CHUNK_VOLUME as u32);
    for voxel in data.voxels {
        push_u16(&mut payload, voxel);
    }
    payload
}

fn decode_blocks(payload: &[u8]) -> Result<ChunkData, SaveError> {
    let mut reader = SliceReader::new(payload);
    let encoding = reader.read_u8()?;
    if encoding != RAW_BLOCK_ENCODING {
        return Err(SaveError::Serialize(format!(
            "未知方块 Section 编码 {encoding}"
        )));
    }
    let dimensions = [
        reader.read_u16()? as usize,
        reader.read_u16()? as usize,
        reader.read_u16()? as usize,
    ];
    if dimensions != [CHUNK_SIZE; 3] {
        return Err(SaveError::Serialize(format!(
            "区块尺寸 {dimensions:?} 与当前尺寸不一致"
        )));
    }
    let voxel_count = reader.read_u32()? as usize;
    if voxel_count != CHUNK_VOLUME {
        return Err(SaveError::Serialize(format!(
            "方块数量 {voxel_count} 与区块容量 {CHUNK_VOLUME} 不一致"
        )));
    }
    let mut data = ChunkData::new();
    for voxel in &mut data.voxels {
        *voxel = reader.read_u16()?;
    }
    reader.finish()?;
    Ok(data)
}

fn reject_duplicate<T>(value: &Option<T>, name: &str) -> Result<(), SaveError> {
    if value.is_some() {
        return Err(SaveError::Serialize(format!(
            "区块包含重复的 {name} Section"
        )));
    }
    Ok(())
}

/// 在限定切片内按小端顺序读取字段，所有游标推进都经过溢出检查。
pub(super) struct SliceReader<'a> {
    bytes: &'a [u8],
    cursor: usize,
}

impl<'a> SliceReader<'a> {
    /// 从完整字段或 Section 切片创建读取器。
    pub(super) const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, cursor: 0 }
    }

    /// 读取指定长度的连续字节，并拒绝加法溢出或截断。
    pub(super) fn take(&mut self, len: usize) -> Result<&'a [u8], SaveError> {
        let end = self
            .cursor
            .checked_add(len)
            .ok_or_else(|| SaveError::Serialize("区块长度计算溢出".into()))?;
        let value = self
            .bytes
            .get(self.cursor..end)
            .ok_or_else(|| SaveError::Serialize("区块 Section 被截断".into()))?;
        self.cursor = end;
        Ok(value)
    }

    /// 读取一个无符号字节。
    pub(super) fn read_u8(&mut self) -> Result<u8, SaveError> {
        Ok(self.take(1)?[0])
    }

    /// 按小端顺序读取 `u16`。
    pub(super) fn read_u16(&mut self) -> Result<u16, SaveError> {
        Ok(u16::from_le_bytes(
            self.take(2)?.try_into().expect("读取长度固定为 2"),
        ))
    }

    /// 按小端顺序读取 `u32`。
    pub(super) fn read_u32(&mut self) -> Result<u32, SaveError> {
        Ok(u32::from_le_bytes(
            self.take(4)?.try_into().expect("读取长度固定为 4"),
        ))
    }

    /// 按小端顺序读取 `i32`。
    pub(super) fn read_i32(&mut self) -> Result<i32, SaveError> {
        Ok(i32::from_le_bytes(
            self.take(4)?.try_into().expect("读取长度固定为 4"),
        ))
    }

    /// 按小端顺序读取有限或非有限的原始 `f64`；业务层随后负责语义校验。
    pub(super) fn read_f64(&mut self) -> Result<f64, SaveError> {
        Ok(f64::from_le_bytes(
            self.take(8)?.try_into().expect("读取长度固定为 8"),
        ))
    }

    /// 返回当前边界内尚未读取的字节数。
    pub(super) fn remaining(&self) -> usize {
        self.bytes.len() - self.cursor
    }

    /// 确认调用方完整消费了当前边界，拒绝静默接受尾随数据。
    pub(super) fn finish(self) -> Result<(), SaveError> {
        if self.cursor != self.bytes.len() {
            return Err(SaveError::Serialize("区块 Section 含尾随字节".into()));
        }
        Ok(())
    }
}

/// 按小端顺序追加 `u16`。
pub(super) fn push_u16(target: &mut Vec<u8>, value: u16) {
    target.extend_from_slice(&value.to_le_bytes());
}

/// 按小端顺序追加 `u32`。
pub(super) fn push_u32(target: &mut Vec<u8>, value: u32) {
    target.extend_from_slice(&value.to_le_bytes());
}

/// 按小端顺序追加 `i32`。
pub(super) fn push_i32(target: &mut Vec<u8>, value: i32) {
    target.extend_from_slice(&value.to_le_bytes());
}

fn push_f64(target: &mut Vec<u8>, value: f64) {
    target.extend_from_slice(&value.to_le_bytes());
}
