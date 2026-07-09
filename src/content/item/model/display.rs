use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// 物品模型支持的展示目标。
///
/// 这里保持和 Minecraft item display 的概念接近：同一个模型在 GUI、第一人称、第三人称、地面掉落物、展示框中可以拥有不同变换。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemModelDisplayTarget {
    /// 背包、快捷栏等 UI 中的图标展示。
    Gui,
    /// 第一人称右手。
    FirstPersonRightHand,
    /// 第一人称左手。
    FirstPersonLeftHand,
    /// 第三人称右手。
    ThirdPersonRightHand,
    /// 第三人称左手。
    ThirdPersonLeftHand,
    /// 掉落在世界里的物品实体。
    Ground,
    /// 展示框、展示台等固定展示场景。
    Fixed,
}

/// 可序列化的物品展示变换。
///
/// JSON 中只保存基础数组，不保存 Bevy Transform，避免 content 层和渲染运行时资源耦合。
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ItemDisplayTransform {
    /// 位置偏移，单位使用渲染世界坐标。
    #[serde(default)]
    pub translation: [f32; 3],
    /// 欧拉角旋转，单位是角度，转换时使用 XYZ 顺序。
    #[serde(default)]
    pub rotation: [f32; 3],
    /// 三轴缩放。
    #[serde(default = "unit_scale")]
    pub scale: [f32; 3],
}

impl ItemDisplayTransform {
    /// 创建三轴等比缩放的展示变换。
    pub fn uniform(translation: [f32; 3], rotation: [f32; 3], scale: f32) -> Self {
        Self {
            translation,
            rotation,
            scale: [scale; 3],
        }
    }

    /// 把纯数据定义转换成 Bevy Transform。
    pub fn to_transform(self) -> Transform {
        Transform {
            translation: Vec3::from_array(self.translation),
            rotation: Quat::from_euler(
                EulerRot::XYZ,
                self.rotation[0].to_radians(),
                self.rotation[1].to_radians(),
                self.rotation[2].to_radians(),
            ),
            scale: Vec3::from_array(self.scale),
        }
    }
}

impl Default for ItemDisplayTransform {
    fn default() -> Self {
        Self {
            translation: [0.0; 3],
            rotation: [0.0; 3],
            scale: unit_scale(),
        }
    }
}

/// serde 默认值：不缩放。
fn unit_scale() -> [f32; 3] {
    [1.0; 3]
}

/// 一个物品模型在各类展示场景中的变换集合。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ItemModelDisplay {
    /// GUI 图标展示变换。
    #[serde(default)]
    pub gui: Option<ItemDisplayTransform>,
    /// 第一人称右手展示变换。
    #[serde(default)]
    pub first_person_right_hand: Option<ItemDisplayTransform>,
    /// 第一人称左手展示变换。
    #[serde(default)]
    pub first_person_left_hand: Option<ItemDisplayTransform>,
    /// 第三人称右手展示变换。
    #[serde(default)]
    pub third_person_right_hand: Option<ItemDisplayTransform>,
    /// 第三人称左手展示变换。
    #[serde(default)]
    pub third_person_left_hand: Option<ItemDisplayTransform>,
    /// 地面掉落物展示变换。
    #[serde(default)]
    pub ground: Option<ItemDisplayTransform>,
    /// 固定展示场景变换。
    #[serde(default)]
    pub fixed: Option<ItemDisplayTransform>,
}

impl ItemModelDisplay {
    /// 获取指定场景的 Bevy Transform；未配置时回退到单位变换。
    pub fn transform_for(&self, target: ItemModelDisplayTarget) -> Transform {
        self.get(target).unwrap_or_default().to_transform()
    }

    /// 获取指定场景的原始展示变换。
    pub fn get(&self, target: ItemModelDisplayTarget) -> Option<ItemDisplayTransform> {
        match target {
            ItemModelDisplayTarget::Gui => self.gui,
            ItemModelDisplayTarget::FirstPersonRightHand => self.first_person_right_hand,
            ItemModelDisplayTarget::FirstPersonLeftHand => self.first_person_left_hand,
            ItemModelDisplayTarget::ThirdPersonRightHand => self.third_person_right_hand,
            ItemModelDisplayTarget::ThirdPersonLeftHand => self.third_person_left_hand,
            ItemModelDisplayTarget::Ground => self.ground,
            ItemModelDisplayTarget::Fixed => self.fixed,
        }
    }

    /// 设置某个展示场景的变换，并返回 self 方便链式构建默认值。
    pub fn set(
        &mut self,
        target: ItemModelDisplayTarget,
        transform: ItemDisplayTransform,
    ) -> &mut Self {
        match target {
            ItemModelDisplayTarget::Gui => self.gui = Some(transform),
            ItemModelDisplayTarget::FirstPersonRightHand => {
                self.first_person_right_hand = Some(transform)
            }
            ItemModelDisplayTarget::FirstPersonLeftHand => {
                self.first_person_left_hand = Some(transform)
            }
            ItemModelDisplayTarget::ThirdPersonRightHand => {
                self.third_person_right_hand = Some(transform)
            }
            ItemModelDisplayTarget::ThirdPersonLeftHand => {
                self.third_person_left_hand = Some(transform)
            }
            ItemModelDisplayTarget::Ground => self.ground = Some(transform),
            ItemModelDisplayTarget::Fixed => self.fixed = Some(transform),
        }
        self
    }

    /// 方块物品的默认展示变换。
    ///
    /// GUI 使用接近 Minecraft 背包图标的 30 度俯视 + 45 度侧向角度；手持和掉落物使用较小缩放，避免挡住视野。
    pub fn block_defaults() -> Self {
        let mut display = Self::default();
        display
            .set(
                ItemModelDisplayTarget::Gui,
                ItemDisplayTransform::uniform([0.0, 0.0, 0.0], [30.0, -45.0, 0.0], 1.08),
            )
            .set(
                ItemModelDisplayTarget::FirstPersonRightHand,
                ItemDisplayTransform::uniform([0.0, -0.04, -0.7], [0.0, 15.0, 0.0], 0.4),
            )
            .set(
                ItemModelDisplayTarget::FirstPersonLeftHand,
                ItemDisplayTransform::uniform([0.0, -0.04, -0.7], [0.0, -15.0, 0.0], 0.4),
            )
            .set(
                ItemModelDisplayTarget::ThirdPersonRightHand,
                ItemDisplayTransform::uniform([0.0, 0.1, 0.0], [0.0, 45.0, 0.0], 0.35),
            )
            .set(
                ItemModelDisplayTarget::ThirdPersonLeftHand,
                ItemDisplayTransform::uniform([0.0, 0.1, 0.0], [0.0, -45.0, 0.0], 0.35),
            )
            .set(
                ItemModelDisplayTarget::Ground,
                ItemDisplayTransform::uniform([0.0, 0.15, 0.0], [0.0, 0.0, 0.0], 0.35),
            )
            .set(
                ItemModelDisplayTarget::Fixed,
                ItemDisplayTransform::uniform([0.0, 0.0, 0.0], [0.0, 0.0, 0.0], 0.5),
            );
        display
    }

    /// 普通贴图物品的默认展示变换。
    ///
    /// GUI 直接显示 2D 贴图，不走这里的 3D 变换；第一人称、掉落物、展示框仍可使用挤出模型。
    pub fn generated_defaults(handheld: bool) -> Self {
        let mut display = Self::default();
        let first_person_rotation = if handheld {
            [0.0, -60.0, 30.0]
        } else {
            [0.0, -25.0, 0.0]
        };

        display
            .set(
                ItemModelDisplayTarget::Gui,
                ItemDisplayTransform::uniform([0.0, 0.0, 0.0], [0.0, 0.0, 0.0], 1.0),
            )
            .set(
                ItemModelDisplayTarget::FirstPersonRightHand,
                ItemDisplayTransform::uniform([0.0, 0.3, -0.3], first_person_rotation, 1.0),
            )
            .set(
                ItemModelDisplayTarget::FirstPersonLeftHand,
                ItemDisplayTransform::uniform([0.0, 0.3, -0.3], [0.0, 60.0, -30.0], 1.0),
            )
            .set(
                ItemModelDisplayTarget::ThirdPersonRightHand,
                ItemDisplayTransform::uniform([0.0, 0.15, 0.0], [0.0, -90.0, 45.0], 0.75),
            )
            .set(
                ItemModelDisplayTarget::ThirdPersonLeftHand,
                ItemDisplayTransform::uniform([0.0, 0.15, 0.0], [0.0, 90.0, -45.0], 0.75),
            )
            .set(
                ItemModelDisplayTarget::Ground,
                ItemDisplayTransform::uniform([0.0, 0.08, 0.0], [0.0, 0.0, 0.0], 0.5),
            )
            .set(
                ItemModelDisplayTarget::Fixed,
                ItemDisplayTransform::uniform([0.0, 0.0, 0.0], [0.0, 0.0, 0.0], 0.75),
            );
        display
    }
}
