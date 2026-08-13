//! 累积区块网格的顶点属性，并转换为 Bevy 网格资产。

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
    /// 顶点光色（RGBA，乘法衰减系数语义），烘焙自权威光级数组。
    pub colors: Vec<[f32; 4]>,
    /// 第二组 UV 中的 12bit 方块 RGB 光级，供区块材质生成稳定远景照明。
    pub block_light_uvs: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
}

impl Default for MeshBufferData {
    fn default() -> Self {
        Self::new()
    }
}

impl MeshBufferData {
    /// 创建具有默认面容量的空网格缓冲区。
    pub fn new() -> Self {
        Self::with_capacity(512)
    }

    /// 按预计面数预分配顶点和索引容量。
    pub fn with_capacity(estimated_faces: usize) -> Self {
        Self {
            positions: Vec::with_capacity(estimated_faces * 4),
            normals: Vec::with_capacity(estimated_faces * 4),
            uvs: Vec::with_capacity(estimated_faces * 4),
            colors: Vec::with_capacity(estimated_faces * 4),
            block_light_uvs: Vec::with_capacity(estimated_faces * 4),
            indices: Vec::with_capacity(estimated_faces * 6),
        }
    }

    /// 判断缓冲区是否尚未写入任何顶点。
    pub fn is_empty(&self) -> bool {
        self.positions.is_empty()
    }

    /// 向缓冲区追加一个面的 4 个顶点
    pub fn append_face(
        &mut self,
        face_vertices: &[[f32; 3]; 4],
        normal: Vec3,
        uvs: &[[f32; 2]; 4],
        color: [f32; 4],
        block_light_uv: [f32; 2],
    ) {
        let start_idx = self.positions.len() as u32;
        self.positions.extend_from_slice(face_vertices);
        for _ in 0..4 {
            self.normals.push([normal.x, normal.y, normal.z]);
            self.colors.push(color);
            self.block_light_uvs.push(block_light_uv);
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

    /// 从缓冲区生成带合成顶点光色与独立方块光级的 Bevy Mesh。
    pub fn build_mesh(self) -> Mesh {
        self.build_mesh_impl(true)
    }

    /// 从缓冲区生成不带顶点光色的 Bevy Mesh。
    ///
    /// 供自定义着色（如水面）的通道使用：顶点色会触发 PBR 的
    /// `VERTEX_COLORS` 着色器变体，而自定义材质可能不支持该变体。
    pub fn build_mesh_plain(self) -> Mesh {
        self.build_mesh_impl(false)
    }

    fn build_mesh_impl(mut self, with_voxel_lighting: bool) -> Mesh {
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
        if with_voxel_lighting {
            mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, std::mem::take(&mut self.colors));
            mesh.insert_attribute(
                Mesh::ATTRIBUTE_UV_1,
                std::mem::take(&mut self.block_light_uvs),
            );
        }
        mesh.insert_indices(Indices::U32(std::mem::take(&mut self.indices)));
        mesh
    }
}
