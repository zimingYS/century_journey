//! World state ownership boundaries.
//!
//! `WorldState` is authoritative simulation data. `ChunkRuntime` is disposable
//! streaming/task state. Client mesh entities live in the client module.

use crate::game::world::chunk::ChunkData;
use crate::game::world::generation::context::ChunkGenContext;
use crate::game::world::storage::{PendingVoxelWrites, WorldStorage};
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

#[derive(Resource, Debug, Default)]
pub struct WorldState {
    pub loaded_chunks: HashMap<IVec3, Arc<ChunkData>>,
    pub chunk_modified_times: HashMap<IVec3, f64>,
    pub pending_writes: PendingVoxelWrites,
    pub block_entities: HashMap<IVec3, Entity>,
}

impl WorldState {
    pub fn from_legacy(storage: WorldStorage) -> (Self, ChunkRuntime) {
        (
            Self {
                loaded_chunks: storage.loaded_chunks,
                chunk_modified_times: storage.chunk_modified_times,
                pending_writes: storage.pending_writes,
                block_entities: HashMap::new(),
            },
            ChunkRuntime {
                chunk_entities: storage.chunk_entities,
                gen_contexts: storage.gen_contexts,
                ..default()
            },
        )
    }
}

#[derive(Resource, Debug, Default)]
pub struct ChunkRuntime {
    pub chunk_entities: HashMap<IVec3, Entity>,
    pub gen_contexts: HashMap<IVec3, ChunkGenContext>,
    pub last_chunk_pos: Option<IVec3>,
    pub expected_chunks: HashSet<IVec3>,
    pub terrain_tasks_in_flight: usize,
    pub structure_tasks_in_flight: usize,
    pub mesh_tasks_in_flight: usize,
}

pub struct HeadlessWorldPlugin;

impl Plugin for HeadlessWorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldState>()
            .init_resource::<ChunkRuntime>()
            .init_resource::<crate::game::world::storage::WorldStorage>()
            .init_resource::<crate::game::block::BlockBehaviorRegistry>()
            .insert_resource(Time::<Fixed>::from_hz(
                crate::game::world::time::SIMULATION_TICKS_PER_SECOND as f64,
            ))
            .add_plugins(crate::game::simulation::SimulationPlugin)
            .add_plugins(crate::game::gameplay::GameplayPlugin)
            .add_plugins(crate::game::inventory::plugin::InventoryPlugin)
            .add_plugins(crate::game::player::plugin::GamePlayerPlugin);
    }
}

#[cfg(test)]
mod tests {
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
}
