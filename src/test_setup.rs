use bevy::prelude::*;

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
){
    // 渲染一个立方体
    // let cube_mesh = Cuboid::default().mesh().build();
    // let mesh_handle = meshes.add(Mesh::from(cube_mesh));
    //
    // let material_handle = materials.add(StandardMaterial{
    //     base_color: Color::srgb(0.3, 0.7, 0.9),
    //     ..default()
    // });
    //
    // commands.spawn((
    //     Mesh3d(mesh_handle),
    //     MeshMaterial3d(material_handle),
    //     Transform::from_xyz(0.0, 0.0, 0.0),
    // ));

    // 添加光源
    commands.spawn((PointLight{
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));
    
    info!("游戏已启动！");
}