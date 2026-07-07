use century_journey::content::tag::definition::TagAction;

#[test]
fn test_append_deser() {
    let json = r##"{"append": ["minecraft:stone", "#minecraft:stone_like"]}"##;
    let def: TagAction = serde_json::from_str(json).unwrap();
    match def {
        TagAction::Append { append } => {
            assert_eq!(append.len(), 2);
            assert_eq!(append[0], "minecraft:stone");
            assert_eq!(append[1], "#minecraft:stone_like");
        }
        _ => panic!("Expected Append"),
    }
}

#[test]
fn test_remove_deser() {
    let json = r#"{"remove": ["minecraft:stone"]}"#;
    let def: TagAction = serde_json::from_str(json).unwrap();
    match def {
        TagAction::Remove { remove } => assert_eq!(remove[0], "minecraft:stone"),
        _ => panic!("Expected Remove"),
    }
}

#[test]
fn test_replace_deser() {
    let json = r#"{"replace": ["minecraft:granite", "minecraft:diorite"]}"#;
    let def: TagAction = serde_json::from_str(json).unwrap();
    match def {
        TagAction::Replace { replace } => assert_eq!(replace.len(), 2),
        _ => panic!("Expected Replace"),
    }
}

#[test]
fn test_values_deser() {
    let json = r#"{"replace": false, "values": ["minecraft:stone"]}"#;
    let def: TagAction = serde_json::from_str(json).unwrap();
    match def {
        TagAction::Values { replace, values } => {
            assert!(!replace);
            assert_eq!(values, ["minecraft:stone"]);
        }
        _ => panic!("Expected Values"),
    }
}
