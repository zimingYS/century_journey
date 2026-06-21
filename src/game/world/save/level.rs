use std::io::{Read, Write};
use bevy::prelude::*;
use bincode::Options;
use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use crate::content::block::registry::BlockRegistry;
use crate::game::world::save::format::LevelData;
use crate::game::world::save::region::{RegionManager, SaveError};

/// 检测存档是否存在
pub fn world_exists(world_name: &str) -> bool {
    RegionManager::level_path(world_name).exists()
}

/// 保存世界配置数据到level.dat
pub fn save_level(
    world_name: &str,
    seed: u64,
    spawn_pos: Vec3,
    time_of_day: f32,
    block_registry: &BlockRegistry,
)-> Result<(), SaveError> {
    // 确保当前世界文件存在
    RegionManager::ensure_dirs(world_name)?;

    // 构建ID映射表(将动态ID转换为方块标识符)
    let block_id_map = block_registry.build_save_id_map();

    let level = LevelData {
        seed,
        spawn_position: [spawn_pos.x, spawn_pos.y, spawn_pos.z],
        time_of_day,
        block_id_map,
        version: LevelData::CURRENT_VERSION,
    };

    // 获取保存路径
    let path = RegionManager::level_path(world_name);
    // 序列化存档数据
    let serialized = bincode::DefaultOptions::new()
        .with_varint_encoding()
        .serialize(&level)?;

    // 压缩存档数据
    let mut encoder = GzEncoder::new(Vec::new(), Compression::fast());
    encoder.write_all(&serialized)?;
    let compressed = encoder.finish()?;

    // 写入保存
    std::fs::write(&path, compressed)?;
    Ok(())
}

/// 从level.dat加载世界元数据
pub fn load_level(world_name: &str) -> Result<LevelData, SaveError> {
    // 读取世界文件
    let path = RegionManager::level_path(world_name);
    let compressed = std::fs::read(&path)?;

    // 解压数据
    let mut decoder = GzDecoder::new(&compressed[..]);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed)?;

    // 反序列化
    let level: LevelData = bincode::DefaultOptions::new()
        .with_varint_encoding()
        .deserialize(&decompressed)?;

    // 版本兼容性检查
    if level.version > LevelData::CURRENT_VERSION {
        log::warn!(
            "存档版本 {} 高于当前支持的 {}，可能存在兼容性问题",
            level.version,
            LevelData::CURRENT_VERSION
        );
    }

    Ok(level)
}

// 根据存档中的ID映射表，将区块数据中的方块ID重映射到当前运行时ID
pub fn remap_chunk_block_ids(
    chunk_data: &mut crate::game::world::chunk::ChunkData,
    saved_id_map: &[(u16, String)],
    current_registry: &BlockRegistry,
) {
    // 构建映射：保存时的 runtime_id -> 当前的 runtime_id
    let remap = current_registry.build_id_remap_table(saved_id_map);

    // 重映射每个方块
    for voxel in chunk_data.voxels.iter_mut() {
        if let Some(&new_id) = remap.get(voxel) {
            *voxel = new_id;
        } else {
            // 未知方块 ID，回退为空气
            log::warn!("未知的方块 ID {}，替换为空气", voxel);
            *voxel = 0;
        }
    }
}