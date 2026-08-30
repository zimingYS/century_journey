//! 把生存物品栏按钮交互转换为权威层命令。

use bevy::prelude::*;

use crate::client::ui::components::{
    CompactBackpackButton, SortBackpackButton, SurvivalCloseButton,
};
use crate::client::ui::navigation::{UiNavigation, UiScreen};
use crate::game::inventory::events::InventoryCommand;

/// 点击右上角关闭按钮时关闭生存物品栏。
pub fn survival_close_button_system(
    button_query: Query<&Interaction, (Changed<Interaction>, With<SurvivalCloseButton>)>,
    mut writer: MessageWriter<UiNavigation>,
) {
    let pressed = button_query
        .iter()
        .any(|interaction| *interaction == Interaction::Pressed);
    if pressed {
        writer.write(UiNavigation::Close(UiScreen::Inventory));
    }
}

/// 把收拢和整理按钮转换为 Game 层物品栏命令。
/// 组合过滤器确保本系统只消费两种背包管理按钮。
#[allow(clippy::type_complexity)]
pub fn backpack_management_button_system(
    mut writer: MessageWriter<InventoryCommand>,
    query: Query<
        (
            &Interaction,
            Option<&CompactBackpackButton>,
            Option<&SortBackpackButton>,
        ),
        (
            Changed<Interaction>,
            With<Button>,
            Or<(With<CompactBackpackButton>, With<SortBackpackButton>)>,
        ),
    >,
) {
    for (interaction, compact, sort) in &query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        if compact.is_some() {
            writer.write(InventoryCommand::CompactBackpack);
        } else if sort.is_some() {
            writer.write(InventoryCommand::SortBackpack);
        }
    }
}
