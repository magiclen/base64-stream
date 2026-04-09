use std::{
    fs::{self, File},
    io::{Cursor, Write},
    path::Path,
};

use base64_stream::ToBase64Writer;

const DATA_FOLDER: &str = "data";
const ENCODE_OUTPUT: &str = "encode_output.txt";

#[test]
fn encode_write() {
    let test_data = b"Hi there, this is a simple sentence used for testing this crate. I hope all cases are correct.".as_ref();

    let file_path = Path::new("tests").join(DATA_FOLDER).join(ENCODE_OUTPUT);

    let base64 = File::create(file_path.as_path()).unwrap();

    let mut writer = ToBase64Writer::new(base64);

    writer.write_all(test_data).unwrap();

    writer.flush().unwrap(); // the flush method is only used when the full plain data has been written

    assert_eq!("SGkgdGhlcmUsIHRoaXMgaXMgYSBzaW1wbGUgc2VudGVuY2UgdXNlZCBmb3IgdGVzdGluZyB0aGlzIGNyYXRlLiBJIGhvcGUgYWxsIGNhc2VzIGFyZSBjb3JyZWN0Lg==", fs::read_to_string(file_path).unwrap());
}

#[test]
fn encode_empty_write() {
    let mut writer = ToBase64Writer::new(Cursor::new(Vec::<u8>::new()));

    writer.flush().unwrap();

    let out = writer.into_inner().into_inner();

    assert!(out.is_empty());
}

#[test]
fn encode_one_byte_write() {
    let mut writer = ToBase64Writer::new(Cursor::new(Vec::<u8>::new()));

    writer.write_all(b"a").unwrap();
    writer.flush().unwrap();

    let out = writer.into_inner().into_inner();

    assert_eq!(out, b"YQ==");
}

#[test]
fn encode_two_bytes_write() {
    let mut writer = ToBase64Writer::new(Cursor::new(Vec::<u8>::new()));

    writer.write_all(b"ab").unwrap();
    writer.flush().unwrap();

    let out = writer.into_inner().into_inner();

    assert_eq!(out, b"YWI=");
}

#[test]
fn encode_three_bytes_write() {
    let mut writer = ToBase64Writer::new(Cursor::new(Vec::<u8>::new()));

    writer.write_all(b"abc").unwrap();
    writer.flush().unwrap();

    let out = writer.into_inner().into_inner();

    assert_eq!(out, b"YWJj");
}

#[test]
fn encode_large_write() {
    use base64_stream::base64::{Engine, engine::general_purpose::STANDARD};

    let plain: Vec<u8> = (0u8..=255).cycle().take(5000).collect();
    let expected = STANDARD.encode(&plain);

    let mut writer = ToBase64Writer::new(Cursor::new(Vec::<u8>::new()));

    writer.write_all(&plain).unwrap();
    writer.flush().unwrap();

    let out = writer.into_inner().into_inner();

    assert_eq!(out, expected.as_bytes());
}

#[test]
fn encode_into_inner_write() {
    let mut writer = ToBase64Writer::new(Cursor::new(Vec::<u8>::new()));

    writer.write_all(b"abc").unwrap();
    writer.flush().unwrap();

    let out = writer.into_inner().into_inner();

    assert_eq!(out, b"YWJj");
}

#[test]
fn encode_small_buffer_write() {
    let mut writer = ToBase64Writer::<_, 4>::new2(Cursor::new(Vec::<u8>::new()));

    writer.write_all(b"abcd").unwrap();
    writer.flush().unwrap();

    let out = writer.into_inner().into_inner();

    assert_eq!(out, b"YWJjZA==");
}
