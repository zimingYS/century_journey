use super::*;
use crate::game::world::chunk::ChunkData;

fn saved_chunk(position: IVec3, modified_time: f64, first_voxel: u16) -> SavedChunk {
    let mut data = ChunkData::default();
    data.voxels[0] = first_voxel;
    SavedChunk {
        position,
        data,
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
}
