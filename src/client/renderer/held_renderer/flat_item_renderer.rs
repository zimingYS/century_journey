use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};

const ITEM_WORLD_WIDTH: f32 = 0.78;
const ALPHA_CUTOFF: u8 = 30;

pub struct HeldFlatItemRenderer;

impl HeldFlatItemRenderer {
    pub fn build_mesh(image: &Image, thickness: f32) -> Mesh {
        let size = image.size();
        let w = size.x as i32;
        let h = size.y as i32;

        let Some(pixels) = image.data.as_deref() else {
            return empty_mesh();
        };

        if pixels.is_empty() || w <= 0 || h <= 0 {
            return empty_mesh();
        }

        let tex_w = w as f32;
        let tex_h = h as f32;
        let world_w = ITEM_WORLD_WIDTH;
        let world_h = world_w * tex_h / tex_w;
        let px = world_w / tex_w;
        let py = world_h / tex_h;
        let front_z = thickness * 0.5;
        let back_z = -front_z;

        let is_opaque = |x: i32, y: i32| -> bool {
            if x < 0 || x >= w || y < 0 || y >= h {
                return false;
            }
            let idx = ((y * w + x) * 4 + 3) as usize;
            idx < pixels.len() && pixels[idx] > ALPHA_CUTOFF
        };

        let mut vertices = Vec::new();
        let mut normals = Vec::new();
        let mut uvs = Vec::new();
        let mut indices = Vec::new();

        push_quad(
            &mut vertices,
            &mut normals,
            &mut uvs,
            &mut indices,
            [0.0, 0.0, 1.0],
            [
                [-world_w * 0.5, world_h * 0.5, front_z],
                [world_w * 0.5, world_h * 0.5, front_z],
                [world_w * 0.5, -world_h * 0.5, front_z],
                [-world_w * 0.5, -world_h * 0.5, front_z],
            ],
            [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]],
            false,
        );
        push_quad(
            &mut vertices,
            &mut normals,
            &mut uvs,
            &mut indices,
            [0.0, 0.0, -1.0],
            [
                [-world_w * 0.5, world_h * 0.5, back_z],
                [-world_w * 0.5, -world_h * 0.5, back_z],
                [world_w * 0.5, -world_h * 0.5, back_z],
                [world_w * 0.5, world_h * 0.5, back_z],
            ],
            [[0.0, 1.0], [0.0, 0.0], [1.0, 0.0], [1.0, 1.0]],
            false,
        );

        for y in 0..h {
            for x in 0..w {
                if !is_opaque(x, y) {
                    continue;
                }

                let x0 = -world_w * 0.5 + x as f32 * px;
                let x1 = x0 + px;
                let y0 = world_h * 0.5 - y as f32 * py;
                let y1 = y0 - py;
                let uv = [(x as f32 + 0.5) / tex_w, 1.0 - (y as f32 + 0.5) / tex_h];

                if !is_opaque(x + 1, y) {
                    push_side(
                        &mut vertices,
                        &mut normals,
                        &mut uvs,
                        &mut indices,
                        [1.0, 0.0, 0.0],
                        [
                            [x1, y1, back_z],
                            [x1, y1, front_z],
                            [x1, y0, front_z],
                            [x1, y0, back_z],
                        ],
                        uv,
                    );
                }
                if !is_opaque(x - 1, y) {
                    push_side(
                        &mut vertices,
                        &mut normals,
                        &mut uvs,
                        &mut indices,
                        [-1.0, 0.0, 0.0],
                        [
                            [x0, y0, back_z],
                            [x0, y0, front_z],
                            [x0, y1, front_z],
                            [x0, y1, back_z],
                        ],
                        uv,
                    );
                }
                if !is_opaque(x, y - 1) {
                    push_side(
                        &mut vertices,
                        &mut normals,
                        &mut uvs,
                        &mut indices,
                        [0.0, 1.0, 0.0],
                        [
                            [x0, y0, back_z],
                            [x1, y0, back_z],
                            [x1, y0, front_z],
                            [x0, y0, front_z],
                        ],
                        uv,
                    );
                }
                if !is_opaque(x, y + 1) {
                    push_side(
                        &mut vertices,
                        &mut normals,
                        &mut uvs,
                        &mut indices,
                        [0.0, -1.0, 0.0],
                        [
                            [x0, y1, front_z],
                            [x1, y1, front_z],
                            [x1, y1, back_z],
                            [x0, y1, back_z],
                        ],
                        uv,
                    );
                }
            }
        }

        let mut mesh = empty_mesh();
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.insert_indices(Indices::U32(indices));
        mesh
    }
}

fn empty_mesh() -> Mesh {
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
}

fn push_side(
    vertices: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    indices: &mut Vec<u32>,
    normal: [f32; 3],
    points: [[f32; 3]; 4],
    uv: [f32; 2],
) {
    push_quad(
        vertices, normals, uvs, indices, normal, points, [uv; 4], false,
    );
}

fn push_quad(
    vertices: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    indices: &mut Vec<u32>,
    normal: [f32; 3],
    points: [[f32; 3]; 4],
    quad_uvs: [[f32; 2]; 4],
    flip: bool,
) {
    let base = vertices.len() as u32;
    vertices.extend_from_slice(&points);
    normals.extend_from_slice(&[normal; 4]);
    uvs.extend_from_slice(&quad_uvs);
    if flip {
        indices.extend_from_slice(&[base, base + 2, base + 1, base + 2, base, base + 3]);
    } else {
        indices.extend_from_slice(&[base, base + 1, base + 2, base + 2, base + 3, base]);
    }
}
