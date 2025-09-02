#![warn(clippy::pedantic)]

use native_tls::{TlsConnector, TlsStream};
use std::net::TcpStream;

#[allow(non_snake_case)]
pub mod HTTP {
    #[derive(Debug)]
    pub struct Request<'a> {
        pub method: &'a str,
        pub path: &'a str,
        pub headers: Headers<'a>,
        pub body: &'a [u8],
    }

    #[derive(PartialEq, Eq, Clone, Copy)]
    pub struct ResponseCode(pub u16);

    impl ResponseCode {
        pub const CONTINUE: ResponseCode = ResponseCode(100);
        pub const SWITCHING_PROTOCOLS: ResponseCode = ResponseCode(101);
        pub const PROCESSING: ResponseCode = ResponseCode(102);
        pub const EARLY_HINTS: ResponseCode = ResponseCode(103);

        pub const OK: ResponseCode = ResponseCode(200);
        pub const CREATED: ResponseCode = ResponseCode(201);
        pub const ACCEPTED: ResponseCode = ResponseCode(202);
        pub const NON_AUTHORITATIVE_INFORMATION: ResponseCode = ResponseCode(203);
        pub const NO_CONTENT: ResponseCode = ResponseCode(204);
        pub const RESET_CONTENT: ResponseCode = ResponseCode(205);
        pub const PARTIAL_CONTENT: ResponseCode = ResponseCode(206);
        pub const MULTI_STATUS: ResponseCode = ResponseCode(207);
        pub const ALREADY_REPORTED: ResponseCode = ResponseCode(208);
        pub const IM_USED: ResponseCode = ResponseCode(226);

        pub const MULTIPLE_CHOICES: ResponseCode = ResponseCode(300);
        pub const MOVED_PERMANENTLY: ResponseCode = ResponseCode(301);
        pub const FOUND: ResponseCode = ResponseCode(302);
        pub const SEE_OTHER: ResponseCode = ResponseCode(303);
        pub const NOT_MODIFIED: ResponseCode = ResponseCode(304);
        pub const TEMPORARY_REDIRECT: ResponseCode = ResponseCode(307);
        pub const PERMANENT_REDIRECT: ResponseCode = ResponseCode(308);

        pub const BAD_REQUEST: ResponseCode = ResponseCode(400);
        pub const UNAUTHORIZED: ResponseCode = ResponseCode(401);
        pub const PAYMENT_REQUIRED: ResponseCode = ResponseCode(402);
        pub const FORBIDDEN: ResponseCode = ResponseCode(403);
        pub const NOT_FOUND: ResponseCode = ResponseCode(404);
        pub const METHOD_NOT_ALLOWED: ResponseCode = ResponseCode(405);
        pub const NOT_ACCEPTABLE: ResponseCode = ResponseCode(406);
        pub const PROXY_AUTHENTICATION_REQUIRED: ResponseCode = ResponseCode(407);
        pub const REQUEST_TIMEOUT: ResponseCode = ResponseCode(408);
        pub const CONFLICT: ResponseCode = ResponseCode(409);
        pub const GONE: ResponseCode = ResponseCode(410);
        pub const LENGTH_REQUIRED: ResponseCode = ResponseCode(411);
        pub const PRECONDITION_FAILED: ResponseCode = ResponseCode(412);
        pub const CONTENT_TOO_LARGE: ResponseCode = ResponseCode(413);
        pub const URI_TOO_LONG: ResponseCode = ResponseCode(414);
        pub const UNSUPPORTED_MEDIA_TYPE: ResponseCode = ResponseCode(415);
        pub const RANGE_NOT_SATISFIABLE: ResponseCode = ResponseCode(416);
        pub const EXPECTATION_FAILED: ResponseCode = ResponseCode(417);
        pub const IM_A_TEAPOT: ResponseCode = ResponseCode(418);
        pub const MISDIRECTED_REQUEST: ResponseCode = ResponseCode(421);
        pub const UNPROCESSABLE_CONTENT: ResponseCode = ResponseCode(422);
        pub const LOCKED: ResponseCode = ResponseCode(423);
        pub const FAILED_DEPENDENCY: ResponseCode = ResponseCode(424);
        pub const TOO_EARLY: ResponseCode = ResponseCode(425);
        pub const UPGRADE_REQUIRED: ResponseCode = ResponseCode(426);
        pub const PRECONDITION_REQUIRED: ResponseCode = ResponseCode(428);
        pub const TOO_MANY_REQUESTS: ResponseCode = ResponseCode(429);
        pub const REQUEST_HEADER_FIELDS_TOO_LARGE: ResponseCode = ResponseCode(431);
        pub const UNAVAILABLE_FOR_LEGAL_REASONS: ResponseCode = ResponseCode(451);

        pub const INTERNAL_SERVER_ERROR: ResponseCode = ResponseCode(500);
        pub const NOT_IMPLEMENTED: ResponseCode = ResponseCode(501);
        pub const BAD_GATEWAY: ResponseCode = ResponseCode(502);
        pub const SERVICE_UNAVAILABLE: ResponseCode = ResponseCode(503);
        pub const GATEWAY_TIMEOUT: ResponseCode = ResponseCode(504);
        pub const HTTP_VERSION_NOT_SUPPORTED: ResponseCode = ResponseCode(505);
        pub const VARIANT_ALSO_NEGOTIATES: ResponseCode = ResponseCode(506);
        pub const INSUFFICIENT_STORAGE: ResponseCode = ResponseCode(507);
        pub const LOOP_DETECTED: ResponseCode = ResponseCode(508);
        pub const NOT_EXTENDED: ResponseCode = ResponseCode(510);
        pub const NETWORK_AUTHENTICATION_REQUIRED: ResponseCode = ResponseCode(511);

        /// # Errors
        ///
        /// returns an `Err` if the input is not a known HTTP response code
        pub fn from_line(item: &str) -> Result<Self, &str> {
            Ok(match item {
                "100 Continue" => Self::CONTINUE,
                "101 Switching Protocols" => Self::SWITCHING_PROTOCOLS,
                "102 Processing" => Self::PROCESSING,
                "103 Early Hints" => Self::EARLY_HINTS,
                "200 OK" => Self::OK,
                "201 Created" => Self::CREATED,
                "202 Accepted" => Self::ACCEPTED,
                "203 Non-Authoritative Information" => Self::NON_AUTHORITATIVE_INFORMATION,
                "204 No Content" => Self::NO_CONTENT,
                "205 Reset Content" => Self::RESET_CONTENT,
                "206 Partial Content" => Self::PARTIAL_CONTENT,
                "207 Multi-Status" => Self::MULTI_STATUS,
                "208 Already Reported" => Self::ALREADY_REPORTED,
                "226 IM Used" => Self::IM_USED,
                "300 Multiple Choices" => Self::MULTIPLE_CHOICES,
                "301 Moved Permanently" => Self::MOVED_PERMANENTLY,
                "302 Found" => Self::FOUND,
                "303 See Other" => Self::SEE_OTHER,
                "304 Not Modified" => Self::NOT_MODIFIED,
                "307 Temporary Redirect" => Self::TEMPORARY_REDIRECT,
                "308 Permanent Redirect" => Self::PERMANENT_REDIRECT,
                "400 Bad Request" => Self::BAD_REQUEST,
                "401 Unauthorized" => Self::UNAUTHORIZED,
                "402 Payment Required" => Self::PAYMENT_REQUIRED,
                "403 Forbidden" => Self::FORBIDDEN,
                "404 Not Found" => Self::NOT_FOUND,
                "405 Method Not Allowed" => Self::METHOD_NOT_ALLOWED,
                "406 Not Acceptable" => Self::NOT_ACCEPTABLE,
                "407 Proxy Authentication Required" => Self::PROXY_AUTHENTICATION_REQUIRED,
                "408 Request Timeout" => Self::REQUEST_TIMEOUT,
                "409 Conflict" => Self::CONFLICT,
                "410 Gone" => Self::GONE,
                "411 Length Required" => Self::LENGTH_REQUIRED,
                "412 Precondition Failed" => Self::PRECONDITION_FAILED,
                "413 Content Too Large" => Self::CONTENT_TOO_LARGE,
                "414 URI Too Long" => Self::URI_TOO_LONG,
                "415 Unsupported Media Type" => Self::UNSUPPORTED_MEDIA_TYPE,
                "416 Range Not Satisfiable" => Self::RANGE_NOT_SATISFIABLE,
                "417 Expectation Failed" => Self::EXPECTATION_FAILED,
                "418 I'm a teapot" => Self::IM_A_TEAPOT,
                "421 Misdirected Request" => Self::MISDIRECTED_REQUEST,
                "422 Unprocessable Content" => Self::UNPROCESSABLE_CONTENT,
                "423 Locked" => Self::LOCKED,
                "424 Failed Dependency" => Self::FAILED_DEPENDENCY,
                "425 Too Early" => Self::TOO_EARLY,
                "426 Upgrade Required" => Self::UPGRADE_REQUIRED,
                "428 Precondition Required" => Self::PRECONDITION_REQUIRED,
                "429 Too Many Requests" => Self::TOO_MANY_REQUESTS,
                "431 Request Header Fields Too Large" => Self::REQUEST_HEADER_FIELDS_TOO_LARGE,
                "451 Unavailable For Legal Reasons" => Self::UNAVAILABLE_FOR_LEGAL_REASONS,
                "500 Internal Server Error" => Self::INTERNAL_SERVER_ERROR,
                "501 Not Implemented" => Self::NOT_IMPLEMENTED,
                "502 Bad Gateway" => Self::BAD_GATEWAY,
                "503 Service Unavailable" => Self::SERVICE_UNAVAILABLE,
                "504 Gateway Timeout" => Self::GATEWAY_TIMEOUT,
                "505 HTTP Version Not Supported" => Self::HTTP_VERSION_NOT_SUPPORTED,
                "506 Variant Also Negotiates" => Self::VARIANT_ALSO_NEGOTIATES,
                "507 Insufficient Storage" => Self::INSUFFICIENT_STORAGE,
                "508 Loop Detected" => Self::LOOP_DETECTED,
                "510 Not Extended" => Self::NOT_EXTENDED,
                "511 Network Authentication Required" => Self::NETWORK_AUTHENTICATION_REQUIRED,
                item => {
                    // Custom status code
                    if let Some(value) = item
                        .split_once(' ')
                        .and_then(|(lhs, _)| lhs.parse::<u16>().ok())
                    {
                        Self(value)
                    } else {
                        return Err(item);
                    }
                }
            })
        }
    }

    pub struct Response<'a> {
        pub code: ResponseCode,
        pub headers: Headers<'a>,
        pub body: Encoding<native_tls::TlsStream<std::net::TcpStream>>,
    }

    #[derive(Clone, Debug)]
    pub struct Headers<'a>(pub std::borrow::Cow<'a, str>);

    impl Headers<'static> {
        #[must_use]
        pub fn empty() -> Headers<'static> {
            Headers(std::borrow::Cow::Borrowed(""))
        }

        /// # Errors
        ///
        /// returns an `Err` if the input is not `utf8`
        pub fn from_raw(on: Vec<u8>) -> Result<Headers<'static>, std::string::FromUtf8Error> {
            String::from_utf8(on).map(Headers::from_string)
        }

        #[must_use]
        pub fn from_string(on: String) -> Headers<'static> {
            Headers(std::borrow::Cow::Owned(on))
        }

        #[must_use]
        pub fn iter(&self) -> HeaderIter<'_> {
            HeaderIter(self.0.lines())
        }
    }

    impl<'a> IntoIterator for &'a Headers<'_> {
        type Item = (&'a str, &'a str);
        type IntoIter = HeaderIter<'a>;

        fn into_iter(self) -> Self::IntoIter {
            HeaderIter(self.0.lines())
        }
    }

    pub struct HeaderIter<'a>(pub(super) std::str::Lines<'a>);

    impl<'a> Iterator for HeaderIter<'a> {
        type Item = (&'a str, &'a str);

        fn next(&mut self) -> Option<Self::Item> {
            let row = self.0.next()?;
            // TODO what if `None` here?
            let (key, value) = row.split_once(':')?;
            Some((key, value.trim()))
        }
    }

    pub enum Encoding<T> {
        Raw(std::io::BufReader<T>),
        Chunked(ChunkedReader<T>),
    }

    impl<T> std::io::Read for Encoding<T>
    where
        T: std::io::Read,
    {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            match self {
                Self::Raw(reader) => reader.read(buf),
                Self::Chunked(reader) => reader.read(buf),
            }
        }
    }

    pub struct ChunkedReader<T> {
        reader: std::io::BufReader<T>,
        to_read: usize,
    }

    impl<T> ChunkedReader<T>
    where
        T: std::io::Read,
    {
        pub fn new(source: T) -> Self {
            Self {
                reader: std::io::BufReader::new(source),
                to_read: 0,
            }
        }

        pub fn new_from_reader(reader: std::io::BufReader<T>) -> Self {
            Self { reader, to_read: 0 }
        }
    }

    impl<T> std::io::Read for ChunkedReader<T>
    where
        T: std::io::Read,
    {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            use std::io::BufRead;

            if self.to_read == 0 {
                let mut chunk_size_buf = String::new();
                self.reader.read_line(&mut chunk_size_buf)?;

                let chunk_size_str = chunk_size_buf.trim_end();
                let hex = u64::from_str_radix(chunk_size_str, 16);
                let Ok(chunk_size) = hex else {
                    let message = format!("invalid chunk length {chunk_size_str:?}");
                    let error = std::io::Error::new(std::io::ErrorKind::InvalidData, message);
                    return Err(error);
                };

                // The terminating chunk is a zero-length chunk
                if chunk_size == 0 {
                    return Ok(0);
                }

                let chunk_size = usize::try_from(chunk_size).unwrap_or(usize::MAX);

                self.to_read = chunk_size;
            }

            let mut reader_over_chunk = self.reader.by_ref().take(self.to_read as u64);
            let bytes_tranferred = reader_over_chunk.read(buf)?;
            self.to_read -= bytes_tranferred;

            if self.to_read == 0 {
                let mut end: [u8; 2] = [0, 0];
                self.reader.read_exact(&mut end)?;

                #[cfg(debug_assertions)]
                if &end != b"\r\n" {
                    let message = "expected '\r\n' at end of chunked frame";
                    let error = std::io::Error::new(std::io::ErrorKind::InvalidData, message);
                    return Err(error);
                }
            }

            Ok(bytes_tranferred)
        }
    }
}

/// Start the HTTP request, sending [`HTTP::Headers<'_>`]
fn initiate_stream(
    root: &str,
    path: &str,
    headers: &HTTP::Headers<'_>,
) -> Result<TlsStream<TcpStream>, Box<dyn std::error::Error>> {
    use std::io::Write;

    // TODO 443
    let url = format!("{root}:443");
    let tcp_stream = TcpStream::connect(url)?;
    let connector = TlsConnector::new()?;
    let mut tls_stream = connector.connect(root, tcp_stream)?;
    let base_request = format!(
        "GET /{path} HTTP/1.1\r\n\
	Host: {root}\r\n\
	Connection: close\r\n\
	User-Agent: yes\r\n"
    );

    tls_stream.write_all(base_request.as_bytes())?;
    if !headers.0.is_empty() {
        tls_stream.write_all(headers.0.as_bytes())?;
        tls_stream.write_all(b"\r\n")?;
    }
    tls_stream.write_all(b"\r\n")?;

    Ok(tls_stream)
}

/// FUTURE response based off slices of a single buffer?
///
/// # Errors
///
/// returns an error if the returned HTTP response is invalid
pub fn make_request(
    root: &str,
    path: &str,
    headers: &HTTP::Headers<'_>,
) -> Result<HTTP::Response<'static>, Box<dyn std::error::Error>> {
    use std::io::{BufRead, BufReader};

    let stream = initiate_stream(root, path, headers)?;

    let mut reader = BufReader::new(stream);

    let code: HTTP::ResponseCode = {
        let mut line = String::new();
        let Ok(_bytes_read) = reader.read_line(&mut line) else {
            return Err("no code".into());
        };
        let line = line.trim_end();
        let code = line
            .split_once(' ')
            .and_then(|(_method, item)| HTTP::ResponseCode::from_line(item).ok());

        let Some(code) = code else {
            return Err(format!("invalid response code: {line}").into());
        };
        code
    };

    let mut headers = String::new();
    let mut chunked = false;

    loop {
        let Ok(bytes_read) = reader.read_line(&mut headers) else {
            return Err("no code".into());
        };

        let last = headers.len() - bytes_read;
        let line = &headers[last..].trim_end();

        if line.is_empty() {
            // finished headers
            break;
        }

        if let Some(transfer_encoding) = line.strip_prefix("Transfer-Encoding: ") {
            chunked = transfer_encoding == "chunked";
        }
    }

    let headers = HTTP::Headers::from_string(headers);

    let body = if chunked {
        HTTP::Encoding::Chunked(HTTP::ChunkedReader::new_from_reader(reader))
    } else {
        HTTP::Encoding::Raw(reader)
    };

    let response = HTTP::Response {
        code,
        headers,
        body,
    };

    Ok(response)
}
