use std::{
    fs::{self, File},
    io::Write,
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
