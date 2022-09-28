use std::{fs, str};

pub struct Response {
    status_line: String,
    content_type: Option<String>,
    filename: String,
    contents: Vec<u8>,
}

impl Response {
    pub fn new(status_line: String, filename: String, content_type: Option<String>) -> Response {
        let contents = fs::read(&filename).unwrap();
        Response { status_line, content_type, filename, contents }
    }
    pub fn len(&self) -> usize {
        self.contents.len()
    }
    pub fn simple_response(&self) -> Vec<u8> {
        let content = str::from_utf8(&self.contents).unwrap();
        let http =
            format!("{}\r\nContent-Length: {}\r\n\r\n{}", self.status_line, self.len(), content);
        return Vec::from(http.as_bytes());
    }
    pub fn file_response(&self) -> Vec<u8> {
        let bytes_string = &self.contents[..];
        let content_type = match &self.content_type {
            None => { "application/octet-stream" }
            Some(c_type) => { c_type }
        };
        let http =
            format!(
                "{}\r\n\
                Connection: keep-alive\r\n\
                Content-Length: {}\r\n\
                Content-Type: {}\r\n\
                content-disposition: attachment; filename*={}\r\n\r\n",
                self.status_line,
                self.len(),
                content_type,
                self.filename
            );
        let response_bytes = &http.as_bytes();
        let response = [response_bytes, bytes_string].concat();
        return response;
    }
}
