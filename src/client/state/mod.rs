use bevy::prelude::*;

/// 当 UI 打开时设置此资源，表示玩家输入应被阻断。
/// 由 UI 层设置，Player/Camera 层读取。
///
/// TODO: 后续使用 Input Intent / Command 解耦 Game 层对 Client 层状态的直接依赖。
#[derive(Resource, Default, Debug)]
pub struct InputBlocked(pub bool);
