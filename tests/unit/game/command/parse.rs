//! 指令解析的单元测试。

use super::*;

#[test]
fn parses_time_set_with_numeric_minute() {
    assert_eq!(
        parse_command("time set 480"),
        Ok(GameCommand::TimeSet { minute_of_day: 480 })
    );
}

#[test]
fn parses_time_set_at_day_boundaries() {
    assert_eq!(
        parse_command("time set 0"),
        Ok(GameCommand::TimeSet { minute_of_day: 0 })
    );
    assert_eq!(
        parse_command("time set 1439"),
        Ok(GameCommand::TimeSet {
            minute_of_day: 1439
        })
    );
}

#[test]
fn parses_time_set_presets_case_insensitively() {
    assert_eq!(
        parse_command("time set NOON"),
        Ok(GameCommand::TimeSet { minute_of_day: 720 })
    );
    assert_eq!(
        parse_command("Time Set Midnight"),
        Ok(GameCommand::TimeSet { minute_of_day: 0 })
    );
    assert_eq!(
        parse_command("time set day"),
        Ok(GameCommand::TimeSet { minute_of_day: 480 })
    );
    assert_eq!(
        parse_command("time set night"),
        Ok(GameCommand::TimeSet {
            minute_of_day: 1200
        })
    );
}

#[test]
fn root_and_subcommands_ignore_case_and_extra_whitespace() {
    assert_eq!(
        parse_command("  TIME   set   100  "),
        Ok(GameCommand::TimeSet { minute_of_day: 100 })
    );
    assert_eq!(
        parse_command("Time SCALE 2.5"),
        Ok(GameCommand::TimeScale { scale: 2.5 })
    );
}

#[test]
fn parses_time_scale_values() {
    assert_eq!(
        parse_command("time scale 0"),
        Ok(GameCommand::TimeScale { scale: 0.0 })
    );
    assert_eq!(
        parse_command("time scale 100"),
        Ok(GameCommand::TimeScale { scale: 100.0 })
    );
    assert_eq!(
        parse_command("time scale 0.5"),
        Ok(GameCommand::TimeScale { scale: 0.5 })
    );
}

#[test]
fn ignores_trailing_arguments() {
    assert_eq!(
        parse_command("time set 100 extra"),
        Ok(GameCommand::TimeSet { minute_of_day: 100 })
    );
}

#[test]
fn rejects_empty_lines() {
    assert_eq!(parse_command(""), Err(CommandParseError::Empty));
    assert_eq!(parse_command("   "), Err(CommandParseError::Empty));
}

#[test]
fn rejects_unknown_command_and_lists_available() {
    assert_eq!(
        parse_command("warp 10"),
        Err(CommandParseError::UnknownCommand {
            name: "warp".to_owned(),
            available: vec!["gamemode", "give", "help", "seed", "time", "tp"],
        })
    );
}

#[test]
fn rejects_missing_subcommands_and_arguments() {
    assert_eq!(
        parse_command("time"),
        Err(CommandParseError::MissingSubcommand { usage: TIME_USAGE })
    );
    assert_eq!(
        parse_command("time warp 10"),
        Err(CommandParseError::MissingSubcommand { usage: TIME_USAGE })
    );
    assert_eq!(
        parse_command("time set"),
        Err(CommandParseError::MissingArgument { usage: TIME_USAGE })
    );
    assert_eq!(
        parse_command("time scale"),
        Err(CommandParseError::MissingArgument { usage: TIME_USAGE })
    );
}

#[test]
fn rejects_time_set_values_out_of_range_or_non_numeric() {
    assert_eq!(
        parse_command("time set 1440"),
        Err(CommandParseError::InvalidTimeValue {
            value: "1440".to_owned()
        })
    );
    assert_eq!(
        parse_command("time set abc"),
        Err(CommandParseError::InvalidTimeValue {
            value: "abc".to_owned()
        })
    );
    assert_eq!(
        parse_command("time set -10"),
        Err(CommandParseError::InvalidTimeValue {
            value: "-10".to_owned()
        })
    );
}

#[test]
fn rejects_time_scale_values_out_of_range_or_non_numeric() {
    assert_eq!(
        parse_command("time scale -1"),
        Err(CommandParseError::ScaleOutOfRange { value: -1.0 })
    );
    assert_eq!(
        parse_command("time scale 100.5"),
        Err(CommandParseError::ScaleOutOfRange { value: 100.5 })
    );
    assert!(matches!(
        parse_command("time scale NaN"),
        Err(CommandParseError::ScaleOutOfRange { .. })
    ));
    assert_eq!(
        parse_command("time scale fast"),
        Err(CommandParseError::InvalidScaleValue {
            value: "fast".to_owned()
        })
    );
}

#[test]
fn parses_help_with_and_without_topic() {
    assert_eq!(parse_command("help"), Ok(GameCommand::Help { topic: None }));
    // 主题规范为注册表小写形式。
    assert_eq!(
        parse_command("HELP TIME"),
        Ok(GameCommand::Help {
            topic: Some("time".to_owned())
        })
    );
}

#[test]
fn rejects_help_for_unknown_topics() {
    assert_eq!(
        parse_command("help warp"),
        Err(CommandParseError::InvalidHelpTopic {
            name: "warp".to_owned(),
            available: vec!["gamemode", "give", "help", "seed", "time", "tp"],
        })
    );
}

#[test]
fn parses_seed_without_arguments() {
    assert_eq!(parse_command("seed"), Ok(GameCommand::Seed));
}

#[test]
fn parses_gamemode_names_and_aliases() {
    assert_eq!(
        parse_command("gamemode survival"),
        Ok(GameCommand::GameModeSet {
            mode: GameMode::Survival
        })
    );
    assert_eq!(
        parse_command("gamemode CREATIVE"),
        Ok(GameCommand::GameModeSet {
            mode: GameMode::Creative
        })
    );
    assert_eq!(
        parse_command("gamemode s"),
        Ok(GameCommand::GameModeSet {
            mode: GameMode::Survival
        })
    );
    assert_eq!(
        parse_command("gamemode C"),
        Ok(GameCommand::GameModeSet {
            mode: GameMode::Creative
        })
    );
}

#[test]
fn rejects_unknown_gamemode_names() {
    assert_eq!(
        parse_command("gamemode hard"),
        Err(CommandParseError::InvalidGameMode {
            value: "hard".to_owned()
        })
    );
    assert_eq!(
        parse_command("gamemode"),
        Err(CommandParseError::MissingArgument {
            usage: GAMEMODE_USAGE
        })
    );
}

#[test]
fn parses_teleport_coordinates() {
    assert_eq!(
        parse_command("tp 1 2 3"),
        Ok(GameCommand::Teleport {
            x: 1.0,
            y: 2.0,
            z: 3.0
        })
    );
    assert_eq!(
        parse_command("tp -10.5 0 100"),
        Ok(GameCommand::Teleport {
            x: -10.5,
            y: 0.0,
            z: 100.0
        })
    );
}

#[test]
fn rejects_teleport_with_missing_or_invalid_coordinates() {
    assert_eq!(
        parse_command("tp 1 2"),
        Err(CommandParseError::MissingArgument { usage: TP_USAGE })
    );
    assert_eq!(
        parse_command("tp 1 2 abc"),
        Err(CommandParseError::InvalidCoordinate {
            value: "abc".to_owned()
        })
    );
    // 超出可表达范围的坐标同样拒绝。
    assert_eq!(
        parse_command("tp 1 2 10000000000"),
        Err(CommandParseError::InvalidCoordinate {
            value: "10000000000".to_owned()
        })
    );
}

#[test]
fn parses_give_with_short_and_full_item_ids() {
    assert_eq!(
        parse_command("give apple"),
        Ok(GameCommand::Give {
            item: ItemId::parse("apple").unwrap(),
            count: 1
        })
    );
    assert_eq!(
        parse_command("give century_journey:coal 10"),
        Ok(GameCommand::Give {
            item: ItemId::parse("century_journey:coal").unwrap(),
            count: 10
        })
    );
    // 数量上限边界可接受。
    assert_eq!(
        parse_command("give apple 1000"),
        Ok(GameCommand::Give {
            item: ItemId::parse("apple").unwrap(),
            count: MAX_GIVE_COUNT
        })
    );
}

#[test]
fn rejects_give_with_invalid_item_or_count() {
    assert_eq!(
        parse_command("give"),
        Err(CommandParseError::MissingArgument { usage: GIVE_USAGE })
    );
    // 冒号后为空等畸形标识在解析期即被拒绝；未知短名由执行期注册表校验。
    assert_eq!(
        parse_command("give mod:"),
        Err(CommandParseError::InvalidItemValue {
            value: "mod:".to_owned()
        })
    );
    assert_eq!(
        parse_command("give apple abc"),
        Err(CommandParseError::InvalidCountValue {
            value: "abc".to_owned()
        })
    );
    assert_eq!(
        parse_command("give apple 0"),
        Err(CommandParseError::CountOutOfRange { value: 0 })
    );
    assert_eq!(
        parse_command("give apple 1001"),
        Err(CommandParseError::CountOutOfRange { value: 1001 })
    );
}

#[test]
fn feedback_texts_are_player_readable() {
    assert_eq!(CommandParseError::Empty.to_feedback(), "输入指令为空");
    let feedback = CommandParseError::UnknownCommand {
        name: "give".to_owned(),
        available: vec!["time"],
    }
    .to_feedback();
    assert!(feedback.contains("/give"));
    assert!(feedback.contains("/time"));
    assert!(
        CommandParseError::MissingSubcommand { usage: TIME_USAGE }
            .to_feedback()
            .contains("/time")
    );
    assert!(
        CommandParseError::InvalidTimeValue {
            value: "abc".to_owned()
        }
        .to_feedback()
        .contains("abc")
    );
}
