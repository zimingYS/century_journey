use crate::meshing::{MeshData, build_section_mesh};
use crate::resources::SectionSnapshots;
use bevy::asset::RenderAssetUsages;
use bevy::log::info;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use cj_core::pos::SectionLocalPos;
use std::time::Instant;

/// 将内部网格数据转换为 BevyMesh
pub fn to_bevy_mesh(data: MeshData) -> Mesh {
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, data.positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, data.normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, data.colors)
    .with_inserted_indices(Indices::U32(data.indices))
}

/// 为当前 Section 快照构建并生成网格实体
pub fn spawn_section_meshes(
    mut commands: Commands,
    snapshots: Option<Res<SectionSnapshots>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Some(snapshots) = snapshots else {
        return;
    };

    let start = Instant::now();

    let material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        ..default()
    });

    let mut spawned = 0usize;
    let mut quads = 0usize;

    for snapshot in snapshots.sections() {
        let section_pos = snapshot.section();
        let data = build_section_mesh(snapshot, section_pos);

        if data.is_empty() {
            continue;
        }

        quads += data.quad_count();

        let origin = section_pos.block_at(SectionLocalPos { x: 0, y: 0, z: 0 });

        commands.spawn((
            Mesh3d(meshes.add(to_bevy_mesh(data))),
            MeshMaterial3d(material.clone()),
            Transform::from_xyz(origin.x as f32, origin.y as f32, origin.z as f32),
        ));

        spawned += 1;
    }

    info!(
        "网格构建完成：section = {spawned}，四边形 = {quads}，耗时 = {:?}",
        start.elapsed()
    );
}
