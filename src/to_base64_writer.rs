use std::intrinsics::copy_nonoverlapping;
use std::io::{self, Write};

use crate::{fmt, BUFFER_SIZE};

// Do not change these
const TEMP_SIZE: usize = BUFFER_SIZE;
const MAX_ENCODE_SIZE: usize = (TEMP_SIZE >> 2) * 3; // (TEMP_SIZE / 4) * 3

/// Write base64 data and encode them to plain data.
#[derive(Educe)]
#[educe(Debug)]
pub struct ToBase64Writer<W: Write> {
    #[educe(Debug(ignore))]
    inner: W,
    buf: [u8; 3],
    buf_length: usize,
    #[educe(Debug(method = "fmt"))]
    temp: [u8; TEMP_SIZE],
}

impl<W: Write> ToBase64Writer<W> {
    #[inline]
    pub fn new(writer: W) -> ToBase64Writer<W> {
        ToBase64Writer {
            inner: writer,
            buf: [0; 3],
            buf_length: 0,
            temp: [0; TEMP_SIZE],
        }
    }
}

impl<R: Write> ToBase64Writer<R> {
    fn drain_block(&mut self) -> Result<(), io::Error> {
        debug_assert!(self.buf_length > 0);

        let encode_length = base64::encode_config_slice(
            &self.buf[..self.buf_length],
            base64::STANDARD,
            &mut self.temp,
        );

        self.inner.write_all(&self.temp[..encode_length])?;

        self.buf_length = 0;

        Ok(())
    }
}

impl<R: Write> Write for ToBase64Writer<R> {
    fn write(&mut self, mut buf: &[u8]) -> Result<usize, io::Error> {
        let original_buf_length = buf.len();

        if self.buf_length == 0 {
            while buf.len() >= 3 {
                let max_available_buf_length = (buf.len() - (buf.len() % 3)).min(MAX_ENCODE_SIZE);

                let encode_length = base64::encode_config_slice(
                    &buf[..max_available_buf_length],
                    base64::STANDARD,
                    &mut self.temp,
                );

                buf = &buf[max_available_buf_length..];

                self.inner.write_all(&self.temp[..encode_length])?;
            }

            let buf_length = buf.len();

            if buf_length > 0 {
                unsafe {
                    copy_nonoverlapping(buf.as_ptr(), self.buf.as_mut_ptr(), buf_length);
                }

                self.buf_length = buf_length;
            }
        } else {
            debug_assert!(self.buf_length < 3);

            let r = 3 - self.buf_length;

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

impl<R: Write> From<R> for ToBase64Writer<R> {
    #[inline]
    fn from(reader: R) -> Self {
        ToBase64Writer::new(reader)
    }
}
