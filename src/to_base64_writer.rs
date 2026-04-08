use std::{
    fmt,
    io::{self, Write},
};

use base64::{
    Engine,
    engine::{GeneralPurpose, general_purpose::STANDARD},
};

/// Write base64 data and encode them to plain data.
pub struct ToBase64Writer<W: Write, const N: usize = 4096> {
    inner:      W,
    buf:        [u8; 3],
    buf_length: usize,
    temp:       [u8; N],
    engine:     &'static GeneralPurpose,
}

impl<W: Write, const N: usize> fmt::Debug for ToBase64Writer<W, N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ToBase64Writer")
            .field("buf", &&self.buf[..self.buf_length])
            .field("buf_length", &self.buf_length)
            .finish_non_exhaustive()
    }
}

impl<W: Write> ToBase64Writer<W> {
    #[inline]
    pub fn new(writer: W) -> ToBase64Writer<W> {
        Self::new2(writer)
    }
}

impl<W: Write, const N: usize> ToBase64Writer<W, N> {
    #[inline]
    pub fn new2(writer: W) -> ToBase64Writer<W, N> {
        const { assert!(N >= 4, "buffer size N must be at least 4") };
        ToBase64Writer {
            inner:      writer,
            buf:        [0; 3],
            buf_length: 0,
            temp:       [0u8; N],
            engine:     &STANDARD,
        }
    }
}

impl<W: Write, const N: usize> ToBase64Writer<W, N> {
    fn drain_block(&mut self) -> Result<(), io::Error> {
        debug_assert!(self.buf_length > 0);

        let encode_length =
            self.engine.encode_slice(&self.buf[..self.buf_length], &mut self.temp).unwrap();

        self.inner.write_all(&self.temp[..encode_length])?;

        self.buf_length = 0;

        Ok(())
    }
}

impl<W: Write, const N: usize> Write for ToBase64Writer<W, N> {
    fn write(&mut self, mut buf: &[u8]) -> Result<usize, io::Error> {
        let original_buf_length = buf.len();

        if self.buf_length == 0 {
            while buf.len() >= 3 {
                let max_available_buf_length = (buf.len() - (buf.len() % 3)).min((N >> 2) * 3); // (N / 4) * 3

                let encode_length = self
                    .engine
                    .encode_slice(&buf[..max_available_buf_length], &mut self.temp)
                    .unwrap();

                buf = &buf[max_available_buf_length..];

                self.inner.write_all(&self.temp[..encode_length])?;
            }

            let buf_length = buf.len();

            if buf_length > 0 {
                self.buf[..buf_length].copy_from_slice(&buf[..buf_length]);

                self.buf_length = buf_length;
            }
        } else {
            debug_assert!(self.buf_length < 3);

            let r = 3 - self.buf_length;

            let buf_length = buf.len();

            let drain_length = r.min(buf_length);

            self.buf[self.buf_length..self.buf_length + drain_length]
                .copy_from_slice(&buf[..drain_length]);

            buf = &buf[drain_length..];

            self.buf_length += drain_length;

            if self.buf_length == 3 {
                self.drain_block()?;

                if buf_length > r {
                    self.write_all(buf)?;
                }
            }
        }

        Ok(original_buf_length)
    }

    fn flush(&mut self) -> Result<(), io::Error> {
        if self.buf_length > 0 {
            self.drain_block()?;
        }

        Ok(())
    }
}

impl<W: Write> From<W> for ToBase64Writer<W> {
    #[inline]
    fn from(reader: W) -> Self {
        ToBase64Writer::new(reader)
    }
}
