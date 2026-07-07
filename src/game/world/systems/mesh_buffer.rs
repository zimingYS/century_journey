use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

/// 定义 6 个方向的相对偏移量，以及对应的三维法线
pub const DIRECTIONS: [(IVec3, Vec3); 6] = [
    (IVec3::new(0, 1, 0), Vec3::new(0.0, 1.0, 0.0)), // 上 (Top)
    (IVec3::new(0, -1, 0), Vec3::new(0.0, -1.0, 0.0)), // 下 (Bottom)
    (IVec3::new(-1, 0, 0), Vec3::new(-1.0, 0.0, 0.0)), // 左 (Left)
    (IVec3::new(1, 0, 0), Vec3::new(1.0, 0.0, 0.0)), // 右 (Right)
    (IVec3::new(0, 0, 1), Vec3::new(0.0, 0.0, 1.0)), // 前 (Front)
    (IVec3::new(0, 0, -1), Vec3::new(0.0, 0.0, -1.0)), // 后 (Back)
];

/// 单个渲染通道的顶点缓冲区
pub struct MeshBufferData {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
}

impl Default for MeshBufferData {
    fn default() -> Self {
        Self::new()
    }
}

impl MeshBufferData {
    pub fn new() -> Self {
        Self::with_capacity(512)
    }

    pub fn with_capacity(estimated_faces: usize) -> Self {
        Self {
            positions: Vec::with_capacity(estimated_faces * 4),
            normals: Vec::with_capacity(estimated_faces * 4),
            uvs: Vec::with_capacity(estimated_faces * 4),
            indices: Vec::with_capacity(estimated_faces * 6),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.positions.is_empty()
    }

    /// 向缓冲区追加一个面的 4 个顶点
    pub fn append_face(
        &mut self,
        face_vertices: &[[f32; 3]; 4],
        normal: Vec3,
        uvs: &[[f32; 2]; 4],
    ) {
        let start_idx = self.positions.len() as u32;
        self.positions.extend_from_slice(face_vertices);
        for _ in 0..4 {
            self.normals.push([normal.x, normal.y, normal.z]);
        }
        self.uvs.extend_from_slice(uvs);
        self.indices.extend_from_slice(&[
            start_idx,
            start_idx + 1,
            start_idx + 2,
            start_idx,
            start_idx + 2,
            start_idx + 3,
        ]);
    }

    /// 从缓冲区生成 Bevy Mesh
    pub fn build_mesh(mut self) -> Mesh {
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        );
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_POSITION,
            std::mem::take(&mut self.positions),
        );
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, std::mem::take(&mut self.normals));
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, std::mem::take(&mut self.uvs));
        mesh.insert_indices(Indices::U32(std::mem::take(&mut self.indices)));
        mesh
    }
}
