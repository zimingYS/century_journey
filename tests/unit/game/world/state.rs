use super::*;
use crate::content::block::registry::BlockRegistry;
use crate::content::item::registry::registry::ItemRegistry;
use crate::engine::asset::AssetPlugin;
use crate::game::gameplay::gamemode::PlayerGameMode;
use crate::game::player::components::{
    Player, PlayerCollider, PlayerGravity, PlayerLifecycle, PlayerMovement, PlayerVelocity,
};
use crate::game::world::time::WorldSimulationClock;

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
        .loaded_chunks
        .insert(IVec3::ZERO, Arc::new(ChunkData::default()));
    app.world_mut().run_schedule(FixedUpdate);

    assert!(
        app.world()
            .resource::<WorldState>()
            .loaded_chunks
            .contains_key(&IVec3::ZERO)
    );
    assert!(app.world().get_entity(player).is_ok());
}
