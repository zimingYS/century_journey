use crate::input::BlockView;
use cj_core::pos::{BlockPos, SectionPos};
use cj_core::spec::{CHUNK_SIZE, SECTION_SIZE};
use cj_core::voxel::BlockId;

/// 单个网格数据
#[derive(Debug, Default)]
pub struct MeshData {
    // 顶点位置。
    pub positions: Vec<[f32; 3]>,
    /// 顶点法线
    pub normals: Vec<[f32; 3]>,
    /// 顶点颜色
    pub colors: Vec<[f32; 4]>,
    /// 三角形索引
    pub indices: Vec<u32>,
}

impl MeshData {
    /// 获取生成的四边形数量
    #[inline]
    pub fn quad_count(&self) -> usize {
        self.indices.len() / 6
    }

    /// 判断网格是否为空
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.positions.is_empty()
    }
}

/// 单个面的几何定义
#[derive(Clone, Copy)]
struct Face {
    /// 面朝向
    normal: [i32; 3],

    /// 面的四个角
    /// 顺序为从外侧看逆时针
    corners: [[i32; 3]; 4],
}

/// 六个立方体面
const FACES: [Face; 6] = [
    // -X
    Face {
        normal: [-1, 0, 0],
        corners: [[0, 0, 0], [0, 0, 1], [0, 1, 1], [0, 1, 0]],
    },
    // +X
    Face {
        normal: [1, 0, 0],
        corners: [[1, 0, 0], [1, 1, 0], [1, 1, 1], [1, 0, 1]],
    },
    // -Y
    Face {
        normal: [0, -1, 0],
        corners: [[0, 0, 0], [1, 0, 0], [1, 0, 1], [0, 0, 1]],
    },
    // +Y
    Face {
        normal: [0, 1, 0],
        corners: [[0, 1, 0], [0, 1, 1], [1, 1, 1], [1, 1, 0]],
    },
    // -Z
    Face {
        normal: [0, 0, -1],
        corners: [[0, 0, 0], [0, 1, 0], [1, 1, 0], [1, 0, 0]],
    },
    // +Z
    Face {
        normal: [0, 0, 1],
        corners: [[0, 0, 1], [1, 0, 1], [1, 1, 1], [0, 1, 1]],
    },
];

/// 调试用方块颜色
/// 此处为临时实现，后续改由方块纹理实现
fn block_color(block: BlockId) -> [f32; 4] {
    match block {
        BlockId::STONE => [0.55, 0.55, 0.58, 1.0],
        BlockId::DIRT => [0.45, 0.30, 0.16, 1.0],
        BlockId::GRASS => [0.35, 0.60, 0.25, 1.0],
        BlockId::SAND => [0.82, 0.74, 0.48, 1.0],
        BlockId::WATER => [0.25, 0.50, 0.85, 1.0],
        _ => [1.0, 1.0, 1.0, 1.0],
    }
}

/// 构建一个区块切片的体素网格
pub fn build_section_mesh<V: BlockView>(view: &V, section: SectionPos) -> MeshData {
    let mut mesh = MeshData::default();

    // Section 在世界中的原点。
    let origin = BlockPos::new(
        section.chunk.x * CHUNK_SIZE,
        section.y * SECTION_SIZE,
        section.chunk.z * CHUNK_SIZE,
    );

    for y in 0..SECTION_SIZE {
        for z in 0..SECTION_SIZE {
            for x in 0..SECTION_SIZE {
                let local_x = x;
                let local_y = y;
                let local_z = z;

                let world_pos =
                    BlockPos::new(origin.x + local_x, origin.y + local_y, origin.z + local_z);

                let block = view.block(world_pos);

                // AIR 不产生几何。
                if block == BlockId::AIR {
                    continue;
                }

                let color = block_color(block);

                for face in FACES {
                    let neighbor_pos = BlockPos::new(
                        world_pos.x + face.normal[0],
                        world_pos.y + face.normal[1],
                        world_pos.z + face.normal[2],
                    );

                    let neighbor = view.block(neighbor_pos);

                    // 遮挡规则由 core 决定，这里不重复实现。
                    if neighbor.occludes() {
                        continue;
                    }

                    let base_index = mesh.positions.len() as u32;

                    for corner in face.corners {
                        mesh.positions.push([
                            (local_x + corner[0]) as f32,
                            (local_y + corner[1]) as f32,
                            (local_z + corner[2]) as f32,
                        ]);

                        mesh.normals.push([
                            face.normal[0] as f32,
                            face.normal[1] as f32,
                            face.normal[2] as f32,
                        ]);

                        mesh.colors.push(color);
                    }

                    mesh.indices.extend_from_slice(&[
                        base_index,
                        base_index + 1,
                        base_index + 2,
                        base_index,
                        base_index + 2,
                        base_index + 3,
                    ]);
                }
            }
        }
    }

    mesh
}

#[cfg(test)]
mod tests {
    use super::*;
    use cj_core::pos::ChunkPos;
    use std::collections::HashMap;

    struct TestView {
        blocks: HashMap<BlockPos, BlockId>,
    }

    impl TestView {
        fn new() -> Self {
            Self {
                blocks: HashMap::new(),
            }
        }

        fn with(mut self, x: i32, y: i32, z: i32) -> Self {
            self.blocks.insert(BlockPos::new(x, y, z), BlockId::STONE);
            self
        }

        fn fill(mut self, size: i32) -> Self {
            for y in 0..size {
                for z in 0..size {
                    for x in 0..size {
                        self.blocks.insert(BlockPos::new(x, y, z), BlockId::STONE);
                    }
                }
            }
            self
        }
    }

    impl BlockView for TestView {
        fn block(&self, pos: BlockPos) -> BlockId {
            self.blocks.get(&pos).copied().unwrap_or(BlockId::AIR)
        }
    }

    fn test_section() -> SectionPos {
        SectionPos::from_index(ChunkPos { x: 0, z: 0 }, 16)
    }

    fn cross_of(e1: [f32; 3], e2: [f32; 3]) -> [f32; 3] {
        [
            e1[1] * e2[2] - e1[2] * e2[1],
            e1[2] * e2[0] - e1[0] * e2[2],
            e1[0] * e2[1] - e1[1] * e2[0],
        ]
    }

    #[test]
    fn empty_view_produces_no_geometry() {
        let mesh = build_section_mesh(&TestView::new(), test_section());

        assert!(mesh.is_empty(), "空视图不应产生任何几何");
        assert_eq!(mesh.quad_count(), 0, "空视图的四边形数应为 0");
    }

    #[test]
    fn single_block_produces_six_quads() {
        let mesh = build_section_mesh(&TestView::new().with(0, 0, 0), test_section());

        assert_eq!(mesh.quad_count(), 6, "孤立方块应产生 6 个面");
        assert_eq!(mesh.positions.len(), 24, "6 个面应产生 24 个顶点");
        assert_eq!(mesh.indices.len(), 36, "6 个面应产生 36 个索引");
    }

    #[test]
    fn adjacent_blocks_cull_shared_face() {
        let view = TestView::new().with(0, 0, 0).with(1, 0, 0);
        let mesh = build_section_mesh(&view, test_section());

        assert_eq!(
            mesh.quad_count(),
            10,
            "相邻两格应剔除两个共用面：12 − 2 = 10"
        );
    }

    #[test]
    fn neighbor_outside_section_culls_boundary_face() {
        let view = TestView::new().with(0, 0, 0).with(-1, 0, 0);
        let mesh = build_section_mesh(&view, test_section());

        assert_eq!(
            mesh.quad_count(),
            5,
            "跨 section 边界的邻居也应参与剔除，本节内那一格只剩 5 个面"
        );
    }

    #[test]
    fn solid_two_cube_produces_twenty_four_quads() {
        let mesh = build_section_mesh(&TestView::new().fill(2), test_section());

        assert_eq!(
            mesh.quad_count(),
            24,
            "2×2×2 实心块的外表面应为 6 × 4 = 24 个面"
        );
    }

    #[test]
    fn solid_section_produces_1536_quads() {
        let mesh = build_section_mesh(&TestView::new().fill(16), test_section());

        assert_eq!(
            mesh.quad_count(),
            1536,
            "16³ 实心段只应剩 6 个外壁 × 16² = 1536 个面 —— 内部面若未剔除会暴涨"
        );
    }

    #[test]
    fn face_winding_matches_normal() {
        let mesh = build_section_mesh(&TestView::new().fill(2), test_section());

        for quad in 0..mesh.quad_count() {
            let base = quad * 4;
            let p0 = mesh.positions[base];
            let p1 = mesh.positions[base + 1];
            let p2 = mesh.positions[base + 2];
            let normal = mesh.normals[base];

            let e1 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
            let e2 = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];
            let cross = cross_of(e1, e2);

            for axis in 0..3 {
                assert!(
                    (cross[axis] - normal[axis]).abs() < 1e-5,
                    "第 {quad} 个面的绕序与法线不一致：叉积 {cross:?}，法线 {normal:?}"
                );
            }
        }
    }

    #[test]
    fn buffers_are_consistent() {
        let mesh = build_section_mesh(&TestView::new().fill(3), test_section());

        assert_eq!(
            mesh.positions.len(),
            mesh.quad_count() * 4,
            "顶点数应为面数的 4 倍"
        );
        assert_eq!(
            mesh.indices.len(),
            mesh.quad_count() * 6,
            "索引数应为面数的 6 倍"
        );
        assert_eq!(
            mesh.positions.len(),
            mesh.normals.len(),
            "法线数应与顶点数一致"
        );
        assert_eq!(
            mesh.positions.len(),
            mesh.colors.len(),
            "颜色数应与顶点数一致"
        );

        for index in &mesh.indices {
            assert!(
                (*index as usize) < mesh.positions.len(),
                "索引 {index} 越界（顶点数 {}）",
                mesh.positions.len()
            );
        }
    }

    #[test]
    fn vertices_use_local_coordinates() {
        let mesh = build_section_mesh(&TestView::new().with(15, 15, 15), test_section());

        assert!(!mesh.is_empty(), "角落方块应产生几何");

        for position in &mesh.positions {
            for axis in position {
                assert!(
                    (0.0..=16.0).contains(axis),
                    "顶点应使用 0..16 的局部坐标，实际为 {position:?}"
                );
            }
        }
    }

    #[test]
    fn air_neighbor_keeps_face_visible() {
        let mesh = build_section_mesh(&TestView::new().with(0, 0, 0), test_section());

        assert_eq!(mesh.quad_count(), 6, "四面皆空气时 6 个面都应保留");
        assert_eq!(mesh.normals.len(), 24, "每个顶点都应带法线");
    }
}
