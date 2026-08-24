//! 解析控制台指令行为类型安全的指令枚举。
//!
//! 本模块是纯函数，不依赖 Bevy 类型；指令注册表集中管理字符串分发，
//! 新增指令只需实现家族解析与候选函数并在注册表追加一行。

use crate::game::gameplay::gamemode::GameMode;
use crate::game::world::time::{MINUTES_PER_GAME_DAY, MINUTES_PER_GAME_HOUR};
use crate::shared::item_id::ItemId;

use super::suggest::SuggestContext;

/// /give 单次给予的数量上限，避免一条指令撑爆背包与反馈文本。
const MAX_GIVE_COUNT: u32 = 1000;

/// /tp 坐标分量的绝对值上限，超出视为明显误输入。
const MAX_COORDINATE: f32 = 1_000_000.0;

/// 已解析的指令及其参数。
#[derive(Debug, Clone, PartialEq)]
pub enum GameCommand {
    /// 显示全部指令或指定指令的用法。
    Help {
        /// 要查询用法的指令根名（已规范为注册表小写形式）。
        topic: Option<String>,
    },
    /// 显示当前世界种子。
    Seed,
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
    /// 设置游戏模式。
    GameModeSet {
        /// 目标游戏模式。
        mode: GameMode,
    },
    /// 传送本地玩家到绝对坐标。
    Teleport {
        /// 目标 X 坐标（方块单位）。
        x: f32,
        /// 目标 Y 坐标（方块单位）。
        y: f32,
        /// 目标 Z 坐标（方块单位）。
        z: f32,
    },
    /// 给予本地玩家物品。
    Give {
        /// 物品标识；短名会按默认命名空间解析。
        item: ItemId,
        /// 给予数量（1..=MAX_GIVE_COUNT）。
        count: u32,
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
    /// 物品标识无法解析为合法的命名空间标识符。
    InvalidItemValue {
        /// 原始输入值。
        value: String,
    },
    /// 数量无法解析为整数。
    InvalidCountValue {
        /// 原始输入值。
        value: String,
    },
    /// 数量为 0 或超出 1..=MAX_GIVE_COUNT 范围。
    CountOutOfRange {
        /// 解析成功但越界的数量。
        value: u32,
    },
    /// 坐标无法解析为数字或超出可表达范围。
    InvalidCoordinate {
        /// 原始输入值。
        value: String,
    },
    /// 游戏模式名无法识别。
    InvalidGameMode {
        /// 原始输入值。
        value: String,
    },
    /// /help 查询的指令名不存在；available 由注册表生成。
    InvalidHelpTopic {
        /// 未识别的指令名。
        name: String,
        /// 注册表中全部可用指令名。
        available: Vec<&'static str>,
    },
}

/// 单条指令家族的解析描述，构成指令注册表。
pub(crate) struct CommandSpec {
    /// 指令根名（小写，不含 '/'）。
    pub(crate) name: &'static str,
    /// 用法说明；由注册表传给家族函数，是唯一的用法事实来源。
    pub(crate) usage: &'static str,
    /// 家族解析函数：输入为用法说明与剥离根名后的参数片段。
    pub(crate) parse: fn(&'static str, &[&str]) -> Result<GameCommand, CommandParseError>,
    /// 家族候选函数：输入为已完成的参数词、正在补全的词前缀与候选上下文。
    pub(crate) suggest: fn(&[&str], &str, &SuggestContext) -> Vec<String>,
}

/// gamemode 指令家族的用法说明。
pub(crate) const GAMEMODE_USAGE: &str = "/gamemode <survival|creative>";
/// give 指令家族的用法说明。
pub(crate) const GIVE_USAGE: &str = "/give <物品ID> [数量]";
/// help 指令家族的用法说明。
pub(crate) const HELP_USAGE: &str = "/help [指令名]";
/// seed 指令家族的用法说明。
pub(crate) const SEED_USAGE: &str = "/seed";
/// time 指令家族的用法说明。
pub(crate) const TIME_USAGE: &str = "/time set <分钟|day|noon|night|midnight> | /time scale <倍率>";
/// tp 指令家族的用法说明。
pub(crate) const TP_USAGE: &str = "/tp <x> <y> <z>";

/// 全部已注册指令。新增指令 = 实现家族解析与候选函数 + 在此追加一行。
pub(crate) const COMMANDS: &[CommandSpec] = &[
    CommandSpec {
        name: "gamemode",
        usage: GAMEMODE_USAGE,
        parse: parse_gamemode,
        suggest: suggest_gamemode,
    },
    CommandSpec {
        name: "give",
        usage: GIVE_USAGE,
        parse: parse_give,
        suggest: suggest_give,
    },
    CommandSpec {
        name: "help",
        usage: HELP_USAGE,
        parse: parse_help,
        suggest: suggest_help,
    },
    CommandSpec {
        name: "seed",
        usage: SEED_USAGE,
        parse: parse_seed,
        suggest: suggest_seed,
    },
    CommandSpec {
        name: "time",
        usage: TIME_USAGE,
        parse: parse_time,
        suggest: suggest_time,
    },
    CommandSpec {
        name: "tp",
        usage: TP_USAGE,
        parse: parse_tp,
        suggest: suggest_tp,
    },
];

/// 预设时间名对应的当日分钟数。
pub(crate) const TIME_PRESETS: &[(&str, u64)] = &[
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
                format_available(available)
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
            Self::InvalidItemValue { value } => {
                format!("无法识别的物品“{value}”，使用物品短名或 命名空间:名称")
            }
            Self::InvalidCountValue { value } => {
                format!("无法识别的数量“{value}”，需要一个整数")
            }
            Self::CountOutOfRange { value } => {
                format!("数量 {value} 超出范围，支持 1..={MAX_GIVE_COUNT}")
            }
            Self::InvalidCoordinate { value } => {
                format!("无法识别的坐标“{value}”，需要一个数字")
            }
            Self::InvalidGameMode { value } => {
                format!("无法识别的模式“{value}”，可用：survival / creative")
            }
            Self::InvalidHelpTopic { name, available } => format!(
                "未知指令 /{name}，可用指令：{}",
                format_available(available)
            ),
        }
    }
}

/// 把可用指令名列表格式化为 "/a /b /c" 形式。
fn format_available(available: &[&'static str]) -> String {
    available
        .iter()
        .map(|name| format!("/{name}"))
        .collect::<Vec<_>>()
        .join(" ")
}

/// gamemode 指令家族：设置生存或创造模式。
fn parse_gamemode(usage: &'static str, args: &[&str]) -> Result<GameCommand, CommandParseError> {
    let Some(token) = args.first().copied() else {
        return Err(CommandParseError::MissingArgument { usage });
    };
    let mode = if token.eq_ignore_ascii_case("survival") || token.eq_ignore_ascii_case("s") {
        GameMode::Survival
    } else if token.eq_ignore_ascii_case("creative") || token.eq_ignore_ascii_case("c") {
        GameMode::Creative
    } else {
        return Err(CommandParseError::InvalidGameMode {
            value: token.to_string(),
        });
    };
    Ok(GameCommand::GameModeSet { mode })
}

/// give 指令家族：给予物品，数量缺省为 1。
fn parse_give(usage: &'static str, args: &[&str]) -> Result<GameCommand, CommandParseError> {
    let Some(token) = args.first().copied() else {
        return Err(CommandParseError::MissingArgument { usage });
    };
    let item = ItemId::parse(token).map_err(|_| CommandParseError::InvalidItemValue {
        value: token.to_string(),
    })?;
    let count = match args.get(1).copied() {
        None => 1,
        Some(raw) => {
            let count = raw
                .parse::<u32>()
                .map_err(|_| CommandParseError::InvalidCountValue {
                    value: raw.to_string(),
                })?;
            if count == 0 || count > MAX_GIVE_COUNT {
                return Err(CommandParseError::CountOutOfRange { value: count });
            }
            count
        }
    };
    Ok(GameCommand::Give { item, count })
}

/// help 指令家族：无参数列出全部指令，带参数校验并规范指令名。
fn parse_help(_usage: &'static str, args: &[&str]) -> Result<GameCommand, CommandParseError> {
    let Some(token) = args.first().copied() else {
        return Ok(GameCommand::Help { topic: None });
    };
    let Some(spec) = COMMANDS
        .iter()
        .find(|spec| spec.name.eq_ignore_ascii_case(token))
    else {
        return Err(CommandParseError::InvalidHelpTopic {
            name: token.to_string(),
            available: COMMANDS.iter().map(|spec| spec.name).collect(),
        });
    };
    Ok(GameCommand::Help {
        topic: Some(spec.name.to_owned()),
    })
}

/// seed 指令家族：无参数，直接返回。
fn parse_seed(_usage: &'static str, _args: &[&str]) -> Result<GameCommand, CommandParseError> {
    Ok(GameCommand::Seed)
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

/// tp 指令家族：解析三个绝对坐标分量。
fn parse_tp(usage: &'static str, args: &[&str]) -> Result<GameCommand, CommandParseError> {
    if args.len() < 3 {
        return Err(CommandParseError::MissingArgument { usage });
    }
    let mut values = [0.0f32; 3];
    for (slot, token) in values.iter_mut().zip(args) {
        let value = token
            .parse::<f32>()
            .map_err(|_| CommandParseError::InvalidCoordinate {
                value: token.to_string(),
            })?;
        if !value.is_finite() || value.abs() > MAX_COORDINATE {
            return Err(CommandParseError::InvalidCoordinate {
                value: token.to_string(),
            });
        }
        *slot = value;
    }
    Ok(GameCommand::Teleport {
        x: values[0],
        y: values[1],
        z: values[2],
    })
}

/// gamemode 家族的参数候选：模式名。
fn suggest_gamemode(args: &[&str], prefix: &str, _context: &SuggestContext) -> Vec<String> {
    if args.is_empty() {
        filter_candidates(["survival", "creative"], prefix)
    } else {
        Vec::new()
    }
}

/// give 家族的参数候选：第一个参数补全物品短名；数量是自由数字不提供候选。
fn suggest_give(args: &[&str], prefix: &str, context: &SuggestContext) -> Vec<String> {
    if args.is_empty() {
        filter_candidates(context.item_names.iter().map(String::as_str), prefix)
    } else {
        Vec::new()
    }
}

/// help 家族的参数候选：指令根名。
fn suggest_help(args: &[&str], prefix: &str, _context: &SuggestContext) -> Vec<String> {
    if args.is_empty() {
        filter_candidates(COMMANDS.iter().map(|spec| spec.name), prefix)
    } else {
        Vec::new()
    }
}

/// seed 家族无参数候选。
fn suggest_seed(_args: &[&str], _prefix: &str, _context: &SuggestContext) -> Vec<String> {
    Vec::new()
}

/// time 家族的参数候选：未到子指令时给子指令名，set 之后给预设时间名。
///
/// scale 的倍率是自由数字、set 的分钟值不可枚举，均不提供候选。
fn suggest_time(args: &[&str], prefix: &str, _context: &SuggestContext) -> Vec<String> {
    match args.first() {
        None => filter_candidates(["set", "scale"], prefix),
        Some(sub) if sub.eq_ignore_ascii_case("set") => {
            filter_candidates(TIME_PRESETS.iter().map(|(name, _)| *name), prefix)
        }
        _ => Vec::new(),
    }
}

/// tp 家族的坐标是自由数字，不提供候选。
fn suggest_tp(_args: &[&str], _prefix: &str, _context: &SuggestContext) -> Vec<String> {
    Vec::new()
}

/// 按忽略大小写的前缀过滤候选词并收集为字符串列表。
fn filter_candidates<'a>(
    candidates: impl IntoIterator<Item = &'a str>,
    prefix: &str,
) -> Vec<String> {
    let lowered = prefix.to_ascii_lowercase();
    candidates
        .into_iter()
        .filter(|candidate| candidate.starts_with(&lowered))
        .map(String::from)
        .collect()
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
