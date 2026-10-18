Base64 Stream
====================

[![CI](https://github.com/magiclen/base64-stream/actions/workflows/ci.yml/badge.svg)](https://github.com/magiclen/base64-stream/actions/workflows/ci.yml)

To encode/decode large data with the standard Base64 encoding.

## Upgrading to 7.0.0

The public `base64` re-export now uses version 0.23.1 instead of 0.22.
Use `DecodeError::InvalidLastSymbol { offset, symbol, .. }` instead of `DecodeError::InvalidLastSymbol(offset, symbol)` in match patterns.

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
use std::io::Write;

use base64_stream::ToBase64Writer;

let test_data = b"Hi there, this is a simple sentence used for testing this crate. I hope all cases are correct.".as_ref();

let mut writer = ToBase64Writer::new(Vec::new());

writer.write_all(test_data).unwrap();

let base64 = writer.finish().unwrap();

assert_eq!(b"SGkgdGhlcmUsIHRoaXMgaXMgYSBzaW1wbGUgc2VudGVuY2UgdXNlZCBmb3IgdGVzdGluZyB0aGlzIGNyYXRlLiBJIGhvcGUgYWxsIGNhc2VzIGFyZSBjb3JyZWN0Lg==", base64.as_slice());
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
use std::io::Write;

use base64_stream::FromBase64Writer;

let base64 = b"SGkgdGhlcmUsIHRoaXMgaXMgYSBzaW1wbGUgc2VudGVuY2UgdXNlZCBmb3IgdGVzdGluZyB0aGlzIGNyYXRlLiBJIGhvcGUgYWxsIGNhc2VzIGFyZSBjb3JyZWN0Lg==".as_ref();

let mut writer = FromBase64Writer::new(Vec::new());

writer.write_all(base64).unwrap();

let test_data = writer.finish().unwrap();

assert_eq!(b"Hi there, this is a simple sentence used for testing this crate. I hope all cases are correct.", test_data.as_slice());
```

## Change the Buffer Size

The default buffer size is 4096 bytes. If you want to change that, you can use the `new2` associated function and define a length explicitly to create an instance of the above structs.
The buffer size must be at least 4 bytes.

For example, to change the buffer size to 256 bytes,

```rust
use std::io::{Cursor, Read};

use base64_stream::ToBase64Reader;

let test_data = b"Hi there, this is a simple sentence used for testing this crate. I hope all cases are correct.".to_vec();

let mut reader: ToBase64Reader<_, 256> = ToBase64Reader::new2(Cursor::new(test_data));

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
