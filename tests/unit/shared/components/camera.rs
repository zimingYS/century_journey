use super::*;

#[test]
fn perspective_cycles_first_second_third() {
    let first = CameraPerspective::FirstPerson;
    assert_eq!(first.next(), CameraPerspective::SecondPerson);
    assert_eq!(first.next().next(), CameraPerspective::ThirdPerson);
    assert_eq!(first.next().next().next(), first);
}
