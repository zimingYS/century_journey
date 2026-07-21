use super::*;
use crate::game::world::chunk::ChunkData;

fn saved_chunk(position: IVec3, voxel: u16) -> SavedChunk {
    let mut data = ChunkData::default();
    data.voxels[0] = voxel;
    SavedChunk {
        position,
        data,
        modified_time: f64::from(voxel),
    }
}

#[test]
fn missing_primary_reads_and_extends_the_valid_backup() {
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let world = format!("region_backup_window_{}_{unique}", std::process::id());
    let first = saved_chunk(IVec3::ZERO, 11);
    let updated = saved_chunk(IVec3::ZERO, 12);
    let second = saved_chunk(IVec3::X, 21);

    RegionManager::write_chunk(&world, &first).unwrap();
    RegionManager::write_chunk(&world, &updated).unwrap();
    let path = RegionManager::region_path(&world, IVec3::ZERO);
    std::fs::remove_file(&path).unwrap();

    let from_backup = RegionManager::read_chunk(&world, IVec3::ZERO)
        .unwrap()
        .unwrap();
    assert_eq!(from_backup.data.voxels[0], 11);

    RegionManager::write_chunk(&world, &second).unwrap();
    assert_eq!(
        RegionManager::read_chunk(&world, IVec3::ZERO)
            .unwrap()
            .unwrap()
            .data
            .voxels[0],
        11
    );
    assert_eq!(
        RegionManager::read_chunk(&world, IVec3::X)
            .unwrap()
            .unwrap()
            .data
            .voxels[0],
        21
    );

    RegionManager::delete_world(&world).unwrap();
}
