use std::{
    fmt,
    io::{self, ErrorKind, Write},
};

use base64::{
    DecodeError, DecodeSliceError, Engine,
    engine::{GeneralPurpose, general_purpose::STANDARD},
};

#[inline]
fn decode_error_to_io_error(error: DecodeSliceError) -> io::Error {
    io::Error::new(ErrorKind::InvalidData, error)
}

/// Write base64 data and decode them to plain data.
///
/// Call [`finish`](Self::finish) to check the final input and flush the inner writer.
/// A write may succeed with output still buffered if the inner writer returns an error.
/// Dropping this wrapper does not finish or flush it.
pub struct FromBase64Writer<W: Write, const N: usize = 4096> {
    inner:        W,
    buf:          [u8; 4],
    buf_length:   usize,
    temp:         [u8; N],
    temp_length:  usize,
    temp_offset:  usize,
    padding_seen: bool,
    engine:       &'static GeneralPurpose,
}

impl<W: Write, const N: usize> fmt::Debug for FromBase64Writer<W, N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FromBase64Writer")
            .field("buf", &&self.buf[..self.buf_length])
            .field("buf_length", &self.buf_length)
            .field("temp_length", &self.temp_length)
            .field("temp_offset", &self.temp_offset)
            .field("padding_seen", &self.padding_seen)
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
    /// Creates a writer with a buffer of `N` bytes, where `N` must be at least 4.
    #[inline]
    pub fn new2(writer: W) -> FromBase64Writer<W, N> {
        const { assert!(N >= 4, "buffer size N must be at least 4") };
        FromBase64Writer {
            inner:        writer,
            buf:          [0; 4],
            buf_length:   0,
            temp:         [0u8; N],
            temp_length:  0,
            temp_offset:  0,
            padding_seen: false,
            engine:       &STANDARD,
        }
    }
}

impl<W: Write, const N: usize> FromBase64Writer<W, N> {
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

    fn decode_block(&mut self) -> Result<(), io::Error> {
        debug_assert!(self.buf_length > 0);
        debug_assert!(self.temp_length == 0);

        self.temp_length = self
            .engine
            .decode_slice(&self.buf[..self.buf_length], &mut self.temp)
            .map_err(decode_error_to_io_error)?;

        self.padding_seen = self.buf[self.buf_length - 1] == b'=';
        self.buf_length = 0;

        Ok(())
    }

    /// Finishes decoding buffered data and returns the inner writer.
    ///
    /// This method consumes the wrapper even if decoding, writing, or flushing fails.
    #[inline]
    pub fn finish(mut self) -> Result<W, io::Error> {
        self.drain_temp()?;

        if self.buf_length > 0 {
            self.decode_block()?;
            self.drain_temp()?;
        }

        self.inner.flush()?;

        Ok(self.inner)
    }

    /// Returns the inner writer, consuming this wrapper without finishing it.
    ///
    /// Any buffered input or output is lost.
    /// Call [`finish`](Self::finish) instead to decode the final buffered data and flush the inner writer.
    #[inline]
    pub fn into_inner(self) -> W {
        self.inner
    }
}

impl<W: Write, const N: usize> Write for FromBase64Writer<W, N> {
    fn write(&mut self, mut buf: &[u8]) -> Result<usize, io::Error> {
        let original_buf_length = buf.len();

        if buf.is_empty() {
            return Ok(0);
        }

        if self.padding_seen {
            return Err(decode_error_to_io_error(DecodeError::InvalidPadding.into()));
        }

        self.drain_temp()?;

        if self.buf_length > 0 {
            debug_assert!(self.buf_length < 4);

            let original_self_buf_length = self.buf_length;
            let drain_length = (4 - self.buf_length).min(buf.len());

            self.buf[self.buf_length..self.buf_length + drain_length]
                .copy_from_slice(&buf[..drain_length]);

            self.buf_length += drain_length;

            if self.buf_length < 4 {
                return Ok(original_buf_length);
            }

            if let Err(error) = self.decode_block() {
                self.buf_length = original_self_buf_length;

                return Err(error);
            }

            buf = &buf[drain_length..];

            if self.drain_temp().is_err() {
                return Ok(original_buf_length - buf.len());
            }
        }

        while buf.len() >= 4 && !self.padding_seen {
            let max_available_buf_length = (buf.len() & !0b11).min((N / 3) << 2); // (N / 3) * 4

            self.temp_length =
                match self.engine.decode_slice(&buf[..max_available_buf_length], &mut self.temp) {
                    Ok(length) => length,
                    Err(error) if buf.len() == original_buf_length => {
                        return Err(decode_error_to_io_error(error));
                    },
                    Err(_) => return Ok(original_buf_length - buf.len()),
                };

            self.padding_seen = buf[max_available_buf_length - 1] == b'=';
            buf = &buf[max_available_buf_length..];

            // The input is already buffered, so report it even if the output must wait.
            if self.drain_temp().is_err() {
                return Ok(original_buf_length - buf.len());
            }
        }

        if self.padding_seen {
            return Ok(original_buf_length - buf.len());
        }

        self.buf[..buf.len()].copy_from_slice(buf);
        self.buf_length = buf.len();

        Ok(original_buf_length)
    }

    /// Flushes pending output without decoding an incomplete input block.
    ///
    /// Use [`finish`](Self::finish) to check the final input.
    #[inline]
    fn flush(&mut self) -> Result<(), io::Error> {
        self.drain_temp()?;

        self.inner.flush()
    }
}

impl<W: Write> From<W> for FromBase64Writer<W> {
    #[inline]
    fn from(reader: W) -> Self {
        FromBase64Writer::new(reader)
    }
}
