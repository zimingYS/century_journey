//! 解析控制台指令行为类型安全的指令枚举。
//!
//! 本模块是纯函数，不依赖 Bevy 类型；指令注册表集中管理字符串分发，
//! 新增指令只需实现家族解析函数并在注册表追加一行。

use crate::game::world::time::{MINUTES_PER_GAME_DAY, MINUTES_PER_GAME_HOUR};

/// 已解析的指令及其参数。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameCommand {
    /// 设置当日时间：当日内的游戏分钟（0..MINUTES_PER_GAME_DAY）。
    TimeSet {
        /// 当日内的游戏分钟。
        minute_of_day: u64,
    },
    /// 设置时间流速倍率（0.0..=100.0，0 表示暂停）。
    TimeScale {
        /// 时间流速倍率。
        scale: f32,
    },
}

/// 指令解析失败原因，用于生成玩家可读反馈。
#[derive(Debug, Clone, PartialEq)]
pub enum CommandParseError {
    /// 指令行为空或只含空白。
    Empty,
    /// 未知指令；available 由注册表生成，供反馈列出可用指令。
    UnknownCommand {
        /// 未识别的指令根名。
        name: String,
        /// 注册表中全部可用指令名。
        available: Vec<&'static str>,
    },
    /// 缺少子指令；usage 为该指令的用法说明。
    MissingSubcommand {
        /// 用法说明文本。
        usage: &'static str,
    },
    /// 缺少必要参数；usage 为该指令的用法说明。
    MissingArgument {
        /// 用法说明文本。
        usage: &'static str,
    },
    /// 时间值无法解析为整数分钟或预设名。
    InvalidTimeValue {
        /// 原始输入值。
        value: String,
    },
    /// 倍率值无法解析为数字。
    InvalidScaleValue {
        /// 原始输入值。
        value: String,
    },
    /// 倍率超出 0.0..=100.0 范围（含 NaN 与无穷）。
    ScaleOutOfRange {
        /// 解析成功但越界的倍率。
        value: f32,
    },
}

/// 单条指令家族的解析描述，构成指令注册表。
struct CommandSpec {
    /// 指令根名（小写，不含 '/'）。
    name: &'static str,
    /// 用法说明；由注册表传给家族函数，是唯一的用法事实来源。
    usage: &'static str,
    /// 家族解析函数：输入为用法说明与剥离根名后的参数片段。
    parse: fn(&'static str, &[&str]) -> Result<GameCommand, CommandParseError>,
}

/// time 指令家族的用法说明。
const TIME_USAGE: &str = "/time set <分钟|day|noon|night|midnight> | /time scale <倍率>";

/// 全部已注册指令。新增指令 = 实现家族解析函数 + 在此追加一行。
const COMMANDS: &[CommandSpec] = &[CommandSpec {
    name: "time",
    usage: TIME_USAGE,
    parse: parse_time,
}];

/// 预设时间名对应的当日分钟数。
const TIME_PRESETS: &[(&str, u64)] = &[
    ("day", 8 * MINUTES_PER_GAME_HOUR),
    ("noon", 12 * MINUTES_PER_GAME_HOUR),
    ("night", 20 * MINUTES_PER_GAME_HOUR),
    ("midnight", 0),
];

/// 把一行指令文本解析为指令。输入为剥离 '/' 后的裸指令行。
pub fn parse_command(line: &str) -> Result<GameCommand, CommandParseError> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    let Some(root) = parts.first().copied() else {
        return Err(CommandParseError::Empty);
    };
    for spec in COMMANDS {
        if root.eq_ignore_ascii_case(spec.name) {
            return (spec.parse)(spec.usage, &parts[1..]);
        }
    }
    Err(CommandParseError::UnknownCommand {
        name: root.to_string(),
        available: COMMANDS.iter().map(|spec| spec.name).collect(),
    })
}

impl CommandParseError {
    /// 生成面向玩家的中文提示文本。
    pub fn to_feedback(&self) -> String {
        match self {
            Self::Empty => "输入指令为空".to_owned(),
            Self::UnknownCommand { name, available } => format!(
                "未知指令 /{name}，可用指令：{}",
                available
                    .iter()
                    .map(|name| format!("/{name}"))
                    .collect::<Vec<_>>()
                    .join(" ")
            ),
            Self::MissingSubcommand { usage } => format!("缺少子指令，用法：{usage}"),
            Self::MissingArgument { usage } => format!("缺少参数，用法：{usage}"),
            Self::InvalidTimeValue { value } => format!(
                "无法识别的时间值“{value}”，支持 0..{MINUTES_PER_GAME_DAY} 的分钟数或 day/noon/night/midnight"
            ),
            Self::InvalidScaleValue { value } => {
                format!("无法识别的倍率“{value}”，需要一个数字")
            }
            Self::ScaleOutOfRange { value } => {
                format!("倍率 {value} 超出范围，支持 0.0..=100.0")
            }
        }
    }
}

/// time 指令家族：子指令 set 设置当日时间，scale 设置流速倍率。
fn parse_time(usage: &'static str, args: &[&str]) -> Result<GameCommand, CommandParseError> {
    let Some(sub) = args.first().copied() else {
        return Err(CommandParseError::MissingSubcommand { usage });
    };
    let Some(token) = args.get(1).copied() else {
        return Err(CommandParseError::MissingArgument { usage });
    };
    if sub.eq_ignore_ascii_case("set") {
        Ok(GameCommand::TimeSet {
            minute_of_day: parse_minute_value(token)?,
        })
    } else if sub.eq_ignore_ascii_case("scale") {
        Ok(GameCommand::TimeScale {
            scale: parse_scale_value(token)?,
        })
    } else {
        Err(CommandParseError::MissingSubcommand { usage })
    }
}

/// 解析 0..MINUTES_PER_GAME_DAY 的整数分钟或预设名（忽略大小写）。
fn parse_minute_value(token: &str) -> Result<u64, CommandParseError> {
    if let Some((_, minute)) = TIME_PRESETS
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case(token))
    {
        return Ok(*minute);
    }
    let Ok(minute) = token.parse::<u64>() else {
        return Err(CommandParseError::InvalidTimeValue {
            value: token.to_string(),
        });
    };
    if minute >= MINUTES_PER_GAME_DAY {
        return Err(CommandParseError::InvalidTimeValue {
            value: token.to_string(),
        });
    }
    Ok(minute)
}

/// 解析 0.0..=100.0 的时间倍率；NaN 与无穷因无法落入范围同样被拒绝。
fn parse_scale_value(token: &str) -> Result<f32, CommandParseError> {
    let Ok(scale) = token.parse::<f32>() else {
        return Err(CommandParseError::InvalidScaleValue {
            value: token.to_string(),
        });
    };
    if !(0.0..=100.0).contains(&scale) {
        return Err(CommandParseError::ScaleOutOfRange { value: scale });
    }
    Ok(scale)
}

#[cfg(test)]
#[path = "../../../tests/unit/game/command/parse.rs"]
mod tests;
