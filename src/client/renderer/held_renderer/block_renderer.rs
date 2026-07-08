use crate::client::renderer::tex_atlas::BlockRenderAssets;
use crate::content::block::registry::BlockRegistry;
use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};

const FACE_COUNT: usize = 6;
const ATLAS_TILES_PER_ROW: f32 = 16.0;

pub struct HeldBlockRenderer;

impl HeldBlockRenderer {
    pub fn build_mesh(block_registry: &BlockRegistry, block_identifier: &str) -> Option<Mesh> {
        let runtime_id = block_registry.get_id_by_identifier(block_identifier)?;
        let total_layers = block_registry.total_layer_count().max(1) as f32;
        let face_layers =
            std::array::from_fn(|face_idx| block_registry.get_layer(runtime_id, face_idx) as f32);

        Some(build_cube_mesh(face_layers, total_layers))
    }

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

fn build_cube_mesh(face_layers: [f32; FACE_COUNT], total_layers: f32) -> Mesh {
    let mut positions = Vec::with_capacity(FACE_COUNT * 4);
    let mut normals = Vec::with_capacity(FACE_COUNT * 4);
    let mut uvs = Vec::with_capacity(FACE_COUNT * 4);
    let mut indices = Vec::with_capacity(FACE_COUNT * 6);

    for face_idx in 0..FACE_COUNT {
        let face = held_cube_face(face_idx);
        let face_uvs = face_uvs(face_idx, face_layers[face_idx], total_layers);
        let base = positions.len() as u32;

        positions.extend_from_slice(&face.positions);
        normals.extend_from_slice(&[face.normal; 4]);
        uvs.extend_from_slice(&face_uvs);
        indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

struct CubeFace {
    positions: [[f32; 3]; 4],
    normal: [f32; 3],
}

fn held_cube_face(face_idx: usize) -> CubeFace {
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
