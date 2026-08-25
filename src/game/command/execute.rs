//! 在固定步内消费指令消息并修改权威玩法状态。

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::content::item::registry::ItemRegistry;
use crate::engine::localization::Localization;
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
    /// 本地化资源，指令反馈文案查表。
    localization: Res<'w, Localization>,
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
                write_help_output(&topic, &context.localization, &mut output);
            }
            Ok(GameCommand::Seed) => {
                output.write(CommandOutput {
                    text: context.localization.format(
                        "command.feedback.seed",
                        &[("seed", &context.generator.seed.to_string())],
                    ),
                });
            }
            Ok(GameCommand::TimeSet { minute_of_day }) => {
                context.clock.set_time_of_day(minute_of_day);
                let snapshot = context.clock.snapshot();
                output.write(CommandOutput {
                    text: context.localization.format(
                        "command.feedback.time-set",
                        &[
                            ("hour", &format!("{:02}", snapshot.hour)),
                            ("minute", &format!("{:02}", snapshot.minute)),
                        ],
                    ),
                });
            }
            Ok(GameCommand::TimeScale { scale }) => {
                context.rules.time_scale = scale;
                let text = if scale == 0.0 {
                    context
                        .localization
                        .get("command.feedback.time-paused")
                        .to_owned()
                } else {
                    context.localization.format(
                        "command.feedback.time-scale",
                        &[("scale", &format!("{scale}"))],
                    )
                };
                output.write(CommandOutput { text });
            }
            Ok(GameCommand::GameModeSet { mode }) => {
                context.gamemode.mode = mode;
                let label = match mode {
                    GameMode::Survival => context.localization.get("gamemode.survival"),
                    GameMode::Creative => context.localization.get("gamemode.creative"),
                };
                output.write(CommandOutput {
                    text: context
                        .localization
                        .format("command.feedback.gamemode", &[("mode", label)]),
                });
            }
            Ok(GameCommand::Teleport { x, y, z }) => {
                let Ok((mut transform, mut velocity, mut gravity, _)) = context.player.single_mut()
                else {
                    output.write(CommandOutput {
                        text: context
                            .localization
                            .get("command.feedback.no-player")
                            .to_owned(),
                    });
                    continue;
                };
                // 与重生同语义：清空水平速度与垂直状态，避免携带旧动量落地。
                transform.translation = Vec3::new(x, y, z);
                *velocity = PlayerVelocity::default();
                *gravity = PlayerGravity::default();
                output.write(CommandOutput {
                    text: context.localization.format(
                        "command.feedback.teleported",
                        &[
                            ("x", &format!("{x:.1}")),
                            ("y", &format!("{y:.1}")),
                            ("z", &format!("{z:.1}")),
                        ],
                    ),
                });
            }
            Ok(GameCommand::Give { item, count }) => {
                if !context.registry.contains(&item) {
                    output.write(CommandOutput {
                        text: context.localization.format(
                            "command.feedback.unknown-item",
                            &[("item", &item.to_string())],
                        ),
                    });
                    continue;
                }
                let Ok((_, _, _, inventory)) = context.player.single_mut() else {
                    output.write(CommandOutput {
                        text: context
                            .localization
                            .get("command.feedback.no-player")
                            .to_owned(),
                    });
                    continue;
                };
                // into_inner 取出原生引用，允许快捷栏与主背包的字段级分别借用。
                let inventory = inventory.into_inner();
                let stack = ItemStack::new(item.clone(), count);
                let text =
                    match insert_into_player(&mut inventory.hotbar, &mut inventory.survival, stack)
                    {
                        InventoryInsertResult::AllInserted => context.localization.format(
                            "command.feedback.give-inserted",
                            &[("count", &count.to_string()), ("item", &item.to_string())],
                        ),
                        InventoryInsertResult::Partial(remaining) => context.localization.format(
                            "command.feedback.give-partial",
                            &[
                                ("count", &(count - remaining.count).to_string()),
                                ("item", &item.to_string()),
                            ],
                        ),
                        InventoryInsertResult::Full(_) => context
                            .localization
                            .format("command.feedback.give-full", &[("item", &item.to_string())]),
                    };
                output.write(CommandOutput { text });
            }
            Err(error) => {
                output.write(CommandOutput {
                    text: error.to_feedback(&context.localization),
                });
            }
        }
    }
}

/// 输出帮助反馈：无主题时逐行列出全部指令用法，有主题时输出单条用法。
fn write_help_output(
    topic: &Option<String>,
    localization: &Localization,
    output: &mut MessageWriter<CommandOutput>,
) {
    match topic {
        None => {
            output.write(CommandOutput {
                text: localization.get("command.feedback.help-header").to_owned(),
            });
            for spec in COMMANDS {
                output.write(CommandOutput {
                    text: localization.get(spec.usage).to_owned(),
                });
            }
        }
        // 解析阶段已校验主题存在，这里按规范名直接查找。
        Some(name) => {
            if let Some(spec) = COMMANDS.iter().find(|spec| spec.name == name) {
                output.write(CommandOutput {
                    text: localization.get(spec.usage).to_owned(),
                });
            }
        }
    }
}
