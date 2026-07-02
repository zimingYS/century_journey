use bevy::prelude::*;

/// 当 UI 打开时设置此资源，表示玩家输入应被阻断。
/// 由 UI 层（Client）设置，Player/Camera 层（Game）读取。
///
/// 放在 Shared 层以便 Game 和 Client 层都能访问，避免层间反向依赖。
#[derive(Resource, Default, Debug)]
pub struct InputBlocked(pub bool);
