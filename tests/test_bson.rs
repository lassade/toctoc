use knocknoc::{Deserialize as KDeserialize, Serialize as KSerialize};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Deserialize, Serialize, KDeserialize, KSerialize)]
struct V {
    string: String,
    b: bool,
    int: i32,
}

/// May be used to deserialize primitive types
#[derive(Deserialize, Serialize)]
struct Primitive<T>{
    #[serde(rename = "")]
    val: T,
}

#[test]
fn test_bson_struct() {
    let v = V { string: "Hi!".to_owned(), b: false, int: 5, };

    let bin = knocknoc::bson::to_bin(&v, & ());

    let mut ground = vec![];
    bson::to_bson(&v).unwrap()
        .as_document().unwrap()
        .to_writer(&mut ground).unwrap();
        
    assert_eq!(bintext::hex::encode(&bin), bintext::hex::encode(&ground));

    let v1: V = bson::from_bson(
        bson::Document::from_reader(&mut &bin[..]).unwrap().into()
    ).unwrap();

    assert_eq!(v, v1);

    let v2 = knocknoc::bson::from_bin(&bin, &mut ()).unwrap();
    assert_eq!(v, v2);
}

macro_rules! test_primitive {
    ($p:expr, $t:ty) => { {
        let bin = knocknoc::bson::to_bin(&$p, & ());
    
        let mut ground = vec![];
        let mut doc = bson::Document::new();
        doc.entry("".to_string()).or_insert(bson::to_bson(&$p).unwrap());
        doc.to_writer(&mut ground).unwrap();
            
        assert_eq!(bintext::hex::encode(&bin), bintext::hex::encode(&ground));
    
        let doc = bson::Document::from_reader(&mut &bin[..]).unwrap().into();
        let v1: Primitive<$t> = bson::from_bson(doc).unwrap();
        assert_eq!($p, v1.val);
    } };
}

#[test]
fn test_bson_primitive() {
    test_primitive!(true, bool);
    test_primitive!(0i32, i32);
    test_primitive!("Hello World!".to_string(), String);
}
