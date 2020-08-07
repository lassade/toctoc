use knocknoc::{json, Deserialize, Serialize};

#[derive(PartialEq, Debug, Serialize, Deserialize)]
struct Message<'str> {
    sender: &'str str,
    text: &'str str,
}

#[test]
fn test_zero_copy() {
    let mut j = r#" { "sender": "you", "text": "hi!" }"#.to_string();
    let actual: Message = json::from_str(&mut j, &mut ()).unwrap();
    let expected = Message {
        sender: "you",
        text: "hi!",
    };
    assert_eq!(actual, expected);
}
