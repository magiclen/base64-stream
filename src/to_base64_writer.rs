use std::io::{self, Write};

/// Write any data and encode them to base64 data.
#[derive(Educe)]
#[educe(Debug)]
pub struct ToBase64Writer<W: Write> {
    #[educe(Debug(ignore))]
    inner: W,
    buf: Vec<u8>,
    remaining: Vec<u8>,
}

impl<W: Write> ToBase64Writer<W> {
    #[inline]
    pub fn new(inner: W) -> ToBase64Writer<W> {
        ToBase64Writer {
            inner,
            buf: Vec::new(),
            remaining: Vec::new(),
        }
    }
}

impl<W: Write> Write for ToBase64Writer<W> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, io::Error> {
        let remaining_len = self.remaining.len();
        let buf_len = buf.len();

        if remaining_len > 0 {
            let new_buf_len = remaining_len + buf_len;

            if new_buf_len < 3 {
                self.remaining.extend_from_slice(&buf);
                Ok(buf_len)
            } else {
                let actual_max_write_size = new_buf_len / 3 * 3;

                let buf_end = actual_max_write_size - remaining_len;

                self.remaining.extend_from_slice(&buf[..buf_end]);

                let c = actual_max_write_size / 3 * 4;

                self.buf.clear();

                self.buf.reserve(c);

                unsafe { self.buf.set_len(c); }

                base64::encode_config_slice(&self.remaining, base64::STANDARD, &mut self.buf);

                self.remaining.clear();

                self.inner.write_all(&self.buf)?;

                if buf_len != buf_end {
                    self.remaining.extend_from_slice(&buf[buf_end..]);
                }

                Ok(buf_len)
            }
        } else if buf_len < 3 {
            self.remaining.extend_from_slice(&buf);
            Ok(buf_len)
        } else {
            let actual_max_write_size = buf_len / 3 * 3;

            let buf = if actual_max_write_size == buf_len {
                buf
            } else {
                self.remaining.extend_from_slice(&buf[actual_max_write_size..]);
                &buf[..actual_max_write_size]
            };

            let c = actual_max_write_size / 3 * 4;

            self.buf.clear();

            self.buf.reserve(c);

            unsafe { self.buf.set_len(c); }

            base64::encode_config_slice(buf, base64::STANDARD, &mut self.buf);

            self.inner.write_all(&self.buf)?;

            Ok(buf_len)
        }
    }

    fn flush(&mut self) -> Result<(), io::Error> {
        let remaining_len = self.remaining.len();

        if remaining_len > 0 {
            let c = (remaining_len + 2) / 3 * 4;

            self.buf.clear();

            self.buf.reserve(c);

            unsafe { self.buf.set_len(c); }

            base64::encode_config_slice(&self.remaining, base64::STANDARD, &mut self.buf);

            self.remaining.clear();

            self.inner.write_all(&self.buf)?;

            self.inner.flush()
        } else {
            self.inner.flush()
        }
    }
}

impl<W: Write> Drop for ToBase64Writer<W> {
    fn drop(&mut self) {
        self.flush().unwrap()
    }
}