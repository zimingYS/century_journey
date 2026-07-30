//! 定义玩家模型部件、关节、网格和锚点等表现组件。

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
    /// 构造右上臂部件。
    pub fn upper_arm_r() -> Self {
        PlayerPart::UpperArmL(true)
    }
    /// 构造左上臂部件。
    pub fn upper_arm_l() -> Self {
        PlayerPart::UpperArmL(false)
    }
    /// 构造右前臂部件。
    pub fn forearm_r() -> Self {
        PlayerPart::ForearmL(true)
    }
    /// 构造左前臂部件。
    pub fn forearm_l() -> Self {
        PlayerPart::ForearmL(false)
    }
    /// 构造右手部件。
    pub fn hand_r() -> Self {
        PlayerPart::HandL(true)
    }
    /// 构造左手部件。
    pub fn hand_l() -> Self {
        PlayerPart::HandL(false)
    }
    /// 构造右大腿部件。
    pub fn thigh_r() -> Self {
        PlayerPart::ThighL(true)
    }
    /// 构造左大腿部件。
    pub fn thigh_l() -> Self {
        PlayerPart::ThighL(false)
    }
    /// 构造右小腿部件。
    pub fn calf_r() -> Self {
        PlayerPart::CalfL(true)
    }
    /// 构造左小腿部件。
    pub fn calf_l() -> Self {
        PlayerPart::CalfL(false)
    }
    /// 构造右脚部件。
    pub fn foot_r() -> Self {
        PlayerPart::FootL(true)
    }
    /// 构造左脚部件。
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
/// 胸甲或胸前附件的模型挂点。
#[derive(Component)]
pub struct ChestAnchor;
/// 背部装备和附件的模型挂点。
#[derive(Component)]
pub struct BackAnchor;
