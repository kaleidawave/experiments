#[cfg(feature = "extras")]
pub fn serve_file(path: &std::path::Path) -> crate::http::Response<'static> {
    let mut headers = crate::http::Headers::empty();

    if let Some(extension) = path.extension().and_then(std::ffi::OsStr::to_str) {
        headers.append(
            "Content-Type",
            crate::server::mimetypes::get_mimetype_from_extension(extension),
        );
    }

    headers.append("Connection", "close");
    if let Ok(file) = std::fs::File::open(path) {
        let size = file.metadata().unwrap().len();
        headers.append("Content-Length", &size.to_string());
        crate::http::Response {
            code: crate::http::ResponseCode::OK,
            headers,
            body: Box::new(file),
        }
    } else {
        crate::http::Response {
            code: crate::http::ResponseCode::NOT_FOUND,
            headers,
            body: Box::new(std::io::empty()),
        }
    }
}
