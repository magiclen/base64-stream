use std::io::{self, Read, ErrorKind};

const READ_SIZE: usize = 4096 * 3;

/// Read any data and encode them to base64 data.
#[derive(Derivative)]
#[derivative(Debug)]
pub struct ToBase64Reader<R: Read> {
    #[derivative(Debug = "ignore")]
    inner: R,
    buf: Vec<u8>,
}

impl<R: Read> ToBase64Reader<R> {
    #[inline]
    pub fn new(inner: R) -> ToBase64Reader<R> {
        ToBase64Reader {
            inner,
            buf: Vec::new(),
        }
    }
}

impl<R: Read> Read for ToBase64Reader<R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, io::Error> {
        let buf_len = buf.len();

        if buf_len < 4 {
            return Err(io::Error::new(ErrorKind::Other, "the buffer needs to be equal to or more than 4 bytes"));
        }

        self.buf.clear();

        let actual_max_read_size = buf_len / 4 * 3;

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

        Ok(base64::encode_config_slice(&self.buf[..c], base64::STANDARD, buf))
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

            let cc = (c + 2) / 3 * 4;

            buf.reserve(cc);

            let old_len = buf.len();

            unsafe { buf.set_len(old_len + cc) };

            base64::encode_config_slice(&self.buf[..c], base64::STANDARD, &mut buf[old_len..]);

            sum += cc;
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

            let cc = (c + 2) / 3 * 4;

            buf.reserve(cc);

            base64::encode_config_buf(&self.buf[..c], base64::STANDARD, buf);

            sum += cc;
        }

        Ok(sum)
    }
}