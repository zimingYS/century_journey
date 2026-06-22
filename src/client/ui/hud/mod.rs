use crate::client::ui::components::HudRoot;
use crate::game::inventory::state::InventoryState;
use bevy::prelude::*;

pub mod armor_bar;
pub mod crosshair;
pub mod health_bar;
pub mod hotbar;
pub mod hunger_bar;

/// HUD 显隐同步 — 控制 HudRoot 整体, 子元素 (准心/血条等) 继承 Visibility
pub fn sync_hud_visibility_system(
    state: Res<InventoryState>,
    mut query: Query<&mut Visibility, With<HudRoot>>,
) {
    let Ok(mut vis) = query.single_mut() else {
        return;
    };
    *vis = if state.opened {
        Visibility::Hidden
    } else {
        Visibility::Visible
    };
}
