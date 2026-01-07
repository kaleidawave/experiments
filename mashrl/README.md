Minimal and simple HTTP(s) request library

Usage:

```rust
use mashrl::{HTTP, make_get_request};
use std::io::Read;

fn main() {
    let mut response = make_get_request("httpbin.org", "get", &HTTP::Headers::empty()).unwrap();

    eprintln!("Code {code}", code = response.code.0);

    for (key, value) in &response.headers {
        eprintln!("{key}: {value}");
    }

    let mut content = String::new();
    response.body.read_to_string(&mut content).unwrap();
    eprintln!("{content}");
}
```

Features

- Simple to use API
- Header creation and parsing API
- Chunked decoding
- Gzip decompression (under `-F decompress`)

Future

- Specify port (default to `443`, the standard for *internet traffic*)
- Improvements to body handling (auto `Content-Length` header?)
- Tracing (under flag)?
- HTTPS under flag?
