use serde::{Deserialize, Serialize};
use toctoc::bytes::Bytes;
use toctoc::{Deserialize as KDeserialize, Serialize as KSerialize};

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, KDeserialize, KSerialize)]
struct V {
    string: String,
    b: bool,
    int: i32,
}

#[derive(Clone, Debug, PartialEq, KDeserialize, KSerialize)]
struct MeshReadOnly<'a> {
    verts: Bytes<&'a [[u32; 2]]>,
}

/// May be used to deserialize primitive types
#[derive(Deserialize, Serialize)]
struct Primitive<T> {
    #[serde(rename = "")]
    val: T,
}

#[test]
fn test_bson_struct() {
    let v = V {
        string: "Hi!".to_owned(),
        b: false,
        int: 5,
    };

    let bin = toctoc::bson::to_bin(&v, &());

    let mut ground = vec![];
    bson::to_bson(&Primitive { val: v.clone() })
        .unwrap()
        .as_document()
        .unwrap()
        .to_writer(&mut ground)
        .unwrap();

    assert_eq!(bintext::hex::encode(&bin), bintext::hex::encode(&ground));

    let v1: Primitive<V> =
        bson::from_bson(bson::Document::from_reader(&mut &bin[..]).unwrap().into()).unwrap();

    assert_eq!(v, v1.val);

    let v2 = toctoc::bson::from_bin(&bin, &mut ()).unwrap();
    assert_eq!(v, v2);
}

macro_rules! test_primitive {
    ($p:expr, $t:ty) => {{
        let bin = toctoc::bson::to_bin(&$p, &());

        let mut ground = vec![];
        let mut doc = bson::Document::new();

        if cfg!(feature = "higher-rank-alignment") {
            doc.entry("algin".to_string())
                .or_insert(bson::to_bson(&4i32).unwrap());
        }

        doc.entry("".to_string())
            .or_insert(bson::to_bson(&$p).unwrap());
        doc.to_writer(&mut ground).unwrap();

        assert_eq!(bintext::hex::encode(&bin), bintext::hex::encode(&ground));

        let doc = bson::Document::from_reader(&mut &bin[..]).unwrap().into();
        let v1: Primitive<$t> = bson::from_bson(doc).unwrap();
        assert_eq!($p, v1.val);
    }};
}

#[test]
fn test_bson_primitive() {
    test_primitive!(true, bool);
    test_primitive!(0i32, i32);
    test_primitive!("Hello World!".to_string(), String);
}

#[test]
fn bson_zero_copy() {
    let m0 = MeshReadOnly {
        verts: Bytes::new(&[[0xAA55AA55, 0], [0, 0], [0, 0], [0, 0]][..]),
    };
    let bson = toctoc::bson::to_bin(&m0, &());
    assert_eq!(bson.as_ptr().align_offset(4), 0);
    let m1: MeshReadOnly = toctoc::bson::from_bin(&bson, &mut ()).unwrap();
    assert_eq!(m0, m1);
}
