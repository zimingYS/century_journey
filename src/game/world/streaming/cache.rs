use super::WorldStreamingConfig;
use bevy::math::{IVec3, Vec2};
use bevy::prelude::Resource;
use std::collections::HashSet;

/// 缓存玩家当前位置对应的区块流式规划结果
#[derive(Resource, Default)]
pub struct PlayerChunkCache {
    last_chunk_pos: Option<IVec3>,
    last_streaming_config: Option<WorldStreamingConfig>,
    expected_chunks: HashSet<IVec3>,
    ordered_chunks: Vec<IVec3>,
}

impl PlayerChunkCache {
    /// 判断当前位置和流式配置是否需要重新规划区块窗口
    pub fn needs_rebuild(&self, position: IVec3, config: &WorldStreamingConfig) -> bool {
        self.last_chunk_pos != Some(position) || self.last_streaming_config.as_ref() != Some(config)
    }

    /// 根据玩家位置和视线方向重建区块流式规划
    pub fn rebuild(
        &mut self,
        config: &WorldStreamingConfig,
        position: IVec3,
        view_forward_xz: Vec2,
    ) {
        self.last_chunk_pos = Some(position);
        self.last_streaming_config = Some(config.clone());
        let (ordered_chunks, expected_chunks) =
            config.rebuild_expected_chunks(position, view_forward_xz);
        self.ordered_chunks = ordered_chunks;
        self.expected_chunks = expected_chunks;
    }

    /// 返回当前玩家所在区块
    pub fn player_chunk_pos(&self) -> Option<IVec3> {
        self.last_chunk_pos
    }

    /// 按加载优先级遍历计划区块
    pub fn ordered_chunks(&self) -> &[IVec3] {
        &self.ordered_chunks
    }

    /// 判断区块是否仍位于当前流式窗口
    pub fn expects_chunk(&self, position: IVec3) -> bool {
        self.expected_chunks.contains(&position)
    }

    /// 返回当前流式窗口中预期区块数量
    pub fn expected_chunk_count(&self) -> usize {
        self.expected_chunks.len()
    }
}
