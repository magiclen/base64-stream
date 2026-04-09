use std::{
    fmt,
    io::{self, Write},
};

use base64::{
    Engine,
    engine::{GeneralPurpose, general_purpose::STANDARD},
};

/// Write base64 data and decode them to plain data.
pub struct FromBase64Writer<W: Write, const N: usize = 4096> {
    inner:      W,
    buf:        [u8; 4],
    buf_length: usize,
    temp:       [u8; N],
    engine:     &'static GeneralPurpose,
}

impl<W: Write, const N: usize> fmt::Debug for FromBase64Writer<W, N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FromBase64Writer")
            .field("buf", &&self.buf[..self.buf_length])
            .field("buf_length", &self.buf_length)
            .finish_non_exhaustive()
    }
}

impl<W: Write> FromBase64Writer<W> {
    #[inline]
    pub fn new(writer: W) -> FromBase64Writer<W> {
        Self::new2(writer)
    }
}

impl<W: Write, const N: usize> FromBase64Writer<W, N> {
    #[inline]
    pub fn new2(writer: W) -> FromBase64Writer<W, N> {
        const { assert!(N >= 4, "buffer size N must be at least 4") };
        FromBase64Writer {
            inner:      writer,
            buf:        [0; 4],
            buf_length: 0,
            temp:       [0u8; N],
            engine:     &STANDARD,
        }
    }
}

impl<W: Write, const N: usize> FromBase64Writer<W, N> {
    fn drain_block(&mut self) -> Result<(), io::Error> {
        debug_assert!(self.buf_length > 0);

        let decode_length = self
            .engine
            .decode_slice(&self.buf[..self.buf_length], &mut self.temp)
            .map_err(io::Error::other)?;

        self.inner.write_all(&self.temp[..decode_length])?;

        self.buf_length = 0;

        Ok(())
    }

    /// Returns the inner writer, consuming this wrapper.
    ///
    /// Call [`flush`](std::io::Write::flush) before this method to ensure all
    /// buffered data is written.
    #[inline]
    pub fn into_inner(self) -> W {
        self.inner
    }
}

impl<W: Write, const N: usize> Write for FromBase64Writer<W, N> {
    fn write(&mut self, mut buf: &[u8]) -> Result<usize, io::Error> {
        let original_buf_length = buf.len();

        if self.buf_length == 0 {
            while buf.len() >= 4 {
                let max_available_buf_length = (buf.len() & !0b11).min((N / 3) << 2); // (N / 3) * 4

                let decode_length = self
                    .engine
                    .decode_slice(&buf[..max_available_buf_length], &mut self.temp)
                    .map_err(io::Error::other)?;

                buf = &buf[max_available_buf_length..];

                self.inner.write_all(&self.temp[..decode_length])?;
            }

            let buf_length = buf.len();

            if buf_length > 0 {
                self.buf[..buf_length].copy_from_slice(&buf[..buf_length]);

                self.buf_length = buf_length;
            }
        } else {
            debug_assert!(self.buf_length < 4);

            let r = 4 - self.buf_length;

            let buf_length = buf.len();

            let drain_length = r.min(buf_length);

            self.buf[self.buf_length..self.buf_length + drain_length]
                .copy_from_slice(&buf[..drain_length]);

            buf = &buf[drain_length..];

            self.buf_length += drain_length;

            if self.buf_length == 4 {
                self.drain_block()?;

                if buf_length > r {
                    self.write_all(buf)?;
                }
            }
        }

        Ok(original_buf_length)
    }

    #[inline]
    fn flush(&mut self) -> Result<(), io::Error> {
        if self.buf_length > 0 {
            self.drain_block()?;
        }

        Ok(())
    }
}

impl<W: Write> From<W> for FromBase64Writer<W> {
    #[inline]
    fn from(reader: W) -> Self {
        FromBase64Writer::new(reader)
    }
}
