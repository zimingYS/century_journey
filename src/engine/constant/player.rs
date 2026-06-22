/// 玩家碰撞箱半尺寸（半宽、半高、半深）
/// 标准玩家碰撞箱：宽 0.6、深 0.6、高 1.8
pub const PLAYER_HALF_WIDTH: f32 = 0.3;
pub const PLAYER_HALF_HEIGHT: f32 = 0.9;
pub const PLAYER_HALF_DEPTH: f32 = 0.3;

/// 重力加速度（世界单位/秒²）
pub const GRAVITY: f32 = -24.0;

/// 玩家最大下落速度（世界单位/秒）
pub const MAX_FALL_SPEED: f32 = -60.0;

/// 玩家跳跃力（世界单位/秒）
pub const JUMP_FORCE: f32 = 4.5;

/// 玩家基础移动速度（世界单位/秒）
pub const PLAYER_MOVEMENT_SPEED: f32 = 15.0;

/// 玩家冲刺速度倍率
pub const PLAYER_SPRINT_FACTOR: f32 = 1.5;

/// 玩家可踩踏的最大台阶高度（世界单位）
pub const STEP_HEIGHT: f32 = 0.6;
