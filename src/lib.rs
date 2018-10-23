//! # Base64 Stream
//!
//! To encode/decode large data with the standard Base64 encoding.
//!
//! ## Examples
//!
//! ### Encode
//!
//! #### ToBase64Reader
//!
//! ```
//! extern crate base64_stream;
//!
//! use std::io::Cursor;
//!
//! use std::io::Read;
//!
//! use base64_stream::ToBase64Reader;
//!
//! let test_data = b"Hi there, this is a simple sentence used for testing this crate. I hope all cases are correct.".to_vec();
//!
//! let mut reader = ToBase64Reader::new(Cursor::new(test_data));
//!
//! let mut base64 = [0u8; 4096];
//!
//! let c = reader.read(&mut base64).unwrap();
//!
//! assert_eq!(b"SGkgdGhlcmUsIHRoaXMgaXMgYSBzaW1wbGUgc2VudGVuY2UgdXNlZCBmb3IgdGVzdGluZyB0aGlzIGNyYXRlLiBJIGhvcGUgYWxsIGNhc2VzIGFyZSBjb3JyZWN0Lg==".to_vec(), base64[..c].to_vec());
//! ```
//!
//! #### ToBase64Writer
//!
//! ```
//! extern crate base64_stream;
//!
//! use std::fs::{self, File};
//!
//! use std::io::Write;
//!
//! use base64_stream::ToBase64Writer;
//!
//! let test_data = b"Hi there, this is a simple sentence used for testing this crate. I hope all cases are correct.".as_ref();
//!
//! let base64 = File::create("encode_output.txt").unwrap();
//!
//! let mut writer = ToBase64Writer::new(base64);
//!
//! writer.write(test_data).unwrap();
//!
//! writer.flush().unwrap(); // the flush method is only used when the full base64 data has been written
//!
//! drop(writer);
//!
//! assert_eq!("SGkgdGhlcmUsIHRoaXMgaXMgYSBzaW1wbGUgc2VudGVuY2UgdXNlZCBmb3IgdGVzdGluZyB0aGlzIGNyYXRlLiBJIGhvcGUgYWxsIGNhc2VzIGFyZSBjb3JyZWN0Lg==", fs::read_to_string("encode_output.txt").unwrap());
//! ```
//!
//! ### Decode
//!
//! #### FromBase64Reader
//!
//! ```
//! extern crate base64_stream;
//!
//! use std::io::Cursor;
//!
//! use std::io::Read;
//!
//! use base64_stream::FromBase64Reader;
//!
//! let base64 = b"SGkgdGhlcmUsIHRoaXMgaXMgYSBzaW1wbGUgc2VudGVuY2UgdXNlZCBmb3IgdGVzdGluZyB0aGlzIGNyYXRlLiBJIGhvcGUgYWxsIGNhc2VzIGFyZSBjb3JyZWN0Lg==".to_vec();
//!
//! let mut reader = FromBase64Reader::new(Cursor::new(base64));
//!
//! let mut test_data = [0u8; 4096];
//!
//! let c = reader.read(&mut test_data).unwrap();
//!
//! assert_eq!(b"Hi there, this is a simple sentence used for testing this crate. I hope all cases are correct.".to_vec(), test_data[..c].to_vec());
//! ```
//!
//! #### FromBase64Writer
//!
//! ```
//! extern crate base64_stream;
//!
//! use std::fs::{self, File};
//!
//! use std::io::Write;
//!
//! use base64_stream::FromBase64Writer;
//!
//! let base64 = b"SGkgdGhlcmUsIHRoaXMgaXMgYSBzaW1wbGUgc2VudGVuY2UgdXNlZCBmb3IgdGVzdGluZyB0aGlzIGNyYXRlLiBJIGhvcGUgYWxsIGNhc2VzIGFyZSBjb3JyZWN0Lg==".as_ref();
//!
//! let test_data = File::create("decode_output.txt").unwrap();
//!
//! let mut writer = FromBase64Writer::new(test_data);
//!
//! writer.write(base64).unwrap();
//!
//! writer.flush().unwrap(); // the flush method is only used when the full base64 data has been written
//!
//! assert_eq!("Hi there, this is a simple sentence used for testing this crate. I hope all cases are correct.", fs::read_to_string("decode_output.txt").unwrap());
//! ```

pub extern crate base64;

use std::io::{self, Read, Write, ErrorKind};

const READ_SIZE: usize = 4096 * 3;

/// Read any data and encode them to base64 data.
pub struct ToBase64Reader<R: Read> {
    inner: R,
    buf: Vec<u8>,
}

impl<R: Read> ToBase64Reader<R> {
    pub fn new(inner: R) -> ToBase64Reader<R> {
        ToBase64Reader {
            inner,
            buf: Vec::new(),
        }
    }
}

impl<R: Read> Read for ToBase64Reader<R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, io::Error> {
        let buf_len = buf.len();

        if buf_len < 4 {
            return Err(io::Error::new(ErrorKind::Other, "the buffer needs to be equal to or more than 4 bytes"));
        }

        self.buf.clear();

        let actual_max_read_size = buf_len / 4 * 3;

        self.buf.reserve(actual_max_read_size);

        unsafe { self.buf.set_len(actual_max_read_size) };

        let c = {
            let mut buf = &mut self.buf[..actual_max_read_size];

            let mut c = 0;

            loop {
                match self.inner.read(buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        let tmp = buf;
                        buf = &mut tmp[n..];
                        c += n;
                    }
                    Err(ref e) if e.kind() == ErrorKind::Interrupted => {}
                    Err(e) => return Err(e),
                }
            }

            c
        };

        Ok(base64::encode_config_slice(&self.buf[..c], base64::STANDARD, buf))
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> Result<usize, io::Error> {
        self.buf.clear();

        let actual_max_read_size = READ_SIZE;

        self.buf.reserve(actual_max_read_size);

        unsafe { self.buf.set_len(actual_max_read_size) };

        let mut sum = 0;

        loop {
            let c = {
                let mut buf = &mut self.buf[..actual_max_read_size];

                let mut c = 0;

                loop {
                    match self.inner.read(buf) {
                        Ok(0) => break,
                        Ok(n) => {
                            let tmp = buf;
                            buf = &mut tmp[n..];
                            c += n;
                        }
                        Err(ref e) if e.kind() == ErrorKind::Interrupted => {}
                        Err(e) => return Err(e),
                    }
                }

                c
            };

            if c == 0 {
                break;
            }

            let cc = (c + 2) / 3 * 4;

            buf.reserve(cc);

            let old_len = buf.len();

            unsafe { buf.set_len(old_len + cc) };

            base64::encode_config_slice(&self.buf[..c], base64::STANDARD, &mut buf[old_len..]);

            sum += cc;
        }

        Ok(sum)
    }

    fn read_to_string(&mut self, buf: &mut String) -> Result<usize, io::Error> {
        self.buf.clear();

        let actual_max_read_size = READ_SIZE;

        self.buf.reserve(actual_max_read_size);

        unsafe { self.buf.set_len(actual_max_read_size) };

        let mut sum = 0;

        loop {
            let c = {
                let mut buf = &mut self.buf[..actual_max_read_size];

                let mut c = 0;

                loop {
                    match self.inner.read(buf) {
                        Ok(0) => break,
                        Ok(n) => {
                            let tmp = buf;
                            buf = &mut tmp[n..];
                            c += n;
                        }
                        Err(ref e) if e.kind() == ErrorKind::Interrupted => {}
                        Err(e) => return Err(e),
                    }
                }

                c
            };

            if c == 0 {
                break;
            }

            let cc = (c + 2) / 3 * 4;

            buf.reserve(cc);

            base64::encode_config_buf(&self.buf[..c], base64::STANDARD, buf);

            sum += cc;
        }

        Ok(sum)
    }
}

/// Read base64 data and decode them to plain data.
pub struct FromBase64Reader<R: Read> {
    inner: R,
    buf: Vec<u8>,
}

impl<R: Read> FromBase64Reader<R> {
    pub fn new(inner: R) -> FromBase64Reader<R> {
        FromBase64Reader {
            inner,
            buf: Vec::new(),
        }
    }
}

impl<R: Read> Read for FromBase64Reader<R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, io::Error> {
        let buf_len = buf.len();

        if buf_len < 3 {
            return Err(io::Error::new(ErrorKind::Other, "the buffer needs to be equal to or more than 3 bytes"));
        }

        self.buf.clear();

        let actual_max_read_size = buf_len / 3 * 4;

        self.buf.reserve(actual_max_read_size);

        unsafe { self.buf.set_len(actual_max_read_size) };

        let c = {
            let mut buf = &mut self.buf[..actual_max_read_size];

            let mut c = 0;

            loop {
                match self.inner.read(buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        let tmp = buf;
                        buf = &mut tmp[n..];
                        c += n;
                    }
                    Err(ref e) if e.kind() == ErrorKind::Interrupted => {}
                    Err(e) => return Err(e),
                }
            }

            c
        };

        Ok(base64::decode_config_slice(&self.buf[..c], base64::STANDARD, buf).map_err(|err| io::Error::new(ErrorKind::Other, err.to_string()))?)
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> Result<usize, io::Error> {
        self.buf.clear();

        let actual_max_read_size = READ_SIZE;

        self.buf.reserve(actual_max_read_size);

        unsafe { self.buf.set_len(actual_max_read_size) };

        let mut sum = 0;

        loop {
            let c = {
                let mut buf = &mut self.buf[..actual_max_read_size];

                let mut c = 0;

                loop {
                    match self.inner.read(buf) {
                        Ok(0) => break,
                        Ok(n) => {
                            let tmp = buf;
                            buf = &mut tmp[n..];
                            c += n;
                        }
                        Err(ref e) if e.kind() == ErrorKind::Interrupted => {}
                        Err(e) => return Err(e),
                    }
                }

                c
            };

            if c == 0 {
                break;
            }

            let old_len = buf.len();

            base64::decode_config_buf(&self.buf[..c], base64::STANDARD, buf).map_err(|err| io::Error::new(ErrorKind::Other, err.to_string()))?;

            sum += buf.len() - old_len;
        }

        Ok(sum)
    }

    fn read_to_string(&mut self, buf: &mut String) -> Result<usize, io::Error> {
        self.buf.clear();

        let actual_max_read_size = READ_SIZE;

        self.buf.reserve(actual_max_read_size);

        unsafe { self.buf.set_len(actual_max_read_size) };

        let mut sum = 0;

        loop {
            let c = {
                let mut buf = &mut self.buf[..actual_max_read_size];

                let mut c = 0;

                loop {
                    match self.inner.read(buf) {
                        Ok(0) => break,
                        Ok(n) => {
                            let tmp = buf;
                            buf = &mut tmp[n..];
                            c += n;
                        }
                        Err(ref e) if e.kind() == ErrorKind::Interrupted => {}
                        Err(e) => return Err(e),
                    }
                }

                c
            };

            if c == 0 {
                break;
            }

            let mut temp = Vec::new();

            base64::decode_config_buf(&self.buf[..c], base64::STANDARD, &mut temp).map_err(|err| io::Error::new(ErrorKind::Other, err.to_string()))?;

            let temp = String::from_utf8(temp).map_err(|_| io::Error::new(ErrorKind::InvalidData, "stream did not contain valid UTF-8"))?;

            sum += temp.len();

            buf.push_str(&temp);
        }

        Ok(sum)
    }
}

/// Write any data and encode them to base64 data.
pub struct ToBase64Writer<W: Write> {
    inner: W,
    buf: Vec<u8>,
    remaining: Vec<u8>,
}

impl<W: Write> ToBase64Writer<W> {
    pub fn new(inner: W) -> ToBase64Writer<W> {
        ToBase64Writer {
            inner,
            buf: Vec::new(),
            remaining: Vec::new(),
        }
    }
}

impl<W: Write> Write for ToBase64Writer<W> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, io::Error> {
        let remaining_len = self.remaining.len();
        let buf_len = buf.len();

        if remaining_len > 0 {
            let new_buf_len = remaining_len + buf_len;

            if new_buf_len < 3 {
                self.remaining.extend_from_slice(&buf);
                Ok(buf_len)
            } else {
                let actual_max_write_size = new_buf_len / 3 * 3;

                let buf_end = actual_max_write_size - remaining_len;

                self.remaining.extend_from_slice(&buf[..buf_end]);

                let c = actual_max_write_size / 3 * 4;

                self.buf.clear();

                self.buf.reserve(c);

                unsafe { self.buf.set_len(c); }

                base64::encode_config_slice(&self.remaining, base64::STANDARD, &mut self.buf);

                self.remaining.clear();

                self.inner.write(&self.buf)?;

                if buf_len != buf_end {
                    self.remaining.extend_from_slice(&buf[buf_end..]);
                }

                Ok(buf_len)
            }
        } else {
            if buf_len < 3 {
                self.remaining.extend_from_slice(&buf);
                Ok(buf_len)
            } else {
                let actual_max_write_size = buf_len / 3 * 3;

                let buf = if actual_max_write_size == buf_len {
                    buf
                } else {
                    self.remaining.extend_from_slice(&buf[actual_max_write_size..]);
                    &buf[..actual_max_write_size]
                };

                let c = actual_max_write_size / 3 * 4;

                self.buf.clear();

                self.buf.reserve(c);

                unsafe { self.buf.set_len(c); }

                base64::encode_config_slice(buf, base64::STANDARD, &mut self.buf);

                self.inner.write(&self.buf)?;

                Ok(buf_len)
            }
        }
    }

    fn flush(&mut self) -> Result<(), io::Error> {
        let remaining_len = self.remaining.len();

        if remaining_len > 0 {
            let c = (remaining_len + 2) / 3 * 4;

            self.buf.clear();

            self.buf.reserve(c);

            unsafe { self.buf.set_len(c); }

            base64::encode_config_slice(&self.remaining, base64::STANDARD, &mut self.buf);

            self.remaining.clear();

            self.inner.write(&self.buf)?;

            self.inner.flush()
        } else {
            self.inner.flush()
        }
    }
}

impl<W: Write> Drop for ToBase64Writer<W> {
    fn drop(&mut self) {
        self.flush().unwrap()
    }
}

/// Write base64 data and decode them to plain data.
pub struct FromBase64Writer<W: Write> {
    inner: W,
    buf: Vec<u8>,
    remaining: Vec<u8>,
}

impl<W: Write> FromBase64Writer<W> {
    pub fn new(inner: W) -> FromBase64Writer<W> {
        FromBase64Writer {
            inner,
            buf: Vec::new(),
            remaining: Vec::new(),
        }
    }
}

impl<W: Write> Write for FromBase64Writer<W> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, io::Error> {
        let remaining_len = self.remaining.len();
        let buf_len = buf.len();

        if remaining_len > 0 {
            let new_buf_len = remaining_len + buf_len;

            if new_buf_len < 4 {
                self.remaining.extend_from_slice(&buf);
                Ok(buf_len)
            } else {
                let actual_max_write_size = new_buf_len / 3 * 3;

                let buf_end = actual_max_write_size - remaining_len;

                self.remaining.extend_from_slice(&buf[..buf_end]);

                self.buf.clear();

                base64::decode_config_buf(&self.remaining, base64::STANDARD, &mut self.buf).map_err(|err| io::Error::new(ErrorKind::Other, err.to_string()))?;

                self.inner.write(&self.buf)?;

                if buf_len != buf_end {
                    self.remaining.extend_from_slice(&buf[buf_end..]);
                }

                Ok(buf_len)
            }
        } else {
            if buf_len < 4 {
                self.remaining.extend_from_slice(&buf);
                Ok(buf_len)
            } else {
                let actual_max_write_size = buf_len / 4 * 4;

                let buf = if actual_max_write_size == buf_len {
                    buf
                } else {
                    self.remaining.extend_from_slice(&buf[actual_max_write_size..]);
                    &buf[..actual_max_write_size]
                };

                self.buf.clear();

                base64::decode_config_buf(buf, base64::STANDARD, &mut self.buf).map_err(|err| io::Error::new(ErrorKind::Other, err.to_string()))?;

                self.inner.write(&self.buf)?;

                Ok(buf_len)
            }
        }
    }

    fn flush(&mut self) -> Result<(), io::Error> {
        let remaining_len = self.remaining.len();

        if remaining_len > 0 {
            self.buf.clear();

            base64::decode_config_buf(&self.remaining, base64::STANDARD, &mut self.buf).map_err(|err| io::Error::new(ErrorKind::Other, err.to_string()))?;

            self.remaining.clear();

            self.inner.write(&self.buf)?;

            self.inner.flush()
        } else {
            self.inner.flush()
        }
    }
}

impl<W: Write> Drop for FromBase64Writer<W> {
    fn drop(&mut self) {
        self.flush().unwrap()
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    use std::io::Cursor;
    use std::fs::{self, File};

    #[test]
    fn encode_writer() {
        let test_data = b"Hi there, this is a simple sentence used for testing this crate. I hope all cases are correct.".as_ref();

        let base64 = File::create("encode_output.txt").unwrap();

        let mut writer = ToBase64Writer::new(base64);

        writer.write(test_data).unwrap();

        writer.flush().unwrap(); // the flush method is only used when the full base64 data has been written

        drop(writer);

        assert_eq!("SGkgdGhlcmUsIHRoaXMgaXMgYSBzaW1wbGUgc2VudGVuY2UgdXNlZCBmb3IgdGVzdGluZyB0aGlzIGNyYXRlLiBJIGhvcGUgYWxsIGNhc2VzIGFyZSBjb3JyZWN0Lg==", fs::read_to_string("encode_output.txt").unwrap());
    }

    #[test]
    fn encode_reader() {
        let test_data = b"Hi there, this is a simple sentence used for testing this crate. I hope all cases are correct.".to_vec();

        let mut reader = ToBase64Reader::new(Cursor::new(test_data));

        let mut base64 = [0u8; 4096];

        let c = reader.read(&mut base64).unwrap();

        assert_eq!(b"SGkgdGhlcmUsIHRoaXMgaXMgYSBzaW1wbGUgc2VudGVuY2UgdXNlZCBmb3IgdGVzdGluZyB0aGlzIGNyYXRlLiBJIGhvcGUgYWxsIGNhc2VzIGFyZSBjb3JyZWN0Lg==".to_vec(), base64[..c].to_vec());
    }

    #[test]
    fn encode_to_end() {
        let test_data = b"Hi there, this is a simple sentence used for testing this crate. I hope all cases are correct.".to_vec();

        let mut reader = ToBase64Reader::new(Cursor::new(test_data));

        let mut base64 = Vec::new();

        reader.read_to_end(&mut base64).unwrap();

        assert_eq!(b"SGkgdGhlcmUsIHRoaXMgaXMgYSBzaW1wbGUgc2VudGVuY2UgdXNlZCBmb3IgdGVzdGluZyB0aGlzIGNyYXRlLiBJIGhvcGUgYWxsIGNhc2VzIGFyZSBjb3JyZWN0Lg==".to_vec(), base64);
    }

    #[test]
    fn encode_to_string() {
        let test_data = b"Hi there, this is a simple sentence used for testing this crate. I hope all cases are correct.".to_vec();

        let mut reader = ToBase64Reader::new(Cursor::new(test_data));

        let mut base64_string = String::new();

        reader.read_to_string(&mut base64_string).unwrap();

        assert_eq!("SGkgdGhlcmUsIHRoaXMgaXMgYSBzaW1wbGUgc2VudGVuY2UgdXNlZCBmb3IgdGVzdGluZyB0aGlzIGNyYXRlLiBJIGhvcGUgYWxsIGNhc2VzIGFyZSBjb3JyZWN0Lg==", base64_string);
    }

    #[test]
    fn decode_writer() {
        let base64 = b"SGkgdGhlcmUsIHRoaXMgaXMgYSBzaW1wbGUgc2VudGVuY2UgdXNlZCBmb3IgdGVzdGluZyB0aGlzIGNyYXRlLiBJIGhvcGUgYWxsIGNhc2VzIGFyZSBjb3JyZWN0Lg==".as_ref();

        let test_data = File::create("decode_output.txt").unwrap();

        let mut writer = FromBase64Writer::new(test_data);

        writer.write(base64).unwrap();

        writer.flush().unwrap(); // the flush method is only used when the full base64 data has been written

        assert_eq!("Hi there, this is a simple sentence used for testing this crate. I hope all cases are correct.", fs::read_to_string("decode_output.txt").unwrap());
    }

    #[test]
    fn decode_reader() {
        let base64 = b"SGkgdGhlcmUsIHRoaXMgaXMgYSBzaW1wbGUgc2VudGVuY2UgdXNlZCBmb3IgdGVzdGluZyB0aGlzIGNyYXRlLiBJIGhvcGUgYWxsIGNhc2VzIGFyZSBjb3JyZWN0Lg==".to_vec();

        let mut reader = FromBase64Reader::new(Cursor::new(base64));

        let mut test_data = [0u8; 4096];

        let c = reader.read(&mut test_data).unwrap();

        assert_eq!(b"Hi there, this is a simple sentence used for testing this crate. I hope all cases are correct.".to_vec(), test_data[..c].to_vec());
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

        assert_eq!("Hi there, this is a simple sentence used for testing this crate. I hope all cases are correct.", test_data);
    }
}