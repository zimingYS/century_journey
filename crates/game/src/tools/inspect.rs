use crate::interaction::target::PlayerTarget;
use bevy::prelude::*;
use cj_core::pos::BlockPos;

/// 假温度场
pub fn fake_temperature(pos: BlockPos) -> f32 {
    30.0 - pos.y as f32 * 0.1
}

/// 根据目标方块采样温度
pub fn sample_temperature(target: Option<BlockPos>) -> Option<(BlockPos, f32)> {
    target.map(|pos| (pos, fake_temperature(pos)))
}
/// 读数文本组件
#[derive(Component)]
pub struct InspectReadout;

/// 创建检视读数 UI
pub fn spawn_inspect_readout(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: px(8.0),
                left: px(8.0),
                padding: UiRect::all(px(8.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.65)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("target: none"),
                TextFont {
                    font_size: FontSize::Px(16.0),
                    ..default()
                },
                TextColor(Color::WHITE),
                InspectReadout,
            ));
        });
}

/// 更新检视读数
pub fn update_inspect_readout(
    target: Res<PlayerTarget>,
    mut readouts: Query<&mut Text, With<InspectReadout>>,
) {
    let content = match sample_temperature(target.block) {
        Some((pos, temperature)) => format!(
            "target: ({}, {}, {}) temp: {:.1} C",
            pos.x, pos.y, pos.z, temperature
        ),
        None => "target: none".to_string(),
    };

    for mut text in &mut readouts {
        if text.0 != content {
            text.0.clone_from(&content);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fake_temperature_is_deterministic() {
        let pos = BlockPos::new(0, 64, 0);

        let first = fake_temperature(pos);
        let second = fake_temperature(pos);

        assert_eq!(first, second);
    }

    #[test]
    fn temperature_decreases_by_one_per_ten_blocks() {
        let low = fake_temperature(BlockPos::new(0, 64, 0));
        let high = fake_temperature(BlockPos::new(0, 74, 0));

        assert!((low - high - 1.0).abs() < 0.00001);
    }

    #[test]
    fn samples_temperature_at_target_block() {
        let target = BlockPos::new(3, 64, 7);

        let result = sample_temperature(Some(target));

        assert!(result.is_some());

        let (pos, temperature) = result.unwrap();

        assert_eq!(pos, target);
        assert!((temperature - 23.6).abs() < 0.001);
    }

    #[test]
    fn no_target_returns_none() {
        let result = sample_temperature(None);

        assert_eq!(result, None);
    }
}
