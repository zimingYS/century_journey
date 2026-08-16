//! 从 Blockbench 模型动画（`player_model.glb` 的 `animation.walk/run/fall`）提取的
//! 关键帧数据。
//!
//! 这些数据把模型自带动画的姿态固化下来，程序化动画按 `locomotion_phase` 采样，
//! 让游戏内 walk/run/fall 姿态与 Blockbench 里制作的动画一致，同时避免 armature
//! 骨骼动画带来的延伸问题（右腿缺失、行为动画冲突等）。
//!
//! - 旋转为局部四元数（x,y,z,w），可直接写入 armature=false 模型的同名 group 关节。
//! - body/head 的 translation 通道取其 y 分量（绝对高度，基准 1.5 与 group 初始 y 一致）。
//! - 模型未导出 `right_leg` 通道，右侧大腿按左腿相位 +0.5 反相生成，保证左右腿对称。

use bevy::math::Quat;

/// 旋转关键帧：归一化时间 + 局部旋转。
pub struct RotKey {
    pub time: f32,
    pub quat: Quat,
}

/// 身体高度关键帧：归一化时间 + 绝对 y。
pub struct YKey {
    pub time: f32,
    pub y: f32,
}

/// 在归一化时间 `time`（[0,1)，循环）采样旋转关键帧（相邻帧 slerp）。
pub fn sample_rot(keys: &[RotKey], time: f32) -> Quat {
    let t = time.rem_euclid(1.0).clamp(0.0, 1.0);
    let span = (keys.len() - 1) as f32;
    let scaled = t * span;
    let i = scaled.floor() as usize;
    let frac = scaled - i as f32;
    let a = &keys[i.min(keys.len() - 1)];
    let b = &keys[(i + 1) % keys.len()];
    a.quat.slerp(b.quat, frac)
}

/// 在归一化时间 `time`（[0,1)，循环）采样身体高度关键帧（线性插值）。
pub fn sample_y(keys: &[YKey], time: f32) -> f32 {
    let t = time.rem_euclid(1.0).clamp(0.0, 1.0);
    let span = (keys.len() - 1) as f32;
    let scaled = t * span;
    let i = scaled.floor() as usize;
    let frac = scaled - i as f32;
    let a = &keys[i.min(keys.len() - 1)];
    let b = &keys[(i + 1) % keys.len()];
    a.y + (b.y - a.y) * frac
}

/// WALK_LEFT_LEG 关键帧（时间归一化到 [0,1]，局部旋转四元数）。
pub const WALK_LEFT_LEG: &[RotKey] = &[
    RotKey {
        time: 0.0000,
        quat: Quat::from_xyzw(0.2588, 0.0000, 0.0000, 0.9659),
    },
    RotKey {
        time: 0.2500,
        quat: Quat::from_xyzw(0.0000, 0.0000, 0.0000, 1.0000),
    },
    RotKey {
        time: 0.5000,
        quat: Quat::from_xyzw(-0.2588, 0.0000, 0.0000, 0.9659),
    },
    RotKey {
        time: 0.7500,
        quat: Quat::from_xyzw(0.0000, 0.0000, 0.0000, 1.0000),
    },
    RotKey {
        time: 1.0000,
        quat: Quat::from_xyzw(0.2588, 0.0000, 0.0000, 0.9659),
    },
];

/// WALK_RIGHT_LEG 关键帧（时间归一化到 [0,1]，局部旋转四元数）。
pub const WALK_RIGHT_LEG: &[RotKey] = &[
    RotKey {
        time: 0.0000,
        quat: Quat::from_xyzw(-0.2588, 0.0000, 0.0000, 0.9659),
    },
    RotKey {
        time: 0.2500,
        quat: Quat::from_xyzw(0.0000, 0.0000, 0.0000, 1.0000),
    },
    RotKey {
        time: 0.5000,
        quat: Quat::from_xyzw(0.2588, 0.0000, 0.0000, 0.9659),
    },
    RotKey {
        time: 0.7500,
        quat: Quat::from_xyzw(0.0000, 0.0000, 0.0000, 1.0000),
    },
    RotKey {
        time: 1.0000,
        quat: Quat::from_xyzw(-0.2588, 0.0000, 0.0000, 0.9659),
    },
];

/// WALK_LEFT_CALF 关键帧（时间归一化到 [0,1]，局部旋转四元数）。
pub const WALK_LEFT_CALF: &[RotKey] = &[
    RotKey {
        time: 0.0000,
        quat: Quat::from_xyzw(-0.2588, 0.0000, 0.0000, 0.9659),
    },
    RotKey {
        time: 0.2500,
        quat: Quat::from_xyzw(0.0000, 0.0000, 0.0000, 1.0000),
    },
    RotKey {
        time: 0.5000,
        quat: Quat::from_xyzw(0.0000, 0.0000, 0.0000, 1.0000),
    },
    RotKey {
        time: 0.7500,
        quat: Quat::from_xyzw(0.0000, 0.0000, 0.0000, 1.0000),
    },
    RotKey {
        time: 1.0000,
        quat: Quat::from_xyzw(-0.2588, 0.0000, 0.0000, 0.9659),
    },
];

/// WALK_RIGHT_CALF 关键帧（时间归一化到 [0,1]，局部旋转四元数）。
pub const WALK_RIGHT_CALF: &[RotKey] = &[
    RotKey {
        time: 0.0000,
        quat: Quat::from_xyzw(0.0000, 0.0000, 0.0000, 1.0000),
    },
    RotKey {
        time: 0.2500,
        quat: Quat::from_xyzw(-0.1736, 0.0000, 0.0000, 0.9848),
    },
    RotKey {
        time: 0.5000,
        quat: Quat::from_xyzw(-0.2588, 0.0000, 0.0000, 0.9659),
    },
    RotKey {
        time: 0.7500,
        quat: Quat::from_xyzw(0.0000, 0.0000, 0.0000, 1.0000),
    },
    RotKey {
        time: 1.0000,
        quat: Quat::from_xyzw(0.0000, 0.0000, 0.0000, 1.0000),
    },
];

/// WALK_RIGHT_ARM 关键帧（时间归一化到 [0,1]，局部旋转四元数）。
pub const WALK_RIGHT_ARM: &[RotKey] = &[
    RotKey {
        time: 0.0000,
        quat: Quat::from_xyzw(0.2588, 0.0000, 0.0000, 0.9659),
    },
    RotKey {
        time: 0.2500,
        quat: Quat::from_xyzw(0.0000, 0.0000, 0.0000, 1.0000),
    },
    RotKey {
        time: 0.5000,
        quat: Quat::from_xyzw(-0.2588, 0.0000, 0.0000, 0.9659),
    },
    RotKey {
        time: 0.7500,
        quat: Quat::from_xyzw(0.0000, 0.0000, 0.0000, 1.0000),
    },
    RotKey {
        time: 1.0000,
        quat: Quat::from_xyzw(0.2588, 0.0000, 0.0000, 0.9659),
    },
];

/// WALK_LEFT_ARM 关键帧（时间归一化到 [0,1]，局部旋转四元数）。
pub const WALK_LEFT_ARM: &[RotKey] = &[
    RotKey {
        time: 0.0000,
        quat: Quat::from_xyzw(-0.2588, 0.0000, 0.0000, 0.9659),
    },
    RotKey {
        time: 0.2500,
        quat: Quat::from_xyzw(0.0000, 0.0000, 0.0000, 1.0000),
    },
    RotKey {
        time: 0.5000,
        quat: Quat::from_xyzw(0.2588, 0.0000, 0.0000, 0.9659),
    },
    RotKey {
        time: 0.7500,
        quat: Quat::from_xyzw(0.0000, 0.0000, 0.0000, 1.0000),
    },
    RotKey {
        time: 1.0000,
        quat: Quat::from_xyzw(-0.2588, 0.0000, 0.0000, 0.9659),
    },
];

/// WALK_BODY_Y 关键帧（时间归一化到 [0,1]，身体绝对 y）。
pub const WALK_BODY_Y: &[YKey] = &[
    YKey {
        time: 0.0000,
        y: 1.5000,
    },
    YKey {
        time: 0.2500,
        y: 1.5625,
    },
    YKey {
        time: 0.5000,
        y: 1.5000,
    },
    YKey {
        time: 0.7500,
        y: 1.5625,
    },
    YKey {
        time: 1.0000,
        y: 1.5000,
    },
];

/// RUN_LEFT_LEG 关键帧（时间归一化到 [0,1]，局部旋转四元数）。
pub const RUN_LEFT_LEG: &[RotKey] = &[
    RotKey {
        time: 0.0000,
        quat: Quat::from_xyzw(0.3827, 0.0000, 0.0000, 0.9239),
    },
    RotKey {
        time: 0.2500,
        quat: Quat::from_xyzw(0.0000, 0.0000, 0.0000, 1.0000),
    },
    RotKey {
        time: 0.5000,
        quat: Quat::from_xyzw(-0.3827, 0.0000, 0.0000, 0.9239),
    },
    RotKey {
        time: 0.7500,
        quat: Quat::from_xyzw(0.0000, 0.0000, 0.0000, 1.0000),
    },
    RotKey {
        time: 1.0000,
        quat: Quat::from_xyzw(0.3827, 0.0000, 0.0000, 0.9239),
    },
];

/// RUN_RIGHT_LEG 关键帧（时间归一化到 [0,1]，局部旋转四元数）。
pub const RUN_RIGHT_LEG: &[RotKey] = &[
    RotKey {
        time: 0.0000,
        quat: Quat::from_xyzw(-0.3827, 0.0000, 0.0000, 0.9239),
    },
    RotKey {
        time: 0.2500,
        quat: Quat::from_xyzw(0.0000, 0.0000, 0.0000, 1.0000),
    },
    RotKey {
        time: 0.5000,
        quat: Quat::from_xyzw(0.3827, 0.0000, 0.0000, 0.9239),
    },
    RotKey {
        time: 0.7500,
        quat: Quat::from_xyzw(0.0000, 0.0000, 0.0000, 1.0000),
    },
    RotKey {
        time: 1.0000,
        quat: Quat::from_xyzw(-0.3827, 0.0000, 0.0000, 0.9239),
    },
];

/// RUN_LEFT_CALF 关键帧（时间归一化到 [0,1]，局部旋转四元数）。
pub const RUN_LEFT_CALF: &[RotKey] = &[
    RotKey {
        time: 0.0000,
        quat: Quat::from_xyzw(-0.3827, 0.0000, 0.0000, 0.9239),
    },
    RotKey {
        time: 0.2500,
        quat: Quat::from_xyzw(-0.2588, 0.0000, 0.0000, 0.9659),
    },
    RotKey {
        time: 0.5000,
        quat: Quat::from_xyzw(0.0000, 0.0000, 0.0000, 1.0000),
    },
    RotKey {
        time: 0.7500,
        quat: Quat::from_xyzw(-0.2588, 0.0000, 0.0000, 0.9659),
    },
    RotKey {
        time: 1.0000,
        quat: Quat::from_xyzw(-0.3827, 0.0000, 0.0000, 0.9239),
    },
];

/// RUN_RIGHT_CALF 关键帧（时间归一化到 [0,1]，局部旋转四元数）。
pub const RUN_RIGHT_CALF: &[RotKey] = &[
    RotKey {
        time: 0.0000,
        quat: Quat::from_xyzw(0.0000, 0.0000, 0.0000, 1.0000),
    },
    RotKey {
        time: 0.2500,
        quat: Quat::from_xyzw(-0.2588, 0.0000, 0.0000, 0.9659),
    },
    RotKey {
        time: 0.5000,
        quat: Quat::from_xyzw(-0.3827, 0.0000, 0.0000, 0.9239),
    },
    RotKey {
        time: 0.7500,
        quat: Quat::from_xyzw(-0.2588, 0.0000, 0.0000, 0.9659),
    },
    RotKey {
        time: 1.0000,
        quat: Quat::from_xyzw(0.0000, 0.0000, 0.0000, 1.0000),
    },
];

/// RUN_RIGHT_ARM 关键帧（时间归一化到 [0,1]，局部旋转四元数）。
pub const RUN_RIGHT_ARM: &[RotKey] = &[
    RotKey {
        time: 0.0000,
        quat: Quat::from_xyzw(0.3420, 0.0000, 0.0000, 0.9397),
    },
    RotKey {
        time: 0.2500,
        quat: Quat::from_xyzw(0.0000, 0.0000, 0.0000, 1.0000),
    },
    RotKey {
        time: 0.5000,
        quat: Quat::from_xyzw(-0.3420, 0.0000, 0.0000, 0.9397),
    },
    RotKey {
        time: 0.7500,
        quat: Quat::from_xyzw(0.0000, 0.0000, 0.0000, 1.0000),
    },
    RotKey {
        time: 1.0000,
        quat: Quat::from_xyzw(0.3420, 0.0000, 0.0000, 0.9397),
    },
];

/// RUN_LEFT_ARM 关键帧（时间归一化到 [0,1]，局部旋转四元数）。
pub const RUN_LEFT_ARM: &[RotKey] = &[
    RotKey {
        time: 0.0000,
        quat: Quat::from_xyzw(-0.3420, 0.0000, 0.0000, 0.9397),
    },
    RotKey {
        time: 0.2500,
        quat: Quat::from_xyzw(0.0000, 0.0000, 0.0000, 1.0000),
    },
    RotKey {
        time: 0.5000,
        quat: Quat::from_xyzw(0.3420, 0.0000, 0.0000, 0.9397),
    },
    RotKey {
        time: 0.7500,
        quat: Quat::from_xyzw(0.0000, 0.0000, 0.0000, 1.0000),
    },
    RotKey {
        time: 1.0000,
        quat: Quat::from_xyzw(-0.3420, 0.0000, 0.0000, 0.9397),
    },
];

/// RUN_LEFT_FOREARM 关键帧（时间归一化到 [0,1]，局部旋转四元数）。
pub const RUN_LEFT_FOREARM: &[RotKey] = &[
    RotKey {
        time: 0.0000,
        quat: Quat::from_xyzw(0.4226, 0.0000, 0.0000, 0.9063),
    },
    RotKey {
        time: 1.0000,
        quat: Quat::from_xyzw(0.4226, 0.0000, 0.0000, 0.9063),
    },
];

/// RUN_RIGHT_FOREARM 关键帧（时间归一化到 [0,1]，局部旋转四元数）。
pub const RUN_RIGHT_FOREARM: &[RotKey] = &[
    RotKey {
        time: 0.0000,
        quat: Quat::from_xyzw(0.4226, 0.0000, 0.0000, 0.9063),
    },
    RotKey {
        time: 0.5000,
        quat: Quat::from_xyzw(0.4226, 0.0000, 0.0000, 0.9063),
    },
    RotKey {
        time: 1.0000,
        quat: Quat::from_xyzw(0.4226, 0.0000, 0.0000, 0.9063),
    },
];

/// RUN_BODY_Y 关键帧（时间归一化到 [0,1]，身体绝对 y）。
pub const RUN_BODY_Y: &[YKey] = &[
    YKey {
        time: 0.0000,
        y: 1.5000,
    },
    YKey {
        time: 0.2500,
        y: 1.5938,
    },
    YKey {
        time: 0.5000,
        y: 1.5000,
    },
    YKey {
        time: 0.7500,
        y: 1.5938,
    },
    YKey {
        time: 1.0000,
        y: 1.5000,
    },
];

/// FALL_LEFT_LEG 关键帧（时间归一化到 [0,1]，局部旋转四元数）。
pub const FALL_LEFT_LEG: &[RotKey] = &[
    RotKey {
        time: 0.0000,
        quat: Quat::from_xyzw(0.5736, 0.0000, 0.0000, 0.8192),
    },
    RotKey {
        time: 1.0000,
        quat: Quat::from_xyzw(0.5736, 0.0000, 0.0000, 0.8192),
    },
];

/// 模型未导出右侧大腿通道，坠落时保持自然伸直。
pub const FALL_RIGHT_LEG: &[RotKey] = &[
    RotKey {
        time: 0.0,
        quat: Quat::IDENTITY,
    },
    RotKey {
        time: 1.0,
        quat: Quat::IDENTITY,
    },
];

/// FALL_LEFT_CALF 关键帧（时间归一化到 [0,1]，局部旋转四元数）。
pub const FALL_LEFT_CALF: &[RotKey] = &[
    RotKey {
        time: 0.0000,
        quat: Quat::from_xyzw(0.0000, 0.0000, 0.0000, 1.0000),
    },
    RotKey {
        time: 1.0000,
        quat: Quat::from_xyzw(0.0000, 0.0000, 0.0000, 1.0000),
    },
];

/// FALL_RIGHT_CALF 关键帧（时间归一化到 [0,1]，局部旋转四元数）。
pub const FALL_RIGHT_CALF: &[RotKey] = &[
    RotKey {
        time: 0.0000,
        quat: Quat::from_xyzw(-0.5736, 0.0000, 0.0000, 0.8192),
    },
    RotKey {
        time: 1.0000,
        quat: Quat::from_xyzw(-0.5736, 0.0000, 0.0000, 0.8192),
    },
];

/// FALL_RIGHT_ARM 关键帧（时间归一化到 [0,1]，局部旋转四元数）。
pub const FALL_RIGHT_ARM: &[RotKey] = &[
    RotKey {
        time: 0.0000,
        quat: Quat::from_xyzw(0.0000, 0.0000, 0.8434, 0.5373),
    },
    RotKey {
        time: 1.0000,
        quat: Quat::from_xyzw(0.0000, 0.0000, 0.8434, 0.5373),
    },
];

/// FALL_LEFT_ARM 关键帧（时间归一化到 [0,1]，局部旋转四元数）。
pub const FALL_LEFT_ARM: &[RotKey] = &[
    RotKey {
        time: 0.0000,
        quat: Quat::from_xyzw(0.0000, 0.0000, -0.8434, 0.5373),
    },
    RotKey {
        time: 1.0000,
        quat: Quat::from_xyzw(0.0000, 0.0000, -0.8434, 0.5373),
    },
];

/// FALL_RIGHT_FOREARM 关键帧（时间归一化到 [0,1]，局部旋转四元数）。
pub const FALL_RIGHT_FOREARM: &[RotKey] = &[
    RotKey {
        time: 0.0000,
        quat: Quat::from_xyzw(0.0000, 0.0000, 0.0000, 1.0000),
    },
    RotKey {
        time: 1.0000,
        quat: Quat::from_xyzw(0.0000, 0.0000, 0.0000, 1.0000),
    },
];

/// FALL_LEFT_FOREARM 关键帧（时间归一化到 [0,1]，局部旋转四元数）。
pub const FALL_LEFT_FOREARM: &[RotKey] = &[
    RotKey {
        time: 0.0000,
        quat: Quat::from_xyzw(0.0000, 0.0000, 0.0000, 1.0000),
    },
    RotKey {
        time: 1.0000,
        quat: Quat::from_xyzw(0.0000, 0.0000, 0.0000, 1.0000),
    },
];
