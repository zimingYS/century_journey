//! 基础数学工具

/// 线性插值
#[inline]
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// 反向插值：把 v 在 [a,b] 中的位置映射到 [0,1]
#[inline]
pub fn inverse_lerp(a: f32, b: f32, v: f32) -> f32 {
    if (b - a).abs() < f32::EPSILON {
        0.0
    } else {
        (v - a) / (b - a)
    }
}

/// 平滑阶跃
#[inline]
pub fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = inverse_lerp(edge0, edge1, x).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
