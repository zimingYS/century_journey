//! 调试浮层纯函数的单元测试。

use super::*;

#[test]
fn compass_direction_covers_all_eight_directions() {
    assert_eq!(compass_direction(Vec3::new(0.0, 0.0, -1.0)), "北");
    assert_eq!(compass_direction(Vec3::new(1.0, 0.0, 0.0)), "东");
    assert_eq!(compass_direction(Vec3::new(0.0, 0.0, 1.0)), "南");
    assert_eq!(compass_direction(Vec3::new(-1.0, 0.0, 0.0)), "西");
    assert_eq!(compass_direction(Vec3::new(1.0, 0.0, -1.0)), "东北");
    assert_eq!(compass_direction(Vec3::new(1.0, 0.0, 1.0)), "东南");
    assert_eq!(compass_direction(Vec3::new(-1.0, 0.0, 1.0)), "西南");
    assert_eq!(compass_direction(Vec3::new(-1.0, 0.0, -1.0)), "西北");
}

#[test]
fn compass_direction_handles_sector_boundaries() {
    // 22.5 度扇区边界附近稳定落入相邻扇区之一。
    let near_north_east = Vec3::new(
        30.0_f32.to_radians().sin(),
        0.0,
        -30.0_f32.to_radians().cos(),
    );
    assert_eq!(compass_direction(near_north_east), "东北");
    // 俯仰分量不影响水平方位判定。
    assert_eq!(compass_direction(Vec3::new(0.0, 0.7, -0.7)), "北");
}

#[test]
fn season_labels_are_single_chinese_characters() {
    assert_eq!(season_label(Season::Spring), "春");
    assert_eq!(season_label(Season::Summer), "夏");
    assert_eq!(season_label(Season::Autumn), "秋");
    assert_eq!(season_label(Season::Winter), "冬");
}

fn full_data() -> DebugOverlayData {
    DebugOverlayData {
        fps: 60.0,
        frame_ms: 16.7,
        position: Some(Vec3::new(1.25, 70.5, -2.75)),
        facing: Some(FacingInfo {
            direction: "北",
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
            season: "春",
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
    let text = build_lines(&full_data());
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
    let text = build_lines(&data);
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
    let text = build_lines(&data);
    assert!(text.contains("区块："));
    assert!(!text.contains("朝向："));
}
