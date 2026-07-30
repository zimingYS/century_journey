//! 定义玩家实体标记、稳定玩家 ID 和本地玩家身份。

use bevy::prelude::Component;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
/// 跨实体重建和存档仍保持稳定的玩家 ID。
pub struct PlayerId(pub u64);

impl PlayerId {
    /// 单机客户端本地玩家使用的保留 ID。
    pub const LOCAL: Self = Self(0);

    /// 使用持久化数值创建玩家 ID。
    pub const fn new(value: u64) -> Self {
        Self(value)
    }
}

#[derive(Component)]
/// 标记属于权威玩家领域的实体。
pub struct Player;

/// 本地玩家标记。
///
/// 联机远程玩家也会拥有 Player / PlayerRig，但只有本地玩家会绑定本机相机、输入和第一人称可见性。
#[derive(Component)]
pub struct LocalPlayer;
