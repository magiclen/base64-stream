use std::{
    fmt,
    io::{self, ErrorKind, Write},
};

use base64::{
    Engine,
    engine::{GeneralPurpose, general_purpose::STANDARD},
};

/// Write any data and encode them to base64 data.
///
/// Call [`finish`](Self::finish) to encode the final bytes and flush the inner writer.
/// A write may succeed with output still buffered if the inner writer returns an error.
/// Dropping this wrapper does not finish or flush it.
pub struct ToBase64Writer<W: Write, const N: usize = 4096> {
    inner:       W,
    buf:         [u8; 3],
    buf_length:  usize,
    temp:        [u8; N],
    temp_length: usize,
    temp_offset: usize,
    engine:      &'static GeneralPurpose,
}

impl<W: Write, const N: usize> fmt::Debug for ToBase64Writer<W, N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ToBase64Writer")
            .field("buf", &&self.buf[..self.buf_length])
            .field("buf_length", &self.buf_length)
            .field("temp_length", &self.temp_length)
            .field("temp_offset", &self.temp_offset)
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
    /// Creates a writer with a buffer of `N` bytes, where `N` must be at least 4.
    #[inline]
    pub fn new2(writer: W) -> ToBase64Writer<W, N> {
        const { assert!(N >= 4, "buffer size N must be at least 4") };
        ToBase64Writer {
            inner:       writer,
            buf:         [0; 3],
            buf_length:  0,
            temp:        [0u8; N],
            temp_length: 0,
            temp_offset: 0,
            engine:      &STANDARD,
        }
    }
}

impl<W: Write, const N: usize> ToBase64Writer<W, N> {
    fn drain_temp(&mut self) -> Result<(), io::Error> {
        while self.temp_offset < self.temp_length {
            match self.inner.write(&self.temp[self.temp_offset..self.temp_length]) {
                Ok(0) => return Err(ErrorKind::WriteZero.into()),
                Ok(length) => self.temp_offset += length,
                Err(error) if error.kind() == ErrorKind::Interrupted => {},
                Err(error) => return Err(error),
            }
        }

        self.temp_length = 0;
        self.temp_offset = 0;

        Ok(())
    }

    fn encode_block(&mut self) {
        debug_assert!(self.buf_length > 0);
        debug_assert!(self.temp_length == 0);

        self.temp_length =
            self.engine.encode_slice(&self.buf[..self.buf_length], &mut self.temp).unwrap();

        self.buf_length = 0;
    }

    /// Finishes encoding buffered data and returns the inner writer.
    ///
    /// This method consumes the wrapper even if writing or flushing fails.
    #[inline]
    pub fn finish(mut self) -> Result<W, io::Error> {
        self.drain_temp()?;

        if self.buf_length > 0 {
            self.encode_block();
            self.drain_temp()?;
        }

        self.inner.flush()?;

        Ok(self.inner)
    }

    /// Returns the inner writer, consuming this wrapper without finishing it.
    ///
    /// Any buffered input or output is lost.
    /// Call [`finish`](Self::finish) instead to encode the final buffered data and flush the inner writer.
    #[inline]
    pub fn into_inner(self) -> W {
        self.inner
    }
}

impl<W: Write, const N: usize> Write for ToBase64Writer<W, N> {
    fn write(&mut self, mut buf: &[u8]) -> Result<usize, io::Error> {
        let original_buf_length = buf.len();

        if buf.is_empty() {
            return Ok(0);
        }

        self.drain_temp()?;

        if self.buf_length > 0 {
            debug_assert!(self.buf_length < 3);

            let drain_length = (3 - self.buf_length).min(buf.len());

            self.buf[self.buf_length..self.buf_length + drain_length]
                .copy_from_slice(&buf[..drain_length]);

            buf = &buf[drain_length..];

            self.buf_length += drain_length;

            if self.buf_length < 3 {
                return Ok(original_buf_length);
            }

            self.encode_block();

            if self.drain_temp().is_err() {
                return Ok(original_buf_length - buf.len());
            }
        }

        while buf.len() >= 3 {
            let max_available_buf_length = (buf.len() - (buf.len() % 3)).min((N >> 2) * 3); // (N / 4) * 3

            self.temp_length =
                self.engine.encode_slice(&buf[..max_available_buf_length], &mut self.temp).unwrap();

            buf = &buf[max_available_buf_length..];

            // The input is already buffered, so report it even if the output must wait.
            if self.drain_temp().is_err() {
                return Ok(original_buf_length - buf.len());
            }
        }

        self.buf[..buf.len()].copy_from_slice(buf);
        self.buf_length = buf.len();

        Ok(original_buf_length)
    }

    /// Flushes pending output without encoding an incomplete input block.
    ///
    /// Use [`finish`](Self::finish) to encode the final bytes and add padding.
    #[inline]
    fn flush(&mut self) -> Result<(), io::Error> {
        self.drain_temp()?;

        self.inner.flush()
    }
}

impl<W: Write> From<W> for ToBase64Writer<W> {
    #[inline]
    fn from(reader: W) -> Self {
        ToBase64Writer::new(reader)
    }
}
