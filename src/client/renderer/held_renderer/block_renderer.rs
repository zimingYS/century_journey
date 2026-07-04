use crate::content::block::registry::BlockRegistry;
use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};

/// 方块手持渲染
pub struct HeldBlockRenderer;

impl HeldBlockRenderer {
    /// 构建完成的网格对象
    pub fn build_mesh(block_registry: &BlockRegistry, block_identifier: &str) -> Option<Mesh> {
        // 获取动态ID
        let runtime_id = block_registry.get_id_by_identifier(block_identifier)?;
        // 获取纹理图集
        let total_layers = block_registry.total_layer_count().max(1);

        // 构建面索引重映射表
        const FACE_REMAP: [usize; 6] = [3, 2, 0, 1, 4, 5];
        // 通过重映射索引查询每个面对应的纹理层编号
        let face_layers: [u32; 6] =
            std::array::from_fn(|i| block_registry.get_layer(runtime_id, FACE_REMAP[i]));
        let total = total_layers as f32 * 16.0;

        // 计算每个面的UV坐标矩形
        let face_uvs: [(f32, f32, f32, f32); 6] = std::array::from_fn(|i| {
            let layer = face_layers[i] as f32;
            let row = layer * 16.0;
            (0.0, row / total, 1.0 / 16.0, (row + 1.0) / total)
        });

        Some(build_uv_mesh(&face_uvs))
    }

    /// 创建手持方块材质
    pub fn create_material(
        materials: &mut ResMut<Assets<StandardMaterial>>,
        block_registry: &BlockRegistry,
    ) -> Handle<StandardMaterial> {
        materials.add(StandardMaterial {
            base_color_texture: Some(block_registry.base_texture().clone()),
            perceptual_roughness: 0.85,
            ..default()
        })
    }
}

/// 底层网格构建
fn build_uv_mesh(face_uvs: &[(f32, f32, f32, f32); 6]) -> Mesh {
    let h = 0.5;
    // 顶点位置
    let positions = [
        [h, -h, -h],
        [h, h, -h],
        [h, h, h],
        [h, -h, h],
        [-h, -h, h],
        [-h, h, h],
        [-h, h, -h],
        [-h, -h, -h],
        [-h, h, -h],
        [-h, h, h],
        [h, h, h],
        [h, h, -h],
        [-h, -h, h],
        [-h, -h, -h],
        [h, -h, -h],
        [h, -h, h],
        [h, -h, h],
        [h, h, h],
        [-h, h, h],
        [-h, -h, h],
        [h, -h, -h],
        [-h, -h, -h],
        [-h, h, -h],
        [h, h, -h],
    ];
    // 顶点法线
    let normals = [
        [1.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [-1.0, 0.0, 0.0],
        [-1.0, 0.0, 0.0],
        [-1.0, 0.0, 0.0],
        [-1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, -1.0, 0.0],
        [0.0, -1.0, 0.0],
        [0.0, -1.0, 0.0],
        [0.0, -1.0, 0.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, -1.0],
        [0.0, 0.0, -1.0],
        [0.0, 0.0, -1.0],
        [0.0, 0.0, -1.0],
    ];

    // 构建UV坐标和索引
    let uvs: Vec<[f32; 2]> = (0..6)
        .flat_map(|f| {
            let (u0, v0, u1, v1) = face_uvs[f];
            [[u0, v1], [u0, v0], [u1, v0], [u1, v1]]
        })
        .collect();
    let indices: Vec<u32> = (0..6)
        .flat_map(|f| {
            let b = f as u32 * 4;
            [b, b + 1, b + 2, b + 2, b + 3, b]
        })
        .collect();

    // 构建网格
    let usage = RenderAssetUsages::default();
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, usage);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions.to_vec());
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals.to_vec());
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}
