use std::intrinsics::{copy, copy_nonoverlapping};
use std::io::{self, ErrorKind, Read};

use generic_array::typenum::{IsGreaterOrEqual, True, U4, U4096};
use generic_array::{ArrayLength, GenericArray};

use base64::{self,
    Engine,
};

/// Read any data and encode them to base64 data.
#[derive(Educe)]
#[educe(Debug)]
pub struct ToBase64Reader<R: Read, N: ArrayLength<u8> + IsGreaterOrEqual<U4, Output = True> = U4096>
{
    #[educe(Debug(ignore))]
    inner: R,
    buf: GenericArray<u8, N>,
    buf_length: usize,
    buf_offset: usize,
    temp: [u8; 3],
    temp_length: usize,
    #[educe(Debug(ignore))]
    engine: &'static base64::engine::general_purpose::GeneralPurpose,
}

impl<R: Read> ToBase64Reader<R> {
    #[inline]
    pub fn new(reader: R) -> ToBase64Reader<R> {
        Self::new2(reader, &base64::engine::general_purpose::STANDARD)
    }
}

impl<R: Read, N: ArrayLength<u8> + IsGreaterOrEqual<U4, Output = True>> ToBase64Reader<R, N> {
    #[inline]
    pub fn new2(reader: R, engine: &'static base64::engine::general_purpose::GeneralPurpose) -> ToBase64Reader<R, N> {
        ToBase64Reader {
            inner: reader,
            buf: GenericArray::default(),
            buf_length: 0,
            buf_offset: 0,
            temp: [0; 3],
            temp_length: 0,
            engine,
        }
    }
}

impl<R: Read, N: ArrayLength<u8> + IsGreaterOrEqual<U4, Output = True>> ToBase64Reader<R, N> {
    fn buf_left_shift(&mut self, distance: usize) {
        debug_assert!(self.buf_length >= distance);

        self.buf_offset += distance;

        if self.buf_offset >= N::USIZE - 4 {
            unsafe {
                copy(
                    self.buf.as_ptr().add(self.buf_offset),
                    self.buf.as_mut_ptr(),
                    self.buf_length,
                );
            }

            self.buf_offset = 0;
        }

        self.buf_length -= distance;
    }

    #[inline]
    fn drain_temp<'a>(&mut self, buf: &'a mut [u8]) -> &'a mut [u8] {
        debug_assert!(self.temp_length > 0);
        debug_assert!(!buf.is_empty());

        let drain_length = buf.len().min(self.temp_length);

        unsafe {
            copy_nonoverlapping(self.temp.as_ptr(), buf.as_mut_ptr(), drain_length);
        }

        self.temp_length -= drain_length;

        unsafe {
            copy(
                self.temp.as_ptr().add(self.temp_length),
                self.temp.as_mut_ptr(),
                self.temp_length,
            );
        }

        &mut buf[drain_length..]
    }

    #[inline]
    fn drain_block<'a>(&mut self, mut buf: &'a mut [u8]) -> &'a mut [u8] {
        debug_assert!(self.buf_length > 0);
        debug_assert!(self.temp_length == 0);
        debug_assert!(!buf.is_empty());

        let drain_length = self.buf_length.min(3);

        let mut b = [0; 4];

        let encode_length = 
         match self.engine.encode_slice(
                self.buf[self.buf_offset..(self.buf_offset + drain_length)].as_ref(),
                &mut b,
            )
        {
          Ok(val)  => val,
          Err(err) => panic!("Error: {}", err),
        };
        // let encode_length = base64::encode_config_slice(
        //     &self.buf[self.buf_offset..(self.buf_offset + drain_length)],
        //     base64::STANDARD,
        //     &mut b,
        // );

        self.buf_left_shift(drain_length);

        let buf_length = buf.len();

        if buf_length >= encode_length {
            unsafe {
                copy_nonoverlapping(b.as_ptr(), buf.as_mut_ptr(), encode_length);
            }

            buf = &mut buf[encode_length..];
        } else {
            unsafe {
                copy_nonoverlapping(b.as_ptr(), buf.as_mut_ptr(), buf_length);
            }

            buf = &mut buf[buf_length..];

            self.temp_length = encode_length - buf_length;

            unsafe {
                copy_nonoverlapping(
                    b.as_ptr().add(buf_length),
                    self.temp.as_mut_ptr(),
                    self.temp_length,
                );
            }
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

            let encode_length = 
                match self.engine.encode_slice(
                    self.buf[self.buf_offset..(self.buf_offset + drain_length)].as_ref(),
                    buf,
                  )
                {
                    Ok(val)  => val,
                    Err(err) => panic!("Error: {}", err),
                };
      
            // let encode_length = base64::encode_config_slice(
            //     &self.buf[self.buf_offset..(self.buf_offset + drain_length)],
            //     base64::STANDARD,
            //     buf,
            // );

            buf = &mut buf[encode_length..];

            self.buf_left_shift(drain_length);
        }

        if !buf.is_empty() && self.buf_length >= 3 {
            self.drain_block(buf)
        } else {
            buf
        }
    }

    #[inline]
    fn drain_end<'a>(&mut self, mut buf: &'a mut [u8]) -> &'a mut [u8] {
        if buf.is_empty() {
            return buf;
        }

        if self.temp_length > 0 {
            buf = self.drain_temp(buf);
        }

        if !buf.is_empty() && self.buf_length > 0 {
            self.drain_block(buf)
        } else {
            buf
        }
    }
}

impl<R: Read, N: ArrayLength<u8> + IsGreaterOrEqual<U4, Output = True>> Read
    for ToBase64Reader<R, N>
{
    fn read(&mut self, mut buf: &mut [u8]) -> Result<usize, io::Error> {
        let original_buf_length = buf.len();

        while self.buf_length < 3 {
            match self.inner.read(&mut self.buf[(self.buf_offset + self.buf_length)..]) {
                Ok(0) => {
                    buf = self.drain_end(buf);

                    return Ok(original_buf_length - buf.len());
                }
                Ok(c) => self.buf_length += c,
                Err(ref e) if e.kind() == ErrorKind::Interrupted => {}
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
