use std::intrinsics::copy_nonoverlapping;
use std::io::{self, ErrorKind, Write};

use crate::{fmt, BUFFER_SIZE};

// Do not change these
const TEMP_SIZE: usize = BUFFER_SIZE;
const MAX_DECODE_SIZE: usize = (TEMP_SIZE / 3) << 2; // (TEMP_SIZE / 3) * 4

/// Write base64 data and decode them to plain data.
#[derive(Educe)]
#[educe(Debug)]
pub struct FromBase64Writer<W: Write> {
    #[educe(Debug(ignore))]
    inner: W,
    buf: [u8; 4],
    buf_length: usize,
    #[educe(Debug(method = "fmt"))]
    temp: [u8; TEMP_SIZE],
}

impl<W: Write> FromBase64Writer<W> {
    #[inline]
    pub fn new(writer: W) -> FromBase64Writer<W> {
        FromBase64Writer {
            inner: writer,
            buf: [0; 4],
            buf_length: 0,
            temp: [0; TEMP_SIZE],
        }
    }
}

impl<R: Write> FromBase64Writer<R> {
    fn drain_block(&mut self) -> Result<(), io::Error> {
        debug_assert!(self.buf_length > 0);

        let decode_length = base64::decode_config_slice(
            &self.buf[..self.buf_length],
            base64::STANDARD,
            &mut self.temp,
        )
        .map_err(|err| io::Error::new(ErrorKind::Other, err))?;

        self.inner.write_all(&self.temp[..decode_length])?;

        self.buf_length = 0;

        Ok(())
    }
}

impl<R: Write> Write for FromBase64Writer<R> {
    fn write(&mut self, mut buf: &[u8]) -> Result<usize, io::Error> {
        let original_buf_length = buf.len();

        if self.buf_length == 0 {
            while buf.len() >= 4 {
                let max_available_buf_length = (buf.len() & !0b11).min(MAX_DECODE_SIZE);

                let decode_length = base64::decode_config_slice(
                    &buf[..max_available_buf_length],
                    base64::STANDARD,
                    &mut self.temp,
                )
                .map_err(|err| io::Error::new(ErrorKind::Other, err))?;

                buf = &buf[max_available_buf_length..];

                self.inner.write_all(&self.temp[..decode_length])?;
            }

            let buf_length = buf.len();

            if buf_length > 0 {
                unsafe {
                    copy_nonoverlapping(buf.as_ptr(), self.buf.as_mut_ptr(), buf_length);
                }

                self.buf_length = buf_length;
            }
        } else {
            debug_assert!(self.buf_length < 4);

            let r = 4 - self.buf_length;

            let buf_length = buf.len();

            let drain_length = r.min(buf_length);

            unsafe {
                copy_nonoverlapping(
                    buf.as_ptr(),
                    self.buf.as_mut_ptr().add(self.buf_length),
                    drain_length,
                );
            }

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

    fn flush(&mut self) -> Result<(), io::Error> {
        if self.buf_length > 0 {
            self.drain_block()?;
        }

        Ok(())
    }
}

impl<R: Write> From<R> for FromBase64Writer<R> {
    #[inline]
    fn from(reader: R) -> Self {
        FromBase64Writer::new(reader)
    }
}
