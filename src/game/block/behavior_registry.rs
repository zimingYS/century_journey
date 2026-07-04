use crate::content::block::behavior::{BlockBehavior, DefaultBlockBehavior};
use crate::content::block::registry::BlockRegistry;
use bevy::prelude::*;
use std::collections::HashMap;

/// Game 层方块行为注册表。
///
/// 负责存储 BlockBehavior 实例和按 ID/名称查询。
/// Content 层的 BlockProperty 通过 `behavior_type: String` 关联行为名称，
/// 具体行为实现和执行由 Game 层负责。
#[derive(Resource, Default)]
pub struct BlockBehaviorRegistry {
    pub behaviors:
        HashMap<String, Box<dyn BlockBehavior<crate::game::world::storage::WorldStorage>>>,
}

impl BlockBehaviorRegistry {
    /// 获取方块行为（按行为类型名称）
    pub fn get_behavior(
        &self,
        behavior_type: &str,
    ) -> Option<&dyn BlockBehavior<crate::game::world::storage::WorldStorage>> {
        self.behaviors.get(behavior_type).map(|b| b.as_ref())
    }

    /// 通过方块运行时 ID 获取行为
    pub fn get_behavior_by_id(
        &self,
        id: u16,
        block_registry: &BlockRegistry,
    ) -> &dyn BlockBehavior<crate::game::world::storage::WorldStorage> {
        let prop = block_registry.get(id);
        match prop {
            Some(p) if !p.behavior_type.is_empty() => self
                .behaviors
                .get(&p.behavior_type)
                .map(|b| b.as_ref())
                .unwrap_or(&DEFAULT_BEHAVIOR),
            _ => &DEFAULT_BEHAVIOR,
        }
    }
}

/// 全局默认行为实例（不可变 static ref 避免重复分配）
static DEFAULT_BEHAVIOR: DefaultBlockBehavior = DefaultBlockBehavior;

/// 初始化方块行为注册表（注册内置行为）
pub fn init_behavior_registry_system(mut registry: ResMut<BlockBehaviorRegistry>) {
    // 只在首次初始化时注册
    if !registry.behaviors.is_empty() {
        return;
    }
    registry
        .behaviors
        .insert("default".to_string(), Box::new(DefaultBlockBehavior));
    // FallingBlockBehavior 等未来行为在此注册
    info!("[方块行为] 已注册 {} 个方块行为", registry.behaviors.len());
}
