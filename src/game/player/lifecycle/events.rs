use crate::game::player::survival::events::DamageSource;
use bevy::prelude::*;

/// 死亡事件
#[derive(Message, Debug, Clone)]
pub struct DeathEvent {
    pub entity: Entity,
    pub source: DamageSource,
}

/// 玩家在死亡界面确认重生。
#[derive(Message, Debug, Clone, Copy)]
pub struct RespawnRequest {
    pub entity: Entity,
}
