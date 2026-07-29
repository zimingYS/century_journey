mod breaking;
mod placement;
mod state;

pub use state::{BlockBreakProgress, BlockBreakState};

pub use placement::{can_place_block, consume_placed_block_item};

pub use breaking::{
    active_tool_data, block_break_seconds, can_break_block, can_harvest_block,
    is_replaceable_block, is_unbreakable_block,
};

#[cfg(test)]
#[path = "../../../../tests/unit/game/gameplay/block_action/mod.rs"]
mod tests;
