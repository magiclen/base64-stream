extern crate base64_stream;

use std::io::{Cursor, Read};

use base64_stream::FromBase64Reader;

#[test]
fn decode_reader() {
    let base64 = b"SGkgdGhlcmUsIHRoaXMgaXMgYSBzaW1wbGUgc2VudGVuY2UgdXNlZCBmb3IgdGVzdGluZyB0aGlzIGNyYXRlLiBJIGhvcGUgYWxsIGNhc2VzIGFyZSBjb3JyZWN0Lg==".to_vec();

    let mut reader = FromBase64Reader::new(Cursor::new(base64));

    let mut test_data = [0u8; 4096];

    let c = reader.read(&mut test_data).unwrap();

    assert_eq!(b"Hi there, this is a simple sentence used for testing this crate. I hope all cases are correct.".to_vec(), test_data[..c].to_vec());
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

    assert_eq!("Hi there, this is a simple sentence used for testing this crate. I hope all cases are correct.", test_data);
}
