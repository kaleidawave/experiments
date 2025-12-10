use mashrl::{http, make_get_request};
use std::io::Read;

fn main() {
    let mut response = make_get_request("httpbin.org", "get", http::Headers::empty()).unwrap();

    eprintln!("Code {code}", code = response.code.0);

    for (key, value) in &response.headers {
        eprintln!("{key}: {value}");
    }

    let mut content = String::new();
    response.body.read_to_string(&mut content).unwrap();
    eprintln!("{content}");
}
