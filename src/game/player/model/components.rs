use bevy::prelude::*;

/// 骨骼关节
#[derive(Component, Debug, Clone, Copy)]
pub struct PlayerJoint(pub PlayerPart);

/// 骨骼网格
#[derive(Component, Debug, Clone, Copy)]
pub struct PlayerMesh(pub PlayerPart);

/// 玩家部件类型
/// 区分所有骨骼部位，布尔标记区分左右肢体
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerPart {
    Head,
    Body,
    UpperArmL(bool),
    ForearmL(bool),
    HandL(bool),
    ThighL(bool),
    CalfL(bool),
    FootL(bool),
}

impl PlayerPart {
    /// 根据部位进行构造
    pub fn upper_arm_r() -> Self {
        PlayerPart::UpperArmL(true)
    }
    pub fn upper_arm_l() -> Self {
        PlayerPart::UpperArmL(false)
    }
    pub fn forearm_r() -> Self {
        PlayerPart::ForearmL(true)
    }
    pub fn forearm_l() -> Self {
        PlayerPart::ForearmL(false)
    }
    pub fn hand_r() -> Self {
        PlayerPart::HandL(true)
    }
    pub fn hand_l() -> Self {
        PlayerPart::HandL(false)
    }
    pub fn thigh_r() -> Self {
        PlayerPart::ThighL(true)
    }
    pub fn thigh_l() -> Self {
        PlayerPart::ThighL(false)
    }
    pub fn calf_r() -> Self {
        PlayerPart::CalfL(true)
    }
    pub fn calf_l() -> Self {
        PlayerPart::CalfL(false)
    }
    pub fn foot_r() -> Self {
        PlayerPart::FootL(true)
    }
    pub fn foot_l() -> Self {
        PlayerPart::FootL(false)
    }

    /// 判断当前部件是否为右侧肢体
    pub fn is_right(&self) -> bool {
        matches!(self, PlayerPart::UpperArmL(r) | PlayerPart::ForearmL(r)
            | PlayerPart::HandL(r) | PlayerPart::ThighL(r) | PlayerPart::CalfL(r)
            | PlayerPart::FootL(r) if *r)
    }
}

/// 玩家Rig根节点
#[derive(Component)]
pub struct PlayerRig;

/// 玩家全局模型
#[derive(Component)]
pub struct PlayerModelMarker;

/// 手持物品挂点
#[derive(Component)]
pub struct HeldItemAnchor;

/// 副手物品挂点
#[derive(Component)]
pub struct OffHandAnchor;

/// 装备挂点
#[derive(Component)]
pub struct HelmetAnchor;
#[derive(Component)]
pub struct ChestAnchor;
#[derive(Component)]
pub struct BackAnchor;
