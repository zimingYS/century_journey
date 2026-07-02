use crate::content::tag::block_tags::TagCache;
use bevy::prelude::*;

/// 缓存的标签数据 (Bevy Resource wrapper)
#[derive(Resource, Clone, Default)]
pub struct CachedTagCache(pub TagCache);
