use bevy::prelude::*;

// 当UI打开时设置此资源,表示玩家输入应被阻断
#[derive(Resource, Default, Debug)]
pub struct InputBlocked(pub bool);