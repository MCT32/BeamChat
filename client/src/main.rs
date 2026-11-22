use std::{io::{Read, Write}, net::TcpListener};

const HOST: &str = "127.0.0.1:5555";

fn main() {
    let listener = TcpListener::bind(HOST).unwrap();
    println!("Listening on {HOST}");

    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        let peer = stream.peer_addr().unwrap();
        println!("new connection: {peer}");

        let mut buf = [0u8; 1024];
        loop {
            let n = stream.read(&mut buf).unwrap();
            if n == 0 {
                break;
            }
            stream.write_all(&buf[..n]).unwrap();
        }
    }
}
