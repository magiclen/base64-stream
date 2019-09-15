use std::io::{self, ErrorKind, Read};

const READ_SIZE: usize = 4096 * 3;

/// Read base64 data and decode them to plain data.
#[derive(Educe)]
#[educe(Debug)]
pub struct FromBase64Reader<R: Read> {
    #[educe(Debug(ignore))]
    inner: R,
    buf: Vec<u8>,
}

impl<R: Read> FromBase64Reader<R> {
    #[inline]
    pub fn new(inner: R) -> FromBase64Reader<R> {
        FromBase64Reader {
            inner,
            buf: Vec::new(),
        }
    }
}

impl<R: Read> Read for FromBase64Reader<R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, io::Error> {
        let buf_len = buf.len();

        if buf_len < 3 {
            return Err(io::Error::new(
                ErrorKind::Other,
                "the buffer needs to be equal to or more than 3 bytes",
            ));
        }

        self.buf.clear();

        let actual_max_read_size = buf_len / 3 * 4;

        self.buf.reserve(actual_max_read_size);

        unsafe { self.buf.set_len(actual_max_read_size) };

        let c = {
            let mut buf = &mut self.buf[..actual_max_read_size];

            let mut c = 0;

            loop {
                match self.inner.read(buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        let tmp = buf;
                        buf = &mut tmp[n..];
                        c += n;
                    }
                    Err(ref e) if e.kind() == ErrorKind::Interrupted => {}
                    Err(e) => return Err(e),
                }
            }

            c
        };

        Ok(base64::decode_config_slice(&self.buf[..c], base64::STANDARD, buf)
            .map_err(|err| io::Error::new(ErrorKind::Other, err.to_string()))?)
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> Result<usize, io::Error> {
        self.buf.clear();

        let actual_max_read_size = READ_SIZE;

        self.buf.reserve(actual_max_read_size);

        unsafe { self.buf.set_len(actual_max_read_size) };

        let mut sum = 0;

        loop {
            let c = {
                let mut buf = &mut self.buf[..actual_max_read_size];

                let mut c = 0;

                loop {
                    match self.inner.read(buf) {
                        Ok(0) => break,
                        Ok(n) => {
                            let tmp = buf;
                            buf = &mut tmp[n..];
                            c += n;
                        }
                        Err(ref e) if e.kind() == ErrorKind::Interrupted => {}
                        Err(e) => return Err(e),
                    }
                }

                c
            };

            if c == 0 {
                break;
            }

            let old_len = buf.len();

            base64::decode_config_buf(&self.buf[..c], base64::STANDARD, buf)
                .map_err(|err| io::Error::new(ErrorKind::Other, err.to_string()))?;

            sum += buf.len() - old_len;
        }

        Ok(sum)
    }

    fn read_to_string(&mut self, buf: &mut String) -> Result<usize, io::Error> {
        self.buf.clear();

        let actual_max_read_size = READ_SIZE;

        self.buf.reserve(actual_max_read_size);

        unsafe { self.buf.set_len(actual_max_read_size) };

        let mut sum = 0;

        loop {
            let c = {
                let mut buf = &mut self.buf[..actual_max_read_size];

                let mut c = 0;

                loop {
                    match self.inner.read(buf) {
                        Ok(0) => break,
                        Ok(n) => {
                            let tmp = buf;
                            buf = &mut tmp[n..];
                            c += n;
                        }
                        Err(ref e) if e.kind() == ErrorKind::Interrupted => {}
                        Err(e) => return Err(e),
                    }
                }

                c
            };

            if c == 0 {
                break;
            }

            let mut temp = Vec::new();

            base64::decode_config_buf(&self.buf[..c], base64::STANDARD, &mut temp)
                .map_err(|err| io::Error::new(ErrorKind::Other, err.to_string()))?;

            let temp = String::from_utf8(temp).map_err(|_| {
                io::Error::new(ErrorKind::InvalidData, "stream did not contain valid UTF-8")
            })?;

            sum += temp.len();

            buf.push_str(&temp);
        }

        Ok(sum)
    }
}
