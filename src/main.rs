use std::{
    io::prelude::*,
    io::BufReader,
    net::{TcpListener, TcpStream},
};

mod pool;

static HTML_PAGE: &str = include_str!("index.html");
static NOTFOUND_PAGE: &str = include_str!("not_found.html");

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        handle_connection(stream);
    }
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);

    let req_line = buf_reader
        .lines()
        .next()
        .unwrap()
        .unwrap();

    let (status_line, content) = if req_line == "GET / HTTP/1.1" {
        ("HTTP/1.1 200 OK", HTML_PAGE)
    } else {
        ("HTTP/1.1 404 NOT FOUND", NOTFOUND_PAGE)
    };

    let length = content.len();
    let resp = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{content}");

    stream.write_all(resp.as_bytes()).unwrap();
}
