//! 验证近战攻击输入只在存在合法目标时产生攻击事件。

use super::*;
use bevy::prelude::{App, IntoScheduleConfigs, ResMut, Resource, Update};

#[derive(Resource, Default)]
struct AttackEventCount(usize);

fn count_attack_events(
    mut reader: MessageReader<AttackEvent>,
    mut count: ResMut<AttackEventCount>,
) {
    count.0 += reader.read().count();
}

#[test]
fn empty_attack_does_not_emit_a_hit_event() {
    let mut app = App::new();
    app.init_resource::<PlayerActionState>()
        .init_resource::<AttackEventCount>()
        .add_message::<AttackEvent>()
        .add_systems(
            Update,
            (melee_attack_input_system, count_attack_events).chain(),
        );
    app.world_mut().spawn((
        Player,
        LocalPlayer,
        Transform::default(),
        PlayerLifecycle::default(),
    ));
    app.world_mut()
        .resource_mut::<PlayerActionState>()
        .update(true, [PlayerAction::Attack]);

    app.update();

    assert_eq!(app.world().resource::<AttackEventCount>().0, 0);
}
