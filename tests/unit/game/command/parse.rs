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
        parse_command("give stone 1"),
        Err(CommandParseError::UnknownCommand {
            name: "give".to_owned(),
            available: vec!["time"],
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
