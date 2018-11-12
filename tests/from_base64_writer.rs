extern crate base64_stream;

use std::io::Write;

use std::fs::{self, File};

use base64_stream::FromBase64Writer;

#[test]
fn decode_write() {
    let base64 = b"SGkgdGhlcmUsIHRoaXMgaXMgYSBzaW1wbGUgc2VudGVuY2UgdXNlZCBmb3IgdGVzdGluZyB0aGlzIGNyYXRlLiBJIGhvcGUgYWxsIGNhc2VzIGFyZSBjb3JyZWN0Lg==".as_ref();

    let test_data = File::create("tests/data/decode_output.txt").unwrap();

    let mut writer = FromBase64Writer::new(test_data);

    writer.write(base64).unwrap();

    writer.flush().unwrap(); // the flush method is only used when the full base64 data has been written

    assert_eq!("Hi there, this is a simple sentence used for testing this crate. I hope all cases are correct.", fs::read_to_string("tests/data/decode_output.txt").unwrap());
}