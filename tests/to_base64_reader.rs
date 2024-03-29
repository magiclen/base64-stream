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
