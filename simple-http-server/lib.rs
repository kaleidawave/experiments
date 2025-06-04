use std::{
    io::{BufRead, BufReader, Write},
    net::TcpListener,
};

#[allow(non_snake_case)]
pub mod HTTP {
    #[derive(Debug)]
    pub struct Request<'a> {
        pub method: &'a str,
        pub path: &'a str,
        pub headers: &'a str,
    }

    #[derive(PartialEq, Eq)]
    pub struct ResponseCode(pub u16);

    impl ResponseCode {
        pub const OK: ResponseCode = ResponseCode(200);
        pub const NOT_FOUND: ResponseCode = ResponseCode(404);
    }

    pub struct Response<'a> {
        pub code: ResponseCode,
        pub body: std::borrow::Cow<'a, [u8]>,
    }
}

pub fn open_server(port: u16, callback: impl for<'a> Fn(HTTP::Request<'a>) -> HTTP::Response<'a>) {
    let listener =
        TcpListener::bind(&format!("127.0.0.1:{port}")).expect("Could not open listener on port");
    for mut stream in listener.incoming().flatten() {
        let mut rdr = BufReader::new(&mut stream);

        let inner = rdr.fill_buf().unwrap();
        let inner = inner.to_vec();
        let (method, inner) = inner.split_at(inner.iter().position(|byte| *byte == b' ').unwrap());
        let inner = &inner[1..];
        let (path, inner) = inner.split_at(inner.iter().position(|byte| *byte == b' ').unwrap());

        // TODO parse headers

        let request = HTTP::Request {
            method: str::from_utf8(method).unwrap(),
            path: str::from_utf8(path).unwrap(),
            headers: str::from_utf8(inner).unwrap(),
        };

        let response = callback(request);

        {
            stream.write_all(b"HTTP/1.1 ").unwrap();
            stream.write_all(b"HTTP/1.1").unwrap();
            let code: &[u8] = match response.code {
                HTTP::ResponseCode::OK => b"200 OK",
                HTTP::ResponseCode::NOT_FOUND => b"404 Not Found",
                code => unimplemented!("{code}", code = code.0),
            };
            stream.write_all(code).unwrap();
            // TODO post headers
            stream.write_all(b"\r\n\r\n").unwrap();
            stream.write_all(&response.body).unwrap();
        }
    }
}
