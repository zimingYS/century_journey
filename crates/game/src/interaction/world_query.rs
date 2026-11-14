use cj_core::pos::BlockPos;
use cj_core::spec::{WORLD_BOTTOM, WORLD_TOP};
use cj_runtime::world_data::WorldData;

/// 世界层的方块实心判断。
pub fn world_is_solid(world: &WorldData, pos: BlockPos) -> bool {
    if pos.y < WORLD_BOTTOM || pos.y >= WORLD_TOP {
        return true;
    }

    let chunk_pos = pos.chunk_pos();

    let Some(chunk) = world.chunks().get(chunk_pos) else {
        return false;
    };

    chunk.get_block(pos).block().is_solid()
}
