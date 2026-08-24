//! 指令补全候选的单元测试。

use super::*;
use crate::game::command::parse::TIME_USAGE;

/// 无物品数据的空上下文，用于不依赖注册表的用例。
fn empty_context() -> SuggestContext {
    SuggestContext::default()
}

/// 带样例物品短名的上下文，用于 /give 候选。
fn context_with_items() -> SuggestContext {
    SuggestContext {
        item_names: vec![
            "apple".to_owned(),
            "coal".to_owned(),
            "iron_axe".to_owned(),
            "raw_iron".to_owned(),
            "water_bottle".to_owned(),
        ],
    }
}

#[test]
fn plain_lines_have_no_completions() {
    let result = completions("你好世界", &empty_context());
    assert!(result.lines.is_empty());
    assert_eq!(result.usage, None);
}

#[test]
fn slash_alone_completes_all_registered_roots() {
    for line in ["/", "/ ", "  /"] {
        let result = completions(line, &empty_context());
        assert_eq!(
            result.lines,
            [
                "/gamemode ".to_owned(),
                "/give ".to_owned(),
                "/help ".to_owned(),
                "/seed ".to_owned(),
                "/time ".to_owned(),
                "/tp ".to_owned(),
            ],
            "line: {line:?}"
        );
        // 根级多候选不展示用法。
        assert_eq!(result.usage, None);
    }
}

#[test]
fn root_prefix_matching_ignores_case() {
    assert_eq!(
        completions("/Ti", &empty_context()).lines,
        ["/time ".to_owned()]
    );
    assert_eq!(
        completions("/TIME", &empty_context()).lines,
        ["/time ".to_owned()]
    );
    // 唯一根候选时展示用法。
    assert_eq!(completions("/Ti", &empty_context()).usage, Some(TIME_USAGE));
}

#[test]
fn unknown_root_prefix_has_no_completions() {
    let result = completions("/xyz", &empty_context());
    assert!(result.lines.is_empty());
    assert_eq!(result.usage, None);
}

#[test]
fn subcommands_complete_after_the_root() {
    assert_eq!(
        completions("/time ", &empty_context()).lines,
        ["/time set ".to_owned(), "/time scale ".to_owned()]
    );
    assert_eq!(
        completions("/time s", &empty_context()).lines,
        ["/time set ".to_owned(), "/time scale ".to_owned()]
    );
    assert_eq!(
        completions("/time se", &empty_context()).lines,
        ["/time set ".to_owned()]
    );
    // 根名补全为规范小写，已输入的子指令原样保留。
    assert_eq!(
        completions("  /TIME SET", &empty_context()).lines,
        ["/time set ".to_owned()]
    );
}

#[test]
fn set_completes_preset_time_names() {
    assert_eq!(
        completions("/time set ", &empty_context()).lines,
        [
            "/time set day ".to_owned(),
            "/time set noon ".to_owned(),
            "/time set night ".to_owned(),
            "/time set midnight ".to_owned(),
        ]
    );
    assert_eq!(
        completions("/time set d", &empty_context()).lines,
        ["/time set day ".to_owned()]
    );
    assert_eq!(
        completions("/time set N", &empty_context()).lines,
        ["/time set noon ".to_owned(), "/time set night ".to_owned()]
    );
    assert_eq!(
        completions("/time set no", &empty_context()).lines,
        ["/time set noon ".to_owned()]
    );
}

#[test]
fn free_number_arguments_have_no_candidates_but_keep_usage() {
    let result = completions("/time scale 5", &empty_context());
    assert!(result.lines.is_empty());
    assert_eq!(result.usage, Some(TIME_USAGE));

    let result = completions("/time set 60", &empty_context());
    assert!(result.lines.is_empty());
    assert_eq!(result.usage, Some(TIME_USAGE));
}

#[test]
fn complete_command_still_offers_itself_and_usage() {
    let result = completions("/time set day", &empty_context());
    assert_eq!(result.lines, ["/time set day ".to_owned()]);
    assert_eq!(result.usage, Some(TIME_USAGE));
}

#[test]
fn give_completes_item_short_names_from_context() {
    let context = context_with_items();
    assert_eq!(
        completions("/give ", &context).lines,
        [
            "/give apple ".to_owned(),
            "/give coal ".to_owned(),
            "/give iron_axe ".to_owned(),
            "/give raw_iron ".to_owned(),
            "/give water_bottle ".to_owned(),
        ]
    );
    assert_eq!(
        completions("/give ir", &context).lines,
        ["/give iron_axe ".to_owned()]
    );
    assert_eq!(
        completions("/give r", &context).lines,
        ["/give raw_iron ".to_owned()]
    );
    assert_eq!(
        completions("/give IR", &context).lines,
        ["/give iron_axe ".to_owned()]
    );
}

#[test]
fn give_count_argument_has_no_candidates_but_keeps_usage() {
    let context = context_with_items();
    let result = completions("/give apple ", &context);
    assert!(result.lines.is_empty());
    assert_eq!(result.usage, Some(crate::game::command::parse::GIVE_USAGE));
}

#[test]
fn help_completes_command_names() {
    assert_eq!(
        completions("/help t", &empty_context()).lines,
        ["/help time ".to_owned(), "/help tp ".to_owned()]
    );
    assert_eq!(
        completions("/help ", &empty_context()).lines.len(),
        6,
        "根名候选应覆盖全部注册指令"
    );
}

#[test]
fn gamemode_completes_mode_names() {
    assert_eq!(
        completions("/gamemode ", &empty_context()).lines,
        [
            "/gamemode survival ".to_owned(),
            "/gamemode creative ".to_owned()
        ]
    );
    assert_eq!(
        completions("/gamemode c", &empty_context()).lines,
        ["/gamemode creative ".to_owned()]
    );
    // 已有模式参数时不再提供候选。
    assert!(
        completions("/gamemode creative ", &empty_context())
            .lines
            .is_empty()
    );
}

#[test]
fn seed_has_no_candidates() {
    let result = completions("/seed ", &empty_context());
    assert!(result.lines.is_empty());
}
