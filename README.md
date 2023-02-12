Base64 Stream
====================

[![CI](https://github.com/magiclen/base64-stream/actions/workflows/ci.yml/badge.svg)](https://github.com/magiclen/base64-stream/actions/workflows/ci.yml)

To encode/decode large data with the standard Base64 encoding.

## Examples

### Encode

#### ToBase64Reader

```rust
use std::io::{Cursor, Read};

use base64_stream::ToBase64Reader;

let test_data = b"Hi there, this is a simple sentence used for testing this crate. I hope all cases are correct.".to_vec();

let mut reader = ToBase64Reader::new(Cursor::new(test_data));

let mut base64 = String::new();

reader.read_to_string(&mut base64).unwrap();

assert_eq!("SGkgdGhlcmUsIHRoaXMgaXMgYSBzaW1wbGUgc2VudGVuY2UgdXNlZCBmb3IgdGVzdGluZyB0aGlzIGNyYXRlLiBJIGhvcGUgYWxsIGNhc2VzIGFyZSBjb3JyZWN0Lg==", base64);
```

#### ToBase64Writer

```rust
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

use base64_stream::ToBase64Writer;

const DATA_FOLDER: &str = "data";
const ENCODE_OUTPUT: &str = "encode_output.txt";

let test_data = b"Hi there, this is a simple sentence used for testing this crate. I hope all cases are correct.".as_ref();

let file_path = Path::new("tests").join(DATA_FOLDER).join(ENCODE_OUTPUT);

let base64 = File::create(file_path.as_path()).unwrap();

let mut writer = ToBase64Writer::new(base64);

writer.write_all(test_data).unwrap();

writer.flush().unwrap(); // the flush method is only used when the full plain data has been written

assert_eq!("SGkgdGhlcmUsIHRoaXMgaXMgYSBzaW1wbGUgc2VudGVuY2UgdXNlZCBmb3IgdGVzdGluZyB0aGlzIGNyYXRlLiBJIGhvcGUgYWxsIGNhc2VzIGFyZSBjb3JyZWN0Lg==", fs::read_to_string(file_path).unwrap());
```

### Decode

#### FromBase64Reader

```rust
use std::io::Cursor;

use std::io::Read;

use base64_stream::FromBase64Reader;

let base64 = b"SGkgdGhlcmUsIHRoaXMgaXMgYSBzaW1wbGUgc2VudGVuY2UgdXNlZCBmb3IgdGVzdGluZyB0aGlzIGNyYXRlLiBJIGhvcGUgYWxsIGNhc2VzIGFyZSBjb3JyZWN0Lg==".to_vec();

let mut reader = FromBase64Reader::new(Cursor::new(base64));

let mut test_data = String::new();

reader.read_to_string(&mut test_data).unwrap();

assert_eq!("Hi there, this is a simple sentence used for testing this crate. I hope all cases are correct.", test_data);
```

#### FromBase64Writer

```rust
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

use base64_stream::FromBase64Writer;

const DATA_FOLDER: &str = "data";
const DECODE_OUTPUT: &str = "decode_output.txt";

let base64 = b"SGkgdGhlcmUsIHRoaXMgaXMgYSBzaW1wbGUgc2VudGVuY2UgdXNlZCBmb3IgdGVzdGluZyB0aGlzIGNyYXRlLiBJIGhvcGUgYWxsIGNhc2VzIGFyZSBjb3JyZWN0Lg==".as_ref();

let file_path = Path::new("tests").join(DATA_FOLDER).join(DECODE_OUTPUT);

let test_data = File::create(file_path.as_path()).unwrap();

let mut writer = FromBase64Writer::new(test_data);

writer.write_all(base64).unwrap();

writer.flush().unwrap(); // the flush method is only used when the full base64 data has been written

assert_eq!("Hi there, this is a simple sentence used for testing this crate. I hope all cases are correct.", fs::read_to_string(file_path).unwrap());
```

## Change the Buffer Size

The default buffer size is 4096 bytes. If you want to change that, you can use the `new2` associated function and define a length explicitly to create an instance of the above structs.

For example, to change the buffer size to 256 bytes,

```rust
use std::io::{Cursor, Read};

use base64_stream::ToBase64Reader;
use base64_stream::generic_array::typenum::U256;

let test_data = b"Hi there, this is a simple sentence used for testing this crate. I hope all cases are correct.".to_vec();

let mut reader: ToBase64Reader<_, U256> = ToBase64Reader::new2(Cursor::new(test_data), &base64::engine::general_purpose::STANDARD);

let mut base64 = String::new();

reader.read_to_string(&mut base64).unwrap();

assert_eq!("SGkgdGhlcmUsIHRoaXMgaXMgYSBzaW1wbGUgc2VudGVuY2UgdXNlZCBmb3IgdGVzdGluZyB0aGlzIGNyYXRlLiBJIGhvcGUgYWxsIGNhc2VzIGFyZSBjb3JyZWN0Lg==", base64);
```

## Crates.io

https://crates.io/crates/base64-stream

## Documentation

https://docs.rs/base64-stream

## License

[MIT](LICENSE)