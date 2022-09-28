mod response;
use crate::response::Response;
use http_parser::HTTPRequest;
use std::{
    io::prelude::*,
    net::{TcpListener, TcpStream},
    collections::HashMap,
    str,
};
use threadpool::ThreadPool;

type HTTPFile = (String, String);
type RouteMap = HashMap<&'static str, fn() -> HTTPFile>;

const HTTP_SUCCESS: &str = "HTTP/1.1 200 OK";
const HTTP_NOT_FOUND: &str = "HTTP/1.1 404 NOT FOUND";
const ADDR: &str = "0.0.0.0";
const PORT: u16 = 8080;
const THREADPOOL_SIZE: usize = 6;


fn main() {
    let pool = ThreadPool::new(THREADPOOL_SIZE);
    let bind_addr = format!("{}:{}", ADDR, PORT);
    let listener = TcpListener::bind(&bind_addr).unwrap();
    println!("Listening for TCP traffic at http://{}", bind_addr);


    let routes = register_routes();

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let r = routes.to_owned();
        pool.execute(move || {
            handle_connection(&stream, &r);
        });
    }
}

fn handle_connection(mut stream: &TcpStream, routes: &RouteMap) {
    let buffer: HTTPRequest = HTTPRequest::new(&stream);
    let response = {
        let request =  buffer.request();
        get_response_parts(&request.path, routes)
    };
    match &buffer.request().path[..] {
        "/download" => {
            let _ = stream.write_all(&response.file_response()).unwrap();
        }
        _ => {
            let _ = stream.write_all(&response.simple_response()).unwrap();
        }
    }
}

fn get_response_parts(path: &str, routes: &RouteMap) -> Response {
    let mut success = true;
    let filename: HTTPFile = match routes.get(path) {
        None => {
            success = false;
            ("404.html".to_string(), "text/html".to_string())
        }
        Some(route) => { route() }
    };
    let status_line = if success { HTTP_SUCCESS.to_string() } else { HTTP_NOT_FOUND.to_string() };
    let response = Response::new(status_line, filename.0, Some(filename.1));
    return response;
}

fn index() -> HTTPFile {
    ("hello.html".to_string(), "text/html".to_string())
}

fn download() -> HTTPFile {
    ("test.mp3".to_string(), "audio/mpeg".to_string())
}

fn register_routes() -> RouteMap {
    let mut routes: RouteMap = HashMap::new();
    routes.insert("/", index);
    routes.insert("/download", download);
    return routes;
}