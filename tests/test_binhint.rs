use toctoc::bytes::Bytes;
use toctoc::json;

macro_rules! bin {
    ($ty:ty, $align:expr, $bytes:expr, $string:expr) => {{
        let bytes = $bytes;
        let string = $string;
        let align = $align;

        let actual = json::to_string(&bytes, &mut ());
        assert_eq!(actual, $string);

        let json = &mut string.to_string();
        let actual: Bytes<$ty> = json::from_str(json, &mut ()).unwrap();
        assert_eq!(actual.0.as_ptr().align_offset(align), 0);
        assert_eq!(actual.0, &bytes.0[..]);
    }};
}

#[test]
fn test_binhint() {
    bin!(&[u8], 1, Bytes::new(vec![2u8, 0, 3, 4]), "\"#02000304\"");
    bin!(
        &[u32],
        4,
        Bytes::new(vec![0x02000304_u32]),
        "\"#----04030002\""
    );
}
