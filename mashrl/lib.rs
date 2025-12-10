#![warn(clippy::pedantic)]

use native_tls::{TlsConnector, TlsStream};
use std::net::TcpStream;

pub mod http;

#[cfg(feature = "server")]
pub mod server;

fn write_request<T: std::io::Read, S: std::io::Write>(
    request: &http::Request<'_, T>,
    mut stream: S,
) -> Result<S, Box<dyn std::error::Error>> {
    let http::Request {
        method,
        path,
        headers,
        body: _,
    } = request;

    let method: &str = &method.0;

    let base = format!("{method} /{path} HTTP/1.1\r\n");

    stream.write_all(base.as_bytes())?;
    // TODO should not be empty
    if !headers.0.is_empty() {
        stream.write_all(headers.0.as_bytes())?;
        stream.write_all(b"\r\n")?;
    }
    stream.write_all(b"\r\n")?;

    Ok(stream)
}

/// Start the http request, sending [`http::Headers<'_>`]
fn initiate_stream_tls<T: std::io::Read>(
    request: &http::Request<'_, T>,
) -> Result<TlsStream<TcpStream>, Box<dyn std::error::Error>> {
    let root = request
        .headers
        .iter()
        .find_map(|(key, value)| (key.eq_ignore_ascii_case("host")).then_some(value))
        .unwrap();
    let port = 443;
    let url = format!("{root}:{port}");
    let tcp_stream = TcpStream::connect(url)?;
    let connector = TlsConnector::new()?;
    let tls_stream = connector.connect(root, tcp_stream)?;
    write_request(request, tls_stream)
}

// fn initiate_stream_non_tls<T: std::io::Read>(
//     request: &http::Request<'_, T>,
// ) -> Result<TcpStream, Box<dyn std::error::Error>> {
//     let url = format!("{root}:443", root = request.root);
//     let tcp_stream = TcpStream::connect(url)?;
//     write_request(&request, tcp_stream)
// }

/// # Errors
///
/// returns an error if the returned http response is invalid
pub fn make_get_request(
    root: &str,
    path: &str,
    mut headers: http::Headers<'_>,
) -> Result<http::Response<'static>, Box<dyn std::error::Error>> {
    headers.append("Host", root);
    headers.append("Connection", "close");
    let request = http::Request {
        method: http::Method::GET,
        path: std::borrow::Cow::Borrowed(path),
        headers,
        body: std::io::empty(),
    };
    make_request(request)
}

/// # Errors
///
/// returns an error if the returned http response is invalid
pub fn make_request<T: std::io::Read + Send>(
    mut request: http::Request<'_, T>,
) -> Result<http::Response<'static>, Box<dyn std::error::Error>> {
    let mut stream = initiate_stream_tls(&request)?;
    let _out = std::io::copy(&mut request.body, &mut stream)?;
    parse_http_response(stream)

    // TODO monomorphism will 2x this...
    // if let Ok(ref response) = response && response.code == http::ResponseCode::UPGRADE_REQUIRED {
    //     // try non TLS
    //     let mut stream = initiate_stream(&request)?;
    //     let _out = std::io::copy(&mut request.content, &mut stream)?;
    //     parse_http_response(&mut stream)
    // } else {
    // response
    // }
}

pub(crate) fn parse_http_response<S: std::io::Read + Send + 'static>(
    stream: S,
) -> Result<http::Response<'static>, Box<dyn std::error::Error + 'static>> {
    use std::io::{BufRead, BufReader};

    let mut reader = BufReader::new(stream);

    let code: http::ResponseCode = {
        let mut line = String::new();
        let Ok(_bytes_read) = reader.read_line(&mut line) else {
            return Err("no code".into());
        };

        let code = line
            .trim_end()
            .split_once(' ')
            .and_then(|(_method, item)| http::ResponseCode::from_line(item).ok());

        let Some(code) = code else {
            return Err(format!("invalid response code: {line:?}").into());
        };
        code
    };

    let mut headers = String::new();
    let mut transfer_encoding_range: Option<std::ops::Range<usize>> = None;
    let mut content_encoding_range: Option<std::ops::Range<usize>> = None;

    loop {
        let Ok(bytes_read) = reader.read_line(&mut headers) else {
            return Err("no code".into());
        };

        let last = headers.len() - bytes_read;
        let line = &headers[last..].trim_end();

        if line.is_empty() {
            // finished headers
            // drop last '\r\n'
            let _ = headers.drain(headers.len() - 2..);
            break;
        }

        if line.starts_with("Transfer-Encoding: ") {
            let start = last + "Transfer-Encoding: ".len();
            transfer_encoding_range = Some(start..headers.len());
        }
        if line.starts_with("Content-Encoding: ") {
            let start = last + "Content-Encoding: ".len();
            content_encoding_range = Some(start..headers.len());
        }
    }

    #[allow(unused_mut)]
    let mut reader: Box<dyn std::io::Read + Send> = Box::new(reader);

    if let Some(range) = transfer_encoding_range {
        let transfer_encoding: &str = &headers[range];
        for part in transfer_encoding.split(',').map(str::trim) {
            match part {
                "chunked" => {
                    // TODO performance
                    reader = Box::new(http::ChunkedReader::new(std::io::BufReader::new(reader)));
                }
                #[cfg(feature = "decompress")]
                "gzip" => {
                    reader = Box::new(flate2::read::GzDecoder::new(reader));
                }
                part => {
                    eprintln!("Unhandled encoding {part:?}");
                }
            }
        }
    }
    if let Some(range) = content_encoding_range {
        let content_encoding: &str = &headers[range];
        for part in content_encoding.split(',').map(str::trim) {
            match part {
                #[cfg(feature = "decompress")]
                "gzip" => {
                    reader = Box::new(flate2::read::GzDecoder::new(reader));
                }
                part => {
                    eprintln!("Unhandled encoding {part:?}");
                }
            }
        }
    }

    let headers = http::Headers::from_string(headers);

    let body = reader;

    let response = http::Response {
        code,
        headers,
        body,
    };

    Ok(response)
}
