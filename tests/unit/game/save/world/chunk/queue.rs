use super::*;
use crate::game::world::chunk::ChunkData;
use crate::game::world::{TreeGrowthStage, TreeInstance};
use crate::shared::identifier::Identifier;
use crate::shared::voxel::CHUNK_SIZE;

fn saved_chunk(position: IVec3, modified_time: f64, first_voxel: u16) -> SavedChunk {
    let mut data = ChunkData::default();
    data.voxels[0] = first_voxel;
    SavedChunk {
        position,
        data,
        tree_instances: vec![
            TreeInstance::from_persisted(
                position * CHUNK_SIZE as i32 + IVec3::ONE,
                Identifier::new("century_journey", "oak"),
                u32::from(first_voxel),
                TreeGrowthStage::Mature,
                0,
                0,
                1_000,
                0,
                None,
            )
            .unwrap(),
        ],
        modified_time,
    }
}

#[test]
fn save_queue_coalesces_snapshots_and_keeps_the_newest() {
    let position = IVec3::new(1, 2, 3);
    let mut queue = SaveQueue::default();
    queue.enqueue(saved_chunk(position, 10.0, 10));
    queue.enqueue(saved_chunk(position, 20.0, 20));
    queue.enqueue(saved_chunk(position, 15.0, 15));

    assert_eq!(queue.queue.len(), 1);
    let saved = queue.queue.front().unwrap();
    assert_eq!(saved.modified_time, 20.0);
    assert_eq!(saved.data.voxels[0], 20);
    assert_eq!(saved.tree_instances[0].shape_seed(), 20);
}
