use bevy::prelude::*;
use crate::game::player::view_model::{ViewModelRoot, ViewModelAnimator, ViewAnimation};

/// 动画系统
pub fn view_model_animation_system(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut ViewModelAnimator), With<ViewModelRoot>>,
) {
    let dt = time.delta_secs();
    for (mut transform, mut anim) in &mut query {
        // ── Equip ──
        if anim.equip_progress < 1.0 {
            anim.equip_progress = (anim.equip_progress + dt * 4.0).min(1.0);
        }
        let equip_t = smoothstep(anim.equip_progress);

        // ── Swing ──
        if anim.swing_progress > 0.0 {
            anim.swing_progress = (anim.swing_progress - dt * 4.0).max(0.0);
        }

        // ── Idle sway ──
        anim.idle_phase += dt * 1.5;
        let sway_x = (anim.idle_phase.sin() * 0.012) as f32;
        let sway_y = (anim.idle_phase.cos() * 0.008) as f32;

        // ── 组合 ──
        let swing_rot = if anim.swing_progress > 0.0 {
            Quat::from_rotation_x((anim.swing_progress * -0.65).to_radians())
        } else {
            Quat::IDENTITY
        };
        let swing_trans = Vec3::new(0.0, anim.swing_progress * -0.12, 0.0);

        let equip_trans = Vec3::new(0.0, (1.0 - equip_t) * -0.25, 0.0);
        let equip_rot = Quat::from_rotation_x(((1.0 - equip_t) * -25_f32).to_radians());

        let idle_rot = Quat::from_rotation_z(sway_x) * Quat::from_rotation_x(sway_y);

        transform.rotation = swing_rot * equip_rot * idle_rot;
        transform.translation += swing_trans + equip_trans;
    }
}

/// 触发挥动动画
pub fn trigger_swing(
    query: &mut Query<&mut ViewModelAnimator>,
    _anim: ViewAnimation,
) {
    for mut anim in query.iter_mut() {
        anim.swing_progress = 1.0;
    }
}

/// 触发装备动画
pub fn trigger_equip(query: &mut Query<&mut ViewModelAnimator>) {
    for mut anim in query.iter_mut() {
        anim.equip_progress = 0.0;
    }
}

fn smoothstep(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
