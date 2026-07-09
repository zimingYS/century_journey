use crate::client::renderer::tex_atlas::BlockRenderAssets;
use crate::content::block::registry::BlockRegistry;
use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};

/// 立方体固定 6 个面。
const FACE_COUNT: usize = 6;
/// 方块 atlas 每行瓦片数。
const ATLAS_TILES_PER_ROW: f32 = 16.0;

/// 方块物品模型的 mesh 构建器。
///
/// 这个构建器只服务于“物品模型里的方块”，不直接复用世界区块 mesh，方便之后接入 Minecraft 风格 BakedModel。
pub struct BlockCubeMeshBuilder;

impl BlockCubeMeshBuilder {
    /// 根据方块注册表生成一个带 atlas UV 的 1x1x1 物品立方体 mesh。
    pub fn build_mesh(block_registry: &BlockRegistry, block_identifier: &str) -> Option<Mesh> {
        let runtime_id = block_registry.get_id_by_identifier(block_identifier)?;
        let total_layers = block_registry.total_layer_count().max(1) as f32;
        let face_layers =
            std::array::from_fn(|face_idx| block_registry.get_layer(runtime_id, face_idx) as f32);

        Some(build_cube_mesh(face_layers, total_layers))
    }

    /// 取得该方块物品应使用的 atlas 材质。
    pub fn material(
        block_registry: &BlockRegistry,
        render_assets: &BlockRenderAssets,
        block_identifier: &str,
    ) -> Option<Handle<StandardMaterial>> {
        let runtime_id = block_registry.get_id_by_identifier(block_identifier)?;
        let render_mode = block_registry.get(runtime_id)?.render_mode;
        Some(render_assets.material(render_mode).clone())
    }
}

/// 构建带 UV、法线和顶点色的方块物品 mesh。
fn build_cube_mesh(face_layers: [f32; FACE_COUNT], total_layers: f32) -> Mesh {
    let mut positions = Vec::with_capacity(FACE_COUNT * 4);
    let mut normals = Vec::with_capacity(FACE_COUNT * 4);
    let mut uvs = Vec::with_capacity(FACE_COUNT * 4);
    let mut colors = Vec::with_capacity(FACE_COUNT * 4);
    let mut indices = Vec::with_capacity(FACE_COUNT * 6);

    for face_idx in 0..FACE_COUNT {
        let face = item_cube_face(face_idx);
        let face_uvs = face_uvs(face_idx, face_layers[face_idx], total_layers);
        let face_color = face_color(face_idx);
        let base = positions.len() as u32;

        positions.extend_from_slice(&face.positions);
        normals.extend_from_slice(&[face.normal; 4]);
        uvs.extend_from_slice(&face_uvs);
        colors.extend_from_slice(&[face_color; 4]);
        indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

/// 单个 cube 面的数据。
struct CubeFace {
    /// 面上 4 个顶点位置。
    positions: [[f32; 3]; 4],
    /// 面法线。
    normal: [f32; 3],
}

/// 取得指定面的位置和法线。
fn item_cube_face(face_idx: usize) -> CubeFace {
    let h = 0.5;
    match face_idx {
        0 => CubeFace {
            positions: [[h, h, -h], [-h, h, -h], [-h, h, h], [h, h, h]],
            normal: [0.0, 1.0, 0.0],
        },
        1 => CubeFace {
            positions: [[-h, -h, -h], [h, -h, -h], [h, -h, h], [-h, -h, h]],
            normal: [0.0, -1.0, 0.0],
        },
        2 => CubeFace {
            positions: [[-h, h, h], [-h, h, -h], [-h, -h, -h], [-h, -h, h]],
            normal: [-1.0, 0.0, 0.0],
        },
        3 => CubeFace {
            positions: [[h, h, -h], [h, h, h], [h, -h, h], [h, -h, -h]],
            normal: [1.0, 0.0, 0.0],
        },
        4 => CubeFace {
            positions: [[h, h, h], [-h, h, h], [-h, -h, h], [h, -h, h]],
            normal: [0.0, 0.0, 1.0],
        },
        5 => CubeFace {
            positions: [[-h, h, -h], [h, h, -h], [h, -h, -h], [-h, -h, -h]],
            normal: [0.0, 0.0, -1.0],
        },
        _ => unreachable!(),
    }
}

/// 为不同面写入接近 Minecraft 方块图标的固定亮度。
///
/// GUI 预览使用 unlit 材质，所以这些顶点色就是图标中的稳定“光照”。
fn face_color(face_idx: usize) -> [f32; 4] {
    let brightness = match face_idx {
        0 => 1.0,
        1 => 0.52,
        2 => 0.72,
        3 => 0.86,
        4 => 0.80,
        5 => 0.66,
        _ => 1.0,
    };
    [brightness, brightness, brightness, 1.0]
}

/// 根据方块 atlas 层计算单个面的 UV。
fn face_uvs(face_idx: usize, layer: f32, total_layers: f32) -> [[f32; 2]; 4] {
    let u0 = 0.0;
    let u1 = 1.0 / ATLAS_TILES_PER_ROW;
    let v0 = layer / total_layers;
    let v1 = (layer * ATLAS_TILES_PER_ROW + 1.0) / (total_layers * ATLAS_TILES_PER_ROW);

    match face_idx {
        0 | 2 | 4 => [[u1, v0], [u0, v0], [u0, v1], [u1, v1]],
        1 | 3 | 5 => [[u0, v0], [u1, v0], [u1, v1], [u0, v1]],
        _ => unreachable!(),
    }
}
