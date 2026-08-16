//! 内容注册表与生成状态的同步系统。
//!
//! 在内容重载（`ContentReloadSet::Consumers`）阶段把生物群系、矿石、标签、
//! 方块 ID 和地表采样器刷新到权威生成状态，供后续生成任务消费。

use bevy::prelude::*;

use crate::content::biome::BiomeRegistry;
use crate::content::block::registry::BlockRegistry;
use crate::content::ore_vein::registry::OreVeinRegistry;
use crate::content::tag::runtime::RuntimeTagRegistry;
use crate::game::simulation::SimulationRng;
use crate::game::world::generation::block_ids::{CachedBlockIds, GenerationBlockIds};
use crate::game::world::generation::generator::WorldGenerator;
use crate::game::world::generation::pipeline::TerrainSurfaceSampler;

/// 把世界种子同步到权威模拟随机源。
pub(super) fn sync_simulation_rng_seed_system(
    world_generator: Res<WorldGenerator>,
    mut simulation_rng: ResMut<SimulationRng>,
) {
    simulation_rng.set_world_seed(world_generator.seed as u64);
}

/// 把内容生物群系注册表刷新到世界生成器。
pub(super) fn sync_world_biomes_system(
    registry: Res<BiomeRegistry>,
    mut world_generator: ResMut<WorldGenerator>,
) {
    if registry.is_empty() {
        log::error!("[世界] 生物群系注册表为空，跳过世界生成器刷新");
        return;
    }

    world_generator.set_biome_registry(registry.clone());
}

/// 缓存内容方块与标签映射到生成期方块 ID 快照。
pub(super) fn cache_block_ids_system(
    registry: Res<BlockRegistry>,
    tag_registry: Option<Res<RuntimeTagRegistry>>,
    mut cached: ResMut<CachedBlockIds>,
) {
    let block_ids = if let Some(tag_registry) = tag_registry {
        GenerationBlockIds::from_registry(&registry, &tag_registry)
    } else {
        log::warn!("[世界] RuntimeTagRegistry 尚未初始化，使用空标签");
        GenerationBlockIds::from_registry(&registry, &RuntimeTagRegistry::default())
    };

    cached.0 = block_ids;
}

/// 把内容矿脉定义刷新到世界生成管线。
pub(super) fn sync_world_ores_system(
    registry: Res<OreVeinRegistry>,
    mut world_generator: ResMut<WorldGenerator>,
) {
    let veins = registry.iter().cloned().collect::<Vec<_>>();
    world_generator.pipeline.replace_ore_veins(veins);
}

/// 在世界生成器和方块 ID 快照都刷新后重建 Client 可调用的只读采样服务。
///
/// 服务是确定性表现数据的唯一跨层出口；Client 不需要也不允许读取生成管线内部字段。
pub(super) fn sync_terrain_surface_sampler_system(
    world_generator: Res<WorldGenerator>,
    cached_block_ids: Res<CachedBlockIds>,
    mut sampler: ResMut<TerrainSurfaceSampler>,
) {
    *sampler = TerrainSurfaceSampler::from_generation_state(
        world_generator.pipeline.clone(),
        cached_block_ids.0.clone(),
    );
}
