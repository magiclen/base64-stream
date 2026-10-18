use std::io::{self, Cursor, ErrorKind, Write};

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

#[derive(Debug, Default)]
struct ShortWriter {
    data:        Vec<u8>,
    write_count: usize,
    block_twice: bool,
}

impl Write for ShortWriter {
    fn write(&mut self, buf: &[u8]) -> Result<usize, io::Error> {
        self.write_count += 1;

        if self.write_count == 2 || (self.block_twice && self.write_count == 3) {
            return Err(ErrorKind::WouldBlock.into());
        }

        let length = buf.len().min(2);

        self.data.extend_from_slice(&buf[..length]);

        Ok(length)
    }

    fn flush(&mut self) -> Result<(), io::Error> {
        Ok(())
    }
}

#[test]
fn encode_write() {
    let test_data = b"Hi there, this is a simple sentence used for testing this crate. I hope all cases are correct.".as_ref();

    let mut writer = ToBase64Writer::new(Vec::new());

    writer.write_all(test_data).unwrap();

    let base64 = writer.finish().unwrap();

    assert_eq!(
        b"SGkgdGhlcmUsIHRoaXMgaXMgYSBzaW1wbGUgc2VudGVuY2UgdXNlZCBmb3IgdGVzdGluZyB0aGlzIGNyYXRlLiBJIGhvcGUgYWxsIGNhc2VzIGFyZSBjb3JyZWN0Lg==",
        base64.as_slice()
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

    assert_eq!(b"YQ==", out.as_slice());
}

#[test]
fn encode_two_bytes_write() {
    let mut writer = ToBase64Writer::new(Cursor::new(Vec::<u8>::new()));

    writer.write_all(b"ab").unwrap();

    let out = writer.finish().unwrap().into_inner();

    assert_eq!(b"YWI=", out.as_slice());
}

#[test]
fn encode_three_bytes_write() {
    let mut writer = ToBase64Writer::new(Cursor::new(Vec::<u8>::new()));

    writer.write_all(b"abc").unwrap();
    writer.flush().unwrap();

    let out = writer.into_inner().into_inner();

    assert_eq!(b"YWJj", out.as_slice());
}

#[test]
fn encode_large_write() {
    use base64_stream::base64::{Engine, engine::general_purpose::STANDARD};

    let plain: Vec<u8> = (0u8..=255).cycle().take(5000).collect();
    let expected = STANDARD.encode(&plain);

    let mut writer = ToBase64Writer::new(Cursor::new(Vec::<u8>::new()));

    writer.write_all(&plain).unwrap();

    let out = writer.finish().unwrap().into_inner();

    assert_eq!(expected.as_bytes(), out);
}

#[test]
fn encode_into_inner_write() {
    let mut writer = ToBase64Writer::new(Cursor::new(Vec::<u8>::new()));

    writer.write_all(b"abc").unwrap();

    let out = writer.finish().unwrap().into_inner();

    assert_eq!(b"YWJj", out.as_slice());
}

#[test]
fn encode_small_buffer_write() {
    let mut writer = ToBase64Writer::<_, 4>::new2(Cursor::new(Vec::<u8>::new()));

    writer.write_all(b"abcd").unwrap();

    let out = writer.finish().unwrap().into_inner();

    assert_eq!(b"YWJjZA==", out.as_slice());
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

fn check_partial_writes<const N: usize>() {
    for split in 0..3 {
        let mut writer = ToBase64Writer::<_, N>::new2(ShortWriter::default());
        let plain = b"abcdefghij";

        writer.write_all(&plain[..split]).unwrap();
        writer.write_all(&plain[split..]).unwrap();
        writer.flush().unwrap();

        let inner = writer.finish().unwrap();

        assert_eq!(b"YWJjZGVmZ2hpag==", inner.data.as_slice());
    }
}

#[test]
fn encode_partial_writes_can_resume() {
    check_partial_writes::<4>();
    check_partial_writes::<4096>();
}

fn check_repeated_would_block<const N: usize>() {
    use base64_stream::base64::{Engine, engine::general_purpose::STANDARD};

    let plain: Vec<u8> = (0u8..=255).cycle().take(5000).collect();
    let expected = STANDARD.encode(&plain);
    let inner = ShortWriter {
        block_twice: true,
        ..Default::default()
    };
    let mut writer = ToBase64Writer::<_, N>::new2(inner);
    let mut offset = 0;
    let mut saw_would_block = false;

    while offset < plain.len() {
        match writer.write(&plain[offset..]) {
            Ok(0) => panic!("writer made no progress"),
            Ok(length) => offset += length,
            Err(error) if error.kind() == ErrorKind::WouldBlock => saw_would_block = true,
            Err(error) => panic!("unexpected write error: {error}"),
        }
    }

    assert!(saw_would_block);

    let inner = writer.finish().unwrap();

    assert_eq!(expected.as_bytes(), inner.data.as_slice());
}

#[test]
fn encode_retries_repeated_would_block() {
    check_repeated_would_block::<4>();
    check_repeated_would_block::<4096>();
}

fn check_chunked_writes<const N: usize>() {
    use base64_stream::base64::{Engine, engine::general_purpose::STANDARD};

    for length in [0, 1, 2, 3, 4, 5000] {
        let plain: Vec<u8> = (0u8..=255).cycle().take(length).collect();
        let expected = STANDARD.encode(&plain);

        for chunk_size in [1, 2, 5, 1024] {
            let mut writer = ToBase64Writer::<_, N>::new2(Vec::new());

            for chunk in plain.chunks(chunk_size) {
                writer.write_all(chunk).unwrap();
            }

            assert_eq!(expected.as_bytes(), writer.finish().unwrap());
        }
    }
}

#[test]
fn encode_chunked_writes() {
    check_chunked_writes::<4>();
    check_chunked_writes::<5>();
    check_chunked_writes::<7>();
    check_chunked_writes::<4096>();
}
