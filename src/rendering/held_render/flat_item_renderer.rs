use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::asset::RenderAssetUsages;

/// 平面物品厚度渲染器
pub struct HeldFlatItemRenderer;

impl HeldFlatItemRenderer {
    /// 根据图片生成带厚度的网格
    pub fn build_mesh(image: &Image, thickness: f32) -> Mesh {
        let size = image.size();
        let w = size.x as i32;
        let h = size.y as i32;

        // 获取像素数据
        let Some(pixels) = image.data.as_deref() else {
            return Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
        };

        if pixels.is_empty() || w <= 0 || h <= 0 {
            return Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
        }

        // 纹理调整
        let half_thick = thickness * 0.5;
        let tex_w = w as f32;
        let tex_h = h as f32;
        let world_w = 0.5;
        let world_h = world_w * tex_h / tex_w;

        // 判断指定像素坐标是否为不透明
        let get_alpha = |x: i32, y: i32| -> bool {
            if x < 0 || x >= w || y < 0 || y >= h { return false; }
            let idx = ((y * w + x) * 4 + 3) as usize;
            idx < pixels.len() && pixels[idx] > 30
        };

        // 计算网格
        let mut vertices: Vec<[f32; 3]> = Vec::new();
        let mut uvs: Vec<[f32; 2]> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();

        let px_to_u = |x: i32| x as f32 / tex_w;
        let px_to_v = |y: i32| y as f32 / tex_h;
        let px_to_wx = |x: i32| (x as f32 / tex_w - 0.5) * world_w;
        let px_to_wy = |y: i32| -(y as f32 / tex_h - 0.5) * world_h;

        // 正面
        let front_z = half_thick;
        for y in 0..h {
            for x in 0..w {
                // 跳过透明像素
                if !get_alpha(x, y) { continue; }

                // 计算UV坐标
                let wx = px_to_wx(x);
                let wy = px_to_wy(y);
                let u = px_to_u(x);
                let v = px_to_v(y);
                let du = 1.0 / tex_w;
                let dv = 1.0 / tex_h;

                // 正面四边形4个顶点
                let b = vertices.len() as u32;
                vertices.extend_from_slice(&[
                    [wx, wy, front_z],
                    [wx + world_w / tex_w, wy, front_z],
                    [wx + world_w / tex_w, wy + world_h / tex_h, front_z],
                    [wx, wy + world_h / tex_h, front_z],
                ]);
                uvs.extend_from_slice(&[
                    [u, v + dv], [u + du, v + dv], [u + du, v], [u, v],
                ]);
                indices.extend_from_slice(&[b, b + 1, b + 2, b + 2, b + 3, b]);
            }
        }

        // 背面
        let back_z = -half_thick;
        for y in 0..h {
            for x in 0..w {
                if !get_alpha(x, y) { continue; }
                let wx = px_to_wx(x);
                let wy = px_to_wy(y);
                let u = px_to_u(x);
                let v = px_to_v(y);
                let du = 1.0 / tex_w;
                let dv = 1.0 / tex_h;

                let b = vertices.len() as u32;
                vertices.extend_from_slice(&[
                    [wx, wy, back_z],
                    [wx, wy + world_h / tex_h, back_z],
                    [wx + world_w / tex_w, wy + world_h / tex_h, back_z],
                    [wx + world_w / tex_w, wy, back_z],
                ]);
                uvs.extend_from_slice(&[
                    [u, v + dv], [u, v], [u + du, v], [u + du, v + dv],
                ]);
                indices.extend_from_slice(&[b, b + 1, b + 2, b + 2, b + 3, b]);
            }
        }

        // 边缘侧面
        add_edge_faces(
            &mut vertices, &mut uvs, &mut indices,
            w, h, half_thick, world_w, world_h, &get_alpha,
            px_to_u, px_to_v, px_to_wx, px_to_wy,
        );

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.insert_indices(Indices::U32(indices));
        mesh.compute_normals();
        mesh
    }
}

// 生成物品轮廓的四个方向边缘侧面
fn add_edge_faces<F>(
    vertices: &mut Vec<[f32; 3]>, uvs: &mut Vec<[f32; 2]>, indices: &mut Vec<u32>,
    w: i32, h: i32, half_thick: f32, world_w: f32, world_h: f32,
    get_alpha: &F,
    px_to_u: impl Fn(i32) -> f32, px_to_v: impl Fn(i32) -> f32,
    px_to_wx: impl Fn(i32) -> f32, px_to_wy: impl Fn(i32) -> f32,
) where F: Fn(i32, i32) -> bool {

    // 右侧边缘
    for y in 0..h {
        for x in 0..w {
            if get_alpha(x, y) && !get_alpha(x + 1, y) {
                let wx = px_to_wx(x + 1);
                let wy0 = px_to_wy(y);
                let wy1 = px_to_wy(y + 1);
                let u = px_to_u(x + 1);
                let v0 = px_to_v(y);
                let v1 = px_to_v(y + 1);

                let b = vertices.len() as u32;
                vertices.extend_from_slice(&[
                    [wx, wy0, half_thick],   // 前下
                    [wx, wy0, -half_thick],  // 后下
                    [wx, wy1, -half_thick],  // 后上
                    [wx, wy1, half_thick],   // 前上
                ]);
                uvs.extend_from_slice(&[
                    [u, v0], [u, v0], [u, v1], [u, v1],
                ]);
                indices.extend_from_slice(&[b, b + 1, b + 2, b + 2, b + 3, b]);
            }
        }
    }

    // 左侧边缘
    for y in 0..h {
        for x in 0..w {
            if get_alpha(x, y) && !get_alpha(x - 1, y) {
                let wx = px_to_wx(x);
                let wy0 = px_to_wy(y);
                let wy1 = px_to_wy(y + 1);
                let u = px_to_u(x);
                let v0 = px_to_v(y);
                let v1 = px_to_v(y + 1);

                let b = vertices.len() as u32;
                vertices.extend_from_slice(&[
                    [wx, wy0, -half_thick],  // 后下
                    [wx, wy0, half_thick],   // 前下
                    [wx, wy1, half_thick],   // 前上
                    [wx, wy1, -half_thick],  // 后上
                ]);
                uvs.extend_from_slice(&[
                    [u, v0], [u, v0], [u, v1], [u, v1],
                ]);
                indices.extend_from_slice(&[b, b + 1, b + 2, b + 2, b + 3, b]);
            }
        }
    }

    // 顶部边缘
    for y in 0..h {
        for x in 0..w {
            if get_alpha(x, y) && !get_alpha(x, y + 1) {
                let wx0 = px_to_wx(x);
                let wx1 = px_to_wx(x + 1);
                let wy = px_to_wy(y + 1);
                let v = px_to_v(y + 1);
                let u0 = px_to_u(x);
                let u1 = px_to_u(x + 1);

                let b = vertices.len() as u32;
                vertices.extend_from_slice(&[
                    [wx0, wy, half_thick],   // 前左
                    [wx1, wy, half_thick],   // 前右
                    [wx1, wy, -half_thick],  // 后右
                    [wx0, wy, -half_thick],  // 后左
                ]);
                uvs.extend_from_slice(&[
                    [u0, v], [u1, v], [u1, v], [u0, v],
                ]);
                indices.extend_from_slice(&[b, b + 1, b + 2, b + 2, b + 3, b]);
            }
        }
    }

    // 底部边缘
    for y in 0..h {
        for x in 0..w {
            if get_alpha(x, y) && !get_alpha(x, y - 1) {
                let wx0 = px_to_wx(x);
                let wx1 = px_to_wx(x + 1);
                let wy = px_to_wy(y);
                let v = px_to_v(y);
                let u0 = px_to_u(x);
                let u1 = px_to_u(x + 1);

                let b = vertices.len() as u32;
                vertices.extend_from_slice(&[
                    [wx0, wy, -half_thick],  // 后左
                    [wx1, wy, -half_thick],  // 后右
                    [wx1, wy, half_thick],   // 前右
                    [wx0, wy, half_thick],   // 前左
                ]);
                uvs.extend_from_slice(&[
                    [u0, v], [u1, v], [u1, v], [u0, v],
                ]);
                indices.extend_from_slice(&[b, b + 1, b + 2, b + 2, b + 3, b]);
            }
        }
    }
}