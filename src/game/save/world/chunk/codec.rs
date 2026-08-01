//! 分派区块 Section 格式与冻结的历史顺序式 bincode 读取器。

mod legacy;
mod section;
mod tree_fields;

use super::model::SavedChunk;
use super::region::SaveError;

/// 区分独立区块载荷与更早裸 bincode 记录的固定 magic。
const CHUNK_RECORD_MAGIC: [u8; 4] = *b"CJCH";
/// 当前 Section 容器格式；普通 Section 或字段扩展不得递增此值。
const CURRENT_CHUNK_FORMAT: u32 = 2;
/// 最后一个顺序式 bincode 区块格式。
const POSITIONAL_CHUNK_FORMAT: u32 = 1;
const CHUNK_RECORD_HEADER_LEN: usize = CHUNK_RECORD_MAGIC.len() + size_of::<u32>();

/// 把权威区块快照编码为可跳过未知段的 Section 记录。
pub(super) fn encode_chunk_record(chunk: &SavedChunk) -> Result<Vec<u8>, SaveError> {
    chunk.validate().map_err(SaveError::Serialize)?;
    section::encode(chunk)
}

/// 解码当前 Section、带头顺序式 bincode 或最早裸 bincode 记录。
pub(super) fn decode_chunk_record(record: &[u8]) -> Result<SavedChunk, SaveError> {
    let saved = if record.starts_with(&CHUNK_RECORD_MAGIC) {
        if record.len() < CHUNK_RECORD_HEADER_LEN {
            return Err(SaveError::Serialize("区块载荷头不完整".into()));
        }
        let format = u32::from_le_bytes(
            record[CHUNK_RECORD_MAGIC.len()..CHUNK_RECORD_HEADER_LEN]
                .try_into()
                .expect("区块载荷头长度已经完成预检"),
        );
        let payload = &record[CHUNK_RECORD_HEADER_LEN..];
        match format {
            POSITIONAL_CHUNK_FORMAT => legacy::decode_positional(payload)?,
            CURRENT_CHUNK_FORMAT => section::decode(payload)?,
            found => {
                return Err(SaveError::UnsupportedVersion {
                    found,
                    supported: CURRENT_CHUNK_FORMAT,
                });
            }
        }
    } else {
        legacy::decode_bare(record)?
    };

    saved.validate().map_err(SaveError::Serialize)?;
    Ok(saved)
}

#[cfg(test)]
#[path = "../../../../../tests/unit/game/save/world/chunk/codec.rs"]
mod tests;
