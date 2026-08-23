//! 在固定步内消费指令消息并修改权威玩法状态。

use bevy::prelude::*;

use crate::game::command::components::{CommandOutput, GameCommandSubmitted};
use crate::game::command::parse::{GameCommand, parse_command};
use crate::game::gameplay::rules::GameRules;
use crate::game::world::time::WorldSimulationClock;

/// 固定步内消费指令消息：解析、修改权威状态并写回反馈。
///
/// 该系统不使用时间源：指令直接修改权威状态，随后续固定步自然生效。
pub fn execute_game_command_system(
    mut commands: MessageReader<GameCommandSubmitted>,
    mut clock: ResMut<WorldSimulationClock>,
    mut rules: ResMut<GameRules>,
    mut output: MessageWriter<CommandOutput>,
) {
    for submitted in commands.read() {
        match parse_command(&submitted.raw) {
            Ok(GameCommand::TimeSet { minute_of_day }) => {
                clock.set_time_of_day(minute_of_day);
                let snapshot = clock.snapshot();
                output.write(CommandOutput {
                    text: format!("时间已设置为 {:02}:{:02}", snapshot.hour, snapshot.minute),
                });
            }
            Ok(GameCommand::TimeScale { scale }) => {
                rules.time_scale = scale;
                let text = if scale == 0.0 {
                    "时间已暂停".to_owned()
                } else {
                    format!("时间流速已设置为 {scale} 倍")
                };
                output.write(CommandOutput { text });
            }
            Err(error) => {
                output.write(CommandOutput {
                    text: error.to_feedback(),
                });
            }
        }
    }
}
