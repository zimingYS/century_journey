use super::*;
use crate::game::world::generation::terrain::context::{ChunkGenContext, ColumnContext};

/// 构造所有列地表高度一致的区块上下文。
fn context_with_height(chunk_pos: IVec3, height: i32) -> ChunkGenContext {
    let mut ctx = ChunkGenContext::new(chunk_pos);
    for x in 0..CHUNK_SIZE {
        for z in 0..CHUNK_SIZE {
            ctx.columns.push(ColumnContext {
                world_x: chunk_pos.x * CHUNK_SIZE as i32 + x as i32,
                world_z: chunk_pos.z * CHUNK_SIZE as i32 + z as i32,
                temperature: 0.5,
                humidity: 0.5,
                biome_index: 0,
                base_height: height,
                roughness: 0.0,
            });
        }
    }
    ctx
}

fn all_stone_chunk() -> ChunkData {
    let mut data = ChunkData::new();
    for y in 0..CHUNK_SIZE {
        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                data.set_voxel(x, y, z, 3); // 3 = stone
            }
        }
    }
    data
}

fn count_block(data: &ChunkData, id: u16) -> usize {
    data.voxels.iter().filter(|&&v| v == id).count()
}

fn permissive_profile(min_ceiling: i32) -> CaveProfile {
    CaveProfile {
        threshold: 1.0,
        scale: 0.05,
        min_y: -100,
        min_ceiling,
    }
}

#[test]
fn apply_caves_only_carves_stone() {
    let mut data = all_stone_chunk();
    // 把一个位置换成泥土(2)，验证洞穴不会挖非石头方块
    data.set_voxel(0, 0, 0, 2);
    let ctx = context_with_height(IVec3::ZERO, 40);

    apply_caves(
        &mut data,
        &ctx,
        &NoiseSampler::new(42),
        3,
        &permissive_profile(0),
    );

    assert_eq!(data.get_voxel(0, 0, 0), 2, "dirt must not be carved");
    // threshold = 1.0 时所有石头都应被挖空
    assert_eq!(
        count_block(&data, 0),
        CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE - 1
    );
}

#[test]
fn apply_caves_respects_min_depth() {
    let mut data = all_stone_chunk();
    let ctx = context_with_height(IVec3::ZERO, 40);
    let profile = CaveProfile {
        threshold: 1.0,
        scale: 0.05,
        min_y: 50, // 高于区块内全部 y，深度保护应拦截一切
        min_ceiling: 0,
    };

    apply_caves(&mut data, &ctx, &NoiseSampler::new(42), 3, &profile);

    assert_eq!(
        count_block(&data, 3),
        CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE,
        "below min_y nothing may be carved"
    );
}

#[test]
fn apply_caves_respects_min_ceiling() {
    let mut data = all_stone_chunk();
    // 地表高度 10，min_ceiling 4：y >= 10-4=6 的石头保留
    let ctx = context_with_height(IVec3::ZERO, 10);

    apply_caves(
        &mut data,
        &ctx,
        &NoiseSampler::new(42),
        3,
        &permissive_profile(4),
    );

    let carved = count_block(&data, 0);
    let kept = count_block(&data, 3);
    assert_eq!(carved, 6 * CHUNK_SIZE * CHUNK_SIZE, "y in 0..6 carved");
    assert_eq!(kept, 10 * CHUNK_SIZE * CHUNK_SIZE, "y in 6..16 kept");
}

#[test]
fn apply_caves_threshold_minus_one_carves_nothing() {
    let mut data = all_stone_chunk();
    let ctx = context_with_height(IVec3::ZERO, 40);
    let profile = CaveProfile {
        threshold: -1.0, // Perlin 值域 [-1,1]，n < -1 恒为假
        scale: 0.05,
        min_y: -100,
        min_ceiling: 0,
    };

    apply_caves(&mut data, &ctx, &NoiseSampler::new(42), 3, &profile);

    assert_eq!(
        count_block(&data, 3),
        CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE,
        "threshold -1.0 must never carve"
    );
}

#[test]
fn apply_caves_is_deterministic_for_same_seed_and_chunk() {
    let ctx = context_with_height(IVec3::ZERO, 40);
    let profile = CaveProfile {
        threshold: -0.30,
        scale: 0.05,
        min_y: -100,
        min_ceiling: 0,
    };
    let mut first = all_stone_chunk();
    let mut second = all_stone_chunk();

    apply_caves(&mut first, &ctx, &NoiseSampler::new(42), 3, &profile);
    apply_caves(&mut second, &ctx, &NoiseSampler::new(42), 3, &profile);

    assert_eq!(first.voxels, second.voxels);
}

#[test]
fn apply_caves_different_seeds_produce_different_carving() {
    let ctx = context_with_height(IVec3::ZERO, 40);
    let profile = CaveProfile {
        threshold: 0.0,
        scale: 0.05,
        min_y: -100,
        min_ceiling: 0,
    };
    let mut first = all_stone_chunk();
    let mut second = all_stone_chunk();

    apply_caves(&mut first, &ctx, &NoiseSampler::new(42), 3, &profile);
    apply_caves(&mut second, &ctx, &NoiseSampler::new(43), 3, &profile);

    // 阈值 0.0 下必然有挖空；确定性种子下不同种子的分布不同
    assert!(count_block(&first, 0) > 0);
    assert!(count_block(&second, 0) > 0);
    assert_ne!(first.voxels, second.voxels);
}
