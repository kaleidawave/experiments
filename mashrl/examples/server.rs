#[cfg(all(feature = "server", feature="extras"))]
fn main() {
    use mashrl::{
        extras::serve_file,
        http::{Headers, Request, Response, ResponseCode},
        server::open_http_server,
    };
    let port: u16 = 6767;

    let address = format!("127.0.0.1:{port}");
    let protocol = "http";
    eprintln!("Live at {protocol}://{address}");

    open_http_server(address, |request| {
        use std::fmt::Write;

        let Request {
            method,
            path,
            headers,
            body: _request_body,
        } = request;

        if let "GET" = &*method.0
            && let "/index.html" = &*path
        {
            serve_file(std::path::Path::new("examples/index.html"))
        } else if let "POST" = &*method.0
            && let "/content.json" = &*path
        {
            serve_file(std::path::Path::new("examples/content.json"))
        } else {
            let mut body = format!("method={method:?} path={path:?}");
            writeln!(&mut body).unwrap();
            for (key, value) in &headers {
                writeln!(&mut body, "{key}: {value}").unwrap();
            }
            writeln!(&mut body, "--- end ---").unwrap();

            Response {
                code: ResponseCode::OK,
                headers: Headers::empty(),
                // TODO hmm?
                body: Box::new(std::io::Cursor::new(body.into_bytes())),
            }
        }
    });
}

#[cfg(not(all(feature = "server", feature="extras")))]
fn main() {
    panic!("requires server and extras features")
}
