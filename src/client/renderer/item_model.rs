use bevy::prelude::*;

use crate::client::renderer::item::ItemRenderer;
use crate::content::item::registry::registry::ItemRegistry;
use crate::content::item::texture::registry::ItemTextureRegistry;
use crate::shared::item_id::ItemId;

pub use crate::client::renderer::item::gui_icon_cache::ItemModelRenderAssets;
pub use crate::client::renderer::item::renderer::prepare_item_model_render_assets_system;

/// 兼容旧 UI 代码的物品图标查询入口。
///
/// 新代码优先直接使用 `client::renderer::item::ItemRenderer`。
pub struct ItemModelRenderer;

impl ItemModelRenderer {
    /// 查询 GUI 中应显示的物品图片。
    pub fn item_icon_image(
        item: &ItemId,
        item_registry: Option<&ItemRegistry>,
        item_textures: Option<&ItemTextureRegistry>,
        previews: &ItemModelRenderAssets,
    ) -> Option<Handle<Image>> {
        let item_textures = item_textures?;
        ItemRenderer::gui_icon_image(item, item_registry, item_textures, previews)
    }
}
