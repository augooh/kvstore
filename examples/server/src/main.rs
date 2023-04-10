use std::io::Read;
use std::net::{TcpListener, TcpStream};
use std::str;
use std::thread;

fn handle_client(mut stream: TcpStream) {
    let mut buf = [0; 128];
    loop {
        // 读取内容
        let len = stream.read(&mut buf).unwrap();
        if len == 0 {
            println!("ok");
            break;
        }
        // 输出读取到的内容
        println!("read {} bytes: {:?}", len, str::from_utf8(&buf[..len]));
    }
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4567").unwrap();

    // 对每一个连接开启一个线程进行处理
    for stream in listener.incoming() {
        thread::spawn(move || {
            handle_client(stream.unwrap());
        });
    }
    println!("Hello, world!");
}
