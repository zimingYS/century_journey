//! 键位 TOML 格式的镜像测试：往返一致、容错解析与手编约定。

use super::*;
use bevy::input::mouse::MouseButton;

use crate::app::settings::{KeyAction, Keybinds, parse_keybinds_toml, save_keybinds_to};

#[test]
fn 解析空配置得到全部默认() {
    let keybinds = parse_keybinds_toml("").expect("空配置应可解析");
    assert_eq!(keybinds, Keybinds::default());
}

#[test]
fn 覆盖条目优先于默认并支持解除绑定() {
    let text = r#"
move_forward = "KeyP"
jump = "none"
"#;
    let keybinds = parse_keybinds_toml(text).expect("配置应可解析");
    assert_eq!(
        keybinds.binding(KeyAction::MoveForward),
        Some(BindingKey::Key(KeyCode::KeyP))
    );
    assert_eq!(keybinds.binding(KeyAction::Jump), None);
    // 未提及的条目保持默认。
    assert_eq!(
        keybinds.binding(KeyAction::Sprint),
        Some(BindingKey::Key(KeyCode::ShiftLeft))
    );
}

#[test]
fn 未知条目与非法键名回退默认() {
    let text = r#"
unknown_action = "KeyP"
move_forward = "NotAKey"
hotbar_1 = 42
"#;
    let keybinds = parse_keybinds_toml(text).expect("容错解析应成功");
    assert_eq!(
        keybinds.binding(KeyAction::MoveForward),
        Some(BindingKey::Key(KeyCode::KeyW))
    );
    assert_eq!(
        keybinds.binding(KeyAction::Hotbar1),
        Some(BindingKey::Key(KeyCode::Digit1))
    );
}

#[test]
fn 语法错误的配置整体失败() {
    assert!(parse_keybinds_toml("move_forward = ").is_err());
    assert!(parse_keybinds_toml("[broken\n").is_err());
}

#[test]
fn 保存后重新解析得到相同绑定() {
    let mut keybinds = Keybinds::default();
    keybinds.set_binding(
        KeyAction::MoveBackward,
        Some(BindingKey::Key(KeyCode::KeyZ)),
    );
    keybinds.set_binding(
        KeyAction::BreakOrAttack,
        Some(BindingKey::Mouse(MouseButton::Right)),
    );
    keybinds.set_binding(KeyAction::Hotbar9, None);

    let path = std::env::temp_dir().join("cj_keybinds_roundtrip_test.toml");
    save_keybinds_to(&path, &keybinds).expect("保存应成功");
    let text = std::fs::read_to_string(&path).expect("文件应可读");
    let reloaded = parse_keybinds_toml(&text).expect("保存的配置应可解析");

    assert_eq!(
        reloaded.binding(KeyAction::MoveBackward),
        keybinds.binding(KeyAction::MoveBackward)
    );
    assert_eq!(
        reloaded.binding(KeyAction::BreakOrAttack),
        keybinds.binding(KeyAction::BreakOrAttack)
    );
    assert_eq!(reloaded.binding(KeyAction::Hotbar9), None);
    assert_eq!(
        reloaded.binding(KeyAction::Jump),
        keybinds.binding(KeyAction::Jump)
    );
    assert_eq!(reloaded, keybinds);
    let _ = std::fs::remove_file(path);
}

#[test]
fn 保存文本含手编说明与全部条目() {
    let path = std::env::temp_dir().join("cj_keybinds_header_test.toml");
    save_keybinds_to(&path, &Keybinds::default()).expect("保存应成功");
    let text = std::fs::read_to_string(&path).expect("文件应可读");
    let _ = std::fs::remove_file(path);

    assert!(text.contains("物理按键名"));
    assert!(text.contains("move_forward = \"KeyW\""));
    assert!(text.contains("hotbar_1 = \"Digit1\""));
}
