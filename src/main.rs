use std::{
    io::prelude::*,
    io::BufReader,
    net::{TcpListener, TcpStream}, thread, time::Duration,
};

mod pool;

static HTML_PAGE: &str = include_str!("index.html");
static NOTFOUND_PAGE: &str = include_str!("not_found.html");

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    let mut thread_pool = pool::ThreadPool::new(5);

    println!("Started listening on 8080 ...");

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        thread_pool.execute(|| handle_connection(stream));
    }

    println!("Exiting ...");
}

fn handle_connection(mut stream: TcpStream) {
    thread::sleep(Duration::from_secs(5));
    
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
