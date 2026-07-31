//! 负责区域文件寻址、原子写入和世界目录生命周期操作。

use super::codec::{decode_chunk_record, encode_chunk_record};
use crate::engine::persistence;
use crate::game::save::path::world_save_root;
use crate::game::save::world::chunk::model::{
    REGION_SIZE, RegionFile, RegionHeader, SavedChunk, chunk_local_index, chunk_to_region_pos,
    local_index_to_flat,
};
use bevy::prelude::*;
use bincode::Options;
use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

/// 世界元数据文件名。
const LEVEL_FILE_NAME: &str = "level.dat";
/// Region 文件目录名。
const REGION_DIR_NAME: &str = "regions";
/// Region 文件名前缀。
const REGION_FILE_PREFIX: &str = "r";
/// 单个区块解压后的硬上限，阻止损坏记录在读取屏障前无限占用内存。
const MAX_DECOMPRESSED_CHUNK_BYTES: usize = 20 * 1024 * 1024;

/// Region 文件的读写管理器
pub struct RegionManager;

/// 存档错误处理
#[derive(Debug)]
pub enum SaveError {
    /// 文件读写错误
    Io(std::io::Error),
    /// 序列化/反序列化错误
    Serialize(String),
    /// 原子替换或备份恢复错误
    Atomic(String),
    /// 文件格式比当前程序支持的版本更新
    UnsupportedVersion { found: u32, supported: u32 },
}

/// 错误显示输出
impl std::fmt::Display for SaveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SaveError::Io(e) => write!(f, "IO error: {e}"),
            SaveError::Serialize(e) => write!(f, "Serialize error: {e}"),
            SaveError::Atomic(e) => write!(f, "Atomic file error: {e}"),
            SaveError::UnsupportedVersion { found, supported } => {
                write!(
                    f,
                    "Unsupported save version {found}, current support is {supported}"
                )
            }
        }
    }
}

#[cfg(test)]
#[path = "../../../../../tests/unit/game/save/world/chunk/region.rs"]
mod tests;

/// 将文件相关错误转换为存档错误
impl From<std::io::Error> for SaveError {
    fn from(e: std::io::Error) -> Self {
        SaveError::Io(e)
    }
}

/// 将序列化相关错误转换为存档错误
impl From<bincode::Error> for SaveError {
    fn from(e: bincode::Error) -> Self {
        SaveError::Serialize(e.to_string())
    }
}

impl From<crate::engine::persistence::AtomicFileError> for SaveError {
    fn from(error: crate::engine::persistence::AtomicFileError) -> Self {
        SaveError::Atomic(error.to_string())
    }
}

// 对外接口
impl RegionManager {
    /// 获取存档根路径。
    pub fn save_root(world_name: &str) -> PathBuf {
        world_save_root(world_name)
    }

    /// 获取保存 Region 文件的目录。
    pub(in crate::game::save::world) fn regions_path(world_name: &str) -> PathBuf {
        Self::save_root(world_name).join(REGION_DIR_NAME)
    }

    /// 从 Region 文件路径解析三维 Region 坐标。
    ///
    /// 只接受 `r.x.y.z.bin` 形式的文件名，其他文件由世界加载器忽略。
    pub(in crate::game::save::world) fn region_position_from_path(path: &Path) -> Option<IVec3> {
        if path.extension()? != "bin" {
            return None;
        }
        let mut parts = path.file_stem()?.to_str()?.split('.');
        if parts.next()? != REGION_FILE_PREFIX {
            return None;
        }
        let position = IVec3::new(
            parts.next()?.parse().ok()?,
            parts.next()?.parse().ok()?,
            parts.next()?.parse().ok()?,
        );
        parts.next().is_none().then_some(position)
    }

    /// 获取 region 文件路径。
    pub fn region_path(world_name: &str, region_pos: IVec3) -> PathBuf {
        Self::regions_path(world_name).join(format!(
            "{}.{}.{}.{}.bin",
            REGION_FILE_PREFIX, region_pos.x, region_pos.y, region_pos.z
        ))
    }

    /// 获取 level.dat 路径
    pub fn level_path(world_name: &str) -> PathBuf {
        Self::save_root(world_name).join(LEVEL_FILE_NAME)
    }

    /// 确保存档目录结构存在
    pub fn ensure_dirs(world_name: &str) -> std::io::Result<()> {
        let root = Self::save_root(world_name);
        fs::create_dir_all(root.join(REGION_DIR_NAME))?;
        Ok(())
    }

    /// 读取单个区块，返回 None 表示该区块未存储
    pub fn read_chunk(world_name: &str, chunk_pos: IVec3) -> Result<Option<SavedChunk>, SaveError> {
        let region_pos = chunk_to_region_pos(chunk_pos);
        let path = Self::region_path(world_name, region_pos);

        if !path.exists() && !persistence::backup_path(&path).exists() {
            return Ok(None);
        }

        let region = Self::read_region_path(&path)?;

        let (lx, ly, lz) = chunk_local_index(chunk_pos);
        let flat = local_index_to_flat(lx, ly, lz);

        // 检查位图
        let byte_idx = flat / 8;
        let bit_idx = flat % 8;
        if byte_idx >= region.header.chunk_present.len() {
            return Ok(None);
        }
        if region.header.chunk_present[byte_idx] & (1 << bit_idx) == 0 {
            return Ok(None);
        }

        // 在 header 的有序列表中找到此区块
        let chunk_idx = Self::find_chunk_index(&region, flat)?;
        let compressed = &region.chunks[chunk_idx];
        let saved = Self::decode_compressed_chunk(compressed)?;
        if saved.position != chunk_pos {
            return Err(SaveError::Serialize(format!(
                "区块索引请求 {chunk_pos:?}，载荷却声明 {:?}",
                saved.position
            )));
        }

        Ok(Some(saved))
    }

    /// 写入单个区块（读取整个 region → 修改 → 写回）
    pub fn write_chunk(world_name: &str, chunk: &SavedChunk) -> Result<(), SaveError> {
        let region_pos = chunk_to_region_pos(chunk.position);
        let path = Self::region_path(world_name, region_pos);
        Self::ensure_dirs(world_name)?;

        let mut region = if path.exists() || persistence::backup_path(&path).exists() {
            Self::read_region_path(&path)?
        } else {
            RegionFile {
                header: RegionHeader {
                    chunk_present: vec![0u8; (REGION_SIZE as usize).pow(3) / 8],
                    chunk_offsets: Vec::new(),
                    chunk_lengths: Vec::new(),
                },
                chunks: Vec::new(),
            }
        };

        let (lx, ly, lz) = chunk_local_index(chunk.position);
        let flat = local_index_to_flat(lx, ly, lz);
        let byte_idx = flat / 8;
        let bit_idx = flat % 8;
        let was_present = region.header.chunk_present[byte_idx] & (1 << bit_idx) != 0;

        let compressed = Self::compress_chunk(chunk)?;

        if was_present {
            // 覆盖已有区块
            let idx = Self::find_chunk_index(&region, flat)?;
            region.chunks[idx] = compressed;
        } else {
            // 新增区块
            region.header.chunk_present[byte_idx] |= 1 << bit_idx;
            // 插入到排序位置以保持有序
            let insert_pos = Self::count_present_before(&region, flat);
            region.chunks.insert(insert_pos, compressed);
        }

        Self::write_region_path(&path, &region)?;

        Ok(())
    }

    /// 批量写入同一 Region 的多个区块（只读写一次文件）
    pub fn write_chunks_batch(world_name: &str, chunks: &[SavedChunk]) -> Result<(), SaveError> {
        if chunks.is_empty() {
            return Ok(());
        }

        // 按 region 分组
        let mut groups: std::collections::HashMap<IVec3, Vec<&SavedChunk>> =
            std::collections::HashMap::new();
        for chunk in chunks {
            let rp = chunk_to_region_pos(chunk.position);
            groups.entry(rp).or_default().push(chunk);
        }

        for (region_pos, group) in groups {
            let path = Self::region_path(world_name, region_pos);
            Self::ensure_dirs(world_name)?;

            let mut region = if path.exists() || persistence::backup_path(&path).exists() {
                Self::read_region_path(&path)?
            } else {
                RegionFile {
                    header: RegionHeader {
                        chunk_present: vec![0u8; (REGION_SIZE as usize).pow(3) / 8],
                        chunk_offsets: Vec::new(),
                        chunk_lengths: Vec::new(),
                    },
                    chunks: Vec::new(),
                }
            };

            for chunk in group {
                let (lx, ly, lz) = chunk_local_index(chunk.position);
                let flat = local_index_to_flat(lx, ly, lz);
                let byte_idx = flat / 8;
                let bit_idx = flat % 8;
                let was_present = region.header.chunk_present[byte_idx] & (1 << bit_idx) != 0;

                let compressed = Self::compress_chunk(chunk)?;

                if was_present {
                    let idx = Self::find_chunk_index(&region, flat)?;
                    region.chunks[idx] = compressed;
                } else {
                    region.header.chunk_present[byte_idx] |= 1 << bit_idx;
                    let insert_pos = Self::count_present_before(&region, flat);
                    region.chunks.insert(insert_pos, compressed);
                }
            }

            Self::write_region_path(&path, &region)?;
        }

        Ok(())
    }

    /// 解压并解码单个区块记录，供流式加载与完整世界加载共享迁移规则。
    pub(in crate::game::save::world) fn decode_compressed_chunk(
        compressed: &[u8],
    ) -> Result<SavedChunk, SaveError> {
        let decompressed = Self::decompress_chunk(compressed)?;
        decode_chunk_record(&decompressed)
    }

    /// 删除指定世界的完整存档目录。
    pub fn delete_world(world_name: &str) -> std::io::Result<()> {
        let root = Self::save_root(world_name);
        if root.exists() {
            fs::remove_dir_all(root)?;
        }
        Ok(())
    }
}

// 内部方法
impl RegionManager {
    /// 从路径读取 Region；主文件损坏时自动恢复最近有效备份。
    pub(crate) fn read_region_path(path: &std::path::Path) -> Result<RegionFile, SaveError> {
        let read_primary = || {
            let bytes = fs::read(path)?;
            Self::decode_region(&bytes)
        };
        match read_primary() {
            Ok(region) => Ok(region),
            Err(SaveError::Io(error))
                if error.kind() == std::io::ErrorKind::NotFound
                    && persistence::has_valid_backup(path, Self::validate_region_bytes) =>
            {
                let bytes = persistence::read_backup_verified(path, Self::validate_region_bytes)?;
                Self::decode_region(&bytes)
            }
            Err(primary_error)
                if persistence::has_valid_backup(path, Self::validate_region_bytes) =>
            {
                log::warn!(
                    "[存档系统] Region 主文件损坏，正在恢复最近备份: {} ({primary_error})",
                    path.display()
                );
                persistence::restore_backup(path, Self::validate_region_bytes)?;
                read_primary()
            }
            Err(error) => Err(error),
        }
    }

    fn write_region_path(path: &std::path::Path, region: &RegionFile) -> Result<(), SaveError> {
        let bytes = Self::encode_region(region)?;
        persistence::atomic_write_verified(path, &bytes, Self::validate_region_bytes)?;
        Ok(())
    }

    fn encode_region(region: &RegionFile) -> Result<Vec<u8>, SaveError> {
        let serialized = bincode::DefaultOptions::new()
            .with_varint_encoding()
            .serialize(region)?;
        Self::compress(&serialized)
    }

    fn decode_region(data: &[u8]) -> Result<RegionFile, SaveError> {
        let decompressed = Self::decompress(data)?;
        let region = bincode::DefaultOptions::new()
            .with_varint_encoding()
            .reject_trailing_bytes()
            .deserialize::<RegionFile>(&decompressed)?;
        Self::validate_region_structure(&region)?;
        Ok(region)
    }

    fn validate_region_bytes(data: &[u8]) -> Result<(), String> {
        Self::decode_region(data)
            .map(|_| ())
            .map_err(|error| error.to_string())
    }

    fn validate_region_structure(region: &RegionFile) -> Result<(), SaveError> {
        let present_chunks: usize = region
            .header
            .chunk_present
            .iter()
            .map(|byte| byte.count_ones() as usize)
            .sum();
        if present_chunks != region.chunks.len() {
            return Err(SaveError::Serialize(format!(
                "Region 位图包含 {present_chunks} 个区块，但数据区包含 {} 个",
                region.chunks.len()
            )));
        }
        Ok(())
    }

    /// 查找区块索引
    fn find_chunk_index(region: &RegionFile, flat: usize) -> Result<usize, SaveError> {
        // 遍历位图，统计在 flat 之前存在的区块数量
        let mut count = 0;
        let total_bits = region.header.chunk_present.len() * 8;
        for i in 0..total_bits.min(flat) {
            let byte_idx = i / 8;
            let bit_idx = i % 8;
            if region.header.chunk_present[byte_idx] & (1 << bit_idx) != 0 {
                count += 1;
            }
        }
        Ok(count)
    }

    /// 计算标记存在区块数量
    fn count_present_before(region: &RegionFile, flat: usize) -> usize {
        let mut count = 0;
        let total_bits = region.header.chunk_present.len() * 8;
        for i in 0..total_bits.min(flat) {
            let byte_idx = i / 8;
            let bit_idx = i % 8;
            if region.header.chunk_present[byte_idx] & (1 << bit_idx) != 0 {
                count += 1;
            }
        }
        count
    }

    /// 编码并压缩带独立格式头的区块载荷。
    fn compress_chunk(chunk: &SavedChunk) -> Result<Vec<u8>, SaveError> {
        let encoded = encode_chunk_record(chunk)?;
        Self::compress(&encoded)
    }

    /// 压缩
    fn compress(data: &[u8]) -> Result<Vec<u8>, SaveError> {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::fast());
        encoder.write_all(data)?;
        encoder.finish().map_err(SaveError::Io)
    }

    /// 解压
    pub(crate) fn decompress(data: &[u8]) -> Result<Vec<u8>, SaveError> {
        let mut decoder = GzDecoder::new(data);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)?;
        Ok(decompressed)
    }

    fn decompress_chunk(data: &[u8]) -> Result<Vec<u8>, SaveError> {
        let mut decoder = GzDecoder::new(data).take((MAX_DECOMPRESSED_CHUNK_BYTES + 1) as u64);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)?;
        if decompressed.len() > MAX_DECOMPRESSED_CHUNK_BYTES {
            return Err(SaveError::Serialize(format!(
                "区块解压后超过 {} 字节上限",
                MAX_DECOMPRESSED_CHUNK_BYTES
            )));
        }
        Ok(decompressed)
    }
}
