/*!
# Base64 Stream

To encode/decode large data with the standard Base64 encoding.

## Examples

### Encode

#### ToBase64Reader

```rust
extern crate base64_stream;

use std::io::Cursor;

use std::io::Read;

use base64_stream::ToBase64Reader;

let test_data = b"Hi there, this is a simple sentence used for testing this crate. I hope all cases are correct.".to_vec();

let mut reader = ToBase64Reader::new(Cursor::new(test_data));

let mut base64 = [0u8; 4096];

let c = reader.read(&mut base64).unwrap();

assert_eq!(b"SGkgdGhlcmUsIHRoaXMgaXMgYSBzaW1wbGUgc2VudGVuY2UgdXNlZCBmb3IgdGVzdGluZyB0aGlzIGNyYXRlLiBJIGhvcGUgYWxsIGNhc2VzIGFyZSBjb3JyZWN0Lg==".to_vec(), base64[..c].to_vec());
```

#### ToBase64Writer

```rust
extern crate base64_stream;

use std::fs::{self, File};

use std::io::Write;

use base64_stream::ToBase64Writer;

let test_data = b"Hi there, this is a simple sentence used for testing this crate. I hope all cases are correct.".as_ref();

let base64 = File::create("encode_output.txt").unwrap();

let mut writer = ToBase64Writer::new(base64);

writer.write(test_data).unwrap();

writer.flush().unwrap(); // the flush method is only used when the full base64 data has been written

drop(writer);

assert_eq!("SGkgdGhlcmUsIHRoaXMgaXMgYSBzaW1wbGUgc2VudGVuY2UgdXNlZCBmb3IgdGVzdGluZyB0aGlzIGNyYXRlLiBJIGhvcGUgYWxsIGNhc2VzIGFyZSBjb3JyZWN0Lg==", fs::read_to_string("encode_output.txt").unwrap());
```

### Decode

#### FromBase64Reader

```rust
extern crate base64_stream;

use std::io::Cursor;

use std::io::Read;

use base64_stream::FromBase64Reader;

let base64 = b"SGkgdGhlcmUsIHRoaXMgaXMgYSBzaW1wbGUgc2VudGVuY2UgdXNlZCBmb3IgdGVzdGluZyB0aGlzIGNyYXRlLiBJIGhvcGUgYWxsIGNhc2VzIGFyZSBjb3JyZWN0Lg==".to_vec();

let mut reader = FromBase64Reader::new(Cursor::new(base64));

let mut test_data = [0u8; 4096];

let c = reader.read(&mut test_data).unwrap();

assert_eq!(b"Hi there, this is a simple sentence used for testing this crate. I hope all cases are correct.".to_vec(), test_data[..c].to_vec());
```

#### FromBase64Writer

```rust
extern crate base64_stream;

use std::fs::{self, File};

use std::io::Write;

use base64_stream::FromBase64Writer;

let base64 = b"SGkgdGhlcmUsIHRoaXMgaXMgYSBzaW1wbGUgc2VudGVuY2UgdXNlZCBmb3IgdGVzdGluZyB0aGlzIGNyYXRlLiBJIGhvcGUgYWxsIGNhc2VzIGFyZSBjb3JyZWN0Lg==".as_ref();

let test_data = File::create("decode_output.txt").unwrap();

let mut writer = FromBase64Writer::new(test_data);

writer.write(base64).unwrap();

writer.flush().unwrap(); // the flush method is only used when the full base64 data has been written

assert_eq!("Hi there, this is a simple sentence used for testing this crate. I hope all cases are correct.", fs::read_to_string("decode_output.txt").unwrap());
```
*/

pub extern crate base64;
#[macro_use]
extern crate educe;

mod to_base64_reader;
mod from_base64_reader;
mod to_base64_writer;
mod from_base64_writer;

pub use to_base64_reader::ToBase64Reader;
pub use from_base64_reader::FromBase64Reader;
pub use to_base64_writer::ToBase64Writer;
pub use from_base64_writer::FromBase64Writer;