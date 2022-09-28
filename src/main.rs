use std::{
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
    collections::HashMap,
};
use std::process::Command;
use threadpool::ThreadPool;
use format_bytes::format_bytes;

type FileResponse = (&'static str, &'static str);
type RouteMap = HashMap<&'static str, fn(&str) -> FileResponse>;


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
    // stream.write_all(response.simple_response().as_bytes()).unwrap();
    stream.write_all(response.file_response().as_bytes()).unwrap();
}

struct Response<'a> {
    status_line: &'a str,
    content_type: &'a str,
    contents: Vec<u8>,
    filename: &'a str,
}

impl<'a> Response<'a> {
    fn new(status_line: &'a str, filename: &'a str, content_type: &'a str) -> Response<'a> {
        let contents = fs::read(filename).unwrap();
        Response { status_line, content_type, contents, filename }
    }
    fn len(&self) -> usize {
        self.contents.len()
    }
    fn simple_response(&self) -> String {
        let http =
            format!("{}\r\nContent-Length: {}\r\n\r\n{:?}", self.status_line, self.len(), self.contents);
        return http;
    }
    fn file_response(&self) -> String {
        let bytes_string = format_bytes!(b"{}", &self.contents);
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
                self.content_type,
                bytes_string
            );
        return http;
    }
}

fn get_response_parts<'a>(path: &str, routes: &RouteMap) -> Response<'a> {
    let parts = if routes.contains_key(path) { routes.get(path).unwrap()(path) } else { unknown_request() };
    let response = Response::new(parts.0, parts.1, "audio/mpeg");
    return response;π
}

fn post_request(path: &str) -> (&str, &str) {
    ("HTTP/1.1 405 METHOD NOT ALLOWED", "404.html")
}

fn unknown_request() -> (&'static str, &'static str) {
    ("HTTP/1.1 404 NOT FOUND", "404.html")
}

fn index(request: &str) -> FileResponse {
    ("HTTP/1.1 200 OK", "hello.html")
}

fn test(request: &str) -> FileResponse {
    ("HTTP/1.1 200 OK", "test.mp3")
}

fn register_routes() -> RouteMap {
    let mut routes: RouteMap = HashMap::new();
    routes.insert("/", index);
    routes.insert("/test", test);
    return routes;
}