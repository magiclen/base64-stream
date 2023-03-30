use std::{
    fs::{self, File},
    io::Write,
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
