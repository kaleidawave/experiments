use simple_http_server::{
    open_server,
    HTTP::{Response, ResponseCode},
};

fn main() {
    let port = 5000;
    eprintln!("Live at http://localhost:{port}");
    open_server(port, |request| {
        dbg!(request);
        Response {
            code: ResponseCode::OK,
            body: b"Hello world".into(),
        }
    });
}
