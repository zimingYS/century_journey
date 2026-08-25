//! 调试浮层纯函数的单元测试。

use super::*;
use crate::engine::localization::{LanguageId, LanguageInfo};
use std::collections::BTreeMap;

/// 构造包含调试浮层全部文案键的中文本地化资源。
fn debug_localization() -> Localization {
    let entries = [
        ("debug.title", "Century Journey 调试（F3 隐藏）"),
        ("debug.fps", "FPS {fps}（{ms} ms/帧）"),
        ("debug.position", "坐标：({x}, {y}, {z})"),
        ("debug.position-none", "坐标：暂无玩家实体"),
        (
            "debug.chunk",
            "区块：({x}, {y}, {z}) 局部 ({lx}, {ly}, {lz})",
        ),
        ("debug.chunk-none", "区块：暂无玩家实体"),
        (
            "debug.facing",
            "朝向：{direction}（偏航 {yaw}°，俯仰 {pitch}°）",
        ),
        (
            "debug.time",
            "时间：第 {day} 日 {hour}:{minute}（{season}，倍率 {scale}）",
        ),
        ("debug.time-none", "时间：世界时钟未就绪"),
        ("debug.seed", "种子：{seed}"),
        (
            "debug.chunk-count",
            "区块计数：已加载 {loaded}，预期 {expected}，已渲染 {rendered}",
        ),
        ("debug.tick", "模拟刻：{tick}"),
        ("debug.direction.north", "北"),
        ("debug.direction.north-east", "东北"),
        ("debug.direction.east", "东"),
        ("debug.direction.south-east", "东南"),
        ("debug.direction.south", "南"),
        ("debug.direction.south-west", "西南"),
        ("debug.direction.west", "西"),
        ("debug.direction.north-west", "西北"),
        ("debug.season.spring", "春"),
        ("debug.season.summer", "夏"),
        ("debug.season.autumn", "秋"),
        ("debug.season.winter", "冬"),
    ];
    let mut zh_table = BTreeMap::new();
    for (key, text) in entries {
        zh_table.insert(key.to_string(), text.to_string());
    }
    let mut tables = BTreeMap::new();
    tables.insert(LanguageId::new("zh-CN"), zh_table);
    Localization::new(
        vec![LanguageInfo {
            id: LanguageId::new("zh-CN"),
            native_name: "简体中文".to_string(),
        }],
        tables,
    )
}

#[test]
fn compass_direction_covers_all_eight_directions() {
    assert_eq!(
        compass_direction(Vec3::new(0.0, 0.0, -1.0)),
        "debug.direction.north"
    );
    assert_eq!(
        compass_direction(Vec3::new(1.0, 0.0, 0.0)),
        "debug.direction.east"
    );
    assert_eq!(
        compass_direction(Vec3::new(0.0, 0.0, 1.0)),
        "debug.direction.south"
    );
    assert_eq!(
        compass_direction(Vec3::new(-1.0, 0.0, 0.0)),
        "debug.direction.west"
    );
    assert_eq!(
        compass_direction(Vec3::new(1.0, 0.0, -1.0)),
        "debug.direction.north-east"
    );
    assert_eq!(
        compass_direction(Vec3::new(1.0, 0.0, 1.0)),
        "debug.direction.south-east"
    );
    assert_eq!(
        compass_direction(Vec3::new(-1.0, 0.0, 1.0)),
        "debug.direction.south-west"
    );
    assert_eq!(
        compass_direction(Vec3::new(-1.0, 0.0, -1.0)),
        "debug.direction.north-west"
    );
}

#[test]
fn compass_direction_handles_sector_boundaries() {
    // 22.5 度扇区边界附近稳定落入相邻扇区之一。
    let near_north_east = Vec3::new(
        30.0_f32.to_radians().sin(),
        0.0,
        -30.0_f32.to_radians().cos(),
    );
    assert_eq!(
        compass_direction(near_north_east),
        "debug.direction.north-east"
    );
    // 俯仰分量不影响水平方位判定。
    assert_eq!(
        compass_direction(Vec3::new(0.0, 0.7, -0.7)),
        "debug.direction.north"
    );
}

#[test]
fn season_labels_map_to_localization_keys() {
    assert_eq!(season_label(Season::Spring), "debug.season.spring");
    assert_eq!(season_label(Season::Summer), "debug.season.summer");
    assert_eq!(season_label(Season::Autumn), "debug.season.autumn");
    assert_eq!(season_label(Season::Winter), "debug.season.winter");
}

#[test]
fn localization_resolves_direction_and_season_keys() {
    let localization = debug_localization();
    assert_eq!(
        localization.get(compass_direction(Vec3::new(0.0, 0.0, -1.0))),
        "北"
    );
    assert_eq!(localization.get(season_label(Season::Spring)), "春");
}

fn full_data() -> DebugOverlayData {
    DebugOverlayData {
        fps: 60.0,
        frame_ms: 16.7,
        position: Some(Vec3::new(1.25, 70.5, -2.75)),
        facing: Some(FacingInfo {
            direction: "debug.direction.north",
            yaw_deg: -15.0,
            pitch_deg: 10.0,
        }),
        chunk: Some(ChunkInfo {
            chunk: IVec3::new(0, 4, -1),
            local: IVec3::new(1, 6, 13),
        }),
        clock: Some(ClockInfo {
            game_day: 3,
            hour: 8,
            minute: 24,
            season: "debug.season.spring",
            time_scale: 2.0,
        }),
        seed: Some(12345),
        chunk_counts: Some(ChunkCounts {
            loaded: 289,
            expected: 289,
            rendered: 180,
        }),
        simulation_tick: Some(987_654),
    }
}

#[test]
fn build_lines_renders_all_data_sections() {
    let text = build_lines(&full_data(), &debug_localization());
    let lines: Vec<&str> = text.split('\n').collect();
    assert!(lines.len() >= 8, "完整数据应生成至少 8 行：{text}");
    assert!(lines[0].contains("F3"));
    assert!(lines[1].contains("FPS 60"));
    assert!(lines[2].contains("(1.2, 70.5, -2.8)"));
    assert!(lines[3].contains("(0, 4, -1)"));
    assert!(lines[3].contains("(1, 6, 13)"));
    assert!(lines[4].contains("北"));
    assert!(lines[5].contains("第 3 日 08:24"));
    assert!(lines[5].contains("春"));
    assert!(lines[5].contains("倍率 2"));
    assert!(lines[6].contains("12345"));
    assert!(lines[7].contains("已加载 289"));
    assert!(text.contains("模拟刻：987654"));
}

#[test]
fn build_lines_reports_missing_player_and_clock() {
    let data = DebugOverlayData {
        fps: 0.0,
        frame_ms: 0.0,
        position: None,
        facing: None,
        chunk: None,
        clock: None,
        seed: None,
        chunk_counts: None,
        simulation_tick: None,
    };
    let text = build_lines(&data, &debug_localization());
    assert!(text.contains("暂无玩家实体"));
    assert!(text.contains("世界时钟未就绪"));
    // 缺失的可选行（种子、区块计数、模拟刻）不应出现占位文本。
    assert!(!text.contains("种子"));
    assert!(!text.contains("模拟刻"));
}

#[test]
fn build_lines_shows_facing_only_with_chunk() {
    // 有坐标但相机缺失时仍渲染区块行，只是省略朝向行。
    let mut data = full_data();
    data.facing = None;
    let text = build_lines(&data, &debug_localization());
    assert!(text.contains("区块："));
    assert!(!text.contains("朝向："));
}

#[test]
fn build_lines_falls_back_to_key_when_translation_missing() {
    // 键缺失时回退为键名本身，便于在浮层上定位漏翻条目。
    let data = DebugOverlayData {
        fps: 0.0,
        frame_ms: 0.0,
        position: None,
        facing: None,
        chunk: None,
        clock: None,
        seed: None,
        chunk_counts: None,
        simulation_tick: None,
    };
    let empty = Localization::new(Vec::new(), BTreeMap::new());
    let text = build_lines(&data, &empty);
    assert!(text.contains("debug.title"));
}
