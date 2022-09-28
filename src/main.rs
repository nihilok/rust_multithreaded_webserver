use std::{
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    collections::HashMap,
    str,
};
use threadpool::ThreadPool;
use format_bytes::format_bytes;

type FileResponse = (&'static str, &'static str);
type RouteMap = HashMap<&'static str, fn() -> FileResponse>;


fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(4);
    let routes = register_routes();

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let r = routes.to_owned();
        pool.execute(move || {
            handle_connection(stream, &r);
        });
    }
}

fn handle_connection(mut stream: TcpStream, routes: &RouteMap) {
    let buf_reader = BufReader::new(&mut stream);
    let request_line = buf_reader.lines().next().unwrap().unwrap();

    let request_parts = request_line.split_whitespace();

    let request_vec = request_parts.collect::<Vec<&str>>();
    let (_method, path, _version) = (request_vec[0], request_vec[1], request_vec[2]);

    let response = get_response_parts(path, routes);
    match path {
        "/download" => {
            stream.write_all(response.file_response().as_bytes()).unwrap();
        }
        _ => {
            stream.write_all(response.simple_response().as_bytes()).unwrap();
        }
    }
    stream.write_all(response.file_response().as_bytes()).unwrap();
}

struct Response<'a> {
    status_line: &'a str,
    content_type: Option<&'a str>,
    contents: Vec<u8>,
}

impl<'a> Response<'a> {
    fn new(status_line: &'a str, filename: &'a str, content_type: Option<&'a str>) -> Response<'a> {
        let contents = fs::read(filename).unwrap();
        Response { status_line, content_type, contents }
    }
    fn len(&self) -> usize {
        self.contents.len()
    }
    fn simple_response(&self) -> String {
        let content = str::from_utf8(&self.contents).unwrap();
        let http =
            format!("{}\r\nContent-Length: {}\r\n\r\n{}", self.status_line, self.len(), content);
        return http;
    }
    fn file_response(&self) -> String {
        let bytes_string = format_bytes!(b"{}", &self.contents);
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
                content-disposition: attachment; filename*=download.mp3\r\n\r\n\
                {:?}",
                self.status_line,
                self.len(),
                content_type,
                bytes_string
            );
        return http;
    }
}

fn get_response_parts<'a>(path: &str, routes: &RouteMap) -> Response<'a> {
    let parts = if routes.contains_key(path) { routes.get(path).unwrap()() } else { unknown_request() };
    let response = Response::new(parts.0, parts.1, Some("audio/mpeg"));
    return response;
}

fn unknown_request() -> (&'static str, &'static str) {
    ("HTTP/1.1 404 NOT FOUND", "404.html")
}

fn index() -> FileResponse {
    ("HTTP/1.1 200 OK", "hello.html")
}

fn download() -> FileResponse {
    ("HTTP/1.1 200 OK", "test.mp3")
}

fn register_routes() -> RouteMap {
    let mut routes: RouteMap = HashMap::new();
    routes.insert("/", index);
    routes.insert("/download", download);
    return routes;
}