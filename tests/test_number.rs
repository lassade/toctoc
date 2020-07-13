use knocknoc::json;
use std::f64;

#[test]
fn test_ser() {
    let cases = &[
        (1.0, "1.0"),
        (f64::NAN, "null"),
        (f64::INFINITY, "null"),
        (f64::NEG_INFINITY, "null"),
    ];

    for (number, expected) in cases {
        let actual = json::to_string(number, None);
        assert_eq!(actual, *expected);
    }
}
