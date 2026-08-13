use super::*;
use bevy::prelude::*;

fn app_with_chunk(position: IVec3) -> (App, Entity, std::sync::mpsc::Sender<TerrainGenResult>) {
    let mut app = App::new();
    app.init_resource::<WorldState>();
    app.init_resource::<ChunkRuntime>();
    let channel = TerrainGenChannel::default();
    let sender = channel.sender.clone();
    app.insert_resource(channel);
    let entity = app
        .world_mut()
        .spawn((ChunkComponents { position }, ChunkState::GeneratingTerrain))
        .id();
    app.world_mut()
        .resource_mut::<ChunkRuntime>()
        .register_chunk_entity(position, entity);
    app.add_systems(Update, receive_terrain_results);
    (app, entity, sender)
}

#[test]
fn stale_result_cannot_commit_to_replacement_entity_at_same_position() {
    let position = IVec3::new(3, 0, 2);
    let (mut app, replacement, sender) = app_with_chunk(position);
    let stale_entity = Entity::from_bits(replacement.to_bits().wrapping_add(1));
    app.world()
        .resource::<TerrainGenChannel>()
        .in_flight
        .store(1, Ordering::Relaxed);
    sender
        .send(TerrainGenResult {
            chunk_pos: position,
            request_entity: stale_entity,
            outcome: TerrainGenOutcome::Generated {
                chunk_data: Box::new(ChunkData::new()),
                gen_context: crate::game::world::generation::terrain::context::ChunkGenContext::new(
                    position,
                ),
            },
        })
        .unwrap();

    app.update();

    assert_eq!(
        *app.world().entity(replacement).get::<ChunkState>().unwrap(),
        ChunkState::GeneratingTerrain
    );
    assert!(
        !app.world()
            .resource::<WorldState>()
            .contains_chunk(position)
    );
}

#[test]
fn restored_snapshot_skips_natural_structure_generation() {
    let position = IVec3::new(1, 0, 1);
    let (mut app, entity, sender) = app_with_chunk(position);
    let mut data = ChunkData::new();
    data.voxels[0] = 27;
    app.world()
        .resource::<TerrainGenChannel>()
        .in_flight
        .store(1, Ordering::Relaxed);
    sender
        .send(TerrainGenResult {
            chunk_pos: position,
            request_entity: entity,
            outcome: TerrainGenOutcome::Restored {
                chunk_data: Box::new(data),
                tree_instances: Vec::new(),
            },
        })
        .unwrap();

    app.update();

    assert_eq!(
        *app.world().entity(entity).get::<ChunkState>().unwrap(),
        ChunkState::LightingPending
    );
    assert_eq!(
        app.world()
            .resource::<WorldState>()
            .chunk(position)
            .unwrap()
            .voxels[0],
        27
    );
}
