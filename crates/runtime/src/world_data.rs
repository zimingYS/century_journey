use bevy::prelude::Resource;
use cj_core::pos::{ChunkPos, SectionPos};
use cj_core::rng::WorldRng;
use cj_core::spec::{SECTION_COUNT_USIZE, SECTION_VOLUME};
use cj_core::voxel::{BlockId, Voxel};
use cj_sim::worldgen::fill_section;
use cj_world::chunk::{ChunkStore, Section};
use cj_world::seed::WorldSeed;

/// 世界数据
#[derive(Resource)]
pub struct WorldData {
    /// 世界种子
    seed: WorldSeed,
    /// 区块数据存储
    chunks: ChunkStore,
}

impl WorldData {
    /// 由种子创建新世界
    pub fn new(seed: WorldSeed) -> Self {
        WorldData {
            seed,
            chunks: ChunkStore::default(),
        }
    }

    /// 获取世界种子
    #[inline]
    pub fn seed(&self) -> WorldSeed {
        self.seed
    }

    /// 获取区块数据
    #[inline]
    pub fn chunks(&self) -> &ChunkStore {
        &self.chunks
    }

    /// 生成初始区域
    pub fn generate_initial_area(&mut self, radius_chunks: i32) {
        let rng = self.seed.rng();
        let mut scratch = [BlockId::AIR; SECTION_VOLUME];

        for z in -radius_chunks..=radius_chunks {
            for x in -radius_chunks..=radius_chunks {
                self.fill_one_chunk(ChunkPos { x, z }, &rng, &mut scratch)
            }
        }
    }

    /// 填充一个区块
    fn fill_one_chunk(
        &mut self,
        chunk_pos: ChunkPos,
        rng: &WorldRng,
        scratch: &mut [BlockId; SECTION_VOLUME],
    ) {
        let chunk = self.chunks.get_or_default(chunk_pos);

        for section_index in 0..SECTION_COUNT_USIZE {
            // 将切片数据填充到缓冲区
            let section_pos = SectionPos::from_index(chunk_pos, section_index);
            fill_section(rng, section_pos, scratch);

            // 对空切片进行优化
            if scratch.iter().all(|&block| block == BlockId::AIR) {
                continue;
            }

            // 写入数据
            let mut section = Section::new();
            for (index, &block) in scratch.iter().enumerate() {
                section.set_raw(index, Voxel::cube(block));
            }
            chunk.insert_section(section_index, section);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cj_core::pos::BlockPos;
    use cj_core::spec::SEA_LEVEL;

    /// 半径 1 覆盖的方块范围：chunk -1..=1 ⇒ 方块 -16..=31
    fn area() -> std::ops::Range<i32> {
        -16..32
    }

    /// 读回指定世界坐标处的方块；未加载的 chunk 视为空气
    fn block_at(world: &WorldData, pos: BlockPos) -> BlockId {
        match world.chunks().get(pos.chunk_pos()) {
            Some(chunk) => chunk.get_block(pos).block(),
            None => BlockId::AIR,
        }
    }

    /// 自上而下找到该列最高的「实体」方块（非空气且非水），返回其世界 Y
    ///
    /// 搜索范围为海平面上下各 64 格，足以覆盖当前地形振幅。
    fn terrain_surface_y(world: &WorldData, x: i32, z: i32) -> Option<i32> {
        (SEA_LEVEL - 64..=SEA_LEVEL + 64).rev().find(|&y| {
            let block = block_at(world, BlockPos::new(x, y, z));
            !block.is_air() && block != BlockId::WATER
        })
    }

    /// 半径 1 的小世界（3×3 chunk），便于快速测试
    fn small_world(seed: u64) -> WorldData {
        let mut world = WorldData::new(WorldSeed::from_seed(seed));
        world.generate_initial_area(1);
        world
    }

    #[test]
    fn generate_area_creates_expected_chunk_count() {
        let mut world = WorldData::new(WorldSeed::from_seed(7));

        world.generate_initial_area(0);
        assert_eq!(world.chunks().len(), 1, "半径 0 应生成 1 个 chunk");

        world.generate_initial_area(1);
        assert_eq!(
            world.chunks().len(),
            9,
            "半径 1 应生成 3×3 = 9 个 chunk；重复生成不应重复计数"
        );
    }

    #[test]
    fn column_surface_is_grass_or_sand() {
        let world = small_world(11);

        for x in area() {
            for z in area() {
                let y = terrain_surface_y(&world, x, z)
                    .unwrap_or_else(|| panic!("({x},{z}) 在搜索范围内未找到实体方块"));
                let block = block_at(&world, BlockPos::new(x, y, z));

                assert!(
                    block == BlockId::GRASS || block == BlockId::SAND,
                    "({x},{z}) 的地表方块应为 GRASS（陆上）或 SAND（水下），实际为 {block:?}（y = {y}）"
                );
            }
        }
    }

    #[test]
    fn column_above_surface_follows_water_then_air_rule() {
        let world = small_world(11);

        for x in area() {
            for z in area() {
                let y = terrain_surface_y(&world, x, z).expect("应能找到地表");

                for wy in (y + 1)..=(SEA_LEVEL + 2) {
                    let expected = if wy <= SEA_LEVEL {
                        BlockId::WATER
                    } else {
                        BlockId::AIR
                    };

                    assert_eq!(
                        block_at(&world, BlockPos::new(x, wy, z)),
                        expected,
                        "({x},{z}) 在 y = {wy}（地表 {y}）处应为 {expected:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn column_below_surface_is_soil_then_stone() {
        let world = small_world(11);

        for x in area().step_by(3) {
            for z in area().step_by(3) {
                let y = terrain_surface_y(&world, x, z).expect("应能找到地表");

                for offset in 1..=3 {
                    assert_eq!(
                        block_at(&world, BlockPos::new(x, y - offset, z)),
                        BlockId::DIRT,
                        "({x},{z}) 地表下 {offset} 格应为 DIRT"
                    );
                }

                assert_eq!(
                    block_at(&world, BlockPos::new(x, y - 4, z)),
                    BlockId::STONE,
                    "({x},{z}) 地表下 4 格应为 STONE（土层厚度为 3）"
                );
            }
        }
    }

    #[test]
    fn same_seed_produces_identical_columns() {
        let a = small_world(42);
        let b = small_world(42);

        for x in area().step_by(5) {
            for z in area().step_by(5) {
                for y in (SEA_LEVEL - 32)..=(SEA_LEVEL + 32) {
                    assert_eq!(
                        block_at(&a, BlockPos::new(x, y, z)),
                        block_at(&b, BlockPos::new(x, y, z)),
                        "相同种子在 ({x},{y},{z}) 处产生了不同结果"
                    );
                }
            }
        }
    }

    #[test]
    fn different_seeds_produce_different_terrain() {
        let a = small_world(1);
        let b = small_world(2);

        let differs = area().any(|x| area().any(|z| surface_differs(&a, &b, x, z)));

        assert!(differs, "不同种子应产生不同地形");
    }

    fn surface_differs(a: &WorldData, b: &WorldData, x: i32, z: i32) -> bool {
        terrain_surface_y(a, x, z) != terrain_surface_y(b, x, z)
    }

    #[test]
    fn air_only_sections_are_skipped_and_solid_ones_are_allocated() {
        let world = small_world(3);
        let chunk_pos = ChunkPos { x: 0, z: 0 };
        let chunk = world.chunks().get(chunk_pos).expect("原点 chunk 应存在");

        let top_index = SectionPos::from_index(chunk_pos, SECTION_COUNT_USIZE - 1).index();
        assert!(
            chunk.section(top_index).is_none(),
            "世界顶部附近的纯空气 section 不应被分配（None 与 Some(AIR) 语义必须区分）"
        );

        let bottom_index = SectionPos::from_index(chunk_pos, 0).index();
        assert!(
            chunk.section(bottom_index).is_some(),
            "世界底部的全石头 section 应被分配"
        );
    }
}
