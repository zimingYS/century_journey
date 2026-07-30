//! 定义玩家生成、死亡、重生、治疗和受伤等生命周期消息。

use crate::game::player::survival::events::DamageSource;
use bevy::prelude::*;

/// 死亡事件
#[derive(Message, Debug, Clone)]
pub struct DeathEvent {
    /// 已进入死亡状态的玩家实体。
    pub entity: Entity,
    /// 触发本次死亡的最终伤害来源。
    pub source: DamageSource,
}

/// 玩家在死亡界面确认重生。
#[derive(Message, Debug, Clone, Copy)]
pub struct RespawnRequest {
    /// 请求从死亡状态开始重生的玩家实体。
    pub entity: Entity,
}
