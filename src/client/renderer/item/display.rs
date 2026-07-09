use crate::content::item::model::ItemModelDisplayTarget;

/// 物品渲染场景。
///
/// 外部系统只需要说明“把这个物品渲染到哪个场景”，不需要知道它最终是方块 cube、挤出贴图还是自定义模型。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ItemDisplayContext {
    /// GUI 图标。
    Gui,
    /// 第一人称右手。
    FirstPersonRightHand,
    /// 第一人称左手。
    FirstPersonLeftHand,
    /// 第三人称右手。
    ThirdPersonRightHand,
    /// 第三人称左手。
    ThirdPersonLeftHand,
    /// 地面掉落物。
    Ground,
    /// 展示框或固定展示。
    Fixed,
}

impl ItemDisplayContext {
    /// 转换到 content 层可序列化 display 目标。
    pub fn model_target(self) -> ItemModelDisplayTarget {
        match self {
            Self::Gui => ItemModelDisplayTarget::Gui,
            Self::FirstPersonRightHand => ItemModelDisplayTarget::FirstPersonRightHand,
            Self::FirstPersonLeftHand => ItemModelDisplayTarget::FirstPersonLeftHand,
            Self::ThirdPersonRightHand => ItemModelDisplayTarget::ThirdPersonRightHand,
            Self::ThirdPersonLeftHand => ItemModelDisplayTarget::ThirdPersonLeftHand,
            Self::Ground => ItemModelDisplayTarget::Ground,
            Self::Fixed => ItemModelDisplayTarget::Fixed,
        }
    }

    /// 当前场景生成的实体是否应该投射阴影。
    pub fn casts_shadows(self) -> bool {
        matches!(
            self,
            Self::ThirdPersonRightHand | Self::ThirdPersonLeftHand | Self::Ground | Self::Fixed
        )
    }
}
