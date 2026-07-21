use super::*;

#[test]
fn workbench_chest_and_furnace_receive_distinct_container_ids() {
    let mut containers = WorldContainers::default();
    let workbench = containers
        .ensure_at(IVec3::ZERO, ContainerKind::Workbench)
        .unwrap();
    let chest = containers
        .ensure_at(IVec3::X, ContainerKind::Chest)
        .unwrap();
    let furnace = containers
        .ensure_at(IVec3::Z, ContainerKind::Furnace)
        .unwrap();

    assert_ne!(workbench, chest);
    assert_ne!(workbench, furnace);
    assert_ne!(chest, furnace);
    assert_eq!(
        containers.get(workbench).unwrap().kind(),
        ContainerKind::Workbench
    );
    assert_eq!(containers.get(chest).unwrap().kind(), ContainerKind::Chest);
    assert_eq!(
        containers.get(furnace).unwrap().kind(),
        ContainerKind::Furnace
    );
    assert_eq!(
        containers.get(furnace).unwrap().slot_role(1),
        ContainerSlotRole::Fuel
    );
    assert_eq!(
        containers.get(furnace).unwrap().slot_role(2),
        ContainerSlotRole::Output
    );
}
