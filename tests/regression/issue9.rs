use knocknoc::{json, Result};

#[test]
fn main() {
    let mut j = " true && false ".to_string();
    let result: Result<bool> = json::from_str(&mut j, &mut ());
    assert!(result.is_err());
}
