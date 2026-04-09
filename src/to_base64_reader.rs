use std::{
    fmt,
    io::{self, ErrorKind, Read},
};

use base64::{
    Engine,
    engine::{GeneralPurpose, general_purpose::STANDARD},
};

/// Read any data and encode them to base64 data.
pub struct ToBase64Reader<R: Read, const N: usize = 4096> {
    inner:       R,
    buf:         [u8; N],
    buf_length:  usize,
    buf_offset:  usize,
    temp:        [u8; 3],
    temp_length: usize,
    engine:      &'static GeneralPurpose,
}

impl<R: Read, const N: usize> fmt::Debug for ToBase64Reader<R, N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ToBase64Reader")
            .field("buf_length", &self.buf_length)
            .field("buf_offset", &self.buf_offset)
            .field("temp", &&self.temp[..self.temp_length])
            .field("temp_length", &self.temp_length)
            .finish_non_exhaustive()
    }
}

impl<R: Read> ToBase64Reader<R> {
    #[inline]
    pub fn new(reader: R) -> ToBase64Reader<R> {
        Self::new2(reader)
    }
}

impl<R: Read, const N: usize> ToBase64Reader<R, N> {
    #[inline]
    pub fn new2(reader: R) -> ToBase64Reader<R, N> {
        const { assert!(N >= 4, "buffer size N must be at least 4") };
        ToBase64Reader {
            inner:       reader,
            buf:         [0u8; N],
            buf_length:  0,
            buf_offset:  0,
            temp:        [0; 3],
            temp_length: 0,
            engine:      &STANDARD,
        }
    }
}

impl<R: Read, const N: usize> ToBase64Reader<R, N> {
    fn buf_left_shift(&mut self, distance: usize) {
        debug_assert!(self.buf_length >= distance);

        self.buf_offset += distance;

        if self.buf_offset >= N - 4 {
            self.buf.copy_within(self.buf_offset..self.buf_offset + self.buf_length, 0);

            self.buf_offset = 0;
        }

        self.buf_length -= distance;
    }

    #[inline]
    fn drain_temp<'a>(&mut self, buf: &'a mut [u8]) -> &'a mut [u8] {
        debug_assert!(self.temp_length > 0);
        debug_assert!(!buf.is_empty());

        let drain_length = buf.len().min(self.temp_length);

        buf[..drain_length].copy_from_slice(&self.temp[..drain_length]);

        self.temp_length -= drain_length;
        self.temp.copy_within(drain_length..drain_length + self.temp_length, 0);

        &mut buf[drain_length..]
    }

    #[inline]
    fn drain_block<'a>(&mut self, mut buf: &'a mut [u8]) -> &'a mut [u8] {
        debug_assert!(self.buf_length > 0);
        debug_assert!(self.temp_length == 0);
        debug_assert!(!buf.is_empty());

        let drain_length = self.buf_length.min(3);

        let mut b = [0; 4];

        let encode_length = self
            .engine
            .encode_slice(&self.buf[self.buf_offset..(self.buf_offset + drain_length)], &mut b)
            .unwrap();

        self.buf_left_shift(drain_length);

        let buf_length = buf.len();

        if buf_length >= encode_length {
            buf[..encode_length].copy_from_slice(&b[..encode_length]);

            buf = &mut buf[encode_length..];
        } else {
            buf[..buf_length].copy_from_slice(&b[..buf_length]);

            buf = &mut buf[buf_length..];

            self.temp_length = encode_length - buf_length;
            self.temp[..self.temp_length].copy_from_slice(&b[buf_length..encode_length]);
        }

        buf
    }

    fn drain<'a>(&mut self, mut buf: &'a mut [u8]) -> &'a mut [u8] {
        if buf.is_empty() {
            return buf;
        }

        if self.temp_length > 0 {
            buf = self.drain_temp(buf);
        }

        debug_assert!(self.buf_length >= 3);

        let buf_length = buf.len();

        if buf_length >= 4 {
            debug_assert!(self.temp_length == 0);

            let actual_max_read_size = (buf_length >> 2) * 3; // (buf_length / 4) * 3
            let max_available_self_buf_length = self.buf_length - (self.buf_length % 3);

            let drain_length = max_available_self_buf_length.min(actual_max_read_size);

            let encode_length = self
                .engine
                .encode_slice(&self.buf[self.buf_offset..(self.buf_offset + drain_length)], buf)
                .unwrap();

            buf = &mut buf[encode_length..];

            self.buf_left_shift(drain_length);
        }

        if !buf.is_empty() && self.buf_length >= 3 { self.drain_block(buf) } else { buf }
    }

    #[inline]
    fn drain_end<'a>(&mut self, mut buf: &'a mut [u8]) -> &'a mut [u8] {
        if buf.is_empty() {
            return buf;
        }

        if self.temp_length > 0 {
            buf = self.drain_temp(buf);
        }

        if !buf.is_empty() && self.buf_length > 0 { self.drain_block(buf) } else { buf }
    }

    /// Returns the inner reader, consuming this wrapper.
    #[inline]
    pub fn into_inner(self) -> R {
        self.inner
    }
}

impl<R: Read, const N: usize> Read for ToBase64Reader<R, N> {
    fn read(&mut self, mut buf: &mut [u8]) -> Result<usize, io::Error> {
        let original_buf_length = buf.len();

        while self.buf_length < 3 {
            match self.inner.read(&mut self.buf[(self.buf_offset + self.buf_length)..]) {
                Ok(0) => {
                    buf = self.drain_end(buf);

                    return Ok(original_buf_length - buf.len());
                },
                Ok(c) => self.buf_length += c,
                Err(ref e) if e.kind() == ErrorKind::Interrupted => {},
                Err(e) => return Err(e),
            }
        }

        buf = self.drain(buf);

        Ok(original_buf_length - buf.len())
    }
}

impl<R: Read> From<R> for ToBase64Reader<R> {
    #[inline]
    fn from(reader: R) -> Self {
        ToBase64Reader::new(reader)
    }
}
