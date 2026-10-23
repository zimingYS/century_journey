use crate::meshing::{MeshData, build_section_mesh};
use crate::resources::{DirtySections, SectionMeshes, SectionSnapshots};
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
    mut section_meshes: ResMut<SectionMeshes>,
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
    section_meshes.set_material(material.clone());

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

        let entity = commands
            .spawn((
                Mesh3d(meshes.add(to_bevy_mesh(data))),
                MeshMaterial3d(material.clone()),
                Transform::from_xyz(origin.x as f32, origin.y as f32, origin.z as f32),
            ))
            .id();
        section_meshes.insert(section_pos, entity);

        spawned += 1;
    }

    info!(
        "网格构建完成：section = {spawned}，四边形 = {quads}，耗时 = {:?}",
        start.elapsed()
    );
}

/// 重构脏区块
pub fn rebuild_dirty_sections(
    mut dirty: ResMut<DirtySections>,
    mut section_meshes: ResMut<SectionMeshes>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let Some(material) = section_meshes.material().cloned() else {
        return;
    };

    for (section_pos, snapshot) in dirty.take() {
        let data = build_section_mesh(&snapshot, section_pos);
        let existing = section_meshes.entity(section_pos);
        if data.is_empty() {
            if let Some(e) = existing {
                commands.entity(e).despawn();
                section_meshes.remove(section_pos);
            }
        } else {
            match existing {
                Some(e) => {
                    // 已有实体只替换网格组件
                    commands
                        .entity(e)
                        .insert(Mesh3d(meshes.add(to_bevy_mesh(data))));
                }
                None => {
                    let origin = section_pos.block_at(SectionLocalPos { x: 0, y: 0, z: 0 });
                    // 没有实体新建实体并登记
                    let e = commands
                        .spawn((
                            Mesh3d(meshes.add(to_bevy_mesh(data))),
                            MeshMaterial3d(material.clone()),
                            Transform::from_xyz(origin.x as f32, origin.y as f32, origin.z as f32),
                        ))
                        .id();
                    section_meshes.insert(section_pos, e);
                }
            }
        }
    }
}
