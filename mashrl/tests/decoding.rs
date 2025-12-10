use mashrl::http::ChunkedReader;
use std::io::Read;

#[test]
fn decode() {
    let source = "5\r\nhello\r\n6\r\n world\r\n0\r\n\r\n";
    let mut reader = ChunkedReader::new(source.as_bytes());
    let mut out = String::new();
    let result = reader.read_to_string(&mut out).unwrap();
    assert_eq!(result, 11);
    assert_eq!(out, "hello world");
}
