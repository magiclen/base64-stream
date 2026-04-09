use std::io::{Cursor, Read};

use base64_stream::FromBase64Reader;

#[test]
fn decode_exact() {
    let base64 = b"SGkgdGhlcmUsIHRoaXMgaXMgYSBzaW1wbGUgc2VudGVuY2UgdXNlZCBmb3IgdGVzdGluZyB0aGlzIGNyYXRlLiBJIGhvcGUgYWxsIGNhc2VzIGFyZSBjb3JyZWN0Lg==".to_vec();

    let mut reader = FromBase64Reader::new(Cursor::new(base64));

    let mut test_data = [0u8; 94];

    reader.read_exact(&mut test_data).unwrap();

    assert_eq!(
        "Hi there, this is a simple sentence used for testing this crate. I hope all cases are \
         correct."
            .as_bytes(),
        test_data.as_ref()
    );
}

#[test]
fn decode_to_end() {
    let base64 = b"SGkgdGhlcmUsIHRoaXMgaXMgYSBzaW1wbGUgc2VudGVuY2UgdXNlZCBmb3IgdGVzdGluZyB0aGlzIGNyYXRlLiBJIGhvcGUgYWxsIGNhc2VzIGFyZSBjb3JyZWN0Lg==".to_vec();

    let mut reader = FromBase64Reader::new(Cursor::new(base64));

    let mut test_data = Vec::new();

    reader.read_to_end(&mut test_data).unwrap();

    assert_eq!(b"Hi there, this is a simple sentence used for testing this crate. I hope all cases are correct.".to_vec(), test_data);
}

#[test]
fn decode_to_string() {
    let base64 = b"SGkgdGhlcmUsIHRoaXMgaXMgYSBzaW1wbGUgc2VudGVuY2UgdXNlZCBmb3IgdGVzdGluZyB0aGlzIGNyYXRlLiBJIGhvcGUgYWxsIGNhc2VzIGFyZSBjb3JyZWN0Lg==".to_vec();

    let mut reader = FromBase64Reader::new(Cursor::new(base64));

    let mut test_data = String::new();

    reader.read_to_string(&mut test_data).unwrap();

    assert_eq!(
        "Hi there, this is a simple sentence used for testing this crate. I hope all cases are \
         correct.",
        test_data
    );
}

#[test]
fn decode_empty() {
    let mut reader = FromBase64Reader::new(Cursor::new(b"" as &[u8]));

    let mut out = Vec::new();

    reader.read_to_end(&mut out).unwrap();

    assert!(out.is_empty());
}

#[test]
fn decode_one_byte() {
    let mut reader = FromBase64Reader::new(Cursor::new(b"YQ==" as &[u8]));

    let mut out = Vec::new();

    reader.read_to_end(&mut out).unwrap();

    assert_eq!(out, b"a");
}

#[test]
fn decode_two_bytes() {
    let mut reader = FromBase64Reader::new(Cursor::new(b"YWI=" as &[u8]));

    let mut out = Vec::new();

    reader.read_to_end(&mut out).unwrap();

    assert_eq!(out, b"ab");
}

#[test]
fn decode_three_bytes() {
    let mut reader = FromBase64Reader::new(Cursor::new(b"YWJj" as &[u8]));

    let mut out = Vec::new();

    reader.read_to_end(&mut out).unwrap();

    assert_eq!(out, b"abc");
}

#[test]
fn decode_large() {
    use base64_stream::base64::{Engine, engine::general_purpose::STANDARD};

    let plain: Vec<u8> = (0u8..=255).cycle().take(5000).collect();
    let encoded = STANDARD.encode(&plain);

    let mut reader = FromBase64Reader::new(Cursor::new(encoded.into_bytes()));
    let mut out = Vec::new();

    reader.read_to_end(&mut out).unwrap();

    assert_eq!(out, plain);
}

#[test]
fn decode_invalid_base64() {
    let mut reader = FromBase64Reader::new(Cursor::new(b"!!!!" as &[u8]));

    let mut out = Vec::new();

    assert!(reader.read_to_end(&mut out).is_err());
}

#[test]
fn decode_into_inner() {
    let cursor = Cursor::new(b"YQ==".to_vec());
    let reader = FromBase64Reader::new(cursor);
    let inner = reader.into_inner();

    assert_eq!(inner.into_inner(), b"YQ==");
}

#[test]
fn decode_small_buffer() {
    let mut reader = FromBase64Reader::<_, 4>::new2(Cursor::new(b"YWJjZA==" as &[u8]));

    let mut out = Vec::new();

    reader.read_to_end(&mut out).unwrap();

    assert_eq!(out, b"abcd");
}
