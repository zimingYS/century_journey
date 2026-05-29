use crate::player::systems::raycast::TargetVoxel;
use bevy::camera::Exposure;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::light::{Atmosphere, AtmosphereEnvironmentMapLight, VolumetricFog};
use bevy::light::atmosphere::ScatteringMedium;
use bevy::pbr::AtmosphereSettings;
use bevy::post_process::bloom::Bloom;
use bevy::prelude::*;

pub mod components;
pub mod systems;
pub mod camera;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<TargetVoxel>()
            .add_systems(Startup,(
                spawn_player,
                camera::convert_mouse_lock_on_startup
            ))
            .add_systems(Update, (
                camera::player_look_system,
                systems::movement::player_movement_system,
                camera::toggle_mouse_lock_system,
                systems::interaction::voxel_interaction_system,
                systems::raycast::draw_voxel_highlight_system,
                systems::raycast::update_raycast_system,
            ));
    }
}

fn spawn_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut scattering_mediums: ResMut<Assets<ScatteringMedium>>,
) {
    // 生成玩家身体
    commands.spawn((
        components::Player,
        components::PlayerMovement::default(),
        Transform::from_xyz(0.0, 70.0, 0.0),
        Visibility::default(),
    )).with_children(|parent| {
        parent.spawn((
            Mesh3d(meshes.add(Capsule3d::default().mesh().build())),
            MeshMaterial3d(materials.add(Color::srgb(0.8, 0.2, 0.2))),
            Transform::from_xyz(0.0, 1.0, 0.0),
        ));

        parent.spawn((
            camera::FpsCamera::default(),
            Camera3d::default(),
            Transform::from_xyz(0.0, 1.65, 0.0),

            // 世界大气
            AtmosphereSettings::default(),
            AtmosphereEnvironmentMapLight::default(),

            // 高曝光补偿
            Exposure { ..default() },
            Tonemapping::AcesFitted,
            Bloom::NATURAL,

            // 开启体积雾
            VolumetricFog {
                ambient_intensity: 0.0,
                ..default()
            },
        ));
    });
}