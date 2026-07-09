use std::collections::HashMap;

use bevy::prelude::*;

use crate::shared::identifier::Identifier;

/// 旧 UI 代码使用的资源别名，实际保存的是 GUI 3D 方块图标缓存。
pub type ItemModelRenderAssets = GuiItemIconCache;

/// GUI 3D 图标缓存。
///
/// 这里只缓存方块物品的 3D 渲染结果；工具、材料和普通物品在 GUI 中直接使用原始 2D 贴图。
#[derive(Resource, Default)]
pub struct GuiItemIconCache {
    /// ItemId -> 3D 图标渲染目标。
    icons: HashMap<Identifier, GuiItemIcon>,
    /// 是否已经完成当前资源版本的方块图标预热。
    prepared: bool,
    /// 图标缓存版本号，UI 可用它感知新图标是否生成或烘焙完成。
    revision: u64,
}

/// 单个 GUI 图标缓存项。
#[derive(Clone)]
pub(super) struct GuiItemIcon {
    /// 最终给 UI ImageNode 使用的图片句柄。
    pub image: Handle<Image>,
    /// 离屏相机是否已经至少保留到可认为渲染完成。
    pub ready: bool,
}

impl GuiItemIconCache {
    /// 查询某个物品已经烘焙完成、可以显示的 3D GUI 图标。
    pub fn icon_image(&self, item_identifier: &Identifier) -> Option<Handle<Image>> {
        self.icons.get(item_identifier).and_then(|preview| {
            if preview.ready {
                Some(preview.image.clone())
            } else {
                None
            }
        })
    }

    /// 返回本轮方块图标预热是否完成。
    pub fn is_prepared(&self) -> bool {
        self.prepared
    }

    /// 设置本轮方块图标预热状态。
    pub fn set_prepared(&mut self, prepared: bool) {
        self.prepared = prepared;
    }

    /// 返回缓存版本号。
    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// 清空 GUI 图标缓存并推进版本号。
    pub fn clear(&mut self) {
        self.icons.clear();
        self.prepared = false;
        self.revision = self.revision.wrapping_add(1);
    }

    /// 计算下一个图标在离屏预览场景中的横向偏移序号。
    pub(super) fn next_icon_index(&self) -> usize {
        self.icons.len()
    }

    /// 查询缓存中的图标句柄，不区分 pending / ready。
    pub(super) fn cached_icon_image(&self, item_identifier: &Identifier) -> Option<Handle<Image>> {
        self.icons
            .get(item_identifier)
            .map(|preview| preview.image.clone())
    }

    /// 插入新图标并推进版本号。
    pub(super) fn insert_icon(&mut self, item_identifier: Identifier, icon: GuiItemIcon) {
        self.icons.insert(item_identifier, icon);
        self.revision = self.revision.wrapping_add(1);
    }

    /// 标记某个图标已经完成离屏烘焙。
    pub(super) fn mark_icon_ready(&mut self, item_identifier: &Identifier) {
        let Some(icon) = self.icons.get_mut(item_identifier) else {
            return;
        };
        if icon.ready {
            return;
        }
        icon.ready = true;
        self.revision = self.revision.wrapping_add(1);
    }
}
