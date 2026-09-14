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

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f32, b: f32) {
        assert!((a - b).abs() < 1e-5, "expected {b}, got {a}");
    }

    #[test]
    fn lerp_endpoints_and_midpoint() {
        approx(lerp(0.0, 10.0, 0.0), 0.0);
        approx(lerp(0.0, 10.0, 1.0), 10.0);
        approx(lerp(0.0, 10.0, 0.5), 5.0);
        approx(lerp(0.0, 10.0, 2.0), 20.0);
        approx(lerp(10.0, 0.0, 0.25), 7.5);
    }

    #[test]
    fn inverse_lerp_is_lerp_inverse() {
        for t in [0.0, 0.25, 0.5, 0.75, 1.0] {
            let v = lerp(3.0, 17.0, t);
            approx(inverse_lerp(3.0, 17.0, v), t);
        }
    }

    #[test]
    fn inverse_lerp_degenerate_range_returns_zero() {
        approx(inverse_lerp(5.0, 5.0, 5.0), 0.0);
        approx(inverse_lerp(5.0, 5.0, 100.0), 0.0);
        assert!(
            inverse_lerp(5.0, 5.0, 5.0).is_finite(),
            "degenerate range must not produce NaN"
        );
    }

    #[test]
    fn smoothstep_clamps_outside_range() {
        approx(smoothstep(0.0, 1.0, -5.0), 0.0);
        approx(smoothstep(0.0, 1.0, 5.0), 1.0);
        approx(smoothstep(0.0, 1.0, 0.0), 0.0);
        approx(smoothstep(0.0, 1.0, 1.0), 1.0);
    }

    #[test]
    fn smoothstep_is_symmetric_around_midpoint() {
        for d in [0.1, 0.2, 0.3, 0.4] {
            let a = smoothstep(0.0, 1.0, 0.5 - d);
            let b = smoothstep(0.0, 1.0, 0.5 + d);
            approx(a + b, 1.0);
        }
        approx(smoothstep(0.0, 1.0, 0.5), 0.5);
    }

    #[test]
    fn smoothstep_is_monotonic() {
        let mut prev = -1.0_f32;
        for i in 0..=100 {
            let v = smoothstep(0.0, 1.0, i as f32 / 100.0);
            assert!(v >= prev, "smoothstep decreased at step {i}");
            prev = v;
        }
    }
}
