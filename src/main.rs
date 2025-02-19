use std::{fs, io::Write, net::{TcpListener,TcpStream}, thread, time::Duration};
use std::io::prelude::*;
use hello_server::ThreadPool;

fn main() {
   let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
   for stream in listener.incoming() {
       let stream = stream.unwrap();
       ThreadPool::new(4).execute(|| {
           handle_connection(stream);
       });
   }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();
    
    let get = b"GET / HTTP/1.1\r\n";
    let sleep = b"GET /sleep HTTP/1.1\r\n";

    let (status_line, filename) = if buffer.starts_with(get) {
        ("HTTP/1.1 200 OK", "hello.html")
    } else if buffer.starts_with(sleep) {
        thread::sleep(Duration::from_secs(5));
        ("HTTP/1.1 200 OK", "hello.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND", "404.html")
    };
    
    let contents = fs::read_to_string(filename).unwrap();
    println!("Request: {}", contents);
    let response = format!("{}{}\r\n\r\n{}", status_line, contents, "\r\n");
    stream.write_all(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}