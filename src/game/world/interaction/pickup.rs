use crate::game::inventory::events::InventoryFeedbackEvent;
use crate::game::inventory::interaction::transfer;
use crate::game::inventory::state::InventoryState;
use crate::game::player::identity::Player;
use crate::game::player::lifecycle::PlayerLifecycle;
use crate::game::world::entity::dropped_item::{DroppedItem, despawn_dropped_item};
use bevy::log::info;
use bevy::prelude::{Commands, Entity, Local, MessageWriter, Query, Res, Time, Transform, With};

/// 鎷惧彇鑼冨洿鍗婂緞
const PICKUP_RANGE: f32 = 2.0;

/// 鑷姩鎷惧彇绯荤粺
/// 姣忓抚妫€娴嬬帺瀹惰寖鍥村唴鎵€鏈夋帀钀界墿锛屽皾璇曟彃鍏ョ帺瀹惰儗鍖呫€?/// 鎴愬姛鍒欏垹闄ゆ帀钀界墿瀹炰綋锛屽け璐ュ垯淇濈暀鍓╀綑鐗╁搧
pub fn pickup_system(
    time: Res<Time>,
    mut player_query: Query<(&Transform, &PlayerLifecycle, &mut InventoryState), With<Player>>,
    mut item_query: Query<(Entity, &Transform, &mut DroppedItem)>,
    mut commands: Commands,
    mut feedback_writer: MessageWriter<InventoryFeedbackEvent>,
    mut full_feedback_cooldown: Local<f32>,
) {
    *full_feedback_cooldown = (*full_feedback_cooldown - time.delta_secs()).max(0.0);
    for (entity, item_transform, mut dropped) in &mut item_query {
        // 鍒氱敓鎴愮殑鎺夎惤鐗╁厛绛夊緟涓€灏忔鏃堕棿锛岄伩鍏嶇帺瀹舵寜 Q 鍚庨┈涓婂張鎹″洖鏉ャ€?
        if !dropped.can_pickup() {
            continue;
        }

        let Some((_, _, mut inventory)) = player_query
            .iter_mut()
            .filter(|(transform, lifecycle, _)| {
                lifecycle.is_alive()
                    && transform.translation.distance(item_transform.translation) <= PICKUP_RANGE
            })
            .min_by(|(left, _, _), (right, _, _)| {
                left.translation
                    .distance_squared(item_transform.translation)
                    .total_cmp(
                        &right
                            .translation
                            .distance_squared(item_transform.translation),
                    )
            })
        else {
            continue;
        };

        // 灏濊瘯鎻掑叆鑳屽寘锛堜紭鍏堝揩鎹锋爮锛屽啀鑳屽寘锛?        // 涓ゆ鎻掑叆閬垮厤鍚屾椂 borrow hotbar 鍜?survival
        let result = transfer::insert_into_container(&mut inventory.hotbar, dropped.stack.clone());
        let result = match result {
            transfer::InventoryInsertResult::AllInserted => result,
            transfer::InventoryInsertResult::Partial(remaining)
            | transfer::InventoryInsertResult::Full(remaining) => transfer::insert_into_range(
                &mut inventory.survival,
                remaining,
                0..crate::game::inventory::container::survival::SurvivalInventory::BACKPACK_SIZE,
            ),
        };

        match result {
            transfer::InventoryInsertResult::AllInserted => {
                info!("Picked up {:?}", dropped.stack);
                despawn_dropped_item(&mut commands, entity);
            }
            transfer::InventoryInsertResult::Partial(remaining) => {
                dropped.stack = remaining;
            }
            transfer::InventoryInsertResult::Full(_) => {
                // 婊¤浇鎻愮ず鍋氳妭娴侊紝閬垮厤鍚屼竴浠跺湴闈㈢墿鍝佹瘡甯ч噸澶嶆挱鏀炬彁绀恒€?
                if *full_feedback_cooldown <= 0.0 {
                    feedback_writer.write(InventoryFeedbackEvent::Full);
                    *full_feedback_cooldown = 1.25;
                }
            }
        }
    }
}
