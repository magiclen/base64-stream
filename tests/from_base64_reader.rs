use std::io::{Cursor, Error, ErrorKind, Read};

use base64_stream::FromBase64Reader;

#[derive(Debug)]
struct CountedReader {
    data:       Cursor<&'static [u8]>,
    read_count: usize,
}

impl CountedReader {
    fn new(data: &'static [u8]) -> Self {
        CountedReader {
            data: Cursor::new(data), read_count: 0
        }
    }
}

impl Read for CountedReader {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        self.read_count += 1;

        self.data.read(buf)
    }
}

#[derive(Debug)]
struct FailAfterFirstRead {
    data:       Cursor<&'static [u8]>,
    read_count: usize,
}

impl FailAfterFirstRead {
    fn new(data: &'static [u8]) -> Self {
        FailAfterFirstRead {
            data: Cursor::new(data), read_count: 0
        }
    }
}

impl Read for FailAfterFirstRead {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        if self.read_count > 0 {
            return Err(Error::new(ErrorKind::WouldBlock, "unexpected read"));
        }

        self.read_count += 1;

        self.data.read(buf)
    }
}

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
fn decode_zero_length_read_does_not_read_inner() {
    let inner = CountedReader::new(b"YWJj");
    let mut reader = FromBase64Reader::new(inner);

    let mut out = [];

    assert_eq!(0, reader.read(&mut out).unwrap());

    let inner = reader.into_inner();

    assert_eq!(0, inner.read_count);
}

#[test]
fn decode_pending_temp_is_read_before_inner() {
    let inner = FailAfterFirstRead::new(b"YWJj");
    let mut reader = FromBase64Reader::new(inner);

    let mut first = [0];

    assert_eq!(1, reader.read(&mut first).unwrap());
    assert_eq!(b"a", first.as_ref());

    let mut second = [0];

    assert_eq!(1, reader.read(&mut second).unwrap());
    assert_eq!(b"b", second.as_ref());
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

    let error = reader.read_to_end(&mut out).unwrap_err();

    assert_eq!(ErrorKind::InvalidData, error.kind());
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
