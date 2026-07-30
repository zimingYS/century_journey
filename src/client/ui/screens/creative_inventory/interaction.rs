//! 处理创造物品栏的显隐、关闭、搜索占位和分类高亮。

use bevy::prelude::*;

use crate::client::ui::components::{
    CreativeCloseButton, CreativeInventoryOverlay, CreativeSearchPlaceholder,
};
use crate::client::ui::navigation::{UiNavigation, UiScreen};
use crate::client::ui::widgets::common::UiControl;
use crate::client::ui::widgets::slot::{CategoryTab, SearchInputState};
use crate::game::gameplay::gamemode::PlayerGameMode;
use crate::game::inventory::state::LocalInventory;
/// 同步创造物品栏遮罩显隐。
pub fn update_creative_visibility_system(
    state: LocalInventory,
    gamemode: Res<PlayerGameMode>,
    mut query: Query<&mut Visibility, With<CreativeInventoryOverlay>>,
) {
    let Ok(mut vis) = query.single_mut() else {
        return;
    };
    let target = if state.opened && gamemode.is_creative() {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    if *vis != target {
        *vis = target;
    }
}

/// 点击右上角关闭按钮时关闭创造物品栏。
pub fn creative_close_button_system(
    button_query: Query<&Interaction, (Changed<Interaction>, With<CreativeCloseButton>)>,
    mut writer: MessageWriter<UiNavigation>,
) {
    let pressed = button_query
        .iter()
        .any(|interaction| *interaction == Interaction::Pressed);
    if pressed {
        writer.write(UiNavigation::Close(UiScreen::Inventory));
    }
}

/// 同步搜索框占位文字显隐，避免占位文字参与真实搜索。
pub fn sync_creative_search_placeholder_system(
    state: LocalInventory,
    search_state: Res<SearchInputState>,
    mut query: Query<&mut Visibility, With<CreativeSearchPlaceholder>>,
) {
    let Ok(mut visibility) = query.single_mut() else {
        return;
    };

    *visibility = if state.opened && state.creative.search_text.is_empty() && !search_state.active {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
}

/// 分类标签高亮。
pub fn update_category_highlight_system(
    state: LocalInventory,
    mut query: Query<(&CategoryTab, &mut UiControl)>,
) {
    for (tab, mut control) in &mut query {
        control.selected = tab.category_index == state.creative.selected_tab;
    }
}
