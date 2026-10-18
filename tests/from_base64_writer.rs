use std::io::{self, Cursor, ErrorKind, Write};

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
fn decode_write() {
    let base64 = b"SGkgdGhlcmUsIHRoaXMgaXMgYSBzaW1wbGUgc2VudGVuY2UgdXNlZCBmb3IgdGVzdGluZyB0aGlzIGNyYXRlLiBJIGhvcGUgYWxsIGNhc2VzIGFyZSBjb3JyZWN0Lg==".as_ref();

    let mut writer = FromBase64Writer::new(Vec::new());

    writer.write_all(base64).unwrap();

    let test_data = writer.finish().unwrap();

    assert_eq!(
        b"Hi there, this is a simple sentence used for testing this crate. I hope all cases are correct.",
        test_data.as_slice()
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

    assert_eq!(b"a", out.as_slice());
}

#[test]
fn decode_two_bytes_write() {
    let mut writer = FromBase64Writer::new(Cursor::new(Vec::<u8>::new()));

    writer.write_all(b"YWI=").unwrap();

    let out = writer.finish().unwrap().into_inner();

    assert_eq!(b"ab", out.as_slice());
}

#[test]
fn decode_large_write() {
    use base64_stream::base64::{Engine, engine::general_purpose::STANDARD};

    let plain: Vec<u8> = (0u8..=255).cycle().take(5000).collect();
    let encoded = STANDARD.encode(&plain);

    let mut writer = FromBase64Writer::new(Cursor::new(Vec::<u8>::new()));

    writer.write_all(encoded.as_bytes()).unwrap();

    let out = writer.finish().unwrap().into_inner();

    assert_eq!(plain, out);
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

    assert_eq!(b"a", out.as_slice());
}

#[test]
fn decode_small_buffer_write() {
    let mut writer = FromBase64Writer::<_, 4>::new2(Cursor::new(Vec::<u8>::new()));

    writer.write_all(b"YWJjZA==").unwrap();

    let out = writer.finish().unwrap().into_inner();

    assert_eq!(b"abcd", out.as_slice());
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

fn check_partial_writes<const N: usize>() {
    for split in 0..4 {
        let mut writer = FromBase64Writer::<_, N>::new2(ShortWriter::default());
        let encoded = b"YWJjZGVmZ2hpag==";

        writer.write_all(&encoded[..split]).unwrap();
        writer.write_all(&encoded[split..]).unwrap();
        writer.flush().unwrap();

        let inner = writer.finish().unwrap();

        assert_eq!(b"abcdefghij", inner.data.as_slice());
    }
}

#[test]
fn decode_partial_writes_can_resume() {
    check_partial_writes::<4>();
    check_partial_writes::<4096>();
}

fn check_repeated_would_block<const N: usize>() {
    use base64_stream::base64::{Engine, engine::general_purpose::STANDARD};

    let plain: Vec<u8> = (0u8..=255).cycle().take(5000).collect();
    let encoded = STANDARD.encode(&plain);
    let inner = ShortWriter {
        block_twice: true,
        ..Default::default()
    };
    let mut writer = FromBase64Writer::<_, N>::new2(inner);
    let mut offset = 0;
    let mut saw_would_block = false;

    while offset < encoded.len() {
        match writer.write(&encoded.as_bytes()[offset..]) {
            Ok(0) => panic!("writer made no progress"),
            Ok(length) => offset += length,
            Err(error) if error.kind() == ErrorKind::WouldBlock => saw_would_block = true,
            Err(error) => panic!("unexpected write error: {error}"),
        }
    }

    assert!(saw_would_block);

    let inner = writer.finish().unwrap();

    assert_eq!(plain, inner.data);
}

#[test]
fn decode_retries_repeated_would_block() {
    check_repeated_would_block::<4>();
    check_repeated_would_block::<4096>();
}

fn check_data_after_padding<const N: usize>() {
    for encoded in [b"YQ==Yg==".as_slice(), b"YWI=Yw==", b"YWJjZA==YQ==", b"YQ==A"] {
        for chunk_size in [1, 3, 4, 5, 8, 16] {
            let mut writer = FromBase64Writer::<_, N>::new2(Vec::new());

            let error = encoded
                .chunks(chunk_size)
                .try_for_each(|chunk| writer.write_all(chunk))
                .and_then(|_| writer.finish().map(|_| ()))
                .unwrap_err();

            assert_eq!(ErrorKind::InvalidData, error.kind());
        }
    }
}

#[test]
fn decode_rejects_data_after_padding() {
    check_data_after_padding::<4>();
    check_data_after_padding::<5>();
    check_data_after_padding::<7>();
    check_data_after_padding::<4096>();
}

fn check_chunked_writes<const N: usize>() {
    use base64_stream::base64::{Engine, engine::general_purpose::STANDARD};

    for length in [0, 1, 2, 3, 4, 5000] {
        let plain: Vec<u8> = (0u8..=255).cycle().take(length).collect();
        let encoded = STANDARD.encode(&plain);

        for chunk_size in [1, 2, 5, 1024] {
            let mut writer = FromBase64Writer::<_, N>::new2(Vec::new());

            for chunk in encoded.as_bytes().chunks(chunk_size) {
                writer.write_all(chunk).unwrap();
            }

            assert_eq!(plain, writer.finish().unwrap());
        }
    }
}

#[test]
fn decode_chunked_writes() {
    check_chunked_writes::<4>();
    check_chunked_writes::<5>();
    check_chunked_writes::<7>();
    check_chunked_writes::<4096>();
}
