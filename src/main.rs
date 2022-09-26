use std::{
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};
use std::process::Command;
mod thread_pool;


fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = thread_pool::ThreadPool::new(4);

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        pool.execute(|| {
            handle_connection(stream);
        });
    }
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let request_line = buf_reader.lines().next().unwrap().unwrap();

    let request_parts = request_line.split_whitespace();

    let request_vec = request_parts.collect::<Vec<&str>>();
    let (method, path, _) = (request_vec[0], request_vec[1], request_vec[2]);

    let response: (&str, &str);
    if method == "GET" {
        response = get_request(path);
    } else if method == "POST" {
        response = post_request(path);
    } else {
        response = unknown_request();
    }

    let http = response.0;
    let filename = response.1;

    let contents = fs::read_to_string(filename).unwrap();
    let length = contents.len();

    let response =
        format!("{http}\r\nContent-Length: {length}\r\n\r\n{contents}");

    stream.write_all(response.as_bytes()).unwrap();
}

fn get_request(path: &str) -> (&str, &str) {
    if path != "/" {
        return unknown_request();
    }
    ("HTTP/1.1 200 OK", "hello.html")
}
fn post_request(path: &str) -> (&str, &str) {
    ("HTTP/1.1 405 METHOD NOT ALLOWED", "404.html")
}
fn unknown_request() -> (&'static str, &'static str) {
    ("HTTP/1.1 404 NOT FOUND", "404.html")
}