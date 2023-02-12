use std::intrinsics::copy_nonoverlapping;
use std::io::{self, ErrorKind, Write};

use generic_array::typenum::{IsGreaterOrEqual, True, U4, U4096};
use generic_array::{ArrayLength, GenericArray};

use base64::{self,
    Engine,
};

/// Write base64 data and decode them to plain data.
#[derive(Educe)]
#[educe(Debug)]
pub struct FromBase64Writer<
    W: Write,
    N: ArrayLength<u8> + IsGreaterOrEqual<U4, Output = True> = U4096,
> {
    #[educe(Debug(ignore))]
    inner: W,
    buf: [u8; 4],
    buf_length: usize,
    temp: GenericArray<u8, N>,
    #[educe(Debug(ignore))]
    engine: &'static base64::engine::general_purpose::GeneralPurpose,
}

impl<W: Write> FromBase64Writer<W> {
    #[inline]
    pub fn new(writer: W) -> FromBase64Writer<W> {
        Self::new2(writer, &base64::engine::general_purpose::STANDARD)
    }
}

impl<W: Write, N: ArrayLength<u8> + IsGreaterOrEqual<U4, Output = True>> FromBase64Writer<W, N> {
    #[inline]
    pub fn new2(writer: W, engine: &'static base64::engine::general_purpose::GeneralPurpose) -> FromBase64Writer<W, N> {
        FromBase64Writer {
            inner: writer,
            buf: [0; 4],
            buf_length: 0,
            temp: GenericArray::default(),
            engine,
        }
    }
}

impl<W: Write, N: ArrayLength<u8> + IsGreaterOrEqual<U4, Output = True>> FromBase64Writer<W, N> {
    fn drain_block(&mut self) -> Result<(), io::Error> {
        debug_assert!(self.buf_length > 0);

        let decode_length = self.engine.decode_slice(
            self.buf[..self.buf_length].as_ref(),
            &mut self.temp,
        )
        .map_err(|err| io::Error::new(ErrorKind::Other, err))?;

        self.inner.write_all(&self.temp[..decode_length])?;

        self.buf_length = 0;

        Ok(())
    }
}

impl<W: Write, N: ArrayLength<u8> + IsGreaterOrEqual<U4, Output = True>> Write
    for FromBase64Writer<W, N>
{
    fn write(&mut self, mut buf: &[u8]) -> Result<usize, io::Error> {
        let original_buf_length = buf.len();

        if self.buf_length == 0 {
            while buf.len() >= 4 {
                let max_available_buf_length = (buf.len() & !0b11).min((N::USIZE / 3) << 2); // (N::USIZE / 3) * 4

                let decode_length = self.engine.decode_slice(
                    buf[..max_available_buf_length].as_ref(),
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
