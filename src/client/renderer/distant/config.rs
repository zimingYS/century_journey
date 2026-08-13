//! 定义远景真实方块 LOD 的视距、分环和采样密度策略。

use crate::shared::voxel::CHUNK_SIZE;
use bevy::prelude::Resource;

/// 远景真实方块 LOD 的客户端表现配置。
///
/// 半径按当前真实区块网格半径自动缩放，而不是扩张 `WorldStreamingConfig` 的权威
/// 数据窗口；这使远景不会额外占用区块存档、生成或体素光的预算。
#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub(crate) struct DistantTerrainConfig {
    /// 远景最远半径相对近景网格半径的固定倍率。
    distance_scale: u32,
    /// 即使近景渲染距离较小时也保留的最小远景半径，单位为区块。
    minimum_radius_chunks: i32,
    /// 防止高视距设置导致表现层无限增加网格和任务数量的硬上限，单位为区块。
    maximum_radius_chunks: i32,
}

impl Default for DistantTerrainConfig {
    fn default() -> Self {
        Self {
            distance_scale: 4,
            minimum_radius_chunks: 16,
            maximum_radius_chunks: 64,
        }
    }
}

impl DistantTerrainConfig {
    /// 返回给定近景网格半径对应的远景最远半径。
    pub(crate) fn far_radius_chunks(&self, near_radius_chunks: i32) -> i32 {
        near_radius_chunks
            .saturating_mul(self.distance_scale as i32)
            .clamp(self.minimum_radius_chunks, self.maximum_radius_chunks)
    }

    /// 返回相机大气和雾效需要覆盖的世界方块距离。
    ///
    /// 最外层瓦片会越过圆形半径延伸以填满边界；斜向观察时对角可多出接近两个
    /// 瓦片跨度，因此这里保守预留该距离，避免远景在雾效开始前被相机裁切。
    pub(crate) fn view_distance_blocks(&self, near_radius_chunks: i32) -> f32 {
        let far_radius = self.far_radius_chunks(near_radius_chunks);
        let far_tile_span = near_tile_span_chunks(near_radius_chunks).saturating_mul(2);
        (far_radius
            .saturating_add(far_tile_span.saturating_mul(2))
            .saturating_mul(CHUNK_SIZE as i32)) as f32
    }

    /// 返回雾效应完成远景遮蔽的圆环边界，单位为世界方块。
    pub(crate) fn fog_distance_blocks(&self, near_radius_chunks: i32) -> f32 {
        self.far_radius_chunks(near_radius_chunks)
            .saturating_mul(CHUNK_SIZE as i32) as f32
    }

    /// 根据近景半径生成两个采样密度递减的远景环。
    pub(super) fn rings(&self, near_radius_chunks: i32) -> [DistantTerrainRing; 2] {
        let near_radius_chunks = near_radius_chunks.max(1);
        let far_radius_chunks = self.far_radius_chunks(near_radius_chunks);
        let transition_radius_chunks = near_radius_chunks
            .saturating_mul(2)
            .clamp(near_radius_chunks.saturating_add(1), far_radius_chunks);
        let near_span_chunks = near_tile_span_chunks(near_radius_chunks);

        [
            DistantTerrainRing {
                lod_level: 0,
                inner_radius_chunks: near_radius_chunks,
                outer_radius_chunks: transition_radius_chunks,
                tile_span_chunks: near_span_chunks,
            },
            DistantTerrainRing {
                lod_level: 1,
                inner_radius_chunks: transition_radius_chunks,
                outer_radius_chunks: far_radius_chunks,
                tile_span_chunks: near_span_chunks.saturating_mul(2),
            },
        ]
    }
}

/// 单个远景环的覆盖范围与固定采样步长。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct DistantTerrainRing {
    /// 从近到远递增的 LOD 编号；外环使用更低的采样密度。
    pub(super) lod_level: u8,
    /// 环的内半径，单位为区块。
    pub(super) inner_radius_chunks: i32,
    /// 环的外半径，单位为区块。
    pub(super) outer_radius_chunks: i32,
    /// 一个瓦片覆盖的区块边长；同样决定瓦片内的方块采样步长。
    pub(super) tile_span_chunks: i32,
}

fn near_tile_span_chunks(near_radius_chunks: i32) -> i32 {
    (near_radius_chunks / 2).clamp(4, 8)
}

#[cfg(test)]
#[path = "../../../../tests/unit/client/renderer/distant/config.rs"]
mod tests;
