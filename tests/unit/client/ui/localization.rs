//! 本地化文本刷新系统的最小 ECS 测试：同实体与子实体两种布局、变更检测与兜底。

use super::*;
use crate::engine::localization::{LanguageId, LanguageInfo};
use std::collections::BTreeMap;

/// 构造中英双语资源，zh-CN 为回退语言。
fn bilingual_localization() -> Localization {
    let languages = vec![
        LanguageInfo {
            id: LanguageId::new("en-US"),
            native_name: "English".to_string(),
        },
        LanguageInfo {
            id: LanguageId::new("zh-CN"),
            native_name: "简体中文".to_string(),
        },
    ];
    let mut zh_table = BTreeMap::new();
    zh_table.insert("menu.play".to_string(), "进入世界".to_string());
    let mut en_table = BTreeMap::new();
    en_table.insert("menu.play".to_string(), "Play World".to_string());
    let mut tables = BTreeMap::new();
    tables.insert(LanguageId::new("zh-CN"), zh_table);
    tables.insert(LanguageId::new("en-US"), en_table);
    Localization::new(languages, tables)
}

/// 构造仅包含刷新系统的调度；同一实例需跨运行复用以保留变更检测状态。
fn refresh_schedule() -> Schedule {
    let mut schedule = Schedule::default();
    schedule.add_systems(refresh_localized_text_system);
    schedule
}

#[test]
fn refresh_rewrites_label_and_button_text_on_language_change() {
    let mut world = World::new();
    world.insert_resource(bilingual_localization());
    // 纯标签：标记与 Text 同实体。
    let label = world
        .spawn((LocalizedText::new("menu.play"), Text::new("占位")))
        .id();
    // 组合按钮：标记在父实体，Text 在子实体；ChildOf 关系自动维护 Children。
    let button = world.spawn(LocalizedText::new("menu.play")).id();
    let child = world.spawn((Text::new("占位"), ChildOf(button))).id();

    let mut schedule = refresh_schedule();
    // 资源刚插入视为已变更，首次运行即按初始语言填充。
    schedule.run(&mut world);
    assert_eq!(world.get::<Text>(label).unwrap().0, "进入世界");
    assert_eq!(world.get::<Text>(child).unwrap().0, "进入世界");

    // 资源未变化时跳过重写，界面上的外部改动得以保留。
    world.get_mut::<Text>(label).unwrap().0 = "外部改动".to_string();
    schedule.run(&mut world);
    assert_eq!(world.get::<Text>(label).unwrap().0, "外部改动");

    // 切换语言触发资源变更，标签与按钮文本同步刷新。
    world
        .resource_mut::<Localization>()
        .set_active(&LanguageId::new("en-US"));
    schedule.run(&mut world);
    assert_eq!(world.get::<Text>(label).unwrap().0, "Play World");
    assert_eq!(world.get::<Text>(child).unwrap().0, "Play World");
}

#[test]
fn refresh_uses_fallback_text_when_key_is_missing() {
    let mut world = World::new();
    world.insert_resource(bilingual_localization());
    let label = world
        .spawn((
            LocalizedText::with_fallback("dynamic.unknown", "数据自带名称"),
            Text::new("占位"),
        ))
        .id();

    let mut schedule = refresh_schedule();
    schedule.run(&mut world);

    // 键缺失时不暴露内部键名，退回调用方提供的兜底文本。
    assert_eq!(world.get::<Text>(label).unwrap().0, "数据自带名称");
}
