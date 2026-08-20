use super::*;

#[test]
fn default_uniform_has_valid_opacity_and_cell_size() {
    let uniform = CloudVolumeUniform::default();
    // 不透明度必须落在有效范围，避免云被完全隐去或过度覆盖。
    assert!((0.0..=1.0).contains(&uniform.tint_day.w));
    // 云体素尺寸必须为正，保证 DDA 网格化有效。
    assert!(uniform.cell_size > 0.0);
    // 密度阈值和能见度都必须是材质可安全使用的有限范围。
    assert!((0.0..=1.0).contains(&uniform.density_threshold));
    assert!((0.0..=1.0).contains(&uniform.visibility));
    // 云体上沿必须高于下沿。
    assert!(uniform.cloud_max_y > uniform.cloud_min_y);
}

#[test]
fn default_sun_direction_points_mostly_upward() {
    let sun = CloudVolumeUniform::default().sun_direction;
    // 太阳方向应主要朝上，且分量不为零（用于自遮蔽光照采样）。
    assert!(sun.y > 0.0);
    assert!(sun.x != 0.0 || sun.z != 0.0);
}
