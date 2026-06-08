use std::{
    fs::{self, File},
    io::{self, Cursor, ErrorKind, Write},
    path::Path,
};

use base64_stream::FromBase64Writer;

#[derive(Debug, Default)]
struct FlushCountingWriter {
    data:        Vec<u8>,
    flush_count: usize,
    flush_error: Option<ErrorKind>,
}

impl Write for FlushCountingWriter {
    fn write(&mut self, buf: &[u8]) -> Result<usize, io::Error> {
        self.data.extend_from_slice(buf);

        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<(), io::Error> {
        self.flush_count += 1;

        match self.flush_error {
            Some(kind) => Err(io::Error::new(kind, "flush error")),
            None => Ok(()),
        }
    }
}

const DATA_FOLDER: &str = "data";
const DECODE_OUTPUT: &str = "decode_output.txt";

#[test]
fn decode_write() {
    let base64 = b"SGkgdGhlcmUsIHRoaXMgaXMgYSBzaW1wbGUgc2VudGVuY2UgdXNlZCBmb3IgdGVzdGluZyB0aGlzIGNyYXRlLiBJIGhvcGUgYWxsIGNhc2VzIGFyZSBjb3JyZWN0Lg==".as_ref();

    let file_path = Path::new("tests").join(DATA_FOLDER).join(DECODE_OUTPUT);

    let test_data = File::create(file_path.as_path()).unwrap();

    let mut writer = FromBase64Writer::new(test_data);

    writer.write_all(base64).unwrap();

    writer.finish().unwrap();

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
fn decode_flush_flushes_inner_writer() {
    let inner = FlushCountingWriter::default();
    let mut writer = FromBase64Writer::new(inner);

    writer.write_all(b"YWI=").unwrap();
    writer.flush().unwrap();

    let inner = writer.into_inner();

    assert_eq!(1, inner.flush_count);
    assert_eq!(b"ab", inner.data.as_slice());
}

#[test]
fn decode_flush_returns_inner_flush_error() {
    let inner = FlushCountingWriter {
        flush_error: Some(ErrorKind::BrokenPipe),
        ..Default::default()
    };
    let mut writer = FromBase64Writer::new(inner);

    let error = writer.flush().unwrap_err();

    assert_eq!(ErrorKind::BrokenPipe, error.kind());
}

#[test]
fn decode_one_byte_write() {
    let mut writer = FromBase64Writer::new(Cursor::new(Vec::<u8>::new()));

    writer.write_all(b"YQ==").unwrap();

    let out = writer.finish().unwrap().into_inner();

    assert_eq!(out, b"a");
}

#[test]
fn decode_two_bytes_write() {
    let mut writer = FromBase64Writer::new(Cursor::new(Vec::<u8>::new()));

    writer.write_all(b"YWI=").unwrap();

    let out = writer.finish().unwrap().into_inner();

    assert_eq!(out, b"ab");
}

#[test]
fn decode_large_write() {
    use base64_stream::base64::{Engine, engine::general_purpose::STANDARD};

    let plain: Vec<u8> = (0u8..=255).cycle().take(5000).collect();
    let encoded = STANDARD.encode(&plain);

    let mut writer = FromBase64Writer::new(Cursor::new(Vec::<u8>::new()));

    writer.write_all(encoded.as_bytes()).unwrap();

    let out = writer.finish().unwrap().into_inner();

    assert_eq!(out, plain);
}

#[test]
fn decode_invalid_write() {
    let mut writer = FromBase64Writer::new(Cursor::new(Vec::<u8>::new()));

    let error = writer.write_all(b"!!!!").unwrap_err();

    assert_eq!(ErrorKind::InvalidData, error.kind());
}

#[test]
fn decode_into_inner_write() {
    let mut writer = FromBase64Writer::new(Cursor::new(Vec::<u8>::new()));

    writer.write_all(b"YQ==").unwrap();

    let out = writer.finish().unwrap().into_inner();

    assert_eq!(out, b"a");
}

#[test]
fn decode_small_buffer_write() {
    let mut writer = FromBase64Writer::<_, 4>::new2(Cursor::new(Vec::<u8>::new()));

    writer.write_all(b"YWJjZA==").unwrap();

    let out = writer.finish().unwrap().into_inner();

    assert_eq!(out, b"abcd");
}

#[test]
fn decode_flush_does_not_decode_pending_tail() {
    let inner = FlushCountingWriter::default();
    let mut writer = FromBase64Writer::new(inner);

    writer.write_all(b"YQ").unwrap();
    writer.flush().unwrap();

    let inner = writer.into_inner();

    assert_eq!(1, inner.flush_count);
    assert!(inner.data.is_empty());
}

#[test]
fn decode_finish_rejects_invalid_pending_tail() {
    let inner = FlushCountingWriter::default();
    let mut writer = FromBase64Writer::new(inner);

    writer.write_all(b"YQ").unwrap();

    let error = writer.finish().unwrap_err();

    assert_eq!(ErrorKind::InvalidData, error.kind());
}

#[test]
fn decode_finish_returns_inner_flush_error() {
    let inner = FlushCountingWriter {
        flush_error: Some(ErrorKind::BrokenPipe),
        ..Default::default()
    };
    let mut writer = FromBase64Writer::new(inner);

    writer.write_all(b"YQ==").unwrap();

    let error = writer.finish().unwrap_err();

    assert_eq!(ErrorKind::BrokenPipe, error.kind());
}
