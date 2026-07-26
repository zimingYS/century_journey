use bevy::prelude::Component;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PlayerId(pub u64);

impl PlayerId {
    pub const LOCAL: Self = Self(0);

    pub const fn new(value: u64) -> Self {
        Self(value)
    }
}

#[derive(Component)]
pub struct Player;

/// 本地玩家标记。
///
/// 联机远程玩家也会拥有 Player / PlayerRig，但只有本地玩家会绑定本机相机、输入和第一人称可见性。
#[derive(Component)]
pub struct LocalPlayer;
