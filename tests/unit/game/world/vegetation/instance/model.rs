use super::*;

fn oak() -> Identifier {
    Identifier::new("century_journey", "oak")
}

#[test]
fn mature_instance_derives_age_and_negative_owner_chunk() {
    let instance = TreeInstance::new_mature(IVec3::new(-1, 31, -17), oak(), 42, 120);

    assert_eq!(instance.owner_chunk(), IVec3::new(-1, 1, -2));
    assert_eq!(150 - instance.born_at_game_minute(), 30);
    assert_eq!(instance.stage(), TreeGrowthStage::Mature);
    assert_eq!(instance.health(), FULL_TREE_HEALTH);
}

#[test]
fn persisted_instance_rejects_reversed_simulation_times() {
    let result = TreeInstance::from_persisted(
        IVec3::ZERO,
        oak(),
        7,
        TreeGrowthStage::Mature,
        100,
        90,
        FULL_TREE_HEALTH,
        100,
        None,
    );

    assert!(result.is_err());
}

#[test]
fn sapling_schedule_advances_through_young_to_mature() {
    let mut instance = TreeInstance::new_sapling(IVec3::ZERO, oak(), 9, 100, 20);

    assert_eq!(instance.stage(), TreeGrowthStage::Sapling);
    assert!(!instance.is_due(119));
    assert!(instance.is_due(120));

    instance.transition_to_young(120, 30).unwrap();
    assert_eq!(instance.stage(), TreeGrowthStage::Young);
    assert_eq!(instance.stage_started_at_game_minute(), 120);
    assert_eq!(instance.next_update_game_minute(), Some(150));

    instance.defer_update(150, 5).unwrap();
    assert_eq!(instance.last_simulated_game_minute(), 150);
    assert_eq!(instance.next_update_game_minute(), Some(155));

    instance.transition_to_mature(155).unwrap();
    assert_eq!(instance.stage(), TreeGrowthStage::Mature);
    assert_eq!(instance.next_update_game_minute(), None);
    assert!(!instance.is_due(u64::MAX));
}

#[test]
fn persisted_schedule_is_normalized_for_each_stage() {
    let sapling = TreeInstance::from_persisted(
        IVec3::ZERO,
        oak(),
        1,
        TreeGrowthStage::Sapling,
        100,
        100,
        FULL_TREE_HEALTH,
        120,
        None,
    )
    .unwrap();
    let mature = TreeInstance::from_persisted(
        IVec3::ZERO,
        oak(),
        1,
        TreeGrowthStage::Mature,
        100,
        100,
        FULL_TREE_HEALTH,
        120,
        Some(180),
    )
    .unwrap();

    assert_eq!(sapling.next_update_game_minute(), Some(120));
    assert_eq!(mature.next_update_game_minute(), None);
}
