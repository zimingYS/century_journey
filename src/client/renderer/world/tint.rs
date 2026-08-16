//! 生物群系与季节的环境着色计算。
//!
//! 把稳定气候（基础温湿度）映射为草/树叶的基础色，再叠加季节乘子得到最终
//! 着色；着色在网格构建时乘进顶点光色，纯表现、不进存档、不参与权威规则。

use crate::client::renderer::world::channel::BlockInfoSnapshot;
use crate::content::block::definition::BlockTint;
use crate::game::world::generation::pipeline::TerrainSurfaceSampler;
use crate::game::world::lighting::chunk_light::LightRgb;
use crate::game::world::time::Season;
use bevy::prelude::*;

/// 着色的最低亮度下限，避免极暗季节 × 极暗光级得到全黑死面。
const MIN_DARK_TINT: f32 = 0.02;
/// 量化档位上限（4-bit 通道），对应浮点 1.0。
const TINT_QUANT_MAX: u8 = 15;

/// 由基础温湿度计算生物群系基础着色（草绿，随温度偏黄/偏蓝、随湿度更绿）。
///
/// 温度 0=极寒、1=极热，湿度 0=极干、1=极湿。生物群系色只依赖基础气候，
/// 不随季节漂移，保证同一位置四季的「底子」稳定。
/// 各通道保持在 [0,1]，通过相对差异体现色相变化（顶点色是乘法衰减）。
pub fn biome_tint(kind: BlockTint, temperature: f64, humidity: f64) -> [f32; 3] {
    let warmth = temperature.clamp(0.0, 1.0) as f32;
    let moisture = humidity.clamp(0.0, 1.0) as f32;

    // 草与树叶的基底略有差异：树叶更深更浓。
    let (base_r, base_g, base_b) = match kind {
        BlockTint::GrassTop => (0.36, 0.62, 0.18),
        BlockTint::Foliage => (0.24, 0.50, 0.16),
    };

    // 温度升高偏黄（稀树草原），降低偏蓝灰（雪原）；湿度升高更绿。
    let r = (base_r + (warmth - 0.5) * 0.30 + (0.5 - moisture) * 0.10).clamp(0.0, 1.0);
    let g = (base_g + (moisture - 0.5) * 0.20 - (warmth - 0.5) * 0.06).clamp(0.0, 1.0);
    let b = (base_b + (0.5 - warmth) * 0.14).clamp(0.0, 1.0);

    [r, g, b]
}

/// 季节着色乘子：春嫩绿、夏浓绿、秋橙红、冬褪色偏白。
/// 各通道保持在 [0,1]，通过相对差异（红升绿降 = 橙色）体现季节色彩；
/// 顶点光色语义是乘法衰减，>1.0 会溢出 GPU 上限并丢失颜色比例。
pub fn season_tint(season: Season) -> [f32; 3] {
    match season {
        Season::Spring => [0.95, 1.00, 0.90],
        Season::Summer => [0.82, 1.00, 0.68],
        Season::Autumn => [1.00, 0.62, 0.32],
        Season::Winter => [0.74, 0.80, 0.88],
    }
}

/// 组合生物群系基础色与季节乘子，得到最终着色。
pub fn final_tint(kind: BlockTint, temperature: f64, humidity: f64, season: Season) -> [f32; 3] {
    let biome = biome_tint(kind, temperature, humidity);
    let seasonal = season_tint(season);
    [
        biome[0] * seasonal[0],
        biome[1] * seasonal[1],
        biome[2] * seasonal[2],
    ]
}

/// 把 [0,1] 浮点着色量化到 4-bit LightRgb（写入 face_key）。
pub fn quantize_tint(tint_rgb: [f32; 3]) -> LightRgb {
    LightRgb {
        r: (tint_rgb[0].clamp(0.0, 1.0) * TINT_QUANT_MAX as f32).round() as u8,
        g: (tint_rgb[1].clamp(0.0, 1.0) * TINT_QUANT_MAX as f32).round() as u8,
        b: (tint_rgb[2].clamp(0.0, 1.0) * TINT_QUANT_MAX as f32).round() as u8,
    }
}

/// 把量化后的 LightRgb 解码回 [0,1] 浮点。
pub fn unquantize_tint(tint: LightRgb) -> [f32; 3] {
    [
        tint.r as f32 / TINT_QUANT_MAX as f32,
        tint.g as f32 / TINT_QUANT_MAX as f32,
        tint.b as f32 / TINT_QUANT_MAX as f32,
    ]
}

/// 把生物群系/季节着色乘进顶点光色（RGBA），并保持最低亮度下限避免纯黑。
pub fn apply_vertex_tint(base: [f32; 4], tint_rgb: [f32; 3]) -> [f32; 4] {
    [
        (base[0] * tint_rgb[0]).clamp(MIN_DARK_TINT, 1.0),
        (base[1] * tint_rgb[1]).clamp(MIN_DARK_TINT, 1.0),
        (base[2] * tint_rgb[2]).clamp(MIN_DARK_TINT, 1.0),
        1.0,
    ]
}

/// "不着色"的白色 tint（量化后为 [15,15,15]，与光色相乘不变）。
pub fn white_tint() -> LightRgb {
    LightRgb {
        r: TINT_QUANT_MAX,
        g: TINT_QUANT_MAX,
        b: TINT_QUANT_MAX,
    }
}

/// 计算某方块某面在某位置的环境着色（已量化）。
///
/// 不参与着色的方块、GrassTop 的非顶面、以及未加载到采样服务的世界坐标都返回白色 tint。
pub fn compute_face_tint(
    voxel_id: u16,
    face_idx: usize,
    world_pos: bevy::math::IVec3,
    block_info: &BlockInfoSnapshot,
    season: Season,
    sampler: &TerrainSurfaceSampler,
) -> LightRgb {
    let Some(kind) = block_info
        .tint_kinds
        .get(voxel_id as usize)
        .copied()
        .flatten()
    else {
        return white_tint();
    };

    // GrassTop 只给顶面着色（侧面是 dirt，不应被季节/群系覆盖）。
    if matches!(kind, BlockTint::GrassTop) && face_idx != 0 {
        return white_tint();
    }

    if !sampler.is_ready() {
        return white_tint();
    }

    let (temp, humidity) = sampler.sample_climate(world_pos.x, world_pos.z);
    quantize_tint(final_tint(kind, temp, humidity, season))
}

/// 客户端季节状态：网格构建时使用的当前季节，季节切换会触发区块重烘焙。
#[derive(Resource, Debug, Clone, Copy)]
pub struct SeasonState {
    /// 上次重烘焙时使用的季节，用于检测变化。
    pub last_seen: Season,
}

impl Default for SeasonState {
    fn default() -> Self {
        Self {
            last_seen: Season::Spring,
        }
    }
}

/// 注册 SeasonState 资源。
pub fn register_season_state(app: &mut App) {
    app.init_resource::<SeasonState>();
}

/// 消费 `SeasonChanged`，更新当前季节并把 `Rendered` 区块打回 `LightingReady` 触发重烘焙。
///
/// 仅在季节真正变化时触发；首次进入游戏的初始 `SeasonChanged` 也会触发一次整体重烘焙。
pub fn sync_season_state(
    mut events: MessageReader<crate::game::world::time::SeasonChanged>,
    mut state: ResMut<SeasonState>,
    mut chunk_query: Query<&mut crate::game::world::chunk::ChunkState>,
) {
    let Some(event) = events.read().last() else {
        return;
    };
    let season = event.0.season;
    if season == state.last_seen {
        return;
    }
    state.last_seen = season;
    // 把所有已渲染的区块打回 LightingReady 让网格生命周期重新派发任务。
    for mut state_ref in &mut chunk_query {
        if *state_ref == crate::game::world::chunk::ChunkState::Rendered {
            *state_ref = crate::game::world::chunk::ChunkState::LightingReady;
        }
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/client/renderer/world/tint.rs"]
mod tests;
