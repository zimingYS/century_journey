use super::*;
use crate::content::ore_vein::definition::OreVeinDefinition;
use crate::shared::identifier::Identifier;

fn vein(
    identifier: &str,
    block_id: u16,
    priority: u32,
    min_y: i32,
    max_y: i32,
    threshold: f64,
    scale: f64,
) -> RuntimeOreVein {
    RuntimeOreVein {
        definition: OreVeinDefinition {
            identifier: Identifier::parse(identifier).unwrap(),
            display_name: identifier.into(),
            block: Identifier::parse("test:ore").unwrap(),
            priority,
            min_y,
            max_y,
            threshold,
            scale,
        },
        block_id,
    }
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

#[test]
fn apply_ores_only_replaces_stone() {
    let mut data = all_stone_chunk();
    // 把一个位置换成泥土(2)，验证矿石不会覆盖非石头方块
    data.set_voxel(0, 0, 0, 2);
    let veins = [vein("test:coal", 9, 1, 0, 64, 1.0, 0.1)];

    apply_ores(&mut data, IVec3::ZERO, &NoiseSampler::new(42), 3, &veins);

    assert_eq!(data.get_voxel(0, 0, 0), 2, "dirt must not be replaced");
    // threshold = 1.0 时所有石头都应被替换
    assert!(count_block(&data, 9) == CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE - 1);
}

#[test]
fn apply_ores_respects_depth_band() {
    let mut data = all_stone_chunk();
    // 深度带只覆盖 y=0 一层
    let veins = [vein("test:coal", 9, 1, 0, 0, 1.0, 0.1)];

    apply_ores(&mut data, IVec3::ZERO, &NoiseSampler::new(42), 3, &veins);

    assert_eq!(count_block(&data, 9), CHUNK_SIZE * CHUNK_SIZE);
    assert_eq!(data.get_voxel(0, 1, 0), 3, "above the band must stay stone");
}

#[test]
fn apply_ores_higher_priority_vein_wins_overlap() {
    let mut data = all_stone_chunk();
    // veins 契约：按优先级从高到低排序传入（与注册表输出一致）
    let veins = [
        vein("test:gold", 11, 3, 0, 64, 1.0, 0.1),
        vein("test:coal", 9, 1, 0, 64, 1.0, 0.1),
    ];

    apply_ores(&mut data, IVec3::ZERO, &NoiseSampler::new(42), 3, &veins);

    // 高优先级(3)的金矿全部命中，低优先级(1)的煤矿无机会
    assert_eq!(count_block(&data, 11), CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE);
    assert_eq!(count_block(&data, 9), 0);
}

#[test]
fn apply_ores_uses_vein_scale_for_sampling() {
    let mut data = all_stone_chunk();
    // 极小 scale 使噪声在世界坐标上变化缓慢，threshold 保证全部命中
    let veins = [vein("test:coal", 9, 1, 0, 64, 1.0, 0.001)];

    apply_ores(&mut data, IVec3::ZERO, &NoiseSampler::new(42), 3, &veins);

    assert_eq!(count_block(&data, 9), CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE);
}

#[test]
fn apply_ores_different_seeds_produce_different_placements() {
    let veins = [vein("test:coal", 9, 1, 0, 64, 0.5, 0.1)];
    let mut first = all_stone_chunk();
    let mut second = all_stone_chunk();

    apply_ores(&mut first, IVec3::ZERO, &NoiseSampler::new(42), 3, &veins);
    apply_ores(&mut second, IVec3::ZERO, &NoiseSampler::new(43), 3, &veins);

    // 阈值 0.5 下必然有矿；确定性种子下不同种子的分布不同
    assert!(count_block(&first, 9) > 0);
    assert!(count_block(&second, 9) > 0);
    assert_ne!(first.voxels, second.voxels);
}
