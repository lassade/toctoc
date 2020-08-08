use knocknoc::{json, Deserialize, Serialize};

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
struct Message<'str> {
    sender: &'str str,
    text: &'str str,
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
struct MessageOwned {
    sender: String,
    text: String,
}

#[test]
fn test_zerocopy() {
    let mut j = r#"{ "sender": "you", "text": "hi!" }"#.to_string();
    let actual: Message = json::from_str(&mut j, &mut ()).unwrap();
    let expected = Message {
        sender: "you",
        text: "hi!",
    };
    assert_eq!(actual, expected);
}

#[test]
fn test_zerocopy_to_owned() {
    let j = r#"{ "sender": "you", "text": "hi!" }"#.to_string();
    let m = json::from_str_owned::<Message>(j, &mut ()).unwrap();

    let mo = unsafe { 
        let m = m.as_ref();
        MessageOwned {
            sender: m.sender.to_string(),
            text: m.text.to_string(),
        }
    };

    std::mem::drop(m);

    let expected = MessageOwned {
        sender: "you".to_string(),
        text: "hi!".to_string(),
    };

    assert_eq!(mo, expected);
}