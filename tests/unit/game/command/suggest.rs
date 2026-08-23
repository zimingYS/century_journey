//! 指令补全候选的单元测试。

use super::*;
use crate::game::command::parse::TIME_USAGE;

#[test]
fn plain_lines_have_no_completions() {
    let result = completions("你好世界");
    assert!(result.lines.is_empty());
    assert_eq!(result.usage, None);
}

#[test]
fn slash_alone_completes_all_registered_roots() {
    for line in ["/", "/ ", "  /"] {
        let result = completions(line);
        assert_eq!(result.lines, ["/time ".to_owned()], "line: {line:?}");
        assert_eq!(result.usage, Some(TIME_USAGE));
    }
}

#[test]
fn root_prefix_matching_ignores_case() {
    assert_eq!(completions("/Ti").lines, ["/time ".to_owned()]);
    assert_eq!(completions("/TIME").lines, ["/time ".to_owned()]);
}

#[test]
fn unknown_root_prefix_has_no_completions() {
    let result = completions("/xyz");
    assert!(result.lines.is_empty());
    assert_eq!(result.usage, None);
}

#[test]
fn subcommands_complete_after_the_root() {
    assert_eq!(
        completions("/time ").lines,
        ["/time set ".to_owned(), "/time scale ".to_owned()]
    );
    assert_eq!(
        completions("/time s").lines,
        ["/time set ".to_owned(), "/time scale ".to_owned()]
    );
    assert_eq!(completions("/time se").lines, ["/time set ".to_owned()]);
    // 根名补全为规范小写，已输入的子指令原样保留。
    assert_eq!(completions("  /TIME SET").lines, ["/time set ".to_owned()]);
}

#[test]
fn set_completes_preset_time_names() {
    assert_eq!(
        completions("/time set ").lines,
        [
            "/time set day ".to_owned(),
            "/time set noon ".to_owned(),
            "/time set night ".to_owned(),
            "/time set midnight ".to_owned(),
        ]
    );
    assert_eq!(
        completions("/time set d").lines,
        ["/time set day ".to_owned()]
    );
    assert_eq!(
        completions("/time set N").lines,
        ["/time set noon ".to_owned(), "/time set night ".to_owned()]
    );
    assert_eq!(
        completions("/time set no").lines,
        ["/time set noon ".to_owned()]
    );
}

#[test]
fn free_number_arguments_have_no_candidates_but_keep_usage() {
    let result = completions("/time scale 5");
    assert!(result.lines.is_empty());
    assert_eq!(result.usage, Some(TIME_USAGE));

    let result = completions("/time set 60");
    assert!(result.lines.is_empty());
    assert_eq!(result.usage, Some(TIME_USAGE));
}

#[test]
fn complete_command_still_offers_itself_and_usage() {
    let result = completions("/time set day");
    assert_eq!(result.lines, ["/time set day ".to_owned()]);
    assert_eq!(result.usage, Some(TIME_USAGE));
}
