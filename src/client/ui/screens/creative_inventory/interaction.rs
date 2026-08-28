//! 处理创造物品栏的显隐、关闭、搜索占位、分类高亮和分页。

use bevy::prelude::*;

use super::catalog::tab_page_count;
use crate::client::ui::components::{
    CreativeCloseButton, CreativeInventoryOverlay, CreativeSearchPlaceholder, CreativeTabLabel,
    CreativeTabPagerLeft, CreativeTabPagerRight,
};
use crate::client::ui::navigation::{UiNavigation, UiScreen};
use crate::client::ui::resources::creative_assets::CreativeUiAssets;
use crate::client::ui::screens::setup::CREATIVE_TABS_PER_PAGE;
use crate::client::ui::theme::ui_theme::UiTheme;
use crate::client::ui::widgets::common::UiControl;
use crate::client::ui::widgets::slot::{CategoryTab, SearchInputState};
use crate::game::gameplay::gamemode::PlayerGameMode;
use crate::game::inventory::state::LocalInventory;

/// 创造物品栏分类标签的当前分页；页码从 0 开始。
#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct CreativeTabPage {
    pub page: usize,
}

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

/// 分类标签高亮：切换选中纹理、文字颜色并叠加悬停提亮。
///
/// 标签纹理在生成时即选定初始状态，选中变化后由本系统随状态切换；
/// 所有写入都先比较再赋值，避免污染变更检测。
pub fn update_category_highlight_system(
    state: LocalInventory,
    theme: Res<UiTheme>,
    assets: Res<CreativeUiAssets>,
    mut query: Query<
        (
            &CategoryTab,
            &Interaction,
            &mut UiControl,
            &mut ImageNode,
            &Children,
        ),
        With<Button>,
    >,
    mut labels: Query<&mut TextColor, With<CreativeTabLabel>>,
) {
    for (tab, interaction, mut control, mut image, children) in &mut query {
        let selected = tab.category_index == state.creative.selected_tab;
        if control.selected != selected {
            control.selected = selected;
        }

        let target = if selected {
            &assets.tab_active
        } else {
            &assets.tab_inactive
        };
        if image.image != *target {
            image.image = target.clone();
        }

        let tint = match interaction {
            Interaction::Hovered | Interaction::Pressed => Color::srgb(1.2, 1.2, 1.2),
            Interaction::None => Color::WHITE,
        };
        if image.color != tint {
            image.color = tint;
        }

        let text_color = if selected {
            theme.text_primary
        } else {
            theme.tab_inactive_text
        };
        for child in children.iter() {
            let Ok(mut color) = labels.get_mut(child) else {
                continue;
            };
            if color.0 != text_color {
                color.0 = text_color;
            }
        }
    }
}

/// 处理分类列表底部翻页按钮：循环翻页，页数变化时由渲染系统钳制。
pub fn creative_tab_pager_click_system(
    state: LocalInventory,
    pager: Res<CreativeTabPage>,
    left_query: Query<&Interaction, (Changed<Interaction>, With<CreativeTabPagerLeft>)>,
    right_query: Query<&Interaction, (Changed<Interaction>, With<CreativeTabPagerRight>)>,
    mut commands: Commands,
) {
    let pressed_left = left_query
        .iter()
        .any(|interaction| *interaction == Interaction::Pressed);
    let pressed_right = right_query
        .iter()
        .any(|interaction| *interaction == Interaction::Pressed);
    if !pressed_left && !pressed_right {
        return;
    }

    let category_count = state.creative.categories.len();
    if category_count <= CREATIVE_TABS_PER_PAGE {
        return;
    }
    let pages = tab_page_count(category_count);
    let next = if pressed_left {
        (pager.page + pages - 1) % pages
    } else {
        (pager.page + 1) % pages
    };
    commands.insert_resource(CreativeTabPage { page: next });
}

/// 翻页按钮悬停/按下时提亮，离开恢复原色。
///
/// 按钮本身是带透明区域的像素图标，通过 `ImageNode.color` 叠加
/// 亮度实现反馈，不改变纹理本身。
#[allow(clippy::type_complexity)]
pub fn update_pager_button_highlight_system(
    mut query: Query<
        (&Interaction, &mut ImageNode),
        (
            Changed<Interaction>,
            Or<(With<CreativeTabPagerLeft>, With<CreativeTabPagerRight>)>,
        ),
    >,
) {
    for (interaction, mut image) in &mut query {
        let tint = match interaction {
            Interaction::Hovered | Interaction::Pressed => Color::srgb(1.3, 1.3, 1.3),
            Interaction::None => Color::WHITE,
        };
        if image.color != tint {
            image.color = tint;
        }
    }
}
