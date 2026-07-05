use crate::game::inventory::state::InventoryState;
use bevy::prelude::*;

pub mod bottom;
pub mod center;
pub mod left;
pub mod left_bottom;
pub mod left_top;
pub mod plugin;
pub mod right;
pub mod right_bottom;
pub mod right_top;
pub mod top;

/// 处理HUD的根节点
#[derive(Component)]
pub struct HudRoot;

/// 生成HUD根节点
pub fn spawn_hud_root_system(mut commands: Commands) {
    commands.spawn((
        HudRoot,
        Name::new("HudRoot"),
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
    ));
}

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
