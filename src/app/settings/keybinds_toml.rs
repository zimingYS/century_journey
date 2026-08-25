//! 键位绑定的 TOML 磁盘格式：按键命名表、加载与原子保存。
//!
//! 配置面向手编优化：未知条目忽略并告警，非法键名回退默认，
//! 删除某行等价于恢复该动作的默认键位。

use std::path::{Path, PathBuf};

use bevy::input::mouse::MouseButton;
use bevy::prelude::*;

use super::keybinds::{BindingKey, KEY_ACTIONS, Keybinds};
use crate::engine::localization::Localization;
use crate::engine::persistence;

/// 返回键位配置文件路径。
pub fn keybinds_path() -> PathBuf {
    PathBuf::from("config").join("keybinds.toml")
}

/// 从默认路径加载键位；文件不存在时返回默认表供调用方决定是否落盘。
pub fn load_keybinds() -> Result<Keybinds, String> {
    load_keybinds_from(&keybinds_path())
}

/// 从指定路径解析键位配置。
///
/// TOML 语法错误整体失败；单条目语义问题（未知动作、非法键名）忽略并告警，
/// 对应动作保持默认，避免手编小错毁掉整个配置。
pub fn load_keybinds_from(path: &Path) -> Result<Keybinds, String> {
    let text =
        std::fs::read_to_string(path).map_err(|error| format!("读取键位配置失败: {error}"))?;
    parse_keybinds_toml(&text)
}

/// 解析键位配置文本；独立函数便于测试与写盘校验复用。
pub fn parse_keybinds_toml(text: &str) -> Result<Keybinds, String> {
    let table: toml::Table =
        toml::from_str(text).map_err(|error| format!("键位配置格式无效: {error}"))?;
    let mut keybinds = Keybinds::default();
    for (key, value) in &table {
        let Some(spec) = KEY_ACTIONS.iter().find(|spec| spec.id == *key) else {
            log::warn!("[键位] 忽略未知配置条目: {key}");
            continue;
        };
        let Some(name) = value.as_str() else {
            log::warn!("[键位] 条目 {key} 的值不是字符串，已回退默认键位");
            continue;
        };
        if name.eq_ignore_ascii_case("none") {
            keybinds.set_binding(spec.action, None);
            continue;
        }
        match parse_binding_key(name) {
            Some(binding) => keybinds.set_binding(spec.action, Some(binding)),
            None => log::warn!("[键位] 条目 {key} 的按键名「{name}」无法识别，已回退默认键位"),
        }
    }
    Ok(keybinds)
}

/// 把键位表原子写入默认路径，并保留可恢复备份。
pub fn save_keybinds(keybinds: &Keybinds) -> Result<(), String> {
    save_keybinds_to(&keybinds_path(), keybinds)
}

/// 把键位表原子写入指定路径；写前生成注释头，写后按解析校验回读。
pub fn save_keybinds_to(path: &Path, keybinds: &Keybinds) -> Result<(), String> {
    let mut text = String::from(
        "# Century Journey 键位绑定\n\
         # 值为物理按键名（按按键位置，不随输入语言变化），如 KeyW、Space、F3、MouseLeft。\n\
         # 设为 \"none\" 表示解除绑定；删除某行表示恢复该动作的默认键位。\n\n",
    );
    let mut table = toml::Table::new();
    for spec in KEY_ACTIONS {
        let value = match keybinds.binding(spec.action) {
            Some(binding) => binding_key_name(binding).to_string(),
            None => "none".to_string(),
        };
        table.insert(spec.id.to_string(), toml::Value::String(value));
    }
    let body = toml::to_string(&table).map_err(|error| format!("生成键位配置失败: {error}"))?;
    text.push_str(&body);
    let validate = |bytes: &[u8]| -> Result<(), String> {
        let text =
            std::str::from_utf8(bytes).map_err(|error| format!("键位配置编码无效: {error}"))?;
        parse_keybinds_toml(text).map(|_| ())
    };
    persistence::atomic_write_verified(path, text.as_bytes(), validate)
        .map_err(|error| error.to_string())
}

/// 返回绑定的标准按键名，与配置文件和界面显示共用。
pub fn binding_key_name(binding: BindingKey) -> String {
    match binding {
        BindingKey::Key(code) => format!("{code:?}"),
        BindingKey::Mouse(MouseButton::Left) => "MouseLeft".into(),
        BindingKey::Mouse(MouseButton::Right) => "MouseRight".into(),
        BindingKey::Mouse(MouseButton::Middle) => "MouseMiddle".into(),
        BindingKey::Mouse(MouseButton::Back) => "MouseBack".into(),
        BindingKey::Mouse(MouseButton::Forward) => "MouseForward".into(),
        BindingKey::Mouse(other) => format!("Mouse{other:?}"),
    }
}

/// 解析标准按键名；只接受玩家可能绑定的常见按键，未知名称返回 None。
pub fn parse_binding_key(name: &str) -> Option<BindingKey> {
    if let Some(button) = match name {
        "MouseLeft" => Some(MouseButton::Left),
        "MouseRight" => Some(MouseButton::Right),
        "MouseMiddle" => Some(MouseButton::Middle),
        "MouseBack" => Some(MouseButton::Back),
        "MouseForward" => Some(MouseButton::Forward),
        _ => None,
    } {
        return Some(BindingKey::Mouse(button));
    }
    parse_key_code(name).map(BindingKey::Key)
}

/// 解析物理键盘按键名。
fn parse_key_code(name: &str) -> Option<KeyCode> {
    Some(match name {
        "KeyA" => KeyCode::KeyA,
        "KeyB" => KeyCode::KeyB,
        "KeyC" => KeyCode::KeyC,
        "KeyD" => KeyCode::KeyD,
        "KeyE" => KeyCode::KeyE,
        "KeyF" => KeyCode::KeyF,
        "KeyG" => KeyCode::KeyG,
        "KeyH" => KeyCode::KeyH,
        "KeyI" => KeyCode::KeyI,
        "KeyJ" => KeyCode::KeyJ,
        "KeyK" => KeyCode::KeyK,
        "KeyL" => KeyCode::KeyL,
        "KeyM" => KeyCode::KeyM,
        "KeyN" => KeyCode::KeyN,
        "KeyO" => KeyCode::KeyO,
        "KeyP" => KeyCode::KeyP,
        "KeyQ" => KeyCode::KeyQ,
        "KeyR" => KeyCode::KeyR,
        "KeyS" => KeyCode::KeyS,
        "KeyT" => KeyCode::KeyT,
        "KeyU" => KeyCode::KeyU,
        "KeyV" => KeyCode::KeyV,
        "KeyW" => KeyCode::KeyW,
        "KeyX" => KeyCode::KeyX,
        "KeyY" => KeyCode::KeyY,
        "KeyZ" => KeyCode::KeyZ,
        "Digit0" => KeyCode::Digit0,
        "Digit1" => KeyCode::Digit1,
        "Digit2" => KeyCode::Digit2,
        "Digit3" => KeyCode::Digit3,
        "Digit4" => KeyCode::Digit4,
        "Digit5" => KeyCode::Digit5,
        "Digit6" => KeyCode::Digit6,
        "Digit7" => KeyCode::Digit7,
        "Digit8" => KeyCode::Digit8,
        "Digit9" => KeyCode::Digit9,
        "F1" => KeyCode::F1,
        "F2" => KeyCode::F2,
        "F3" => KeyCode::F3,
        "F4" => KeyCode::F4,
        "F5" => KeyCode::F5,
        "F6" => KeyCode::F6,
        "F7" => KeyCode::F7,
        "F8" => KeyCode::F8,
        "F9" => KeyCode::F9,
        "F10" => KeyCode::F10,
        "F11" => KeyCode::F11,
        "F12" => KeyCode::F12,
        "Escape" => KeyCode::Escape,
        "Tab" => KeyCode::Tab,
        "CapsLock" => KeyCode::CapsLock,
        "Space" => KeyCode::Space,
        "Enter" => KeyCode::Enter,
        "Backspace" => KeyCode::Backspace,
        "Delete" => KeyCode::Delete,
        "Insert" => KeyCode::Insert,
        "Home" => KeyCode::Home,
        "End" => KeyCode::End,
        "PageUp" => KeyCode::PageUp,
        "PageDown" => KeyCode::PageDown,
        "PrintScreen" => KeyCode::PrintScreen,
        "ScrollLock" => KeyCode::ScrollLock,
        "Pause" => KeyCode::Pause,
        "Backquote" => KeyCode::Backquote,
        "Minus" => KeyCode::Minus,
        "Equal" => KeyCode::Equal,
        "Comma" => KeyCode::Comma,
        "Period" => KeyCode::Period,
        "Semicolon" => KeyCode::Semicolon,
        "Quote" => KeyCode::Quote,
        "Slash" => KeyCode::Slash,
        "Backslash" => KeyCode::Backslash,
        "BracketLeft" => KeyCode::BracketLeft,
        "BracketRight" => KeyCode::BracketRight,
        "ShiftLeft" | "ShiftRight" => KeyCode::ShiftLeft,
        "ControlLeft" | "ControlRight" => KeyCode::ControlLeft,
        "AltLeft" | "AltRight" => KeyCode::AltLeft,
        "ArrowUp" => KeyCode::ArrowUp,
        "ArrowDown" => KeyCode::ArrowDown,
        "ArrowLeft" => KeyCode::ArrowLeft,
        "ArrowRight" => KeyCode::ArrowRight,
        "Numpad0" => KeyCode::Numpad0,
        "Numpad1" => KeyCode::Numpad1,
        "Numpad2" => KeyCode::Numpad2,
        "Numpad3" => KeyCode::Numpad3,
        "Numpad4" => KeyCode::Numpad4,
        "Numpad5" => KeyCode::Numpad5,
        "Numpad6" => KeyCode::Numpad6,
        "Numpad7" => KeyCode::Numpad7,
        "Numpad8" => KeyCode::Numpad8,
        "Numpad9" => KeyCode::Numpad9,
        "NumpadEnter" => KeyCode::NumpadEnter,
        "NumpadAdd" => KeyCode::NumpadAdd,
        "NumpadSubtract" => KeyCode::NumpadSubtract,
        "NumpadMultiply" => KeyCode::NumpadMultiply,
        "NumpadDivide" => KeyCode::NumpadDivide,
        "NumpadDecimal" => KeyCode::NumpadDecimal,
        "NumpadEqual" => KeyCode::NumpadEqual,
        _ => return None,
    })
}

/// 生成绑定的本地化显示文本；未绑定显示「未绑定」。
///
/// 鼠标键与符号键走 `keybind.key.*` 翻译；字母、数字等功能键名
/// （`KeyW`、`7`、`Shift` 等）本身即可直接显示，回退链会原样返回。
pub fn binding_display_localized(
    binding: Option<BindingKey>,
    localization: &Localization,
) -> String {
    match binding {
        None => localization.get("keybind.key.unbound").to_string(),
        Some(BindingKey::Mouse(MouseButton::Other(code))) => {
            localization.format("keybind.key.mouse-other", &[("code", &code.to_string())])
        }
        Some(BindingKey::Mouse(MouseButton::Left)) => {
            localization.get("keybind.key.mouse-left").to_string()
        }
        Some(BindingKey::Mouse(MouseButton::Right)) => {
            localization.get("keybind.key.mouse-right").to_string()
        }
        Some(BindingKey::Mouse(MouseButton::Middle)) => {
            localization.get("keybind.key.mouse-middle").to_string()
        }
        Some(BindingKey::Mouse(MouseButton::Back)) => {
            localization.get("keybind.key.mouse-back").to_string()
        }
        Some(BindingKey::Mouse(MouseButton::Forward)) => {
            localization.get("keybind.key.mouse-forward").to_string()
        }
        Some(BindingKey::Key(KeyCode::Space)) => localization.get("keybind.key.space").to_string(),
        Some(BindingKey::Key(KeyCode::Enter)) | Some(BindingKey::Key(KeyCode::NumpadEnter)) => {
            localization.get("keybind.key.enter").to_string()
        }
        Some(BindingKey::Key(KeyCode::Backspace)) => {
            localization.get("keybind.key.backspace").to_string()
        }
        Some(BindingKey::Key(KeyCode::CapsLock)) => {
            localization.get("keybind.key.caps-lock").to_string()
        }
        Some(BindingKey::Key(KeyCode::Comma)) => localization.get("keybind.key.comma").to_string(),
        Some(BindingKey::Key(KeyCode::Period)) => {
            localization.get("keybind.key.period").to_string()
        }
        Some(BindingKey::Key(KeyCode::Semicolon)) => {
            localization.get("keybind.key.semicolon").to_string()
        }
        Some(BindingKey::Key(KeyCode::Quote)) => localization.get("keybind.key.quote").to_string(),
        Some(BindingKey::Key(KeyCode::Slash)) => localization.get("keybind.key.slash").to_string(),
        Some(BindingKey::Key(KeyCode::Backslash)) => {
            localization.get("keybind.key.backslash").to_string()
        }
        Some(BindingKey::Key(KeyCode::Minus)) => localization.get("keybind.key.minus").to_string(),
        Some(BindingKey::Key(KeyCode::Equal)) => localization.get("keybind.key.equal").to_string(),
        Some(BindingKey::Key(KeyCode::Backquote)) => {
            localization.get("keybind.key.backquote").to_string()
        }
        Some(BindingKey::Key(KeyCode::BracketLeft)) => {
            localization.get("keybind.key.bracket-left").to_string()
        }
        Some(BindingKey::Key(KeyCode::BracketRight)) => {
            localization.get("keybind.key.bracket-right").to_string()
        }
        Some(BindingKey::Key(KeyCode::ArrowUp)) => {
            localization.get("keybind.key.arrow-up").to_string()
        }
        Some(BindingKey::Key(KeyCode::ArrowDown)) => {
            localization.get("keybind.key.arrow-down").to_string()
        }
        Some(BindingKey::Key(KeyCode::ArrowLeft)) => {
            localization.get("keybind.key.arrow-left").to_string()
        }
        Some(BindingKey::Key(KeyCode::ArrowRight)) => {
            localization.get("keybind.key.arrow-right").to_string()
        }
        Some(BindingKey::Key(code)) => {
            let name = format!("{code:?}");
            name.strip_prefix("Digit")
                .map(|digit| digit.to_string())
                .unwrap_or(name)
        }
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/app/settings/keybinds_toml.rs"]
mod tests;
