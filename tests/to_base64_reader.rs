use std::io::{Cursor, Read};

use base64_stream::ToBase64Reader;

#[test]
fn encode_exact() {
    let test_data = b"Hi there, this is a simple sentence used for testing this crate. I hope all cases are correct.".to_vec();

    let mut reader = ToBase64Reader::new(Cursor::new(test_data));

    let mut base64 = [0u8; 128];

    reader.read_exact(&mut base64).unwrap();

    assert_eq!("SGkgdGhlcmUsIHRoaXMgaXMgYSBzaW1wbGUgc2VudGVuY2UgdXNlZCBmb3IgdGVzdGluZyB0aGlzIGNyYXRlLiBJIGhvcGUgYWxsIGNhc2VzIGFyZSBjb3JyZWN0Lg==".as_bytes(), base64.as_ref());
}

#[test]
fn encode_read_to_end() {
    let test_data = b"Hi there, this is a simple sentence used for testing this crate. I hope all cases are correct.".to_vec();

    let mut reader = ToBase64Reader::new(Cursor::new(test_data));

    let mut base64 = Vec::new();

    reader.read_to_end(&mut base64).unwrap();

    assert_eq!(b"SGkgdGhlcmUsIHRoaXMgaXMgYSBzaW1wbGUgc2VudGVuY2UgdXNlZCBmb3IgdGVzdGluZyB0aGlzIGNyYXRlLiBJIGhvcGUgYWxsIGNhc2VzIGFyZSBjb3JyZWN0Lg==".to_vec(), base64);
}

#[test]
fn encode_read_to_string() {
    let test_data = b"Hi there, this is a simple sentence used for testing this crate. I hope all cases are correct.".to_vec();

    let mut reader = ToBase64Reader::new(Cursor::new(test_data));

    let mut base64_string = String::new();

    reader.read_to_string(&mut base64_string).unwrap();

    assert_eq!("SGkgdGhlcmUsIHRoaXMgaXMgYSBzaW1wbGUgc2VudGVuY2UgdXNlZCBmb3IgdGVzdGluZyB0aGlzIGNyYXRlLiBJIGhvcGUgYWxsIGNhc2VzIGFyZSBjb3JyZWN0Lg==", base64_string);
}

#[test]
fn encode_empty() {
    let mut reader = ToBase64Reader::new(Cursor::new(b"" as &[u8]));

    let mut out = Vec::new();

    reader.read_to_end(&mut out).unwrap();

    assert!(out.is_empty());
}

#[test]
fn encode_one_byte() {
    let mut reader = ToBase64Reader::new(Cursor::new(b"a" as &[u8]));

    let mut out = Vec::new();

    reader.read_to_end(&mut out).unwrap();

    assert_eq!(out, b"YQ==");
}

#[test]
fn encode_two_bytes() {
    let mut reader = ToBase64Reader::new(Cursor::new(b"ab" as &[u8]));

    let mut out = Vec::new();

    reader.read_to_end(&mut out).unwrap();

    assert_eq!(out, b"YWI=");
}

#[test]
fn encode_three_bytes() {
    let mut reader = ToBase64Reader::new(Cursor::new(b"abc" as &[u8]));

    let mut out = Vec::new();

    reader.read_to_end(&mut out).unwrap();

    assert_eq!(out, b"YWJj");
}

#[test]
fn encode_large() {
    use base64_stream::base64::{Engine, engine::general_purpose::STANDARD};

    let plain: Vec<u8> = (0u8..=255).cycle().take(5000).collect();
    let expected = STANDARD.encode(&plain);

    let mut reader = ToBase64Reader::new(Cursor::new(plain));
    let mut out = Vec::new();

    reader.read_to_end(&mut out).unwrap();

    assert_eq!(out, expected.as_bytes());
}

#[test]
fn encode_into_inner() {
    let cursor = Cursor::new(b"abc".to_vec());
    let reader = ToBase64Reader::new(cursor);
    let inner = reader.into_inner();

    assert_eq!(inner.into_inner(), b"abc");
}

#[test]
fn encode_small_buffer() {
    let mut reader = ToBase64Reader::<_, 4>::new2(Cursor::new(b"abcd" as &[u8]));

    let mut out = Vec::new();

    reader.read_to_end(&mut out).unwrap();

    assert_eq!(out, b"YWJjZA==");
}
