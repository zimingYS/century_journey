//! 键位绑定模型的镜像测试：注册表完整性、修饰键规范化、冲突与过滤。

use super::*;
use bevy::input::mouse::MouseButton;

use crate::app::settings::{
    BindingKey, KEY_ACTIONS, KeyAction, Keybinds, keybinds_path, parse_binding_key,
};

/// 构造只包含指定按键按下状态的键盘输入。
fn keyboard_with(pressed: &[KeyCode]) -> ButtonInput<KeyCode> {
    let mut input = ButtonInput::<KeyCode>::default();
    for code in pressed {
        input.press(*code);
    }
    input
}

/// 构造只包含指定鼠标按键按下状态的鼠标输入。
fn mouse_with(pressed: &[MouseButton]) -> ButtonInput<MouseButton> {
    let mut input = ButtonInput::<MouseButton>::default();
    for button in pressed {
        input.press(*button);
    }
    input
}

#[test]
fn 注册表覆盖全部动作且默认键唯一() {
    assert_eq!(KEY_ACTIONS.len(), KeyAction::COUNT);
    let mut defaults = Vec::new();
    for spec in KEY_ACTIONS {
        assert!(
            spec.default.is_some(),
            "动作 {:?} 缺少默认绑定",
            spec.action
        );
        assert!(!spec.id.is_empty() && !spec.display.is_empty());
        defaults.push(spec.default);
    }
    // 默认布局不应自带冲突。
    let keybinds = Keybinds::default();
    for spec in KEY_ACTIONS {
        assert!(
            !keybinds.has_conflict(spec.action),
            "默认布局中 {:?} 不应冲突",
            spec.action
        );
    }
}

#[test]
fn 左右修饰键按住均视为触发() {
    let keybinds = Keybinds::default();
    for shift in [KeyCode::ShiftLeft, KeyCode::ShiftRight] {
        let keyboard = keyboard_with(&[shift]);
        let mouse = mouse_with(&[]);
        assert!(
            keybinds.is_held(KeyAction::Sprint, &keyboard, &mouse),
            "{shift:?} 按住应触发疾跑"
        );
    }
    let keyboard = keyboard_with(&[]);
    let mouse = mouse_with(&[]);
    assert!(!keybinds.is_held(KeyAction::Sprint, &keyboard, &mouse));
}

#[test]
fn 鼠标绑定按住与单击均生效() {
    let keybinds = Keybinds::default();
    let keyboard = keyboard_with(&[]);
    let mut mouse = mouse_with(&[]);
    mouse.press(MouseButton::Left);
    assert!(keybinds.is_held(KeyAction::BreakOrAttack, &keyboard, &mouse));

    let mouse = mouse_with(&[MouseButton::Left]);
    assert!(keybinds.is_just_pressed(KeyAction::BreakOrAttack, &keyboard, &mouse));
}

#[test]
fn 未绑定动作不触发任何输入() {
    let mut keybinds = Keybinds::default();
    keybinds.set_binding(KeyAction::Jump, None);
    let keyboard = keyboard_with(&[KeyCode::Space]);
    let mouse = mouse_with(&[]);
    assert!(!keybinds.is_held(KeyAction::Jump, &keyboard, &mouse));
    assert!(!keybinds.is_just_pressed(KeyAction::Jump, &keyboard, &mouse));
}

#[test]
fn 绑定右侧修饰键被规范化为左侧() {
    let mut keybinds = Keybinds::default();
    keybinds.set_binding(
        KeyAction::Squat,
        Some(BindingKey::Key(KeyCode::ControlRight)),
    );
    assert_eq!(
        keybinds.binding(KeyAction::Squat),
        Some(BindingKey::Key(KeyCode::ControlLeft))
    );
}

#[test]
fn 同键绑定产生双向冲突提示() {
    let mut keybinds = Keybinds::default();
    keybinds.set_binding(KeyAction::DropItem, Some(BindingKey::Key(KeyCode::KeyW)));
    let partners = keybinds.conflict_partners(KeyAction::DropItem);
    assert_eq!(partners, vec!["前进"]);
    assert!(keybinds.has_conflict(KeyAction::MoveForward));
}

#[test]
fn 搜索匹配动作名或键名() {
    let keybinds = Keybinds::default();
    assert!(keybinds.matches_filter(KeyAction::MoveForward, "前进", false, false));
    assert!(keybinds.matches_filter(KeyAction::MoveForward, "keyw", false, false));
    assert!(!keybinds.matches_filter(KeyAction::MoveForward, "跳跃", false, false));
}

#[test]
fn 过滤开关筛出冲突与未绑定条目() {
    let mut keybinds = Keybinds::default();
    keybinds.set_binding(KeyAction::Hotbar1, Some(BindingKey::Key(KeyCode::KeyW)));
    keybinds.set_binding(KeyAction::Hotbar2, None);

    assert!(keybinds.matches_filter(KeyAction::Hotbar1, "", true, false));
    assert!(!keybinds.matches_filter(KeyAction::Jump, "", true, false));
    assert!(keybinds.matches_filter(KeyAction::Hotbar2, "", false, true));
    assert!(!keybinds.matches_filter(KeyAction::Jump, "", false, true));
    // 未绑定条目没有键名，搜索词一律不匹配。
    assert!(!keybinds.matches_filter(KeyAction::Hotbar2, "none", false, true));
}

#[test]
fn 重置恢复全部默认() {
    let mut keybinds = Keybinds::default();
    keybinds.set_binding(KeyAction::MoveForward, Some(BindingKey::Key(KeyCode::KeyP)));
    keybinds.set_binding(KeyAction::Jump, None);
    keybinds.reset_all();
    assert_eq!(keybinds, Keybinds::default());
}

#[test]
fn 按键名解析覆盖字母数字功能键与鼠标() {
    assert_eq!(
        parse_binding_key("KeyW"),
        Some(BindingKey::Key(KeyCode::KeyW))
    );
    assert_eq!(parse_binding_key("F3"), Some(BindingKey::Key(KeyCode::F3)));
    assert_eq!(
        parse_binding_key("Digit1"),
        Some(BindingKey::Key(KeyCode::Digit1))
    );
    assert_eq!(
        parse_binding_key("MouseLeft"),
        Some(BindingKey::Mouse(MouseButton::Left))
    );
    assert_eq!(parse_binding_key("NotAKey"), None);
    // 右侧修饰键解析后即为左侧代表键。
    assert_eq!(
        parse_binding_key("ShiftRight"),
        Some(BindingKey::Key(KeyCode::ShiftLeft))
    );
}

#[test]
fn 配置路径位于config目录() {
    assert!(keybinds_path().ends_with("keybinds.toml"));
}
