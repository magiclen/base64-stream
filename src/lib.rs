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

pub extern crate base64;

use std::io::{self, Read, ErrorKind};

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

        let actually_max_read_size = buf_len / 4 * 3;

        self.buf.reserve(actually_max_read_size);

        unsafe { self.buf.set_len(actually_max_read_size) };

        let c = {
            let mut buf = &mut self.buf[..actually_max_read_size];

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

        let actually_max_read_size = READ_SIZE;

        self.buf.reserve(actually_max_read_size);

        unsafe { self.buf.set_len(actually_max_read_size) };

        let mut sum = 0;

        loop {
            let c = {
                let mut buf = &mut self.buf[..actually_max_read_size];

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

        let actually_max_read_size = READ_SIZE;

        self.buf.reserve(actually_max_read_size);

        unsafe { self.buf.set_len(actually_max_read_size) };

        let mut sum = 0;

        loop {
            let c = {
                let mut buf = &mut self.buf[..actually_max_read_size];

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

        let actually_max_read_size = buf_len / 3 * 4;

        self.buf.reserve(actually_max_read_size);

        unsafe { self.buf.set_len(actually_max_read_size) };

        let c = {
            let mut buf = &mut self.buf[..actually_max_read_size];

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

        let actually_max_read_size = READ_SIZE;

        self.buf.reserve(actually_max_read_size);

        unsafe { self.buf.set_len(actually_max_read_size) };

        let mut sum = 0;

        loop {
            let c = {
                let mut buf = &mut self.buf[..actually_max_read_size];

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

        let actually_max_read_size = READ_SIZE;

        self.buf.reserve(actually_max_read_size);

        unsafe { self.buf.set_len(actually_max_read_size) };

        let mut sum = 0;

        loop {
            let c = {
                let mut buf = &mut self.buf[..actually_max_read_size];

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

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::Cursor;

    #[test]
    fn encode() {
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
    fn decode() {
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