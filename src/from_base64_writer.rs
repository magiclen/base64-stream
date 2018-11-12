use std::io::{self, Write, ErrorKind};

/// Write base64 data and decode them to plain data.
pub struct FromBase64Writer<W: Write> {
    inner: W,
    buf: Vec<u8>,
    remaining: Vec<u8>,
}

impl<W: Write> FromBase64Writer<W> {
    pub fn new(inner: W) -> FromBase64Writer<W> {
        FromBase64Writer {
            inner,
            buf: Vec::new(),
            remaining: Vec::new(),
        }
    }
}

impl<W: Write> Write for FromBase64Writer<W> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, io::Error> {
        let remaining_len = self.remaining.len();
        let buf_len = buf.len();

        if remaining_len > 0 {
            let new_buf_len = remaining_len + buf_len;

            if new_buf_len < 4 {
                self.remaining.extend_from_slice(&buf);
                Ok(buf_len)
            } else {
                let actual_max_write_size = new_buf_len / 3 * 3;

                let buf_end = actual_max_write_size - remaining_len;

                self.remaining.extend_from_slice(&buf[..buf_end]);

                self.buf.clear();

                base64::decode_config_buf(&self.remaining, base64::STANDARD, &mut self.buf).map_err(|err| io::Error::new(ErrorKind::Other, err.to_string()))?;

                self.inner.write(&self.buf)?;

                if buf_len != buf_end {
                    self.remaining.extend_from_slice(&buf[buf_end..]);
                }

                Ok(buf_len)
            }
        } else {
            if buf_len < 4 {
                self.remaining.extend_from_slice(&buf);
                Ok(buf_len)
            } else {
                let actual_max_write_size = buf_len / 4 * 4;

                let buf = if actual_max_write_size == buf_len {
                    buf
                } else {
                    self.remaining.extend_from_slice(&buf[actual_max_write_size..]);
                    &buf[..actual_max_write_size]
                };

                self.buf.clear();

                base64::decode_config_buf(buf, base64::STANDARD, &mut self.buf).map_err(|err| io::Error::new(ErrorKind::Other, err.to_string()))?;

                self.inner.write(&self.buf)?;

                Ok(buf_len)
            }
        }
    }

    fn flush(&mut self) -> Result<(), io::Error> {
        let remaining_len = self.remaining.len();

        if remaining_len > 0 {
            self.buf.clear();

            base64::decode_config_buf(&self.remaining, base64::STANDARD, &mut self.buf).map_err(|err| io::Error::new(ErrorKind::Other, err.to_string()))?;

            self.remaining.clear();

            self.inner.write(&self.buf)?;

            self.inner.flush()
        } else {
            self.inner.flush()
        }
    }
}

impl<W: Write> Drop for FromBase64Writer<W> {
    fn drop(&mut self) {
        self.flush().unwrap()
    }
}