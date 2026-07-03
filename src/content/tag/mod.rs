pub mod block_tags;
pub mod cache;
pub mod plugin;
pub mod populate;

pub use block_tags::{
    TagCache, auto_populate_from_block_tags, get_block_tag_ids, is_block_id_tagged,
    validate_tags_against_block_registry,
};
pub use cache::CachedTagCache;
pub use plugin::TagContentPlugin;
