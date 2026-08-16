use super::*;
use crate::content::block::registry::BlockRegistry;
use crate::content::item::ItemRegistry;
use crate::engine::asset::AssetPlugin;
use crate::game::gameplay::gamemode::PlayerGameMode;
use crate::game::player::identity::Player;
use crate::game::player::lifecycle::PlayerLifecycle;
use crate::game::player::movement::components::{PlayerMovement, PlayerVelocity};
use crate::game::player::physics::components::{PlayerCollider, PlayerGravity};
use crate::game::world::chunk::ChunkData;
use crate::game::world::state::{ChunkRuntime, WorldState};
use crate::game::world::time::WorldSimulationClock;
use bevy::math::IVec3;
use bevy::prelude::{FixedUpdate, MinimalPlugins, Query, Transform, With};
use std::sync::Arc;

fn headless_player_step(mut query: Query<(&mut Transform, &PlayerVelocity), With<Player>>) {
    for (mut transform, velocity) in &mut query {
        transform.translation += velocity.horizontal * (1.0 / 20.0);
    }
}

#[test]
fn minimal_plugins_can_create_world_and_simulate_player_without_a_window() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin))
        .add_plugins(HeadlessWorldPlugin)
        .init_resource::<WorldState>()
        .init_resource::<ChunkRuntime>()
        .init_resource::<WorldSimulationClock>()
        .init_resource::<BlockRegistry>()
        .init_resource::<ItemRegistry>()
        .init_resource::<PlayerGameMode>()
        .add_systems(FixedUpdate, headless_player_step);

    let player = app
        .world_mut()
        .spawn((
            Player,
            PlayerLifecycle::default(),
            PlayerCollider::default(),
            PlayerMovement::default(),
            PlayerVelocity::default(),
            PlayerGravity::default(),
            Transform::from_xyz(0.0, 70.0, 0.0),
        ))
        .id();

    app.world_mut()
        .resource_mut::<WorldState>()
        .insert_chunk(IVec3::ZERO, Arc::new(ChunkData::default()));
    app.world_mut().run_schedule(FixedUpdate);

    assert!(
        app.world()
            .resource::<WorldState>()
            .contains_chunk(IVec3::ZERO)
    );
    assert!(app.world().get_entity(player).is_ok());
}
