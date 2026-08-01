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

#[test]
fn read_barrier_returns_in_flight_snapshot_before_disk_can_catch_up() {
    let position = IVec3::new(2, 0, -1);
    let queue = SaveQueue::default();
    let mut worker = SaveWorker::default();
    worker
        .in_flight_snapshots
        .insert(position, saved_chunk(position, 10.0, 31));
    worker.in_flight_batches = 1;

    let visible = latest_snapshot_for_load(&queue, &worker, position).unwrap();
    assert_eq!(visible.data.voxels[0], 31);
    assert_eq!(visible.tree_instances[0].shape_seed(), 31);
}

#[test]
fn queued_snapshot_wins_over_in_flight_snapshot_even_with_equal_time() {
    let position = IVec3::new(2, 0, -1);
    let mut queue = SaveQueue::default();
    queue.enqueue(saved_chunk(position, 10.0, 42));
    let mut worker = SaveWorker::default();
    worker
        .in_flight_snapshots
        .insert(position, saved_chunk(position, 10.0, 31));
    worker.in_flight_batches = 1;

    let visible = latest_snapshot_for_load(&queue, &worker, position).unwrap();
    assert_eq!(visible.data.voxels[0], 42);
    assert_eq!(visible.tree_instances[0].shape_seed(), 42);
}

#[test]
fn completed_older_batch_never_removes_newer_queued_snapshot() {
    let position = IVec3::new(1, 0, 1);
    let older = saved_chunk(position, 10.0, 10);
    let newer = saved_chunk(position, 10.0, 20);
    let mut queue = SaveQueue::default();
    queue.enqueue(newer);
    let mut worker = SaveWorker::default();
    worker.in_flight_snapshots.insert(position, older.clone());
    worker.in_flight_batches = 1;

    apply_completion(
        &mut queue,
        &mut worker,
        SaveBatchCompletion {
            chunks: vec![older],
            error: None,
        },
    )
    .unwrap();

    assert_eq!(queue.queue.front().unwrap().data.voxels[0], 20);
    assert!(worker.is_idle());
}

#[test]
fn failed_older_batch_keeps_newer_snapshot_but_requeues_when_no_newer_exists() {
    let position = IVec3::new(3, 0, 3);
    let older = saved_chunk(position, 10.0, 10);
    let mut queue = SaveQueue::default();
    queue.enqueue(saved_chunk(position, 10.0, 20));
    let mut worker = SaveWorker::default();
    worker.in_flight_snapshots.insert(position, older.clone());
    worker.in_flight_batches = 1;

    assert!(
        apply_completion(
            &mut queue,
            &mut worker,
            SaveBatchCompletion {
                chunks: vec![older.clone()],
                error: Some("disk full".into()),
            },
        )
        .is_err()
    );
    assert_eq!(queue.queue.len(), 1);
    assert_eq!(queue.queue.front().unwrap().data.voxels[0], 20);

    queue.queue.clear();
    worker.in_flight_snapshots.insert(position, older.clone());
    worker.in_flight_batches = 1;
    let _ = apply_completion(
        &mut queue,
        &mut worker,
        SaveBatchCompletion {
            chunks: vec![older],
            error: Some("disk full".into()),
        },
    );
    assert_eq!(queue.queue.front().unwrap().data.voxels[0], 10);
}
