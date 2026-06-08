use std::{
    fs::{self, File},
    io::{self, Cursor, ErrorKind, Write},
    path::Path,
};

use base64_stream::ToBase64Writer;

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
const ENCODE_OUTPUT: &str = "encode_output.txt";

#[test]
fn encode_write() {
    let test_data = b"Hi there, this is a simple sentence used for testing this crate. I hope all cases are correct.".as_ref();

    let file_path = Path::new("tests").join(DATA_FOLDER).join(ENCODE_OUTPUT);

    let base64 = File::create(file_path.as_path()).unwrap();

    let mut writer = ToBase64Writer::new(base64);

    writer.write_all(test_data).unwrap();

    writer.finish().unwrap();

    assert_eq!(
        "SGkgdGhlcmUsIHRoaXMgaXMgYSBzaW1wbGUgc2VudGVuY2UgdXNlZCBmb3IgdGVzdGluZyB0aGlzIGNyYXRlLiBJIGhvcGUgYWxsIGNhc2VzIGFyZSBjb3JyZWN0Lg==",
        fs::read_to_string(file_path).unwrap()
    );
}

#[test]
fn encode_empty_write() {
    let mut writer = ToBase64Writer::new(Cursor::new(Vec::<u8>::new()));

    writer.flush().unwrap();

    let out = writer.into_inner().into_inner();

    assert!(out.is_empty());
}

#[test]
fn encode_flush_flushes_inner_writer() {
    let inner = FlushCountingWriter::default();
    let mut writer = ToBase64Writer::new(inner);

    writer.write_all(b"ab").unwrap();
    writer.flush().unwrap();

    let inner = writer.into_inner();

    assert_eq!(1, inner.flush_count);
    assert!(inner.data.is_empty());
}

#[test]
fn encode_flush_returns_inner_flush_error() {
    let inner = FlushCountingWriter {
        flush_error: Some(ErrorKind::BrokenPipe),
        ..Default::default()
    };
    let mut writer = ToBase64Writer::new(inner);

    let error = writer.flush().unwrap_err();

    assert_eq!(ErrorKind::BrokenPipe, error.kind());
}

#[test]
fn encode_one_byte_write() {
    let mut writer = ToBase64Writer::new(Cursor::new(Vec::<u8>::new()));

    writer.write_all(b"a").unwrap();

    let out = writer.finish().unwrap().into_inner();

    assert_eq!(out, b"YQ==");
}

#[test]
fn encode_two_bytes_write() {
    let mut writer = ToBase64Writer::new(Cursor::new(Vec::<u8>::new()));

    writer.write_all(b"ab").unwrap();

    let out = writer.finish().unwrap().into_inner();

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

    let out = writer.finish().unwrap().into_inner();

    assert_eq!(out, expected.as_bytes());
}

#[test]
fn encode_into_inner_write() {
    let mut writer = ToBase64Writer::new(Cursor::new(Vec::<u8>::new()));

    writer.write_all(b"abc").unwrap();

    let out = writer.finish().unwrap().into_inner();

    assert_eq!(out, b"YWJj");
}

#[test]
fn encode_small_buffer_write() {
    let mut writer = ToBase64Writer::<_, 4>::new2(Cursor::new(Vec::<u8>::new()));

    writer.write_all(b"abcd").unwrap();

    let out = writer.finish().unwrap().into_inner();

    assert_eq!(out, b"YWJjZA==");
}

#[test]
fn encode_finish_flushes_inner_writer() {
    let inner = FlushCountingWriter::default();
    let mut writer = ToBase64Writer::new(inner);

    writer.write_all(b"ab").unwrap();

    let inner = writer.finish().unwrap();

    assert_eq!(1, inner.flush_count);
    assert_eq!(b"YWI=", inner.data.as_slice());
}

#[test]
fn encode_finish_returns_inner_flush_error() {
    let inner = FlushCountingWriter {
        flush_error: Some(ErrorKind::BrokenPipe),
        ..Default::default()
    };
    let mut writer = ToBase64Writer::new(inner);

    writer.write_all(b"ab").unwrap();

    let error = writer.finish().unwrap_err();

    assert_eq!(ErrorKind::BrokenPipe, error.kind());
}
