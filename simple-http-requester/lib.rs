#![warn(clippy::pedantic)]

use native_tls::{TlsConnector, TlsStream};
use std::io::{Read, Write};
use std::net::TcpStream;

pub fn make_request(root: &str, path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut stream = initiate_stream(root, path)?;

    let mut response = String::new();
    stream.read_to_string(&mut response)?;

    // {
    //     let mut file = std::fs::File::create("private/total.http")?;
    //     file.write_all(response.as_bytes())?;
    // }

    // Skip headers
    let mut lines = response.lines();
    let mut chunked = false;

    for line in lines.by_ref() {
        if let Some(location) = line.strip_prefix("Location: ") {
            eprintln!("Redirect = {location}");
        }
        if let Some(transfer_encoding) = line.strip_prefix("Transfer-Encoding: ") {
            chunked = transfer_encoding == "chunked";
        }

        if line.is_empty() {
            break;
        }
    }

    if let Some(next_line) = lines.next() {
        // TODO sub_ptr https://github.com/rust-lang/rust/issues/95892
        let headers_end = next_line.as_ptr() as usize - response.as_ptr() as usize;
        // WIP
        let body = response[headers_end..].to_owned();

        let body = if chunked {
            let (chunk_length_str, rest) = body.split_once("\r\n").expect("no chunk length");
            let first_size =
                u64::from_str_radix(chunk_length_str, 16).expect("invalid chunk length") as usize;
            let mut size = first_size;

            let mut new_body = rest[..size].to_owned();
            let mut pos = chunk_length_str.len() + "\r\n".len() + first_size + "\r\n".len();

            while size > 0 && pos < body.len() {
                let (chunk_length_str, rest) =
                    body[pos..].split_once("\r\n").expect("no chunk length");
                if chunk_length_str.is_empty() {
                    break;
                }
                size = u64::from_str_radix(chunk_length_str, 16).expect("invalid chunk length")
                    as usize;
                pos += chunk_length_str.len() + "\r\n".len() + size + "\r\n".len();
                new_body.push_str(&rest[..size]);
            }
            new_body
        } else {
            body
        };

        // {
        //     let mut file = std::fs::File::create("private/out.html")?;
        //     file.write_all(body.as_bytes())?;
        // }

        Ok(body)
    } else {
        Err(Box::<dyn std::error::Error>::from(String::from("No body")))
    }
}

fn initiate_stream(
    root: &str,
    path: &str,
) -> Result<TlsStream<TcpStream>, Box<dyn std::error::Error>> {
    // TODO 443
    let url = format!("{root}:443");
    let tcp_stream = TcpStream::connect(url)?;
    let connector = TlsConnector::new()?;
    let mut tls_stream = connector.connect(root, tcp_stream)?;
    let request = format!(
        "GET /{path} HTTP/1.1\r\n\
	Host: {root}\r\n\
	Connection: close\r\n\
	User-Agent: yes\r\n"
    );

    tls_stream.write_all(request.as_bytes())?;
    tls_stream.write_all(b"\r\n")?;

    Ok(tls_stream)
}
