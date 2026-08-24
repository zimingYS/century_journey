//! 在固定步内消费指令消息并修改权威玩法状态。

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::content::item::registry::ItemRegistry;
use crate::game::command::components::{CommandOutput, GameCommandSubmitted};
use crate::game::command::parse::{COMMANDS, GameCommand, parse_command};
use crate::game::gameplay::gamemode::{GameMode, PlayerGameMode};
use crate::game::gameplay::rules::GameRules;
use crate::game::inventory::interaction::transfer::{InventoryInsertResult, insert_into_player};
use crate::game::inventory::item::stack::ItemStack;
use crate::game::inventory::state::InventoryState;
use crate::game::player::identity::LocalPlayer;
use crate::game::player::movement::components::PlayerVelocity;
use crate::game::player::physics::components::PlayerGravity;
use crate::game::world::generation::generator::WorldGenerator;
use crate::game::world::time::WorldSimulationClock;

/// 本地玩家的可变权威状态查询，传送与给予指令共用。
type LocalPlayerQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static mut Transform,
        &'static mut PlayerVelocity,
        &'static mut PlayerGravity,
        &'static mut InventoryState,
    ),
    With<LocalPlayer>,
>;

/// 指令执行系统在单步内可访问的权威状态集合。
///
/// 参数集中封装以保持系统签名精简；全部为指令执行所需的权威数据源。
#[derive(SystemParam)]
pub struct CommandExecutionContext<'w, 's> {
    /// 世界权威时钟，/time set 的修改目标。
    clock: ResMut<'w, WorldSimulationClock>,
    /// 会话规则，/time scale 的修改目标。
    rules: ResMut<'w, GameRules>,
    /// 玩家游戏模式；切回生存时由既有飞行清理系统自动关闭飞行。
    gamemode: ResMut<'w, PlayerGameMode>,
    /// 世界生成器，/seed 的种子来源。
    generator: Res<'w, WorldGenerator>,
    /// 物品注册表，/give 的物品存在性校验。
    registry: Res<'w, ItemRegistry>,
    /// 本地玩家实体状态，/tp 与 /give 的修改目标。
    player: LocalPlayerQuery<'w, 's>,
}

/// 固定步内消费指令消息：解析、修改权威状态并写回反馈。
///
/// 该系统不使用时间源：指令直接修改权威状态，随后续固定步自然生效。
pub fn execute_game_command_system(
    mut commands: MessageReader<GameCommandSubmitted>,
    mut context: CommandExecutionContext,
    mut output: MessageWriter<CommandOutput>,
) {
    for submitted in commands.read() {
        match parse_command(&submitted.raw) {
            Ok(GameCommand::Help { topic }) => {
                write_help_output(&topic, &mut output);
            }
            Ok(GameCommand::Seed) => {
                output.write(CommandOutput {
                    text: format!("世界种子：{}", context.generator.seed),
                });
            }
            Ok(GameCommand::TimeSet { minute_of_day }) => {
                context.clock.set_time_of_day(minute_of_day);
                let snapshot = context.clock.snapshot();
                output.write(CommandOutput {
                    text: format!("时间已设置为 {:02}:{:02}", snapshot.hour, snapshot.minute),
                });
            }
            Ok(GameCommand::TimeScale { scale }) => {
                context.rules.time_scale = scale;
                let text = if scale == 0.0 {
                    "时间已暂停".to_owned()
                } else {
                    format!("时间流速已设置为 {scale} 倍")
                };
                output.write(CommandOutput { text });
            }
            Ok(GameCommand::GameModeSet { mode }) => {
                context.gamemode.mode = mode;
                let label = match mode {
                    GameMode::Survival => "生存",
                    GameMode::Creative => "创造",
                };
                output.write(CommandOutput {
                    text: format!("游戏模式已切换为 {label}模式"),
                });
            }
            Ok(GameCommand::Teleport { x, y, z }) => {
                let Ok((mut transform, mut velocity, mut gravity, _)) = context.player.single_mut()
                else {
                    output.write(CommandOutput {
                        text: "未找到本地玩家".to_owned(),
                    });
                    continue;
                };
                // 与重生同语义：清空水平速度与垂直状态，避免携带旧动量落地。
                transform.translation = Vec3::new(x, y, z);
                *velocity = PlayerVelocity::default();
                *gravity = PlayerGravity::default();
                output.write(CommandOutput {
                    text: format!("已传送到 ({x:.1}, {y:.1}, {z:.1})"),
                });
            }
            Ok(GameCommand::Give { item, count }) => {
                if !context.registry.contains(&item) {
                    output.write(CommandOutput {
                        text: format!("未知物品 {item}"),
                    });
                    continue;
                }
                let Ok((_, _, _, inventory)) = context.player.single_mut() else {
                    output.write(CommandOutput {
                        text: "未找到本地玩家".to_owned(),
                    });
                    continue;
                };
                // into_inner 取出原生引用，允许快捷栏与主背包的字段级分别借用。
                let inventory = inventory.into_inner();
                let stack = ItemStack::new(item.clone(), count);
                let text =
                    match insert_into_player(&mut inventory.hotbar, &mut inventory.survival, stack)
                    {
                        InventoryInsertResult::AllInserted => format!("已给予 {count} × {item}"),
                        InventoryInsertResult::Partial(remaining) => {
                            format!("背包空间不足，已给予 {} × {item}", count - remaining.count)
                        }
                        InventoryInsertResult::Full(_) => format!("背包已满，未能给予 {item}"),
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

/// 输出帮助反馈：无主题时逐行列出全部指令用法，有主题时输出单条用法。
fn write_help_output(topic: &Option<String>, output: &mut MessageWriter<CommandOutput>) {
    match topic {
        None => {
            output.write(CommandOutput {
                text: "可用指令（/help <指令> 查看用法）：".to_owned(),
            });
            for spec in COMMANDS {
                output.write(CommandOutput {
                    text: spec.usage.to_owned(),
                });
            }
        }
        // 解析阶段已校验主题存在，这里按规范名直接查找。
        Some(name) => {
            if let Some(spec) = COMMANDS.iter().find(|spec| spec.name == name) {
                output.write(CommandOutput {
                    text: spec.usage.to_owned(),
                });
            }
        }
    }
}
