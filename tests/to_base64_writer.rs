extern crate base64_stream;

use std::io::Write;

use std::fs::{self, File};

use base64_stream::ToBase64Writer;

#[test]
fn encode_write() {
    let test_data = b"Hi there, this is a simple sentence used for testing this crate. I hope all cases are correct.".as_ref();

    let base64 = File::create("tests/data/encode_output.txt").unwrap();

    let mut writer = ToBase64Writer::new(base64);

    writer.write(test_data).unwrap();

    writer.flush().unwrap(); // the flush method is only used when the full base64 data has been written

    drop(writer);

    assert_eq!("SGkgdGhlcmUsIHRoaXMgaXMgYSBzaW1wbGUgc2VudGVuY2UgdXNlZCBmb3IgdGVzdGluZyB0aGlzIGNyYXRlLiBJIGhvcGUgYWxsIGNhc2VzIGFyZSBjb3JyZWN0Lg==", fs::read_to_string("tests/data/encode_output.txt").unwrap());
}