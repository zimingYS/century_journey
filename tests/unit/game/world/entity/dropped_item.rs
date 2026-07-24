use super::*;
use crate::game::inventory::item::stack::ItemInstanceData;
use crate::shared::item_id::ItemId;

#[test]
fn requested_fix_embedded_drop_is_not_lifted_to_block_top() {
    assert!(!crossed_ground_surface(4.5, 4.4, 5.06));
    assert!(crossed_ground_surface(5.2, 5.0, 5.06));
}

fn grounded_drop(item: ItemStack, position: Vec3) -> (DroppedItem, Transform) {
    let mut dropped = DroppedItem::new(item);
    dropped.grounded = true;
    (dropped, Transform::from_translation(position))
}

#[test]
fn nearby_drops_partially_merge_without_losing_overflow() {
    let item = ItemId::item("century_journey:apple");
    let mut app = App::new();
    app.add_systems(Update, dropped_item_merge_system);
    app.world_mut()
        .spawn(grounded_drop(ItemStack::new(item.clone(), 60), Vec3::ZERO));
    app.world_mut().spawn(grounded_drop(
        ItemStack::new(item, 10),
        Vec3::new(0.5, 0.0, 0.0),
    ));

    app.update();

    let mut counts: Vec<_> = app
        .world_mut()
        .query::<&DroppedItem>()
        .iter(app.world())
        .map(|dropped| dropped.stack.count)
        .collect();
    counts.sort_unstable();
    assert_eq!(counts, vec![6, 64]);
}

#[test]
fn three_nearby_drops_merge_to_the_minimum_stack_count() {
    let item = ItemId::item("century_journey:stick");
    let mut app = App::new();
    app.add_systems(Update, dropped_item_merge_system);
    for count in [40, 40, 40] {
        app.world_mut().spawn(grounded_drop(
            ItemStack::new(item.clone(), count),
            Vec3::ZERO,
        ));
    }

    app.update();

    let mut counts: Vec<_> = app
        .world_mut()
        .query::<&DroppedItem>()
        .iter(app.world())
        .map(|dropped| dropped.stack.count)
        .collect();
    counts.sort_unstable();
    assert_eq!(counts, vec![56, 64]);
}

#[test]
fn drops_with_different_instance_data_do_not_merge() {
    let item = ItemId::item("century_journey:wooden_pickaxe");
    let mut app = App::new();
    app.add_systems(Update, dropped_item_merge_system);
    app.world_mut().spawn(grounded_drop(
        ItemStack::with_instance(
            item.clone(),
            1,
            ItemInstanceData {
                durability: Some(10),
            },
        ),
        Vec3::ZERO,
    ));
    app.world_mut().spawn(grounded_drop(
        ItemStack::with_instance(
            item,
            1,
            ItemInstanceData {
                durability: Some(9),
            },
        ),
        Vec3::ZERO,
    ));

    app.update();

    assert_eq!(
        app.world_mut()
            .query::<&DroppedItem>()
            .iter(app.world())
            .count(),
        2
    );
}
