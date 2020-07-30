use knocknoc::{json, Result};

#[test]
fn main() {
    let result: Result<bool> = json::from_str(" true && false ", &mut ());
    assert!(result.is_err());
}
