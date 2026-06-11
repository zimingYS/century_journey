use bevy::prelude::*;
use crate::inventory::state::InventoryState;
use crate::ui::components::HudRoot;

pub mod crosshair;
pub mod hotbar;
pub mod health_bar;
pub mod hunger_bar;

/// HUD 显隐同步
pub fn sync_hud_visibility_system(
    state: Res<InventoryState>,
    mut query: Query<&mut Visibility, With<HudRoot>>,
) {
    let Ok(mut vis) = query.single_mut() else { return; };
    *vis = if state.opened { Visibility::Hidden } else { Visibility::Visible };
}
