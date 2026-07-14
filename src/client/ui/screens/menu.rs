use bevy::input_focus::InputFocus;
use bevy::prelude::*;
use bevy::text::{EditableText, TextCursorStyle};
use bevy::ui_widgets::SelectAllOnFocus;

use crate::app::flow::{
    DialogKind, DialogState, FlowCommand, GameSettings, LoadingStatus, MenuPage, SettingAction,
    WorldCatalog,
};
use crate::client::ui::navigation::{UiScreen, UiScreenRoot, UiScreenStack};
use crate::client::ui::resources::frame_assets::UiFrameKind;
use crate::client::ui::resources::ui_font::UiFont;
use crate::client::ui::theme::ui_theme::UiTheme;
use crate::client::ui::widgets::common::UiScrollArea;
use crate::client::ui::widgets::common::{UiControl, UiControlKind, spawn_text_button};
use crate::shared::states::{AppState, InputContextState};

#[derive(Component)]
pub(crate) struct WorldList;

#[derive(Component)]
pub(crate) struct WorldNameInput;

#[derive(Component)]
pub(crate) struct WorldEntryButton {
    id: String,
}

#[derive(Component, Default)]
pub(crate) struct PlayButton;
#[derive(Component, Default)]
pub(crate) struct CreateButton;
#[derive(Component, Default)]
pub(crate) struct DeleteButton;
#[derive(Component, Default)]
pub(crate) struct MainSettingsButton;
#[derive(Component, Default)]
pub(crate) struct QuitButton;
#[derive(Component, Default)]
pub(crate) struct ResumeButton;
#[derive(Component, Default)]
pub(crate) struct PauseSettingsButton;
#[derive(Component, Default)]
pub(crate) struct SaveQuitButton;
#[derive(Component, Default)]
pub(crate) struct SettingsBackButton;
#[derive(Component, Default)]
pub(crate) struct DialogConfirmButton;
#[derive(Component, Default)]
pub(crate) struct DialogCancelButton;

#[derive(Component)]
pub(crate) struct SettingsRoot;
#[derive(Component)]
pub(crate) struct DialogRoot;
#[derive(Component)]
pub(crate) struct DialogTitle;
#[derive(Component)]
pub(crate) struct DialogMessage;
#[derive(Component)]
pub(crate) struct LoadingTitle;
#[derive(Component)]
pub(crate) struct LoadingDetail;

#[derive(Component, Clone, Copy)]
pub(crate) struct SettingButton(SettingAction);

#[derive(Component, Clone, Copy)]
pub(crate) enum SettingValue {
    RenderDistance,
    MasterVolume,
    MouseSensitivity,
    UiScale,
    Fullscreen,
    Vsync,
}

#[derive(Resource, Debug, Clone)]
pub(crate) struct WorldNameDraft(String);

impl Default for WorldNameDraft {
    fn default() -> Self {
        Self("new_world".into())
    }
}

pub(crate) fn init_menu_resources(app: &mut App) {
    app.init_resource::<WorldNameDraft>();
}

pub(crate) fn spawn_menu_screens_system(
    mut commands: Commands,
    theme: Res<UiTheme>,
    ui_font: Res<UiFont>,
) {
    spawn_loading_screen(&mut commands, &theme, &ui_font);
    spawn_main_menu(&mut commands, &theme, &ui_font);
    spawn_pause_menu(&mut commands, &theme, &ui_font);
    spawn_settings(&mut commands, &theme, &ui_font);
    spawn_dialog(&mut commands, &theme, &ui_font);
}

fn overlay_node() -> Node {
    Node {
        position_type: PositionType::Absolute,
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        padding: UiRect::all(Val::Px(24.0)),
        ..default()
    }
}

fn spawn_loading_screen(commands: &mut Commands, theme: &UiTheme, ui_font: &UiFont) {
    commands
        .spawn((
            UiScreenRoot::new(UiScreen::Loading),
            Name::new("LoadingScreen"),
            overlay_node(),
            BackgroundColor(Color::srgb(0.025, 0.03, 0.035)),
            GlobalZIndex(3000),
            Visibility::Hidden,
        ))
        .with_children(|root| {
            root.spawn((
                UiFrameKind::Generic,
                Node {
                    width: Val::Px(520.0),
                    min_height: Val::Px(180.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    padding: UiRect::all(Val::Px(28.0)),
                    row_gap: Val::Px(14.0),
                    ..default()
                },
            ))
            .with_children(|panel| {
                panel.spawn((
                    LoadingTitle,
                    Text::new("正在启动"),
                    title_font(ui_font, 28.0),
                    TextColor(theme.text_primary),
                ));
                panel.spawn((
                    LoadingDetail,
                    Text::new("正在加载内容资源..."),
                    body_font(ui_font, 15.0),
                    TextColor(theme.text_secondary),
                ));
            });
        });
}

fn spawn_main_menu(commands: &mut Commands, theme: &UiTheme, ui_font: &UiFont) {
    commands
        .spawn((
            UiScreenRoot::new(UiScreen::MainMenu),
            Name::new("MainMenu"),
            overlay_node(),
            BackgroundColor(Color::srgba(0.02, 0.025, 0.03, 0.97)),
            GlobalZIndex(2000),
            Visibility::Hidden,
        ))
        .with_children(|root| {
            root.spawn((
                UiFrameKind::Generic,
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    max_width: Val::Px(1080.0),
                    max_height: Val::Px(700.0),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(22.0)),
                    row_gap: Val::Px(16.0),
                    ..default()
                },
            ))
            .with_children(|panel| {
                panel.spawn((
                    Text::new("CENTURY JOURNEY"),
                    title_font(ui_font, 34.0),
                    TextColor(theme.text_primary),
                ));
                panel.spawn((
                    Text::new("世界"),
                    title_font(ui_font, 20.0),
                    TextColor(theme.text_secondary),
                ));
                panel
                    .spawn(Node {
                        width: Val::Percent(100.0),
                        min_height: Val::Px(0.0),
                        flex_grow: 1.0,
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(14.0),
                        ..default()
                    })
                    .with_children(|body| {
                        body.spawn((
                            WorldList,
                            UiScrollArea,
                            Interaction::None,
                            Pickable::default(),
                            ScrollPosition::default(),
                            Node {
                                width: Val::Percent(64.0),
                                min_height: Val::Px(0.0),
                                flex_direction: FlexDirection::Column,
                                padding: UiRect::all(Val::Px(10.0)),
                                row_gap: Val::Px(6.0),
                                overflow: Overflow::scroll_y(),
                                ..default()
                            },
                            BackgroundColor(theme.bg_content),
                            BorderColor::all(theme.border_default),
                        ));
                        body.spawn((Node {
                            flex_grow: 1.0,
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(10.0),
                            ..default()
                        },))
                            .with_children(|actions| {
                                actions.spawn((
                                    Text::new("新世界名称"),
                                    body_font(ui_font, 14.0),
                                    TextColor(theme.text_secondary),
                                ));
                                actions.spawn((
                                    WorldNameInput,
                                    EditableText::new("new_world"),
                                    TextCursorStyle::default(),
                                    SelectAllOnFocus,
                                    TextLayout::no_wrap(),
                                    body_font(ui_font, 15.0),
                                    TextColor(theme.text_primary),
                                    Node {
                                        width: Val::Percent(100.0),
                                        height: Val::Px(42.0),
                                        padding: UiRect::all(Val::Px(10.0)),
                                        border: UiRect::all(Val::Px(1.0)),
                                        overflow: Overflow::clip_x(),
                                        ..default()
                                    },
                                    BackgroundColor(theme.search_bg),
                                    BorderColor::all(theme.search_border),
                                ));
                                spawn_text_button(
                                    actions,
                                    CreateButton,
                                    "创建世界",
                                    UiControlKind::Button,
                                    theme,
                                    ui_font,
                                );
                                spawn_text_button(
                                    actions,
                                    PlayButton,
                                    "进入世界",
                                    UiControlKind::Button,
                                    theme,
                                    ui_font,
                                );
                                spawn_text_button(
                                    actions,
                                    DeleteButton,
                                    "删除世界",
                                    UiControlKind::Button,
                                    theme,
                                    ui_font,
                                );
                                actions.spawn(Node {
                                    flex_grow: 1.0,
                                    ..default()
                                });
                                spawn_text_button(
                                    actions,
                                    MainSettingsButton,
                                    "设置",
                                    UiControlKind::Button,
                                    theme,
                                    ui_font,
                                );
                                spawn_text_button(
                                    actions,
                                    QuitButton,
                                    "退出游戏",
                                    UiControlKind::Button,
                                    theme,
                                    ui_font,
                                );
                            });
                    });
            });
        });
}

fn spawn_pause_menu(commands: &mut Commands, theme: &UiTheme, ui_font: &UiFont) {
    commands
        .spawn((
            UiScreenRoot::new(UiScreen::PauseMenu),
            Name::new("PauseMenu"),
            overlay_node(),
            BackgroundColor(theme.modal_scrim),
            GlobalZIndex(2500),
            Visibility::Hidden,
        ))
        .with_children(|root| {
            root.spawn((
                UiFrameKind::Generic,
                Node {
                    width: Val::Px(420.0),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(24.0)),
                    row_gap: Val::Px(12.0),
                    ..default()
                },
            ))
            .with_children(|panel| {
                panel.spawn((
                    Text::new("游戏已暂停"),
                    title_font(ui_font, 28.0),
                    TextColor(theme.text_primary),
                ));
                spawn_text_button(
                    panel,
                    ResumeButton,
                    "继续游戏",
                    UiControlKind::Button,
                    theme,
                    ui_font,
                );
                spawn_text_button(
                    panel,
                    PauseSettingsButton,
                    "设置",
                    UiControlKind::Button,
                    theme,
                    ui_font,
                );
                spawn_text_button(
                    panel,
                    SaveQuitButton,
                    "保存并退出",
                    UiControlKind::Button,
                    theme,
                    ui_font,
                );
            });
        });
}

fn spawn_settings(commands: &mut Commands, theme: &UiTheme, ui_font: &UiFont) {
    commands
        .spawn((
            SettingsRoot,
            Name::new("SettingsMenu"),
            overlay_node(),
            BackgroundColor(theme.modal_scrim),
            GlobalZIndex(4000),
            Visibility::Hidden,
        ))
        .with_children(|root| {
            root.spawn((
                UiFrameKind::Generic,
                Node {
                    width: Val::Px(650.0),
                    max_width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(24.0)),
                    row_gap: Val::Px(10.0),
                    ..default()
                },
                BackgroundColor(theme.bg_panel),
                BorderColor::all(theme.border_default),
            ))
            .with_children(|panel| {
                panel.spawn((
                    Text::new("设置"),
                    title_font(ui_font, 28.0),
                    TextColor(theme.text_primary),
                ));
                spawn_setting_row(
                    panel,
                    "画面 / 渲染距离",
                    SettingValue::RenderDistance,
                    SettingAction::RenderDistance(-1),
                    SettingAction::RenderDistance(1),
                    theme,
                    ui_font,
                );
                spawn_setting_row(
                    panel,
                    "音频 / 主音量",
                    SettingValue::MasterVolume,
                    SettingAction::MasterVolume(-0.1),
                    SettingAction::MasterVolume(0.1),
                    theme,
                    ui_font,
                );
                spawn_setting_row(
                    panel,
                    "控制 / 鼠标灵敏度",
                    SettingValue::MouseSensitivity,
                    SettingAction::MouseSensitivity(-0.1),
                    SettingAction::MouseSensitivity(0.1),
                    theme,
                    ui_font,
                );
                spawn_setting_row(
                    panel,
                    "界面 / UI 缩放",
                    SettingValue::UiScale,
                    SettingAction::UiScale(-0.1),
                    SettingAction::UiScale(0.1),
                    theme,
                    ui_font,
                );
                spawn_toggle_row(
                    panel,
                    "画面 / 全屏",
                    SettingValue::Fullscreen,
                    SettingAction::ToggleFullscreen,
                    theme,
                    ui_font,
                );
                spawn_toggle_row(
                    panel,
                    "画面 / 垂直同步",
                    SettingValue::Vsync,
                    SettingAction::ToggleVsync,
                    theme,
                    ui_font,
                );
                spawn_text_button(
                    panel,
                    SettingsBackButton,
                    "返回",
                    UiControlKind::Button,
                    theme,
                    ui_font,
                );
            });
        });
}

fn spawn_setting_row(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    value: SettingValue,
    decrease: SettingAction,
    increase: SettingAction,
    theme: &UiTheme,
    ui_font: &UiFont,
) {
    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Px(46.0),
            align_items: AlignItems::Center,
            column_gap: Val::Px(8.0),
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                Text::new(label.to_string()),
                body_font(ui_font, 14.0),
                TextColor(theme.text_secondary),
                Node {
                    flex_grow: 1.0,
                    ..default()
                },
            ));
            spawn_text_button(
                row,
                SettingButton(decrease),
                "-",
                UiControlKind::IconButton,
                theme,
                ui_font,
            );
            row.spawn((
                value,
                Text::new(""),
                body_font(ui_font, 14.0),
                TextColor(theme.text_primary),
                Node {
                    width: Val::Px(92.0),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
            ));
            spawn_text_button(
                row,
                SettingButton(increase),
                "+",
                UiControlKind::IconButton,
                theme,
                ui_font,
            );
        });
}

fn spawn_toggle_row(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    value: SettingValue,
    action: SettingAction,
    theme: &UiTheme,
    ui_font: &UiFont,
) {
    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Px(44.0),
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                Text::new(label.to_string()),
                body_font(ui_font, 14.0),
                TextColor(theme.text_secondary),
                Node {
                    flex_grow: 1.0,
                    ..default()
                },
            ));
            row.spawn((
                value,
                Text::new(""),
                body_font(ui_font, 14.0),
                TextColor(theme.text_primary),
                Node {
                    width: Val::Px(72.0),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
            ));
            spawn_text_button(
                row,
                SettingButton(action),
                "切换",
                UiControlKind::Button,
                theme,
                ui_font,
            );
        });
}

fn spawn_dialog(commands: &mut Commands, theme: &UiTheme, ui_font: &UiFont) {
    commands
        .spawn((
            DialogRoot,
            Name::new("Dialog"),
            overlay_node(),
            BackgroundColor(theme.modal_scrim),
            GlobalZIndex(10_000),
            Visibility::Hidden,
        ))
        .with_children(|root| {
            root.spawn((
                UiFrameKind::Modal,
                Node {
                    width: Val::Px(520.0),
                    max_width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(24.0)),
                    row_gap: Val::Px(14.0),
                    ..default()
                },
            ))
            .with_children(|panel| {
                panel.spawn((
                    DialogTitle,
                    Text::new("提示"),
                    title_font(ui_font, 24.0),
                    TextColor(theme.text_primary),
                ));
                panel.spawn((
                    DialogMessage,
                    Text::new(""),
                    body_font(ui_font, 15.0),
                    TextColor(theme.text_secondary),
                ));
                panel
                    .spawn(Node {
                        justify_content: JustifyContent::End,
                        column_gap: Val::Px(10.0),
                        ..default()
                    })
                    .with_children(|actions| {
                        spawn_text_button(
                            actions,
                            DialogCancelButton,
                            "取消",
                            UiControlKind::Button,
                            theme,
                            ui_font,
                        );
                        spawn_text_button(
                            actions,
                            DialogConfirmButton,
                            "确认",
                            UiControlKind::Button,
                            theme,
                            ui_font,
                        );
                    });
            });
        });
}

pub(crate) fn sync_flow_screen_stack_system(
    state: Res<State<AppState>>,
    mut previous: Local<Option<AppState>>,
    mut stack: ResMut<UiScreenStack>,
    mut context: ResMut<InputContextState>,
    mut menu_page: ResMut<MenuPage>,
    mut focus: ResMut<InputFocus>,
) {
    if previous.as_ref() == Some(state.get()) {
        return;
    }
    *previous = Some(state.get().clone());
    focus.clear();
    *menu_page = MenuPage::Worlds;
    match state.get() {
        AppState::Boot | AppState::Loading | AppState::WorldLoading => {
            stack.clear();
            stack.open(UiScreen::Loading);
            context.set_menu_open(true);
        }
        AppState::MainMenu => {
            stack.clear();
            stack.open(UiScreen::MainMenu);
            context.set_menu_open(true);
        }
        AppState::InGame => {
            stack.clear();
            context.set_menu_open(false);
        }
        AppState::Paused => {
            stack.close(UiScreen::Settings);
            stack.open(UiScreen::PauseMenu);
            context.set_menu_open(true);
        }
    }
}

pub(crate) fn sync_menu_visibility_system(
    state: Res<State<AppState>>,
    page: Res<MenuPage>,
    dialog: Res<DialogState>,
    mut settings_query: Query<&mut Visibility, (With<SettingsRoot>, Without<DialogRoot>)>,
    mut dialog_query: Query<&mut Visibility, (With<DialogRoot>, Without<SettingsRoot>)>,
    mut screen_query: Query<
        (&UiScreenRoot, &mut Visibility),
        (Without<SettingsRoot>, Without<DialogRoot>),
    >,
) {
    let settings_open =
        *page == MenuPage::Settings && matches!(state.get(), AppState::MainMenu | AppState::Paused);
    if let Ok(mut visibility) = settings_query.single_mut() {
        *visibility = if settings_open {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    for (root, mut visibility) in &mut screen_query {
        match root.screen {
            UiScreen::MainMenu if *state.get() == AppState::MainMenu => {
                *visibility = if settings_open {
                    Visibility::Hidden
                } else {
                    Visibility::Visible
                };
            }
            UiScreen::PauseMenu if *state.get() == AppState::Paused => {
                *visibility = if settings_open {
                    Visibility::Hidden
                } else {
                    Visibility::Visible
                };
            }
            _ => {}
        }
    }
    if let Ok(mut visibility) = dialog_query.single_mut() {
        *visibility = if dialog.kind.is_some() {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

pub(crate) fn sync_loading_text_system(
    status: Res<LoadingStatus>,
    mut title_query: Query<&mut Text, (With<LoadingTitle>, Without<LoadingDetail>)>,
    mut detail_query: Query<&mut Text, (With<LoadingDetail>, Without<LoadingTitle>)>,
) {
    if !status.is_changed() {
        return;
    }
    if let Ok(mut text) = title_query.single_mut() {
        *text = Text::new(status.title.clone());
    }
    if let Ok(mut text) = detail_query.single_mut() {
        *text = Text::new(status.detail.clone());
    }
}

pub(crate) fn sync_dialog_text_system(
    dialog: Res<DialogState>,
    mut title_query: Query<&mut Text, (With<DialogTitle>, Without<DialogMessage>)>,
    mut message_query: Query<&mut Text, (With<DialogMessage>, Without<DialogTitle>)>,
    mut cancel_query: Query<&mut Visibility, With<DialogCancelButton>>,
) {
    if !dialog.is_changed() {
        return;
    }
    if let Ok(mut text) = title_query.single_mut() {
        *text = Text::new(dialog.title.clone());
    }
    if let Ok(mut text) = message_query.single_mut() {
        *text = Text::new(dialog.message.clone());
    }
    if let Ok(mut visibility) = cancel_query.single_mut() {
        *visibility = if dialog
            .kind
            .as_ref()
            .is_some_and(DialogKind::requires_confirmation)
        {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

pub(crate) fn populate_world_list_system(
    catalog: Res<WorldCatalog>,
    list_query: Query<Entity, With<WorldList>>,
    children_query: Query<&Children>,
    mut commands: Commands,
    theme: Res<UiTheme>,
    ui_font: Res<UiFont>,
) {
    if !catalog.is_changed() {
        return;
    }
    let Ok(list) = list_query.single() else {
        return;
    };
    if let Ok(children) = children_query.get(list) {
        for child in children.iter() {
            commands.entity(child).despawn();
        }
    }
    commands.entity(list).with_children(|parent| {
        if catalog.worlds.is_empty() {
            parent.spawn((
                Text::new("还没有世界"),
                body_font(&ui_font, 15.0),
                TextColor(theme.text_hint),
            ));
            return;
        }
        for world in &catalog.worlds {
            let label = format!("{}    种子 {}", world.id, world.seed);
            let entity = spawn_text_button(
                parent,
                WorldEntryButton {
                    id: world.id.clone(),
                },
                &label,
                UiControlKind::Tab,
                &theme,
                &ui_font,
            );
            parent.commands().entity(entity).insert(UiControl {
                kind: UiControlKind::Tab,
                selected: catalog.selected.as_deref() == Some(world.id.as_str()),
                disabled: false,
            });
        }
    });
}

pub(crate) fn menu_button_system(
    static_query: Query<
        (
            &Interaction,
            Option<&PlayButton>,
            Option<&CreateButton>,
            Option<&DeleteButton>,
            Option<&MainSettingsButton>,
            Option<&QuitButton>,
            Option<&ResumeButton>,
            Option<&PauseSettingsButton>,
            Option<&SaveQuitButton>,
            Option<&SettingsBackButton>,
        ),
        Changed<Interaction>,
    >,
    world_query: Query<(&Interaction, &WorldEntryButton), Changed<Interaction>>,
    setting_query: Query<(&Interaction, &SettingButton), Changed<Interaction>>,
    dialog_query: Query<
        (
            &Interaction,
            Option<&DialogConfirmButton>,
            Option<&DialogCancelButton>,
        ),
        Changed<Interaction>,
    >,
    draft: Res<WorldNameDraft>,
    mut writer: MessageWriter<FlowCommand>,
) {
    for (
        interaction,
        play,
        create,
        delete,
        main_settings,
        quit,
        resume,
        pause_settings,
        save_quit,
        back,
    ) in &static_query
    {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let command = if play.is_some() {
            FlowCommand::PlaySelected
        } else if create.is_some() {
            FlowCommand::CreateWorld(draft.0.clone())
        } else if delete.is_some() {
            FlowCommand::RequestDeleteSelected
        } else if main_settings.is_some() || pause_settings.is_some() {
            FlowCommand::OpenSettings
        } else if quit.is_some() {
            FlowCommand::QuitApplication
        } else if resume.is_some() {
            FlowCommand::Resume
        } else if save_quit.is_some() {
            FlowCommand::SaveAndQuit
        } else if back.is_some() {
            FlowCommand::CloseSettings
        } else {
            continue;
        };
        writer.write(command);
    }
    for (interaction, entry) in &world_query {
        if *interaction == Interaction::Pressed {
            writer.write(FlowCommand::SelectWorld(entry.id.clone()));
        }
    }
    for (interaction, button) in &setting_query {
        if *interaction == Interaction::Pressed {
            writer.write(FlowCommand::AdjustSetting(button.0));
        }
    }
    for (interaction, confirm, cancel) in &dialog_query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        if confirm.is_some() {
            writer.write(FlowCommand::ConfirmDialog);
        } else if cancel.is_some() {
            writer.write(FlowCommand::CancelDialog);
        }
    }
}

pub(crate) fn sync_world_name_draft_system(
    query: Query<&EditableText, (With<WorldNameInput>, Changed<EditableText>)>,
    mut draft: ResMut<WorldNameDraft>,
) {
    let Ok(editable) = query.single() else {
        return;
    };
    draft.0 = editable.value().to_string();
}

pub(crate) fn sync_setting_values_system(
    settings: Res<GameSettings>,
    mut query: Query<(&SettingValue, &mut Text)>,
) {
    if !settings.is_changed() {
        return;
    }
    for (value, mut text) in &mut query {
        *text = Text::new(match value {
            SettingValue::RenderDistance => settings.render_distance.to_string(),
            SettingValue::MasterVolume => format!("{:.0}%", settings.master_volume * 100.0),
            SettingValue::MouseSensitivity => format!("{:.1}x", settings.mouse_sensitivity),
            SettingValue::UiScale => format!("{:.1}x", settings.ui_scale),
            SettingValue::Fullscreen => if settings.fullscreen {
                "开启"
            } else {
                "关闭"
            }
            .to_string(),
            SettingValue::Vsync => if settings.vsync { "开启" } else { "关闭" }.to_string(),
        });
    }
}

fn title_font(ui_font: &UiFont, size: f32) -> TextFont {
    TextFont {
        font: FontSource::from(ui_font.default.clone()),
        font_size: FontSize::Px(size),
        ..default()
    }
}

fn body_font(ui_font: &UiFont, size: f32) -> TextFont {
    TextFont {
        font: FontSource::from(ui_font.default.clone()),
        font_size: FontSize::Px(size),
        ..default()
    }
}
