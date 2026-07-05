use crate::client::ui::hud::HudRoot;
use bevy::prelude::*;

#[derive(Component)]
pub struct TopHud;

/// 生成顶部HUD节点
pub fn spawn_top_hud_system(mut commands: Commands, hud: Query<Entity, With<HudRoot>>) {
    let Ok(root) = hud.single() else {
        log::error!("HUD根节点未生成！");
        return;
    };

    commands.entity(root).with_children(|parent| {
        parent.spawn((
            TopHud,
            Name::new("TopHud"),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                width: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Start,
                ..default()
            },
        ));
    });
}
