use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;
use bevy::prelude::*;
use bincode::Options;
use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use crate::core::constant::world::*;
use crate::world::save::format::{chunk_local_index, chunk_to_region_pos, local_index_to_flat, RegionFile, RegionHeader, SavedChunk};

/// Region 文件的读写管理器
pub struct RegionManager;

/// 存档错误处理
#[derive(Debug)]
pub enum SaveError {
    /// 文件读写错误
    Io(std::io::Error),
    /// 序列化/反序列化错误
    Serialize(String),
}

/// 错误显示输出
impl std::fmt::Display for SaveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SaveError::Io(e) => write!(f, "IO error: {e}"),
            SaveError::Serialize(e) => write!(f, "Serialize error: {e}"),
        }
    }
}

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

// 对外接口
impl RegionManager {
    /// 获取存档根路径
    pub fn save_root(world_name: &str) -> PathBuf {
        PathBuf::from(SAVE_DIR_NAME).join(world_name)
    }

    /// 获取 region 文件路径
    pub fn region_path(world_name: &str, region_pos: IVec3) -> PathBuf {
        Self::save_root(world_name)
            .join(REGION_DIR_NAME)
            .join(format!(
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
    pub fn read_chunk(
        world_name: &str,
        chunk_pos: IVec3,
    ) -> Result<Option<SavedChunk>, SaveError> {
        let region_pos = chunk_to_region_pos(chunk_pos);
        let path = Self::region_path(world_name, region_pos);

        if !path.exists() {
            return Ok(None);
        }

        let mut file = fs::File::open(&path)?;
        let region = Self::read_region_file(&mut file)?;

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
        let decompressed = Self::decompress(compressed)?;
        let saved: SavedChunk = bincode::DefaultOptions::new()
            .with_varint_encoding()
            .deserialize(&decompressed)?;

        Ok(Some(saved))
    }

    /// 写入单个区块（读取整个 region → 修改 → 写回）
    pub fn write_chunk(
        world_name: &str,
        chunk: &SavedChunk,
    ) -> Result<(), SaveError> {
        let region_pos = chunk_to_region_pos(chunk.position);
        let path = Self::region_path(world_name, region_pos);
        Self::ensure_dirs(world_name)?;

        let mut region = if path.exists() {
            let mut file = fs::File::open(&path)?;
            Self::read_region_file(&mut file)?
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

        let mut file = fs::File::create(&path)?;
        Self::write_region_file(&mut file, &region)?;

        Ok(())
    }

    /// 批量写入同一 Region 的多个区块（只读写一次文件）
    pub fn write_chunks_batch(
        world_name: &str,
        chunks: &[SavedChunk],
    ) -> Result<(), SaveError> {
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

            let mut region = if path.exists() {
                let mut file = fs::File::open(&path)?;
                Self::read_region_file(&mut file)?
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
                let was_present =
                    region.header.chunk_present[byte_idx] & (1 << bit_idx) != 0;

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

            let mut file = fs::File::create(&path)?;
            Self::write_region_file(&mut file, &region)?;
        }

        Ok(())
    }

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
    /// Region文件读取
    pub(crate) fn read_region_file(file: &mut fs::File) -> Result<RegionFile, SaveError> {
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;

        // 先解压整个 region 文件
        let decompressed = Self::decompress(&data)?;
        let region: RegionFile = bincode::DefaultOptions::new()
            .with_varint_encoding()
            .deserialize(&decompressed)?;

        Ok(region)
    }

    /// Region文件写入
    fn write_region_file(
        file: &mut fs::File,
        region: &RegionFile,
    ) -> Result<(), SaveError> {
        let serialized = bincode::DefaultOptions::new()
            .with_varint_encoding()
            .serialize(region)?;

        let compressed = Self::compress(&serialized)?;
        file.write_all(&compressed)?;
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

    /// 压缩区块
    fn compress_chunk(chunk: &SavedChunk) -> Result<Vec<u8>, SaveError> {
        let serialized = bincode::DefaultOptions::new()
            .with_varint_encoding()
            .serialize(chunk)?;
        Self::compress(&serialized)
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
}