use std::intrinsics::copy_nonoverlapping;
use std::io::{self, Write};

use crate::generic_array::typenum::U4096;
use crate::generic_array::{ArrayLength, GenericArray};

/// Write base64 data and encode them to plain data.
#[derive(Educe)]
#[educe(Debug)]
pub struct ToBase64Writer<W: Write, N: ArrayLength<u8> = U4096> {
    #[educe(Debug(ignore))]
    inner: W,
    buf: [u8; 3],
    buf_length: usize,
    temp: GenericArray<u8, N>,
}

impl<W: Write> ToBase64Writer<W> {
    #[inline]
    pub fn new(writer: W) -> ToBase64Writer<W> {
        ToBase64Writer {
            inner: writer,
            buf: [0; 3],
            buf_length: 0,
            temp: GenericArray::default(),
        }
    }
}

impl<W: Write, N: ArrayLength<u8>> ToBase64Writer<W, N> {
    #[inline]
    pub fn new2(writer: W) -> Result<ToBase64Writer<W, N>, &'static str> {
        if N::USIZE >= 4 {
            Ok(ToBase64Writer {
                inner: writer,
                buf: [0; 3],
                buf_length: 0,
                temp: GenericArray::default(),
            })
        } else {
            Err("The buffer size must be bigger than or equal to 4.")
        }
    }
}

impl<W: Write, N: ArrayLength<u8>> ToBase64Writer<W, N> {
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

impl<W: Write, N: ArrayLength<u8>> Write for ToBase64Writer<W, N> {
    fn write(&mut self, mut buf: &[u8]) -> Result<usize, io::Error> {
        let original_buf_length = buf.len();

        if self.buf_length == 0 {
            while buf.len() >= 3 {
                let max_available_buf_length =
                    (buf.len() - (buf.len() % 3)).min((N::USIZE >> 2) * 3); // (N::USIZE / 4) * 3

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

impl<W: Write> From<W> for ToBase64Writer<W> {
    #[inline]
    fn from(reader: W) -> Self {
        ToBase64Writer::new(reader)
    }
}
