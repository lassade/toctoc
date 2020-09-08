use toctoc::json::{self, Value};

#[test]
#[cfg(feature = "deeply-nested")]
fn test_round_trip_deeply_nested() {
    let mut j = String::new();
    for _ in 0..100_000 {
        j.push_str("{\"x\":[");
    }
    for _ in 0..100_000 {
        j.push_str("]}");
    }

    let mut jc = j.clone();
    let value: Value = json::from_str(&mut jc, &mut ()).unwrap();
    let j2 = json::to_string(&value, &());
    assert_eq!(j, j2);
}
