use std::{fs, str};

pub struct Response<'a> {
    status_line: &'a str,
    content_type: Option<&'a str>,
    contents: Vec<u8>,
}

impl<'a> Response<'a> {
    pub fn new(status_line: &'a str, filename: &'a str, content_type: Option<&'a str>) -> Response<'a> {
        let contents = fs::read(filename).unwrap();
        Response { status_line, content_type, contents }
    }
    pub fn len(&self) -> usize {
        self.contents.len()
    }
    pub fn simple_response(&self) -> String {
        let content = str::from_utf8(&self.contents).unwrap();
        let http =
            format!("{}\r\nContent-Length: {}\r\n\r\n{}", self.status_line, self.len(), content);
        return http;
    }
    pub fn file_response(&self) -> Vec<u8> {
        let bytes_string = &self.contents[..];
        let content_type = match self.content_type {
            None => { "application/octet-stream" }
            Some(c_type) => { c_type }
        };
        let http =
            format!(
                "{}\r\n\
                Connection: keep-alive\r\n\
                Content-Length: {}\r\n\
                Content-Type: {}\r\n\
                content-disposition: attachment; filename*=download.mp3\r\n\r\n",
                self.status_line,
                self.len(),
                content_type,
            );
        let response_bytes = &http.as_bytes();
        let response = [response_bytes, bytes_string].concat();
        return response;
    }
}
