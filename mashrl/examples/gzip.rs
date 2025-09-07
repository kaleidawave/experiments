#[cfg(feature = "decompress")]
fn main() {
    use mashrl::{HTTP, make_get_request};
    use std::io::Read;

    let mut response = make_get_request("httpbin.org", "gzip", &HTTP::Headers::empty()).unwrap();

    eprintln!("Code {code}", code = response.code.0);

    for (key, value) in &response.headers {
        eprintln!("{key}: {value}");
    }

    let mut content = String::new();
    response.body.read_to_string(&mut content).unwrap();
    eprintln!("{content}");
}

#[cfg(not(feature = "decompress"))]
fn main() {
    panic!("requires 'decompress' feature")
}
