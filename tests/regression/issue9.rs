use knocknoc::json;

#[test]
fn main() {
    let result = json::from_str::<bool>(" true && false ");
    assert!(result.is_err());
}
