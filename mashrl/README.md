Minimal and simple HTTP(s) request library

Usage:

```rust
use mashrl::{HTTP, make_request};

fn main() {
	let site = "example.com";
	let path = "";
	let headers = HTTP::Headers::empty();

	// Makes a request to `https://example.com`
    let response = make_request(site, path, &headers).unwrap();

    eprintln!("Code {code}", code = response.code.0);

    for (key, value) in &response.headers {
        eprintln!("{key}: {value}");
    }

    match str::from_utf8(&response.body) {
        Ok(body) => {
            eprintln!("{body}");
        }
        Err(_err) => {
            eprintln!("body not string");
        }
    }
}
```
