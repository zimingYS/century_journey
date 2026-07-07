use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// 方块音效材质分类（决定破坏/放置/脚步声的音色）
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
pub enum SoundMaterial {
    /// 石头类
    #[default]
    Stone,
    /// 泥土类
    Dirt,
    /// 草地类
    Grass,
    /// 木头类
    Wood,
    /// 沙子类
    Sand,
    /// 金属类
    Metal,
    /// 玻璃类
    Glass,
    /// 布料类
    Cloth,
    /// 雪
    Snow,
    /// 水
    Water,
}

/// 方块音效配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockSoundConfig {
    /// 音效材质
    pub sound_material: SoundMaterial,
    /// 破坏音量 (0.0~1.0)
    pub break_volume: f32,
    /// 放置音量 (0.0~1.0)
    pub place_volume: f32,
    /// 脚步音量 (0.0~1.0)
    pub step_volume: f32,
}

impl Default for BlockSoundConfig {
    fn default() -> Self {
        Self {
            sound_material: SoundMaterial::Stone,
            break_volume: 1.0,
            place_volume: 1.0,
            step_volume: 0.6,
        }
    }
}

/// 音效事件（由方块交互系统发送，由音频系统监听）
#[derive(Message)]
pub struct BlockSoundEvent {
    /// 播放位置
    pub position: Vec3,
    /// 音效材质
    pub sound_material: SoundMaterial,
    /// 音效动作类型
    pub action: SoundAction,
    /// 音量
    pub volume: f32,
}

/// 音效动作
#[derive(Debug, Clone, Copy)]
pub enum SoundAction {
    /// 破坏方块
    Break,
    /// 放置方块
    Place,
    /// 踩上方块
    Step,
    /// 挖掘方块
    Dig,
    /// 落地时踩到方块
    FallOn,
    /// 与方块交互
    Interact,
    /// 打开门
    Open,
    /// 关闭门
    Close,
    /// 重置
    Reset,
    /// 植物生长/方块自然更新
    Grow,
    /// 点燃方块
    Ignite,
    /// 方块被熄灭
    Extinguish,
    /// 流体流动
    Flow,
}
