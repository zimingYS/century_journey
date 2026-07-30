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
