use std::{
    fs::{self, File},
    io::{Cursor, Write},
    path::Path,
};

use base64_stream::FromBase64Writer;

const DATA_FOLDER: &str = "data";
const DECODE_OUTPUT: &str = "decode_output.txt";

#[test]
fn decode_write() {
    let base64 = b"SGkgdGhlcmUsIHRoaXMgaXMgYSBzaW1wbGUgc2VudGVuY2UgdXNlZCBmb3IgdGVzdGluZyB0aGlzIGNyYXRlLiBJIGhvcGUgYWxsIGNhc2VzIGFyZSBjb3JyZWN0Lg==".as_ref();

    let file_path = Path::new("tests").join(DATA_FOLDER).join(DECODE_OUTPUT);

    let test_data = File::create(file_path.as_path()).unwrap();

    let mut writer = FromBase64Writer::new(test_data);

    writer.write_all(base64).unwrap();

    writer.flush().unwrap(); // the flush method is only used when the full base64 data has been written

    assert_eq!(
        "Hi there, this is a simple sentence used for testing this crate. I hope all cases are \
         correct.",
        fs::read_to_string(file_path).unwrap()
    );
}

#[test]
fn decode_empty_write() {
    let mut writer = FromBase64Writer::new(Cursor::new(Vec::<u8>::new()));

    writer.flush().unwrap();

    let out = writer.into_inner().into_inner();

    assert!(out.is_empty());
}

#[test]
fn decode_one_byte_write() {
    let mut writer = FromBase64Writer::new(Cursor::new(Vec::<u8>::new()));

    writer.write_all(b"YQ==").unwrap();
    writer.flush().unwrap();

    let out = writer.into_inner().into_inner();

    assert_eq!(out, b"a");
}

#[test]
fn decode_two_bytes_write() {
    let mut writer = FromBase64Writer::new(Cursor::new(Vec::<u8>::new()));

    writer.write_all(b"YWI=").unwrap();
    writer.flush().unwrap();

    let out = writer.into_inner().into_inner();

    assert_eq!(out, b"ab");
}

#[test]
fn decode_large_write() {
    use base64_stream::base64::{Engine, engine::general_purpose::STANDARD};

    let plain: Vec<u8> = (0u8..=255).cycle().take(5000).collect();
    let encoded = STANDARD.encode(&plain);

    let mut writer = FromBase64Writer::new(Cursor::new(Vec::<u8>::new()));

    writer.write_all(encoded.as_bytes()).unwrap();
    writer.flush().unwrap();

    let out = writer.into_inner().into_inner();

    assert_eq!(out, plain);
}

#[test]
fn decode_invalid_write() {
    let mut writer = FromBase64Writer::new(Cursor::new(Vec::<u8>::new()));

    assert!(writer.write_all(b"!!!!").is_err());
}

#[test]
fn decode_into_inner_write() {
    let mut writer = FromBase64Writer::new(Cursor::new(Vec::<u8>::new()));

    writer.write_all(b"YQ==").unwrap();
    writer.flush().unwrap();

    let out = writer.into_inner().into_inner();

    assert_eq!(out, b"a");
}

#[test]
fn decode_small_buffer_write() {
    let mut writer = FromBase64Writer::<_, 4>::new2(Cursor::new(Vec::<u8>::new()));

    writer.write_all(b"YWJjZA==").unwrap();
    writer.flush().unwrap();

    let out = writer.into_inner().into_inner();

    assert_eq!(out, b"abcd");
}
