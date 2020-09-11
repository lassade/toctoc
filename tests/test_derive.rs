use toctoc::{json, Deserialize, Serialize};

#[allow(dead_code)]
#[derive(PartialEq, Debug, Serialize, Deserialize)]
enum Tag {
    A,
    #[toctoc(rename = "renamedB")]
    B,
    C(i32),
    D(i32, i32),
    E {
        #[toctoc(skip_deserializing)]
        a: i32,
        #[toctoc(skip_serializing, rename = "bb")]
        b: i32,
    },
    #[toctoc(skip)]
    F {
        a: i32,
    },
}

#[derive(PartialEq, Debug, Serialize, Deserialize)]
struct Example {
    x: String,
    t1: Tag,
    t2: Tag,
    n: Nested,
}

#[derive(PartialEq, Debug, Serialize, Deserialize)]
struct Nested {
    y: Option<Vec<String>>,
    z: Option<String>,
}

#[test]
fn test_de() {
    let mut j = r#" {"x": "X", "t1": "A", "t2": "renamedB", "n": {"y": ["Y", "Y"]}} "#.to_string();
    let actual: Example = json::from_str(&mut j, &mut ()).unwrap();
    let expected = Example {
        x: "X".to_owned(),
        t1: Tag::A,
        t2: Tag::B,
        n: Nested {
            y: Some(vec!["Y".to_owned(), "Y".to_owned()]),
            z: None,
        },
    };
    assert_eq!(actual, expected);
}

#[test]
fn test_ser() {
    let example = Example {
        x: "X".to_owned(),
        t1: Tag::A,
        t2: Tag::B,
        n: Nested {
            y: Some(vec!["Y".to_owned(), "Y".to_owned()]),
            z: None,
        },
    };
    let actual = json::to_string(&example, &());
    let expected = r#"{"x":"X","t1":"A","t2":"renamedB","n":{"y":["Y","Y"],"z":null}}"#;
    assert_eq!(actual, expected);
}
