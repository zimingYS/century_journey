//! 定义飞行状态组件、切换请求与飞行判定纯函数。

use bevy::prelude::*;

/// 双击切换飞行请求；Client 在渲染帧检测到双击后写入，Game 在固定步消费
#[derive(Message, Debug, Clone, Copy)]
pub struct ToggleFlightRequest;

/// 玩家飞行状态组件，随玩家实体存在；是否启用由切换系统在固定步维护
#[derive(Component, Debug, Clone, Copy)]
pub struct PlayerFlight {
    /// 当前是否处于飞行状态
    pub enabled: bool,
    /// 飞行目标速度（方块/秒）
    pub fly_speed: f32,
}

impl Default for PlayerFlight {
    fn default() -> PlayerFlight {
        Self {
            enabled: false,
            fly_speed: 10.0,
        }
    }
}
